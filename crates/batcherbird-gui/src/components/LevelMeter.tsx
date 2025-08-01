import { useAudioMonitoring } from "@/hooks/useTauri"

interface LevelMeterProps {
  isVisible: boolean
  className?: string
}

export function LevelMeter({ isVisible, className = "" }: LevelMeterProps) {
  const { levels, isMonitoring } = useAudioMonitoring()

  if (!isVisible || !isMonitoring) {
    return null
  }

  // Convert dB to percentage for visual display (assuming -60dB to 0dB range)
  const peakPercent = Math.max(0, Math.min(100, ((levels.peak_db + 60) / 60) * 100))
  const rmsPercent = Math.max(0, Math.min(100, ((levels.rms_db + 60) / 60) * 100))

  // Professional color thresholds based on broadcast standards
  const getPeakColor = (db: number): string => {
    if (db > -10) return 'bg-red-500'      // Professional red zone (broadcast limit)
    if (db > -20) return 'bg-yellow-500'   // Professional caution zone  
    return 'bg-green-500'                  // Professional safe zone
  }

  const getRMSColor = (db: number): string => {
    if (db > -10) return 'bg-red-400'      // Hot RMS (danger zone)
    if (db > -20) return 'bg-yellow-400'   // Caution RMS
    return 'bg-green-400'                  // Safe RMS
  }

  // Professional gain staging analysis
  const getGainRecommendation = (): { message: string; color: string; urgency: 'low' | 'medium' | 'high' } => {
    const targetRMS = -18.0 // Professional standard
    const currentRMS = levels.rms_db
    
    if (levels.peak_db > -10) {
      return { message: 'REDUCE GAIN', color: 'text-red-400', urgency: 'high' }
    }
    
    if (currentRMS > -15) {
      return { message: 'SLIGHTLY HOT', color: 'text-yellow-400', urgency: 'medium' }
    }
    
    if (currentRMS < -25) {
      return { message: 'INCREASE GAIN', color: 'text-yellow-400', urgency: 'medium' }
    }
    
    if (Math.abs(currentRMS - targetRMS) < 3) {
      return { message: 'OPTIMAL', color: 'text-green-400', urgency: 'low' }
    }
    
    return { message: 'GOOD', color: 'text-green-400', urgency: 'low' }
  }

  return (
    <div className={`flex items-center space-x-3 ${className}`}>
      <span className="text-sm text-gray-300 font-mono min-w-[60px]">
        INPUT
      </span>
      
      {/* Professional Level Bars with Target Indicators */}
      <div className="flex flex-col space-y-1">
        {/* Peak level with professional markings */}
        <div className="flex items-center space-x-2">
          <span className="text-xs text-gray-400 w-8">PEAK</span>
          <div className="relative w-32 h-3 bg-gray-800 rounded-sm overflow-hidden">
            {/* Professional zone markings */}
            <div className="absolute inset-0 flex">
              <div className="flex-1 bg-green-900/30"></div> {/* Safe zone: -∞ to -20dB */}
              <div className="w-1/3 bg-yellow-900/30"></div>    {/* Caution: -20 to -10dB */}
              <div className="w-1/6 bg-red-900/30"></div>       {/* Danger: -10 to 0dB */}
            </div>
            
            {/* Broadcast limit marker at -10dBFS */}
            <div 
              className="absolute top-0 bottom-0 w-px bg-red-300/80"
              style={{ left: '83.33%' }} // -10dB position
            >
            </div>
            
            {/* Level bar */}
            <div 
              className={`h-full transition-all duration-75 ${getPeakColor(levels.peak_db)} relative z-10`}
              style={{ width: `${peakPercent}%` }}
            />
          </div>
          <span className="text-xs text-gray-300 font-mono w-12">
            {levels.peak_db.toFixed(1)}
          </span>
        </div>
        
        {/* RMS level with -18dBFS target indicator */}
        <div className="flex items-center space-x-2">
          <span className="text-xs text-gray-400 w-8">RMS</span>
          <div className="relative w-32 h-3 bg-gray-800 rounded-sm overflow-hidden">
            {/* Professional zone markings */}
            <div className="absolute inset-0 flex">
              <div className="flex-1 bg-green-900/30"></div> {/* Safe zone */}
              <div className="w-1/3 bg-yellow-900/30"></div>    {/* Caution */}
              <div className="w-1/6 bg-red-900/30"></div>       {/* Danger */}
            </div>
            
            {/* Professional -18dBFS target marker */}
            <div 
              className="absolute top-0 bottom-0 w-px bg-blue-300/80"
              style={{ left: '70%' }} // -18dB position
              title="Professional -18dBFS target"
            >
            </div>
            
            {/* Level bar */}
            <div 
              className={`h-full transition-all duration-75 ${getRMSColor(levels.rms_db)} relative z-10`}
              style={{ width: `${rmsPercent}%` }}
            />
          </div>
          <span className="text-xs text-gray-300 font-mono w-12">
            {levels.rms_db.toFixed(1)}
          </span>
        </div>
      </div>

      {/* Professional Status Indicators */}
      <div className="flex flex-col space-y-1 min-w-[90px]">
        {(() => {
          const recommendation = getGainRecommendation()
          return (
            <div className={`text-xs font-bold ${recommendation.color} ${
              recommendation.urgency === 'high' ? 'animate-pulse' : ''
            }`}>
              {recommendation.message}
            </div>
          )
        })()}
        
        {/* Accessibility text indicators */}
        <div className="text-xs text-gray-500">
          {levels.peak_db > -10 && '⚠️ BROADCAST LIMIT'}
          {levels.rms_db >= -21 && levels.rms_db <= -15 && '🎯 TARGET RANGE'}
          {levels.peak_db < -40 && '🔇 VERY QUIET'}
        </div>
        
        {/* Professional target info */}
        <div className="text-xs text-blue-300/60">
          Target: -18dB RMS
        </div>
      </div>
    </div>
  )
}