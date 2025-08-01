import { useState, useCallback, useRef } from 'react'

export interface CountdownState {
  isCountingDown: boolean
  countdownValue: number
  totalDuration: number
}

export function useRecordingCountdown() {
  const [countdownState, setCountdownState] = useState<CountdownState>({
    isCountingDown: false,
    countdownValue: 0,
    totalDuration: 0
  })
  
  const countdownIntervalRef = useRef<number | null>(null)
  const resolveRef = useRef<(() => void) | null>(null)

  const startCountdown = useCallback((durationMs: number): Promise<void> => {
    return new Promise((resolve) => {
      resolveRef.current = resolve
      
      const steps = Math.ceil(durationMs / 100) // 10fps updates for smooth countdown
      let currentStep = steps
      
      setCountdownState({
        isCountingDown: true,
        countdownValue: durationMs,
        totalDuration: durationMs
      })
      
      countdownIntervalRef.current = window.setInterval(() => {
        currentStep--
        const remainingMs = Math.max(0, (currentStep * 100))
        
        setCountdownState(prev => ({
          ...prev,
          countdownValue: remainingMs
        }))
        
        if (remainingMs <= 0) {
          if (countdownIntervalRef.current) {
            clearInterval(countdownIntervalRef.current)
            countdownIntervalRef.current = null
          }
          
          setCountdownState({
            isCountingDown: false,
            countdownValue: 0,
            totalDuration: 0
          })
          
          if (resolveRef.current) {
            resolveRef.current()
            resolveRef.current = null
          }
        }
      }, 100)
    })
  }, [])

  const cancelCountdown = useCallback(() => {
    if (countdownIntervalRef.current) {
      clearInterval(countdownIntervalRef.current)
      countdownIntervalRef.current = null
    }
    
    setCountdownState({
      isCountingDown: false,
      countdownValue: 0,
      totalDuration: 0
    })
    
    if (resolveRef.current) {
      resolveRef.current()
      resolveRef.current = null
    }
  }, [])

  const getCountdownDisplay = useCallback((): string => {
    if (!countdownState.isCountingDown) return ''
    
    const seconds = Math.ceil(countdownState.countdownValue / 1000)
    if (seconds > 0) {
      return `${seconds}`
    } else {
      return 'GO!'
    }
  }, [countdownState])

  const getCountdownProgress = useCallback((): number => {
    if (!countdownState.isCountingDown || countdownState.totalDuration === 0) return 0
    
    const progress = ((countdownState.totalDuration - countdownState.countdownValue) / countdownState.totalDuration) * 100
    return Math.min(100, Math.max(0, progress))
  }, [countdownState])

  // Cleanup on unmount
  const cleanup = useCallback(() => {
    if (countdownIntervalRef.current) {
      clearInterval(countdownIntervalRef.current)
    }
  }, [])

  return {
    countdownState,
    startCountdown,
    cancelCountdown,
    getCountdownDisplay,
    getCountdownProgress,
    cleanup
  }
}