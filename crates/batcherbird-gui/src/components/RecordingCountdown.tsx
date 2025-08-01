import { useEffect } from "react"

interface RecordingCountdownProps {
  isCountingDown: boolean
  countdownValue: number
  totalDuration: number
  onComplete?: () => void
}

export function RecordingCountdown({ 
  isCountingDown, 
  countdownValue, 
  totalDuration,
  onComplete 
}: RecordingCountdownProps) {
  
  const getCountdownDisplay = (): string => {
    if (!isCountingDown) return ''
    
    const seconds = Math.ceil(countdownValue / 1000)
    if (seconds > 0) {
      return `${seconds}`
    } else {
      return 'GO!'
    }
  }

  const getProgress = (): number => {
    if (!isCountingDown || totalDuration === 0) return 0
    
    return ((totalDuration - countdownValue) / totalDuration) * 100
  }

  const isGoTime = countdownValue <= 500 && isCountingDown

  useEffect(() => {
    if (countdownValue <= 0 && isCountingDown && onComplete) {
      onComplete()
    }
  }, [countdownValue, isCountingDown, onComplete])

  if (!isCountingDown) {
    return null
  }

  return (
    <div className="fixed inset-0 bg-black bg-opacity-75 flex items-center justify-center z-50">
      <div className="flex flex-col items-center space-y-6">
        {/* Circular Progress - Custom SVG */}
        <div className="w-32 h-32 relative">
          <svg className="w-full h-full transform -rotate-90" viewBox="0 0 120 120">
            {/* Background circle */}
            <circle
              cx="60"
              cy="60" 
              r="54"
              stroke="#374151"
              strokeWidth="8"
              fill="none"
            />
            {/* Progress circle */}
            <circle
              cx="60"
              cy="60"
              r="54"
              stroke={isGoTime ? '#ef4444' : '#f59e0b'}
              strokeWidth="8"
              fill="none"
              strokeLinecap="round"
              strokeDasharray={`${2 * Math.PI * 54}`}
              strokeDashoffset={`${2 * Math.PI * 54 * (1 - getProgress() / 100)}`}
              className="transition-all duration-100"
            />
          </svg>
        </div>
        
        {/* Countdown Display */}
        <div className={`text-8xl font-bold transition-all duration-150 ${
          isGoTime 
            ? 'text-red-400 animate-pulse scale-110' 
            : 'text-amber-400'
        }`}>
          {getCountdownDisplay()}
        </div>
        
        {/* Recording Indicator */}
        <div className="flex items-center space-x-3">
          <div className={`w-4 h-4 rounded-full ${
            isGoTime ? 'bg-red-500 animate-pulse' : 'bg-amber-500'
          }`}></div>
          <span className="text-xl text-gray-300 font-medium">
            {isGoTime ? 'Recording Starting...' : 'Get Ready'}
          </span>
        </div>
        
        {/* Cancel instruction */}
        <div className="text-sm text-gray-500 mt-4">
          Press ESC to cancel
        </div>
      </div>
    </div>
  )
}