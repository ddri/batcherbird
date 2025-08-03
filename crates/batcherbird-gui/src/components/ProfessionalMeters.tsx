import { useRef, useEffect } from 'react'
import { useProfessionalMeters, useGainStaging, GainStagingStatus, RecommendationUrgency, HeadroomStatus } from '@/hooks/useTauri'

interface ProfessionalMetersProps {
  isVisible: boolean
  className?: string
}

// Professional meter color schemes (matching Pro Tools/Logic Pro)
const METER_COLORS = {
  GREEN: '#00FF00',      // Safe zone
  YELLOW: '#FFFF00',     // Caution zone  
  ORANGE: '#FF8000',     // Warning zone
  RED: '#FF0000',        // Danger zone
  BACKGROUND: '#1a1a1a', // Meter background
  SCALE: '#666666',      // Scale markings
  HOLD_PEAK: '#FFFFFF',  // Peak hold indicator
  TARGET: '#00BFFF',     // -18dBFS target marker
  TEXT: '#CCCCCC'        // Text color
}

// Professional dB scale markings
const DB_SCALE = [0, -6, -12, -18, -24, -30, -36, -42, -48, -54, -60]

export function ProfessionalMeters({ isVisible, className = '' }: ProfessionalMetersProps) {
  const vuCanvasRef = useRef<HTMLCanvasElement>(null)
  const ppmCanvasRef = useRef<HTMLCanvasElement>(null)
  const peakCanvasRef = useRef<HTMLCanvasElement>(null)
  
  const { readings, isMonitoring, startProfessionalMetering, stopProfessionalMetering } = useProfessionalMeters()
  const { analysis, startGainStagingAnalysis, stopGainStagingAnalysis } = useGainStaging()

  // Auto-start/stop metering when visibility changes
  useEffect(() => {
    if (isVisible) {
      startProfessionalMetering()
      startGainStagingAnalysis()
    } else {
      stopProfessionalMetering() 
      stopGainStagingAnalysis()
    }
  }, [isVisible, startProfessionalMetering, stopProfessionalMetering, startGainStagingAnalysis, stopGainStagingAnalysis])

  // Convert dB to pixel position (0dB = top, -60dB = bottom)
  const dbToPixel = (db: number, height: number): number => {
    const clampedDb = Math.max(-60, Math.min(0, db))
    return ((60 + clampedDb) / 60) * height
  }

  // Get color for given dB level (currently unused but may be needed for future enhancements)
  // const getColorForDb = (db: number): string => {
  //   if (db > -6) return METER_COLORS.RED      // -6dB to 0dB
  //   if (db > -18) return METER_COLORS.ORANGE  // -18dB to -6dB  
  //   if (db > -30) return METER_COLORS.YELLOW  // -30dB to -18dB
  //   return METER_COLORS.GREEN                 // Below -30dB
  // }

  // Draw professional meter with proper ballistics
  const drawMeter = (canvas: HTMLCanvasElement, level: number, peakHold: number, label: string, showTarget: boolean = false) => {
    const ctx = canvas.getContext('2d')
    if (!ctx) return

    const width = canvas.width
    const height = canvas.height
    const meterWidth = 20
    const scaleWidth = 30
    const labelWidth = 40

    // Clear canvas
    ctx.fillStyle = '#000000'
    ctx.fillRect(0, 0, width, height)

    // Draw scale background
    ctx.fillStyle = METER_COLORS.BACKGROUND
    ctx.fillRect(labelWidth, 10, meterWidth, height - 20)

    // Draw dB scale markings
    ctx.fillStyle = METER_COLORS.SCALE
    ctx.font = '10px monospace'
    ctx.textAlign = 'right'
    
    DB_SCALE.forEach(db => {
      const y = dbToPixel(db, height - 20) + 10
      
      // Scale line
      ctx.fillRect(labelWidth + meterWidth, y - 0.5, 8, 1)
      
      // Scale text
      ctx.fillText(db.toString(), labelWidth + meterWidth + scaleWidth - 2, y + 3)
    })

    // Draw -18dBFS target marker (for VU meter)
    if (showTarget) {
      const targetY = dbToPixel(-18, height - 20) + 10
      ctx.strokeStyle = METER_COLORS.TARGET
      ctx.lineWidth = 2
      ctx.beginPath()
      ctx.moveTo(labelWidth - 5, targetY)
      ctx.lineTo(labelWidth + meterWidth + 5, targetY)
      ctx.stroke()
    }

    // Draw level meter (bottom to top)
    if (level > -60) {
      const levelHeight = dbToPixel(level, height - 20)
      const meterBottom = height - 10
      
      // Create gradient from bottom to current level
      const gradient = ctx.createLinearGradient(0, meterBottom, 0, meterBottom - levelHeight)
      
      // Add color stops based on dB ranges
      const segments = [
        { db: -60, color: METER_COLORS.GREEN },
        { db: -30, color: METER_COLORS.GREEN },
        { db: -18, color: METER_COLORS.YELLOW },
        { db: -6, color: METER_COLORS.ORANGE },
        { db: 0, color: METER_COLORS.RED }
      ]
      
      segments.forEach((segment) => {
        if (level >= segment.db) {
          const stop = Math.max(0, (60 + segment.db) / 60)
          gradient.addColorStop(stop, segment.color)
        }
      })
      
      ctx.fillStyle = gradient
      ctx.fillRect(labelWidth, meterBottom - levelHeight, meterWidth, levelHeight)
    }

    // Draw peak hold indicator
    if (peakHold > -60) {
      const peakY = dbToPixel(peakHold, height - 20) + 10
      ctx.fillStyle = METER_COLORS.HOLD_PEAK
      ctx.fillRect(labelWidth, peakY - 1, meterWidth, 2)
    }

    // Draw meter label
    ctx.fillStyle = METER_COLORS.TEXT
    ctx.font = 'bold 12px sans-serif'
    ctx.textAlign = 'left'
    ctx.fillText(label, 5, 20)

    // Draw current value
    ctx.font = '10px monospace'
    ctx.fillText(`${level.toFixed(1)}dB`, 5, height - 10)
  }

  // Draw meters when readings change
  useEffect(() => {
    if (!isVisible || !isMonitoring) return

    // VU Meter (with -18dBFS target)
    if (vuCanvasRef.current) {
      drawMeter(vuCanvasRef.current, readings.vu_db, readings.vu_db, 'VU', true)
    }

    // PPM Meter  
    if (ppmCanvasRef.current) {
      drawMeter(ppmCanvasRef.current, readings.ppm_db, readings.ppm_db, 'PPM', false)
    }

    // Digital Peak Meter (with hold)
    if (peakCanvasRef.current) {
      drawMeter(peakCanvasRef.current, readings.peak_db, readings.peak_hold_db, 'PEAK', false)
    }
  }, [readings, isVisible, isMonitoring])

  // Get gain staging color based on status
  const getGainStagingColor = (status: GainStagingStatus): string => {
    switch (status) {
      case GainStagingStatus.Optimal: return 'text-green-400'
      case GainStagingStatus.Acceptable: return 'text-blue-400'  
      case GainStagingStatus.TooQuiet: return 'text-yellow-400'
      case GainStagingStatus.TooLoud: return 'text-orange-400'
      case GainStagingStatus.Clipping: return 'text-red-400 animate-pulse'
      default: return 'text-gray-400'
    }
  }

  // Get recommendation urgency styling
  const getUrgencyStyle = (urgency: RecommendationUrgency): string => {
    switch (urgency) {
      case RecommendationUrgency.Critical: return 'text-red-400 animate-pulse font-bold'
      case RecommendationUrgency.High: return 'text-orange-400 font-semibold'
      case RecommendationUrgency.Medium: return 'text-yellow-400'
      case RecommendationUrgency.Low: return 'text-green-400'
      default: return 'text-gray-400'
    }
  }

  if (!isVisible) {
    return null
  }

  return (
    <div className={`flex items-start space-x-6 ${className}`}>
      {/* Professional Meters */}
      <div className="flex space-x-3">
        {/* VU Meter */}
        <div className="flex flex-col items-center">
          <canvas
            ref={vuCanvasRef}
            width={100}
            height={200}
            className="border border-gray-600 rounded bg-black"
          />
        </div>

        {/* PPM Meter */}
        <div className="flex flex-col items-center">
          <canvas
            ref={ppmCanvasRef}
            width={100}
            height={200}
            className="border border-gray-600 rounded bg-black"
          />
        </div>

        {/* Digital Peak Meter */}
        <div className="flex flex-col items-center">
          <canvas
            ref={peakCanvasRef}
            width={100}
            height={200}
            className="border border-gray-600 rounded bg-black"
          />
        </div>
      </div>

      {/* Professional Readouts */}
      <div className="flex flex-col space-y-3 min-w-[200px]">
        {/* Numeric Readings */}
        <div className="bg-gray-900 border border-gray-600 rounded p-3">
          <h3 className="text-sm font-semibold text-gray-200 mb-2">LEVELS</h3>
          <div className="grid grid-cols-2 gap-2 text-xs font-mono">
            <div>VU: <span className="text-green-400">{readings.vu_db.toFixed(1)}dB</span></div>
            <div>PPM: <span className="text-yellow-400">{readings.ppm_db.toFixed(1)}dB</span></div>
            <div>PEAK: <span className="text-orange-400">{readings.peak_db.toFixed(1)}dB</span></div>
            <div>HOLD: <span className="text-white">{readings.peak_hold_db.toFixed(1)}dB</span></div>
            <div className="col-span-2">LUFS: <span className="text-blue-400">{readings.lufs.toFixed(1)}</span></div>
          </div>
        </div>

        {/* Gain Staging Status */}
        <div className="bg-gray-900 border border-gray-600 rounded p-3">
          <h3 className="text-sm font-semibold text-gray-200 mb-2">GAIN STAGING</h3>
          <div className={`text-sm font-bold mb-1 ${getGainStagingColor(readings.gain_staging)}`}>
            {readings.gain_staging.toUpperCase()}
          </div>
          {analysis && (
            <>
              <div className="text-xs text-gray-400 mb-2">
                Target: {analysis.target_level_db.toFixed(1)}dB | Current: {analysis.current_level_db.toFixed(1)}dB
              </div>
              <div className={`text-xs ${getUrgencyStyle(analysis.recommendation.urgency)}`}>
                {analysis.recommendation.description}
              </div>
              {analysis.recommendation.adjustment_db !== 0 && (
                <div className="text-xs text-blue-300 mt-1">
                  Adjust: {analysis.recommendation.adjustment_db > 0 ? '+' : ''}{analysis.recommendation.adjustment_db.toFixed(1)}dB
                </div>
              )}
            </>
          )}
        </div>

        {/* Headroom Monitor */}
        {analysis && (
          <div className="bg-gray-900 border border-gray-600 rounded p-3">
            <h3 className="text-sm font-semibold text-gray-200 mb-2">HEADROOM</h3>
            <div className="text-xs">
              <div className={`font-semibold ${
                analysis.headroom_status === HeadroomStatus.SafeHeadroom ? 'text-green-400' :
                analysis.headroom_status === HeadroomStatus.LowHeadroom ? 'text-yellow-400' :
                'text-red-400 animate-pulse'
              }`}>
                {analysis.headroom_status === HeadroomStatus.SafeHeadroom && '✓ SAFE'}
                {analysis.headroom_status === HeadroomStatus.LowHeadroom && '⚠ LOW'}
                {analysis.headroom_status === HeadroomStatus.Clipping && '🔥 CLIPPING'}
              </div>
              <div className="text-gray-400 mt-1">
                Available: {analysis.headroom_db.toFixed(1)}dB
              </div>
            </div>
          </div>
        )}

        {/* Professional Target Info */}
        <div className="bg-gray-800 border border-gray-700 rounded p-2">
          <div className="text-xs text-gray-400">
            <div>• VU Target: -18dBFS (0 VU)</div>
            <div>• PPM Limit: -10dBFS (BBC)</div>
            <div>• Peak Limit: -6dBFS (Digital)</div>
          </div>
        </div>
      </div>
    </div>
  )
}