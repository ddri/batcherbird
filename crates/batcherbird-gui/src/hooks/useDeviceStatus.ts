import { useMemo } from "react"
import { useDeviceConnection, useAudioMonitoring } from "@/hooks/useTauri"
import { useRecordingState } from "@/hooks/useRecordingState"

export type AudioStatus = 'ready' | 'armed' | 'recording' | 'no-signal' | 'levels-high' | 'no-device'
export type MidiStatus = 'ready' | 'activity' | 'no-device'

export function useDeviceStatus(selectedAudioInput?: string, audioInputDevices?: string[]) {
  const { midiConnected, lastMidiActivity } = useDeviceConnection()
  const { levels, isMonitoring } = useAudioMonitoring()
  const { state: recordingState, isArmed } = useRecordingState()

  const audioStatus: AudioStatus = useMemo(() => {
    // No device selected
    if (!selectedAudioInput || !audioInputDevices || audioInputDevices.length === 0) {
      return 'no-device'
    }

    // Recording state takes priority
    if (recordingState === 'recording') {
      return 'recording'
    }

    // Armed state (monitoring levels)
    if (isArmed && isMonitoring) {
      const peakDb = levels.peak_db
      
      // Signal too high (clipping risk)
      if (peakDb > -3) {
        return 'levels-high'
      }
      
      // No signal detected (too quiet)
      if (peakDb < -50) {
        return 'no-signal'
      }
      
      // Good armed state with signal
      return 'armed'
    }
    
    // Device ready but not armed
    return 'ready'
  }, [selectedAudioInput, audioInputDevices?.length, recordingState, isArmed, isMonitoring, levels.peak_db])

  const midiStatus: MidiStatus = useMemo(() => {
    // No MIDI connection
    if (!midiConnected) {
      return 'no-device'
    }

    // Recent MIDI activity (within last 3 seconds)
    if (lastMidiActivity && (Date.now() - lastMidiActivity) < 3000) {
      return 'activity'
    }

    // Connected but no recent activity
    return 'ready'
  }, [midiConnected, lastMidiActivity])

  const getAudioStatusText = (): string => {
    switch (audioStatus) {
      case 'ready': return 'Audio Ready'
      case 'armed': return 'ARMED'
      case 'recording': return 'RECORDING'
      case 'no-signal': return 'No Signal'
      case 'levels-high': return 'Levels High'
      case 'no-device': return 'No Audio Device'
      default: return 'Audio Status'
    }
  }

  const getMidiStatusText = (): string => {
    switch (midiStatus) {
      case 'ready': return 'MIDI Ready'
      case 'activity': return 'MIDI Activity'
      case 'no-device': return 'No MIDI Device'
      default: return 'MIDI Status'
    }
  }

  const getAudioStatusColor = (): string => {
    switch (audioStatus) {
      case 'ready': return 'text-green-400'
      case 'armed': return 'text-yellow-400'
      case 'recording': return 'text-red-400'
      case 'no-signal': return 'text-yellow-400'
      case 'levels-high': return 'text-red-400'
      case 'no-device': return 'text-red-400'
      default: return 'text-gray-400'
    }
  }

  const getMidiStatusColor = (): string => {
    switch (midiStatus) {
      case 'ready': return 'text-green-400'
      case 'activity': return 'text-blue-400'
      case 'no-device': return 'text-red-400'
      default: return 'text-gray-400'
    }
  }

  const getAudioStatusIcon = (): string => {
    switch (audioStatus) {
      case 'ready': return '✓'
      case 'armed': return '●'
      case 'recording': return '●'
      case 'no-signal': return '⚠'
      case 'levels-high': return '⚠'
      case 'no-device': return '✗'
      default: return '?'
    }
  }

  const getMidiStatusIcon = (): string => {
    switch (midiStatus) {
      case 'ready': return '✓'
      case 'activity': return '●'
      case 'no-device': return '✗'
      default: return '?'
    }
  }

  return {
    audioStatus,
    midiStatus,
    getAudioStatusText,
    getMidiStatusText,
    getAudioStatusColor,
    getMidiStatusColor,
    getAudioStatusIcon,
    getMidiStatusIcon,
  }
}