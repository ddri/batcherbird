import { useRef, useEffect, useState, useCallback } from 'react'
import { WaveformData, VizChunk } from '@/hooks/useTauri'
import { Button } from '@/components/ui/button'
import { Play, Pause, ZoomIn, ZoomOut, RotateCcw } from 'lucide-react'

interface WaveformDisplayProps {
  waveformData: WaveformData | null
  isLoading?: boolean
  isTransitioning?: boolean
  error?: string | null
  onSeek?: (position: number) => void
  playbackPosition?: number
  fileName?: string
  duration?: string
  isPlaying?: boolean
  onPlayPause?: () => void
  // Real-time recording props
  realtimeVizChunks?: VizChunk[] | null
  isRecording?: boolean
}

export function WaveformDisplay({
  waveformData,
  isLoading,
  isTransitioning,
  error,
  onSeek,
  playbackPosition = 0,
  fileName,
  duration,
  isPlaying = false,
  onPlayPause,
  realtimeVizChunks,
  isRecording = false
}: WaveformDisplayProps) {
  const canvasRef = useRef<HTMLCanvasElement>(null)
  const [zoomLevel, setZoomLevel] = useState(1)
  const [scrollPosition, setScrollPosition] = useState(0)

  // Draw waveform on canvas with proper state prioritization
  useEffect(() => {
    console.log('🌊 WaveformDisplay: Drawing waveform', { 
      hasWaveformData: !!waveformData, 
      isRecording,
      isTransitioning,
      isLoading,
      fileName,
      peaksLength: waveformData?.peaks.positive.length,
      waveformDataSummary: waveformData ? {
        duration: waveformData.duration,
        sampleRate: waveformData.sample_rate,
        channels: waveformData.channels,
        format: waveformData.format
      } : null
    })
    
    console.log('🔍 WaveformDisplay: State check', {
      isTransitioning,
      hasWaveformData: !!waveformData,
      isRecording,
      canvasExists: !!canvasRef.current
    })
    
    const canvas = canvasRef.current
    if (!canvas) return
    
    // Priority order: Transitioning > File Data > Recording
    if (isTransitioning) {
      // During transition, don't draw anything - let loading state handle it
      return
    }
    
    if (!waveformData) {
      // Only skip if actively recording AND no waveform data
      if (isRecording) return
      return
    }

    const ctx = canvas.getContext('2d')
    if (!ctx) return

    // Set canvas size
    const rect = canvas.getBoundingClientRect()
    canvas.width = rect.width * window.devicePixelRatio
    canvas.height = rect.height * window.devicePixelRatio
    ctx.scale(window.devicePixelRatio, window.devicePixelRatio)

    // Clear canvas
    ctx.clearRect(0, 0, rect.width, rect.height)

    // Calculate drawing parameters
    const { peaks } = waveformData
    const numPeaks = peaks.positive.length
    const peakWidth = (rect.width * zoomLevel) / numPeaks
    const centerY = rect.height / 2
    
    console.log('🎨 WaveformDisplay: Drawing parameters', {
      numPeaks,
      peakWidth,
      centerY,
      canvasWidth: rect.width,
      canvasHeight: rect.height,
      firstFewPositivePeaks: peaks.positive.slice(0, 5),
      firstFewNegativePeaks: peaks.negative.slice(0, 5)
    })

    // Set styles
    ctx.strokeStyle = '#d1d5db' // gray-300
    ctx.lineWidth = 1

    // Draw waveform
    ctx.beginPath()
    
    // Draw positive peaks
    for (let i = 0; i < numPeaks; i++) {
      const x = i * peakWidth - scrollPosition
      
      // Skip if outside visible area
      if (x + peakWidth < 0 || x > rect.width) continue
      
      const positiveHeight = peaks.positive[i] * centerY * 0.9
      const negativeHeight = peaks.negative[i] * centerY * 0.9
      
      // Draw vertical line from negative to positive peak
      ctx.moveTo(x, centerY - positiveHeight)
      ctx.lineTo(x, centerY + negativeHeight)
    }
    
    ctx.stroke()

    // Draw center line
    ctx.strokeStyle = '#6b7280' // gray-500
    ctx.lineWidth = 0.5
    ctx.beginPath()
    ctx.moveTo(0, centerY)
    ctx.lineTo(rect.width, centerY)
    ctx.stroke()

    // Draw playback position
    if (playbackPosition > 0 && playbackPosition <= 1) {
      const playheadX = playbackPosition * rect.width * zoomLevel - scrollPosition
      if (playheadX >= 0 && playheadX <= rect.width) {
        ctx.strokeStyle = '#3b82f6' // blue-500
        ctx.lineWidth = 2
        ctx.beginPath()
        ctx.moveTo(playheadX, 0)
        ctx.lineTo(playheadX, rect.height)
        ctx.stroke()
      }
    }
  }, [waveformData, zoomLevel, scrollPosition, playbackPosition, isRecording, isTransitioning])

  // Real-time waveform drawing during recording using VizChunk data
  useEffect(() => {
    // Only log when starting/stopping recording to avoid 60fps spam
    if (isRecording && realtimeVizChunks?.length === 1) {
      console.log('🎨 WaveformDisplay: Starting real-time visualization')
    } else if (!isRecording && realtimeVizChunks?.length === 0) {
      console.log('🎨 WaveformDisplay: Stopping real-time visualization')  
    }
    
    // Don't draw real-time during transition or if not recording
    if (isTransitioning || !isRecording || !realtimeVizChunks || realtimeVizChunks.length === 0) return
    
    const canvas = canvasRef.current
    if (!canvas) return
    
    const ctx = canvas.getContext('2d')
    if (!ctx) return

    // Set canvas size
    const rect = canvas.getBoundingClientRect()
    canvas.width = rect.width * window.devicePixelRatio
    canvas.height = rect.height * window.devicePixelRatio
    ctx.scale(window.devicePixelRatio, window.devicePixelRatio)

    // Clear canvas
    ctx.clearRect(0, 0, rect.width, rect.height)

    // Draw center line
    const centerY = rect.height / 2
    ctx.strokeStyle = '#6b7280' // gray-500
    ctx.lineWidth = 0.5
    ctx.beginPath()
    ctx.moveTo(0, centerY)
    ctx.lineTo(rect.width, centerY)
    ctx.stroke()

    // Draw real-time waveform from VizChunk peak data with smooth connected lines
    const numChunks = realtimeVizChunks.length
    const chunkWidth = rect.width / Math.max(numChunks, 1)

    if (numChunks > 0) {
      // Draw positive peaks as connected line
      ctx.strokeStyle = '#22c55e' // green-500 for recording
      ctx.lineWidth = 2
      ctx.beginPath()
      
      for (let i = 0; i < numChunks; i++) {
        const chunk = realtimeVizChunks[i]
        const x = i * chunkWidth
        const peakHeight = chunk.peak * centerY * 0.9
        const y = centerY - peakHeight
        
        if (i === 0) {
          ctx.moveTo(x, y)
        } else {
          ctx.lineTo(x, y)
        }
      }
      ctx.stroke()
      
      // Draw negative peaks as connected line
      ctx.beginPath()
      for (let i = 0; i < numChunks; i++) {
        const chunk = realtimeVizChunks[i]
        const x = i * chunkWidth
        const peakHeight = chunk.peak * centerY * 0.9
        const y = centerY + peakHeight
        
        if (i === 0) {
          ctx.moveTo(x, y)
        } else {
          ctx.lineTo(x, y)
        }
      }
      ctx.stroke()
    }

    // Add polished recording indicator with pulsing effect
    const time = Date.now() / 1000
    const pulse = 0.8 + 0.2 * Math.sin(time * 4) // Pulsing between 0.8 and 1.0
    
    ctx.fillStyle = `rgba(239, 68, 68, ${pulse})` // red-500 with pulsing alpha
    ctx.beginPath()
    ctx.arc(20, 20, 8, 0, 2 * Math.PI)
    ctx.fill()

    // Add "RECORDING" text with shadow for better visibility
    ctx.shadowColor = 'rgba(0, 0, 0, 0.5)'
    ctx.shadowOffsetX = 1
    ctx.shadowOffsetY = 1
    ctx.shadowBlur = 2
    ctx.fillStyle = '#ef4444'
    ctx.font = 'bold 14px sans-serif'
    ctx.fillText('RECORDING', 35, 25)
    
    // Reset shadow
    ctx.shadowColor = 'transparent'
    ctx.shadowOffsetX = 0
    ctx.shadowOffsetY = 0
    ctx.shadowBlur = 0

    // Add enhanced live peak level indicator
    if (numChunks > 0) {
      const latestChunk = realtimeVizChunks[numChunks - 1]
      
      // Background for better readability
      ctx.fillStyle = 'rgba(0, 0, 0, 0.7)'
      ctx.fillRect(rect.width - 110, 10, 100, 40)
      
      // Peak and RMS display
      ctx.fillStyle = '#22c55e'
      ctx.font = 'bold 12px monospace'
      ctx.fillText(`Peak: ${(latestChunk.peak * 100).toFixed(1)}%`, rect.width - 105, 28)
      
      // Color-code RMS based on level
      const rmsPercent = latestChunk.rms * 100
      if (rmsPercent > 70) {
        ctx.fillStyle = '#ef4444' // red for high levels
      } else if (rmsPercent > 30) {
        ctx.fillStyle = '#f59e0b' // yellow for medium levels
      } else {
        ctx.fillStyle = '#22c55e' // green for low levels
      }
      ctx.fillText(`RMS:  ${rmsPercent.toFixed(1)}%`, rect.width - 105, 43)
    }

  }, [realtimeVizChunks, isRecording, isTransitioning])

  // Handle canvas click for seeking
  const handleCanvasClick = useCallback((e: React.MouseEvent<HTMLCanvasElement>) => {
    if (!onSeek || !canvasRef.current) return
    
    const rect = canvasRef.current.getBoundingClientRect()
    const x = e.clientX - rect.left
    const position = (x + scrollPosition) / (rect.width * zoomLevel)
    
    if (position >= 0 && position <= 1) {
      onSeek(position)
    }
  }, [onSeek, scrollPosition, zoomLevel])

  const handleZoomIn = () => {
    setZoomLevel(prev => Math.min(prev * 1.5, 10))
  }

  const handleZoomOut = () => {
    setZoomLevel(prev => Math.max(prev / 1.5, 1))
    setScrollPosition(0)
  }

  const handleReset = () => {
    setZoomLevel(1)
    setScrollPosition(0)
  }

  const handlePlay = () => {
    if (onPlayPause) {
      onPlayPause()
    }
  }

  if (error) {
    return (
      <div className="bg-gray-950 rounded-lg p-4 h-48 flex items-center justify-center">
        <p className="text-red-400">Error loading waveform: {error}</p>
      </div>
    )
  }

  if (isLoading || isTransitioning) {
    return (
      <div className="bg-gray-950 rounded-lg p-4 h-48 flex items-center justify-center">
        <div className="flex items-center space-x-2">
          <div className="w-2 h-2 bg-blue-500 rounded-full animate-pulse"></div>
          <span className="text-gray-400">
            {isTransitioning ? 'Switching to playback...' : 'Loading waveform...'}
          </span>
        </div>
      </div>
    )
  }

  if (!waveformData) {
    // Show placeholder waveform
    return (
      <div className="bg-gray-950 rounded-lg p-4 h-48 flex items-center justify-center">
        <svg width="100%" height="100%" viewBox="0 0 800 150" className="text-gray-300">
          <path
            d="M0,75 Q50,25 100,75 T200,75 Q250,125 300,75 T400,75 Q450,25 500,75 T600,75 Q650,125 700,75 T800,75"
            stroke="#d1d5db"
            strokeWidth="2"
            fill="none"
          />
          <path
            d="M0,75 Q50,125 100,75 T200,75 Q250,25 300,75 T400,75 Q450,125 500,75 T600,75 Q650,25 700,75 T800,75"
            stroke="#d1d5db"
            strokeWidth="2"
            fill="none"
            opacity="0.6"
          />
        </svg>
      </div>
    )
  }

  return (
    <>
      <div className="bg-gray-950 rounded-lg p-4 h-48 relative overflow-hidden">
        <canvas
          ref={canvasRef}
          className="w-full h-full cursor-pointer"
          onClick={handleCanvasClick}
          style={{ imageRendering: 'pixelated' }}
        />
        {fileName && (
          <div className="absolute top-2 right-2 text-xs text-gray-400">
            {fileName}
          </div>
        )}
        {duration && (
          <div className="absolute top-2 left-2 text-xs text-gray-400">
            Duration: {duration}
          </div>
        )}
      </div>
      <div className="flex items-center justify-center space-x-2 mt-4">
        <Button
          onClick={handlePlay}
          variant="outline"
          size="sm"
          className="border-gray-600 text-gray-100 hover:bg-gray-800 bg-transparent"
        >
          {isPlaying ? (
            <>
              <Pause className="w-4 h-4 mr-2" />
              Pause
            </>
          ) : (
            <>
              <Play className="w-4 h-4 mr-2" />
              Play
            </>
          )}
        </Button>
        <Button
          onClick={handleZoomIn}
          variant="outline"
          size="sm"
          className="border-gray-600 text-gray-100 hover:bg-gray-800 bg-transparent"
          disabled={zoomLevel >= 10}
        >
          <ZoomIn className="w-4 h-4" />
        </Button>
        <Button
          onClick={handleZoomOut}
          variant="outline"
          size="sm"
          className="border-gray-600 text-gray-100 hover:bg-gray-800 bg-transparent"
          disabled={zoomLevel <= 1}
        >
          <ZoomOut className="w-4 h-4" />
        </Button>
        <Button
          onClick={handleReset}
          variant="outline"
          size="sm"
          className="border-gray-600 text-gray-100 hover:bg-gray-800 bg-transparent"
        >
          <RotateCcw className="w-4 h-4" />
        </Button>
      </div>
    </>
  )
}