import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { useState, useEffect, useCallback, useRef } from 'react'

// Types matching our Rust backend
export interface AudioLevels {
  peak: number
  rms: number
  peak_db: number
  rms_db: number
}

// Professional meter types (Epic 3.1.3)
export interface ProfessionalMeterReadings {
  vu_db: number
  ppm_db: number
  peak_db: number
  peak_hold_db: number
  lufs: number
  gain_staging: GainStagingStatus
}

export enum GainStagingStatus {
  Optimal = "Optimal",
  TooQuiet = "TooQuiet", 
  TooLoud = "TooLoud",
  Acceptable = "Acceptable",
  Clipping = "Clipping"
}

export enum RecommendationUrgency {
  Low = "Low",
  Medium = "Medium",
  High = "High",
  Critical = "Critical"
}

export enum HeadroomStatus {
  SafeHeadroom = "SafeHeadroom",
  LowHeadroom = "LowHeadroom",
  Clipping = "Clipping"
}

export enum LevelTrend {
  Rising = "Rising",
  Falling = "Falling",
  Stable = "Stable"
}

export interface GainRecommendation {
  adjustment_db: number
  confidence: number
  urgency: RecommendationUrgency
  description: string
}

export interface GainStagingAnalysis {
  current_level_db: number
  target_level_db: number
  target_distance_db: number
  is_optimal: boolean
  trend: LevelTrend
  recommendation: GainRecommendation
  headroom_status: HeadroomStatus
  peak_db: number
  headroom_db: number
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

export interface AudioDeviceInfo {
  device_name: string
  total_channels: number
  sample_rate: number
  channel_names: string[]
}

export interface VizChunk {
  peak: number        // Peak amplitude for this chunk (0.0 to 1.0)
  rms: number         // RMS level for this chunk (0.0 to 1.0)
  peak_db: number     // Peak in dBFS
  rms_db: number      // RMS in dBFS
  timestamp: number   // Timestamp in samples since recording start
  chunk_size: number  // Number of samples in this chunk
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

// Get Audio Device Info Hook
export function useAudioDeviceInfo() {
  const [deviceInfo, setDeviceInfo] = useState<AudioDeviceInfo | null>(null)
  const [isLoading, setIsLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)

  const getDeviceInfo = useCallback(async (deviceIndex: number) => {
    setIsLoading(true)
    setError(null)
    try {
      const info = await invoke<AudioDeviceInfo>('get_audio_device_info', { deviceIndex })
      setDeviceInfo(info)
      return info
    } catch (err) {
      const errorMsg = err as string
      setError(errorMsg)
      console.error('Failed to get audio device info:', errorMsg)
      throw err
    } finally {
      setIsLoading(false)
    }
  }, [])

  return { deviceInfo, isLoading, error, getDeviceInfo }
}

// Device Connection
export function useDeviceConnection() {
  const [midiConnected, setMidiConnected] = useState(false)
  const [isConnecting, setIsConnecting] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [lastMidiActivity, setLastMidiActivity] = useState<number | null>(null)

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
      // Mark MIDI activity when test succeeds
      setLastMidiActivity(Date.now())
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
      // Mark MIDI activity when panic is sent
      setLastMidiActivity(Date.now())
      return result
    } catch (err) {
      console.error('MIDI panic failed:', err)
      throw err
    }
  }, [])

  // Function to manually mark MIDI activity (for external calls)
  const markMidiActivity = useCallback(() => {
    setLastMidiActivity(Date.now())
  }, [])

  // Query actual backend MIDI connection status for state recovery
  const checkMidiConnectionStatus = useCallback(async () => {
    try {
      const backendConnected = await invoke<boolean>('get_midi_connection_status')
      console.log('🔍 Backend MIDI status:', backendConnected, 'Frontend state:', midiConnected)
      
      // Sync frontend state with backend reality
      if (backendConnected !== midiConnected) {
        console.log('🔄 Syncing MIDI connection state: backend =', backendConnected)
        setMidiConnected(backendConnected)
      }
      
      return backendConnected
    } catch (err) {
      console.error('Failed to check MIDI connection status:', err)
      return false
    }
  }, [midiConnected])

  return {
    midiConnected,
    isConnecting,
    error,
    lastMidiActivity,
    connectMidi,
    testMidiConnection,
    sendMidiPanic,
    markMidiActivity,
    checkMidiConnectionStatus
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

  const startMonitoring = useCallback(async (enablePlaythrough: boolean = false) => {
    try {
      if (enablePlaythrough) {
        await invoke<string>('start_input_monitoring_with_playthrough', { enablePlaythrough })
      } else {
        await invoke<string>('start_input_monitoring')
      }
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

// Professional meter monitoring (Epic 3.1.3)
export function useProfessionalMeters() {
  const [readings, setReadings] = useState<ProfessionalMeterReadings>({
    vu_db: -60.0,
    ppm_db: -60.0,
    peak_db: -60.0,
    peak_hold_db: -60.0,
    lufs: -70.0,
    gain_staging: GainStagingStatus.TooQuiet
  })
  const [isMonitoring, setIsMonitoring] = useState(false)

  const startProfessionalMetering = useCallback(() => {
    setIsMonitoring(true)
  }, [])

  const stopProfessionalMetering = useCallback(() => {
    setIsMonitoring(false)
  }, [])

  // Poll for professional meter readings when monitoring
  useEffect(() => {
    if (!isMonitoring) return

    const interval = setInterval(async () => {
      try {
        const newReadings = await invoke<ProfessionalMeterReadings>('get_professional_meter_readings')
        setReadings(newReadings)
      } catch (err) {
        console.error('Failed to get professional meter readings:', err)
      }
    }, 50) // 20fps updates for smooth meter ballistics

    return () => clearInterval(interval)
  }, [isMonitoring])

  return {
    readings,
    isMonitoring,
    startProfessionalMetering,
    stopProfessionalMetering
  }
}

// Gain staging analysis (Epic 3.1.3)
export function useGainStaging() {
  const [analysis, setAnalysis] = useState<GainStagingAnalysis | null>(null)
  const [isMonitoring, setIsMonitoring] = useState(false)
  const [error, setError] = useState<string | null>(null)

  const startGainStagingAnalysis = useCallback(() => {
    setIsMonitoring(true)
    setError(null)
  }, [])

  const stopGainStagingAnalysis = useCallback(() => {
    setIsMonitoring(false)
    setAnalysis(null)
  }, [])

  // Poll for gain staging analysis when monitoring
  useEffect(() => {
    if (!isMonitoring) return

    const interval = setInterval(async () => {
      try {
        const newAnalysis = await invoke<GainStagingAnalysis>('get_gain_staging_analysis')
        setAnalysis(newAnalysis)
        setError(null)
      } catch (err) {
        console.error('Failed to get gain staging analysis:', err)
        setError(err as string)
      }
    }, 200) // 5fps updates - slower for analysis

    return () => clearInterval(interval)
  }, [isMonitoring])

  return {
    analysis,
    isMonitoring,
    error,
    startGainStagingAnalysis,
    stopGainStagingAnalysis
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
  ): Promise<string> => {
    console.log('🎧 Frontend: Starting SYNCHRONOUS recording')
    setIsRecording(true)
    setError(null)
    
    try {
      console.log('🚀 Frontend: Calling synchronous start_recording_with_viz', {
        note, velocity, duration, outputDirectory, sampleName, exportFormat
      })
      
      const filePath = await invoke<string>('start_recording_with_viz', {
        note,
        velocity,
        duration,
        outputDirectory,
        sampleName,
        exportFormat,
        creatorName,
        instrumentDescription
      })
      
      console.log('✅ Frontend: SYNCHRONOUS recording completed:', filePath)
      setIsRecording(false)
      return filePath
      
    } catch (err) {
      console.error('❌ Frontend: SYNCHRONOUS recording failed:', err)
      setError(err as string)
      setIsRecording(false)
      throw err
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

  const selectAudioFile = useCallback(async () => {
    try {
      const result = await invoke<string>('select_audio_file')
      return result
    } catch (err) {
      console.error('Audio file selection failed:', err)
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

  const createDirectory = useCallback(async (path: string) => {
    try {
      const result = await invoke<boolean>('create_directory', { path })
      return result
    } catch (err) {
      console.error('Create directory failed:', err)
      throw err
    }
  }, [])

  return {
    selectOutputDirectory,
    selectAudioFile,
    showSamplesInFinder,
    generateInstrumentFiles,
    createDirectory
  }
}

// Waveform Visualization with Transition Management
export function useWaveform() {
  const [waveformData, setWaveformData] = useState<WaveformData | null>(null)
  const [isLoading, setIsLoading] = useState(false)
  const [isTransitioning, setIsTransitioning] = useState(false)
  const [error, setError] = useState<string | null>(null)

  const loadWaveform = useCallback(async (filePath: string, resolution?: number) => {
    console.log('🌊 Loading waveform for:', filePath)
    console.log('📊 Resolution parameter:', resolution)
    setIsLoading(true)
    setError(null)
    try {
      console.log('📡 Calling Tauri get_waveform_data command...')
      const data = await invoke<WaveformData>('get_waveform_data', {
        filePath,
        resolution
      })
      console.log('✅ Tauri command completed successfully')
      console.log('🌊 Waveform loaded successfully:', { 
        duration: data.duration, 
        channels: data.channels, 
        peaksLength: data.peaks.positive.length,
        sampleRate: data.sample_rate,
        format: data.format
      })
      setWaveformData(data)
      console.log('✅ Waveform data set in React state')
      return data
    } catch (err) {
      setError(err as string)
      console.error('❌ Failed to load waveform:', err)
      console.error('❌ Tauri command error details:', JSON.stringify(err))
      throw err
    } finally {
      setIsLoading(false)
      console.log('🏁 loadWaveform completed, isLoading set to false')
    }
  }, [])

  const clearWaveform = useCallback(() => {
    setWaveformData(null)
    setError(null)
  }, [])

  // Transition from recording to file playback
  const transitionToFilePlayback = useCallback(async (filePath: string, resolution?: number) => {
    console.log('🔄 Transitioning from recording to file playback:', filePath)
    setIsTransitioning(true)
    setError(null)
    
    try {
      // Clear any existing waveform data first
      console.log('🧹 Clearing existing waveform data...')
      setWaveformData(null)
      
      // Small delay to ensure real-time visualization has stopped
      console.log('⏱️ Waiting for real-time visualization to stop...')
      await new Promise(resolve => setTimeout(resolve, 100))
      
      // Verify file exists before attempting to load
      console.log('📁 Attempting to load waveform from:', filePath)
      console.log('📊 Using resolution:', resolution || 'default')
      
      // Load the file waveform
      const data = await loadWaveform(filePath, resolution)
      
      console.log('✅ Transition to file playback complete:', {
        peaksLength: data.peaks.positive.length,
        duration: data.duration,
        sampleRate: data.sample_rate
      })
      return data
    } catch (err) {
      console.error('❌ Failed to transition to file playback:', err)
      console.error('❌ Error type:', typeof err)
      console.error('❌ Error string:', String(err))
      if (err instanceof Error) {
        console.error('❌ Error message:', err.message)
        console.error('❌ Error stack:', err.stack)
      }
      setError(`Failed to load waveform: ${err}`)
      throw err
    } finally {
      setIsTransitioning(false)
    }
  }, [loadWaveform])

  return {
    waveformData,
    isLoading,
    isTransitioning,
    error,
    loadWaveform,
    clearWaveform,
    transitionToFilePlayback
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

// Real-time Tauri-based visualization hook (replaces Web Audio API)
export function useRealTimeVisualization() {
  const [isRecording, setIsRecording] = useState(false)
  const [vizChunks, setVizChunks] = useState<VizChunk[]>([])
  const [error, setError] = useState<string | null>(null)
  const unlistenRef = useRef<(() => void) | null>(null)

  // Start recording visualization
  const startRecording = useCallback(async () => {
    setError(null)
    setVizChunks([]) // Clear previous data
    
    try {
      console.log('🎤 Starting real-time visualization via Tauri channels')
      
      // Set up Tauri channel listener for waveform chunks
      const unlisten = await listen<VizChunk>('waveform_chunk', (event) => {
        const vizChunk = event.payload
        
        // Only log every 60th chunk (once per second) to avoid spam
        if (vizChunk.timestamp % 60 === 0) {
          console.log('📊 VizChunk stream active - Peak:', vizChunk.peak.toFixed(3), 'RMS:', vizChunk.rms.toFixed(3))
        }
        
        setVizChunks(prev => {
          // Keep a rolling buffer of recent chunks (last 3 seconds at 60fps = 180 chunks)
          const newChunks = [...prev, vizChunk]
          return newChunks.slice(-180)
        })
      })
      
      unlistenRef.current = unlisten
      setIsRecording(true)
      console.log('✅ Real-time visualization started')
      
    } catch (err) {
      const errorMsg = `Failed to start real-time visualization: ${err}`
      setError(errorMsg)
      console.error('❌', errorMsg)
      throw err
    }
  }, [])

  // Stop recording visualization
  const stopRecording = useCallback(() => {
    console.log('🛑 Stopping real-time visualization')
    
    // Clean up Tauri listener
    if (unlistenRef.current) {
      unlistenRef.current()
      unlistenRef.current = null
    }
    
    setIsRecording(false)
    console.log('✅ Real-time visualization stopped')
  }, [])

  // Cleanup on unmount
  useEffect(() => {
    return () => {
      if (unlistenRef.current) {
        unlistenRef.current()
      }
    }
  }, [])

  return {
    isRecording,
    vizChunks,
    error,
    startRecording,
    stopRecording
  }
}

