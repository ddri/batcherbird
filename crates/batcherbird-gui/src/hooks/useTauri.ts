import { invoke } from '@tauri-apps/api/core'
import { useState, useEffect, useCallback, useRef } from 'react'

// Types matching our Rust backend
export interface AudioLevels {
  peak: number
  rms: number
  peak_db: number
  rms_db: number
}

export interface LoopCandidate {
  start_sample: number
  end_sample: number
  length_samples: number
  quality_score: number
  zero_crossing_aligned: boolean
  correlation: number
}

export interface LoopDetectionResponse {
  success: boolean
  sample_rate: number
  candidates: LoopCandidate[]
  best_candidate: LoopCandidate | null
  failure_reason: string | null
}

export interface WaveformPeaks {
  positive: number[]
  negative: number[]
}

export interface WaveformData {
  peaks: WaveformPeaks
  sample_rate: number
  duration: number
  channels: number
  format: 'mono' | 'stereo'
}

// Device Management Hooks
export function useMidiDevices() {
  const [devices, setDevices] = useState<string[]>([])
  const [isLoading, setIsLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)

  const loadDevices = useCallback(async () => {
    setIsLoading(true)
    setError(null)
    try {
      const result = await invoke<string[]>('list_midi_devices')
      console.log('MIDI devices loaded:', result)
      setDevices(result || [])
    } catch (err) {
      setError(err as string)
      console.error('Failed to load MIDI devices:', err)
      console.error('Full error object:', err)
      console.error('Error stack:', (err as any)?.stack)
    } finally {
      setIsLoading(false)
    }
  }, [])

  return { devices, isLoading, error, loadDevices }
}

export function useAudioInputDevices() {
  const [devices, setDevices] = useState<string[]>([])
  const [isLoading, setIsLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)

  const loadDevices = useCallback(async () => {
    setIsLoading(true)
    setError(null)
    try {
      const result = await invoke<string[]>('list_audio_input_devices')
      console.log('Audio input devices loaded:', result)
      setDevices(result || [])
    } catch (err) {
      setError(err as string)
      console.error('Failed to load audio input devices:', err)
    } finally {
      setIsLoading(false)
    }
  }, [])

  return { devices, isLoading, error, loadDevices }
}

export function useAudioOutputDevices() {
  const [devices, setDevices] = useState<string[]>([])
  const [isLoading, setIsLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)

  const loadDevices = useCallback(async () => {
    setIsLoading(true)
    setError(null)
    try {
      const result = await invoke<string[]>('list_audio_output_devices')
      console.log('Audio output devices loaded:', result)
      setDevices(result || [])
    } catch (err) {
      setError(err as string)
      console.error('Failed to load audio output devices:', err)
    } finally {
      setIsLoading(false)
    }
  }, [])

  return { devices, isLoading, error, loadDevices }
}

// Device Connection
export function useDeviceConnection() {
  const [midiConnected, setMidiConnected] = useState(false)
  const [audioConnected] = useState(false)
  const [isConnecting, setIsConnecting] = useState(false)
  const [error, setError] = useState<string | null>(null)

  const connectMidi = useCallback(async (deviceIndex: number) => {
    setIsConnecting(true)
    setError(null)
    try {
      await invoke<string>('connect_midi_device', { deviceIndex })
      setMidiConnected(true)
    } catch (err) {
      setError(err as string)
      console.error('Failed to connect MIDI device:', err)
    } finally {
      setIsConnecting(false)
    }
  }, [])

  const testMidiConnection = useCallback(async () => {
    try {
      const result = await invoke<string>('test_midi_connection')
      console.log('MIDI test result:', result)
      return result
    } catch (err) {
      console.error('MIDI test failed:', err)
      throw err
    }
  }, [])

  const sendMidiPanic = useCallback(async () => {
    try {
      const result = await invoke<string>('send_midi_panic')
      console.log('MIDI panic result:', result)
      return result
    } catch (err) {
      console.error('MIDI panic failed:', err)
      throw err
    }
  }, [])

  return {
    midiConnected,
    audioConnected,
    isConnecting,
    error,
    connectMidi,
    testMidiConnection,
    sendMidiPanic
  }
}

// Audio Monitoring
export function useAudioMonitoring() {
  const [isMonitoring, setIsMonitoring] = useState(false)
  const [levels, setLevels] = useState<AudioLevels>({
    peak: 0,
    rms: 0,
    peak_db: -60,
    rms_db: -60
  })

  const startMonitoring = useCallback(async () => {
    try {
      await invoke<string>('start_input_monitoring')
      setIsMonitoring(true)
    } catch (err) {
      console.error('Failed to start monitoring:', err)
      throw err
    }
  }, [])

  const stopMonitoring = useCallback(async () => {
    try {
      await invoke<string>('stop_input_monitoring')
      setIsMonitoring(false)
    } catch (err) {
      console.error('Failed to stop monitoring:', err)
      throw err
    }
  }, [])

  // Poll for audio levels when monitoring
  useEffect(() => {
    if (!isMonitoring) return

    const interval = setInterval(async () => {
      try {
        const newLevels = await invoke<AudioLevels>('get_audio_levels')
        setLevels(newLevels)
      } catch (err) {
        console.error('Failed to get audio levels:', err)
      }
    }, 50) // 20fps updates

    return () => clearInterval(interval)
  }, [isMonitoring])

  return {
    isMonitoring,
    levels,
    startMonitoring,
    stopMonitoring
  }
}

// Recording Functions
export function useRecording() {
  const [isRecording, setIsRecording] = useState(false)
  const [error, setError] = useState<string | null>(null)

  const recordSample = useCallback(async (
    note: number,
    velocity: number,
    duration: number,
    outputDirectory?: string,
    sampleName?: string,
    exportFormat?: string,
    creatorName?: string,
    instrumentDescription?: string
  ) => {
    setIsRecording(true)
    setError(null)
    try {
      const result = await invoke<string>('record_sample', {
        note,
        velocity,
        duration,
        outputDirectory,
        sampleName,
        exportFormat,
        creatorName,
        instrumentDescription
      })
      return result
    } catch (err) {
      setError(err as string)
      console.error('Recording failed:', err)
      throw err
    } finally {
      setIsRecording(false)
    }
  }, [])

  const recordRange = useCallback(async (
    startNote: number,
    endNote: number,
    velocity: number,
    duration: number,
    outputDirectory?: string,
    sampleName?: string,
    exportFormat?: string,
    creatorName?: string,
    instrumentDescription?: string
  ) => {
    setIsRecording(true)
    setError(null)
    try {
      const result = await invoke<string>('record_range', {
        startNote,
        endNote,
        velocity,
        duration,
        outputDirectory,
        sampleName,
        exportFormat,
        creatorName,
        instrumentDescription
      })
      return result
    } catch (err) {
      setError(err as string)
      console.error('Range recording failed:', err)
      throw err
    } finally {
      setIsRecording(false)
    }
  }, [])

  const previewNote = useCallback(async (note: number, velocity: number, duration: number) => {
    try {
      const result = await invoke<string>('preview_note', { note, velocity, duration })
      return result
    } catch (err) {
      console.error('Preview failed:', err)
      throw err
    }
  }, [])

  return {
    isRecording,
    error,
    recordSample,
    recordRange,
    previewNote
  }
}

// Loop Detection
export function useLoopDetection() {
  const [isDetecting, setIsDetecting] = useState(false)
  const [error, setError] = useState<string | null>(null)

  const detectLoopPoints = useCallback(async (
    filePath: string,
    minLoopLength?: number,
    maxLoopLength?: number,
    correlationThreshold?: number
  ) => {
    setIsDetecting(true)
    setError(null)
    try {
      const resultJson = await invoke<string>('detect_loop_points', {
        filePath,
        minLoopLength,
        maxLoopLength,
        correlationThreshold
      })
      const result: LoopDetectionResponse = JSON.parse(resultJson)
      return result
    } catch (err) {
      setError(err as string)
      console.error('Loop detection failed:', err)
      throw err
    } finally {
      setIsDetecting(false)
    }
  }, [])

  const applyLoopMetadata = useCallback(async (
    filePath: string,
    startSample: number,
    endSample: number,
    sampleRate: number
  ) => {
    try {
      const result = await invoke<string>('apply_loop_metadata', {
        filePath,
        startSample,
        endSample,
        sampleRate
      })
      return result
    } catch (err) {
      console.error('Apply loop metadata failed:', err)
      throw err
    }
  }, [])

  const getLastRecordedSamplePath = useCallback(async (
    outputDirectory?: string,
    sampleName?: string
  ) => {
    try {
      const result = await invoke<string>('get_last_recorded_sample_path', {
        outputDirectory,
        sampleName
      })
      return result
    } catch (err) {
      console.error('Get last sample path failed:', err)
      throw err
    }
  }, [])

  return {
    isDetecting,
    error,
    detectLoopPoints,
    applyLoopMetadata,
    getLastRecordedSamplePath
  }
}

// File System Operations
export function useFileSystem() {
  const selectOutputDirectory = useCallback(async () => {
    try {
      const result = await invoke<string>('select_output_directory')
      return result
    } catch (err) {
      console.error('Directory selection failed:', err)
      throw err
    }
  }, [])

  const showSamplesInFinder = useCallback(async () => {
    try {
      const result = await invoke<string>('show_samples_in_finder')
      return result
    } catch (err) {
      console.error('Show samples in finder failed:', err)
      throw err
    }
  }, [])

  const generateInstrumentFiles = useCallback(async (
    directory: string,
    exportFormat: string,
    sampleName?: string,
    creatorName?: string,
    instrumentDescription?: string
  ) => {
    try {
      const result = await invoke<string>('generate_instrument_files', {
        directory,
        exportFormat,
        sampleName,
        creatorName,
        instrumentDescription
      })
      return result
    } catch (err) {
      console.error('Generate instrument files failed:', err)
      throw err
    }
  }, [])

  return {
    selectOutputDirectory,
    showSamplesInFinder,
    generateInstrumentFiles
  }
}

// Waveform Visualization
export function useWaveform() {
  const [waveformData, setWaveformData] = useState<WaveformData | null>(null)
  const [isLoading, setIsLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)

  const loadWaveform = useCallback(async (filePath: string, resolution?: number) => {
    setIsLoading(true)
    setError(null)
    try {
      const data = await invoke<WaveformData>('get_waveform_data', {
        filePath,
        resolution
      })
      setWaveformData(data)
      return data
    } catch (err) {
      setError(err as string)
      console.error('Failed to load waveform:', err)
      throw err
    } finally {
      setIsLoading(false)
    }
  }, [])

  const clearWaveform = useCallback(() => {
    setWaveformData(null)
    setError(null)
  }, [])

  return {
    waveformData,
    isLoading,
    error,
    loadWaveform,
    clearWaveform
  }
}

// Audio Playback
export function useAudioPlayback() {
  const [isPlaying, setIsPlaying] = useState(false)
  const [isLoading, setIsLoading] = useState(false)
  const [currentFile, setCurrentFile] = useState<string | null>(null)
  const [playbackPosition, setPlaybackPosition] = useState(0)
  const [error, setError] = useState<string | null>(null)
  const positionIntervalRef = useRef<number | null>(null)

  // Load audio file for playback
  const loadAudioFile = useCallback(async (filePath: string) => {
    setIsLoading(true)
    setError(null)
    try {
      const result = await invoke<string>('load_sample_for_playback', { filePath })
      setCurrentFile(filePath)
      setPlaybackPosition(0)
      console.log('Audio file loaded:', result)
      return result
    } catch (err) {
      setError(err as string)
      console.error('Failed to load audio file:', err)
      throw err
    } finally {
      setIsLoading(false)
    }
  }, [])

  // Start or resume playback
  const play = useCallback(async () => {
    if (!currentFile) {
      setError('No audio file loaded')
      return
    }
    setError(null)
    try {
      const result = await invoke<string>('start_playback')
      setIsPlaying(true)
      console.log('Playback started:', result)
      
      // Start position polling
      if (positionIntervalRef.current) {
        clearInterval(positionIntervalRef.current)
      }
      positionIntervalRef.current = window.setInterval(async () => {
        try {
          const position = await invoke<number>('get_playback_position')
          setPlaybackPosition(position)
          
          // Check if still playing
          const playing = await invoke<boolean>('is_playing')
          if (!playing) {
            setIsPlaying(false)
            if (positionIntervalRef.current) {
              clearInterval(positionIntervalRef.current)
              positionIntervalRef.current = null
            }
          }
        } catch (err) {
          console.error('Failed to get playback position:', err)
        }
      }, 50) // 20fps update rate
    } catch (err) {
      setError(err as string)
      console.error('Failed to start playback:', err)
      throw err
    }
  }, [currentFile])

  // Pause playback
  const pause = useCallback(async () => {
    setError(null)
    try {
      const result = await invoke<string>('pause_playback')
      setIsPlaying(false)
      console.log('Playback paused:', result)
      
      // Stop position polling
      if (positionIntervalRef.current) {
        clearInterval(positionIntervalRef.current)
        positionIntervalRef.current = null
      }
    } catch (err) {
      setError(err as string)
      console.error('Failed to pause playback:', err)
      throw err
    }
  }, [])

  // Stop playback
  const stop = useCallback(async () => {
    setError(null)
    try {
      const result = await invoke<string>('stop_playback')
      setIsPlaying(false)
      setPlaybackPosition(0)
      console.log('Playback stopped:', result)
      
      // Stop position polling
      if (positionIntervalRef.current) {
        clearInterval(positionIntervalRef.current)
        positionIntervalRef.current = null
      }
    } catch (err) {
      setError(err as string)
      console.error('Failed to stop playback:', err)
      throw err
    }
  }, [])

  // Seek to position
  const seek = useCallback(async (position: number) => {
    setError(null)
    try {
      const result = await invoke<string>('seek_playback', { position })
      setPlaybackPosition(position)
      console.log('Seeked to position:', result)
    } catch (err) {
      setError(err as string)
      console.error('Failed to seek:', err)
      throw err
    }
  }, [])

  // Toggle play/pause
  const togglePlayPause = useCallback(async () => {
    if (isPlaying) {
      await pause()
    } else {
      await play()
    }
  }, [isPlaying, play, pause])

  // Cleanup on unmount
  useEffect(() => {
    return () => {
      if (positionIntervalRef.current) {
        clearInterval(positionIntervalRef.current)
      }
    }
  }, [])

  return {
    isPlaying,
    isLoading,
    currentFile,
    playbackPosition,
    error,
    loadAudioFile,
    play,
    pause,
    stop,
    seek,
    togglePlayPause
  }
}