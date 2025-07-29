import { useRef, useEffect, useState, useCallback } from 'react'
import { WaveformData } from '@/hooks/useTauri'
import { Button } from '@/components/ui/button'
import { Play, ZoomIn, ZoomOut, RotateCcw } from 'lucide-react'

interface WaveformDisplayProps {
  waveformData: WaveformData | null
  isLoading?: boolean
  error?: string | null
  onSeek?: (position: number) => void
  playbackPosition?: number
  fileName?: string
  duration?: string
}

export function WaveformDisplay({
  waveformData,
  isLoading,
  error,
  onSeek,
  playbackPosition = 0,
  fileName,
  duration
}: WaveformDisplayProps) {
  const canvasRef = useRef<HTMLCanvasElement>(null)
  const [zoomLevel, setZoomLevel] = useState(1)
  const [scrollPosition, setScrollPosition] = useState(0)
  const [isPlaying, setIsPlaying] = useState(false)

  // Draw waveform on canvas
  useEffect(() => {
    const canvas = canvasRef.current
    if (!canvas || !waveformData) return

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
  }, [waveformData, zoomLevel, scrollPosition, playbackPosition])

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
    setIsPlaying(!isPlaying)
    // TODO: Implement actual playback control
  }

  if (error) {
    return (
      <div className="bg-gray-950 rounded-lg p-4 h-48 flex items-center justify-center">
        <p className="text-red-400">Error loading waveform: {error}</p>
      </div>
    )
  }

  if (isLoading) {
    return (
      <div className="bg-gray-950 rounded-lg p-4 h-48 flex items-center justify-center">
        <div className="flex items-center space-x-2">
          <div className="w-2 h-2 bg-blue-500 rounded-full animate-pulse"></div>
          <span className="text-gray-400">Loading waveform...</span>
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
          <Play className="w-4 h-4 mr-2" />
          {isPlaying ? 'Pause' : 'Play'}
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