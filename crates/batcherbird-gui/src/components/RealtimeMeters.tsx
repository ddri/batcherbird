import { useEffect, useState, useRef } from 'react'
import { listen } from '@tauri-apps/api/event'
import { invoke } from '@tauri-apps/api/core'

interface MeterData {
  peak_left: number
  peak_right: number
  rms_left: number
  rms_right: number
  is_clipping: boolean
  timestamp: number
}

interface RealtimeMetersProps {
  isActive: boolean
  className?: string
}

// Professional meter color scheme
const METER_COLORS = {
  BACKGROUND: '#1a1a1a',
  GREEN: '#00ff00',
  YELLOW: '#ffff00',
  ORANGE: '#ff8000',
  RED: '#ff0000',
  CLIP: '#ff0000',
  GRID: '#333333',
  TEXT: '#cccccc'
}

// dB scale points for grid
const DB_MARKS = [0, -6, -12, -18, -24, -30, -36, -42, -48, -54, -60]

export function RealtimeMeters({ isActive, className = '' }: RealtimeMetersProps) {
  const [meterData, setMeterData] = useState<MeterData>({
    peak_left: -60,
    peak_right: -60,
    rms_left: -60,
    rms_right: -60,
    is_clipping: false,
    timestamp: 0
  })
  
  const canvasRef = useRef<HTMLCanvasElement>(null)
  const unlistenRef = useRef<(() => void) | null>(null)
  const animationFrameRef = useRef<number | null>(null)
  
  // Peak hold values for display
  const peakHoldRef = useRef({ left: -60, right: -60 })
  const peakHoldTimeRef = useRef({ left: 0, right: 0 })
  
  // Start/stop meter streaming based on isActive
  useEffect(() => {
    const setupMeterStream = async () => {
      if (isActive) {
        // Start the real-time meter stream
        try {
          await invoke('start_realtime_meter_stream')
          
          // Listen for meter updates via Tauri channel
          unlistenRef.current = await listen<MeterData>('meter_update', (event) => {
            setMeterData(event.payload)
            
            // Update peak hold values
            if (event.payload.peak_left > peakHoldRef.current.left) {
              peakHoldRef.current.left = event.payload.peak_left
              peakHoldTimeRef.current.left = Date.now()
            }
            if (event.payload.peak_right > peakHoldRef.current.right) {
              peakHoldRef.current.right = event.payload.peak_right
              peakHoldTimeRef.current.right = Date.now()
            }
          })
        } catch (error) {
          // Failed to start meter stream
        }
      } else {
        // Stop listening
        if (unlistenRef.current) {
          unlistenRef.current()
          unlistenRef.current = null
        }
      }
    }
    
    setupMeterStream()
    
    return () => {
      if (unlistenRef.current) {
        unlistenRef.current()
        unlistenRef.current = null
      }
    }
  }, [isActive])
  
  // Draw meters on canvas
  useEffect(() => {
    const drawMeters = () => {
      const canvas = canvasRef.current
      if (!canvas) return
      
      const ctx = canvas.getContext('2d')
      if (!ctx) return
      
      const width = canvas.width
      const height = canvas.height
      const meterWidth = 40
      const meterSpacing = 20
      const leftMeterX = 50
      const rightMeterX = leftMeterX + meterWidth + meterSpacing
      const meterTop = 20
      const meterBottom = height - 40
      const meterHeight = meterBottom - meterTop
      
      // Clear canvas
      ctx.fillStyle = '#000000'
      ctx.fillRect(0, 0, width, height)
      
      // Draw background
      ctx.fillStyle = METER_COLORS.BACKGROUND
      ctx.fillRect(leftMeterX, meterTop, meterWidth, meterHeight)
      ctx.fillRect(rightMeterX, meterTop, meterWidth, meterHeight)
      
      // Draw dB scale
      ctx.strokeStyle = METER_COLORS.GRID
      ctx.fillStyle = METER_COLORS.TEXT
      ctx.font = '10px monospace'
      ctx.textAlign = 'right'
      
      DB_MARKS.forEach(db => {
        const y = meterTop + ((60 + db) / 60) * meterHeight
        
        // Grid lines
        ctx.beginPath()
        ctx.moveTo(leftMeterX - 5, y)
        ctx.lineTo(rightMeterX + meterWidth + 5, y)
        ctx.stroke()
        
        // dB labels
        ctx.fillText(`${db}`, leftMeterX - 10, y + 3)
      })
      
      // Helper function to convert dB to pixel position
      const dbToY = (db: number) => {
        const clampedDb = Math.max(-60, Math.min(0, db))
        return meterTop + ((60 + clampedDb) / 60) * meterHeight
      }
      
      // Helper function to get color for dB level
      const getColorForDb = (db: number) => {
        if (db > -6) return METER_COLORS.RED
        if (db > -18) return METER_COLORS.ORANGE
        if (db > -30) return METER_COLORS.YELLOW
        return METER_COLORS.GREEN
      }
      
      // Draw left channel meters
      const drawMeter = (x: number, peak: number, rms: number, peakHold: number) => {
        // RMS bar (wider, darker)
        if (rms > -60) {
          const rmsY = dbToY(rms)
          const rmsHeight = meterBottom - rmsY
          
          // Create gradient for RMS
          const rmsGradient = ctx.createLinearGradient(0, meterBottom, 0, rmsY)
          rmsGradient.addColorStop(0, METER_COLORS.GREEN)
          if (rms > -30) rmsGradient.addColorStop(0.5, METER_COLORS.YELLOW)
          if (rms > -18) rmsGradient.addColorStop(0.7, METER_COLORS.ORANGE)
          if (rms > -6) rmsGradient.addColorStop(0.9, METER_COLORS.RED)
          
          ctx.fillStyle = rmsGradient
          ctx.globalAlpha = 0.7
          ctx.fillRect(x, rmsY, meterWidth, rmsHeight)
          ctx.globalAlpha = 1.0
        }
        
        // Peak line (bright, thin)
        if (peak > -60) {
          const peakY = dbToY(peak)
          ctx.strokeStyle = getColorForDb(peak)
          ctx.lineWidth = 2
          ctx.beginPath()
          ctx.moveTo(x, peakY)
          ctx.lineTo(x + meterWidth, peakY)
          ctx.stroke()
        }
        
        // Peak hold indicator
        if (peakHold > -60) {
          const holdY = dbToY(peakHold)
          ctx.fillStyle = '#ffffff'
          ctx.fillRect(x, holdY - 1, meterWidth, 2)
        }
      }
      
      // Decay peak hold values (3 second hold, then decay)
      const now = Date.now()
      if (now - peakHoldTimeRef.current.left > 3000) {
        peakHoldRef.current.left = Math.max(peakHoldRef.current.left - 0.5, -60)
      }
      if (now - peakHoldTimeRef.current.right > 3000) {
        peakHoldRef.current.right = Math.max(peakHoldRef.current.right - 0.5, -60)
      }
      
      // Draw meters
      drawMeter(leftMeterX, meterData.peak_left, meterData.rms_left, peakHoldRef.current.left)
      drawMeter(rightMeterX, meterData.peak_right, meterData.rms_right, peakHoldRef.current.right)
      
      // Draw channel labels
      ctx.fillStyle = METER_COLORS.TEXT
      ctx.font = 'bold 12px sans-serif'
      ctx.textAlign = 'center'
      ctx.fillText('L', leftMeterX + meterWidth/2, meterTop - 5)
      ctx.fillText('R', rightMeterX + meterWidth/2, meterTop - 5)
      
      // Draw clipping indicator
      if (meterData.is_clipping) {
        ctx.fillStyle = METER_COLORS.CLIP
        ctx.font = 'bold 14px sans-serif'
        ctx.textAlign = 'center'
        ctx.fillText('CLIP', width/2, height - 10)
      }
      
      // Draw current values
      ctx.fillStyle = METER_COLORS.TEXT
      ctx.font = '10px monospace'
      ctx.textAlign = 'left'
      ctx.fillText(`L: ${meterData.peak_left.toFixed(1)}dB`, 10, height - 20)
      ctx.fillText(`R: ${meterData.peak_right.toFixed(1)}dB`, 10, height - 10)
      
      // Continue animation
      animationFrameRef.current = requestAnimationFrame(drawMeters)
    }
    
    // Start animation loop
    if (isActive) {
      drawMeters()
    }
    
    return () => {
      if (animationFrameRef.current) {
        cancelAnimationFrame(animationFrameRef.current)
        animationFrameRef.current = null
      }
    }
  }, [isActive, meterData])
  
  return (
    <div className={`realtime-meters ${className}`}>
      <canvas
        ref={canvasRef}
        width={200}
        height={300}
        className="meter-canvas"
        style={{
          backgroundColor: '#000',
          border: '1px solid #333',
          borderRadius: '4px'
        }}
      />
    </div>
  )
}