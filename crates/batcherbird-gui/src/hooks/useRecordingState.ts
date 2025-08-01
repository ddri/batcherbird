import { useState, useCallback } from "react"
import { useAudioMonitoring } from "@/hooks/useTauri"

export type RecordingState = 'idle' | 'armed' | 'counting-down' | 'recording' | 'preview'

export function useRecordingState() {
  const [state, setState] = useState<RecordingState>('idle')
  const { startMonitoring, stopMonitoring, isMonitoring } = useAudioMonitoring()

  const arm = useCallback(async () => {
    try {
      setState('armed')
      await startMonitoring()
      console.log('Recording armed - audio monitoring started')
    } catch (error) {
      console.error('Failed to arm recording:', error)
      setState('idle')
      throw error
    }
  }, [startMonitoring])

  const disarm = useCallback(async () => {
    try {
      setState('idle')
      await stopMonitoring()
      console.log('Recording disarmed - audio monitoring stopped')
    } catch (error) {
      console.error('Failed to disarm recording:', error)
      // Even if stopping fails, we should go back to idle
      setState('idle')
    }
  }, [stopMonitoring])

  const startCountdown = useCallback(() => {
    if (state !== 'armed') {
      throw new Error('Cannot start countdown - not armed')
    }
    setState('counting-down')
    console.log('Recording countdown started')
  }, [state])

  const startRecording = useCallback(() => {
    if (state !== 'counting-down' && state !== 'armed') {
      throw new Error('Cannot start recording - not in countdown or armed state')
    }
    setState('recording')
    console.log('Recording started')
  }, [state])

  const stopRecording = useCallback(async () => {
    if (state !== 'recording') {
      console.warn('Stop recording called but not in recording state')
      return
    }
    // Go back to armed state after recording
    setState('armed')
    console.log('Recording stopped - returning to armed state')
  }, [state])

  const startPreview = useCallback(async () => {
    try {
      setState('preview')
      await startMonitoring(true) // Enable playthrough for preview
      console.log('Preview mode - audio monitoring with playthrough started')
    } catch (error) {
      console.error('Failed to start preview:', error)
      setState('idle')
      throw error
    }
  }, [startMonitoring])

  const stopPreview = useCallback(async () => {
    try {
      setState('idle')
      await stopMonitoring()
      console.log('Preview stopped - audio monitoring stopped')
    } catch (error) {
      console.error('Failed to stop preview:', error)
      setState('idle')
    }
  }, [stopMonitoring])

  const cancelCountdown = useCallback(() => {
    if (state !== 'counting-down') {
      return
    }
    setState('armed')
    console.log('Recording countdown cancelled - returning to armed state')
  }, [state])

  // Computed states
  const canRecord = state === 'armed'
  const canArm = state === 'idle'
  const canDisarm = state === 'armed' || state === 'preview'
  const isArmed = state === 'armed' || state === 'preview'
  const isRecording = state === 'recording'
  const isCountingDown = state === 'counting-down'
  const showLevelMeter = (isArmed || isCountingDown) && isMonitoring

  const getStateDisplayText = (): string => {
    switch (state) {
      case 'idle': return 'Ready'
      case 'armed': return 'ARMED'
      case 'counting-down': return 'COUNTING IN'
      case 'recording': return 'RECORDING'
      case 'preview': return 'PREVIEW'
      default: return 'Unknown'
    }
  }

  const getStateColor = (): string => {
    switch (state) {
      case 'idle': return 'text-gray-400'
      case 'armed': return 'text-yellow-400'
      case 'counting-down': return 'text-orange-400'
      case 'recording': return 'text-red-400'
      case 'preview': return 'text-blue-400'
      default: return 'text-gray-400'
    }
  }

  return {
    // State
    state,
    canRecord,
    canArm,
    canDisarm,
    isArmed,
    isRecording,
    isCountingDown,
    showLevelMeter,

    // Actions
    arm,
    disarm,
    startCountdown,
    startRecording,
    stopRecording,
    cancelCountdown,
    startPreview,
    stopPreview,

    // Display helpers
    getStateDisplayText,
    getStateColor,
  }
}