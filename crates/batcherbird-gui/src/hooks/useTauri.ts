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

// Intelligent Detection Types (Epic 3.2)
export interface IntelligentDetectionConfig {
  profile: string
  rms_threshold: number
  spectral_flux_threshold: number
  phase_deviation_threshold: number
  min_length_ms: number
  pre_attack_ms: number
  post_release_ms: number
  fft_size: number
  overlap_factor: number
}

export interface IntelligentDetectionResult {
  start_sample: number
  end_sample: number
  confidence_score: number
  algorithm_results: AlgorithmResult[]
  profile_used: string
  processing_time_ms: number
}

export interface AlgorithmResult {
  algorithm: string
  start_sample: number
  end_sample: number
  confidence: number
  metadata: Record<string, any>
}

export interface TrimmingResult {
  trimmed_audio: number[]
  original_length: number
  trimmed_length: number
  removed_start_samples: number
  removed_end_samples: number
  fade_in_samples: number
  fade_out_samples: number
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
      setDevices(result || [])
    } catch (err) {
      setError(err as string)
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
      setDevices(result || [])
    } catch (err) {
      setError(err as string)
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
      setDevices(result || [])
    } catch (err) {
      setError(err as string)
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
    } finally {
      setIsConnecting(false)
    }
  }, [])

  const testMidiConnection = useCallback(async () => {
    try {
      const result = await invoke<string>('test_midi_connection')
      // Mark MIDI activity when test succeeds
      setLastMidiActivity(Date.now())
      return result
    } catch (err) {
      throw err
    }
  }, [])

  const sendMidiPanic = useCallback(async () => {
    try {
      const result = await invoke<string>('send_midi_panic')
      // Mark MIDI activity when panic is sent
      setLastMidiActivity(Date.now())
      return result
    } catch (err) {
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

      // Sync frontend state with backend reality
      if (backendConnected !== midiConnected) {
        setMidiConnected(backendConnected)
      }

      return backendConnected
    } catch (err) {
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
      throw err
    }
  }, [])

  const stopMonitoring = useCallback(async () => {
    try {
      await invoke<string>('stop_input_monitoring')
      setIsMonitoring(false)
    } catch (err) {
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
        // Silently ignore polling errors
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
        // Silently ignore polling errors
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
    setIsRecording(true)
    setError(null)

    try {
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

      setIsRecording(false)
      return filePath

    } catch (err) {
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
      throw err
    } finally {
      setIsRecording(false)
    }
  }, [])

  const recordRangeWithVelocityLayers = useCallback(async (
    startNote: number,
    endNote: number,
    velocityLayers: number[],
    duration: number,
    outputDirectory?: string,
    sampleName?: string,
    exportFormat?: string,
    creatorName?: string,
    instrumentDescription?: string,
    noteToNoteDelay?: number,
    layerToLayerDelay?: number
  ) => {
    setIsRecording(true)
    setError(null)
    try {
      const result = await invoke<string>('record_range_with_velocity_layers', {
        startNote,
        endNote,
        velocityLayers,
        duration,
        outputDirectory,
        sampleName,
        exportFormat,
        creatorName,
        instrumentDescription,
        noteToNoteDelay,
        layerToLayerDelay
      })
      return result
    } catch (err) {
      setError(err as string)
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
      throw err
    }
  }, [])

  const cancelRecording = useCallback(async () => {
    try {
      const result = await invoke<string>('cancel_recording')
      return result
    } catch (err) {
      throw err
    }
  }, [])

  return {
    isRecording,
    error,
    recordSample,
    recordRange,
    recordRangeWithVelocityLayers,
    previewNote,
    cancelRecording
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
      throw err
    }
  }, [])

  const selectAudioFile = useCallback(async () => {
    try {
      const result = await invoke<string>('select_audio_file')
      return result
    } catch (err) {
      throw err
    }
  }, [])

  const showSamplesInFinder = useCallback(async () => {
    try {
      const result = await invoke<string>('show_samples_in_finder')
      return result
    } catch (err) {
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
      throw err
    }
  }, [])

  const createDirectory = useCallback(async (path: string) => {
    try {
      const result = await invoke<boolean>('create_directory', { path })
      return result
    } catch (err) {
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
      throw err
    } finally {
      setIsLoading(false)
    }
  }, [])

  const clearWaveform = useCallback(() => {
    setWaveformData(null)
    setError(null)
  }, [])

  // Transition from recording to file playback
  const transitionToFilePlayback = useCallback(async (filePath: string, resolution?: number) => {
    setIsTransitioning(true)
    setError(null)

    try {
      // Clear any existing waveform data first
      setWaveformData(null)

      // Small delay to ensure real-time visualization has stopped
      await new Promise(resolve => setTimeout(resolve, 100))

      // Load the file waveform
      const data = await loadWaveform(filePath, resolution)

      return data
    } catch (err) {
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
      return result
    } catch (err) {
      setError(err as string)
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
      await invoke<string>('start_playback')
      setIsPlaying(true)

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
          // Silently ignore polling errors
        }
      }, 50) // 20fps update rate
    } catch (err) {
      setError(err as string)
      throw err
    }
  }, [currentFile])

  // Pause playback
  const pause = useCallback(async () => {
    setError(null)
    try {
      await invoke<string>('pause_playback')
      setIsPlaying(false)

      // Stop position polling
      if (positionIntervalRef.current) {
        clearInterval(positionIntervalRef.current)
        positionIntervalRef.current = null
      }
    } catch (err) {
      setError(err as string)
      throw err
    }
  }, [])

  // Stop playback
  const stop = useCallback(async () => {
    setError(null)
    try {
      await invoke<string>('stop_playback')
      setIsPlaying(false)
      setPlaybackPosition(0)

      // Stop position polling
      if (positionIntervalRef.current) {
        clearInterval(positionIntervalRef.current)
        positionIntervalRef.current = null
      }
    } catch (err) {
      setError(err as string)
      throw err
    }
  }, [])

  // Seek to position
  const seek = useCallback(async (position: number) => {
    setError(null)
    try {
      await invoke<string>('seek_playback', { position })
      setPlaybackPosition(position)
    } catch (err) {
      setError(err as string)
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
      // Set up Tauri channel listener for waveform chunks
      const unlisten = await listen<VizChunk>('waveform_chunk', (event) => {
        const vizChunk = event.payload

        setVizChunks(prev => {
          // Keep a rolling buffer of recent chunks (last 3 seconds at 60fps = 180 chunks)
          const newChunks = [...prev, vizChunk]
          return newChunks.slice(-180)
        })
      })

      unlistenRef.current = unlisten
      setIsRecording(true)

    } catch (err) {
      const errorMsg = `Failed to start real-time visualization: ${err}`
      setError(errorMsg)
      throw err
    }
  }, [])

  // Stop recording visualization
  const stopRecording = useCallback(() => {
    // Clean up Tauri listener
    if (unlistenRef.current) {
      unlistenRef.current()
      unlistenRef.current = null
    }

    setIsRecording(false)
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

// Intelligent Detection Hook (Epic 3.2)
export function useIntelligentDetection() {
  const [isDetecting, setIsDetecting] = useState(false)
  const [isTrimming, setIsTrimming] = useState(false)
  const [error, setError] = useState<string | null>(null)

  const getSynthesizerProfiles = useCallback(async () => {
    try {
      const profiles = await invoke<string[]>('get_synthesizer_profiles')
      return profiles
    } catch (err) {
      throw err
    }
  }, [])

  const getDetectionConfig = useCallback(async (profile: string) => {
    try {
      const configJson = await invoke<string>('get_detection_config', { profile })
      const config: IntelligentDetectionConfig = JSON.parse(configJson)
      return config
    } catch (err) {
      throw err
    }
  }, [])

  const detectSampleBoundaries = useCallback(async (
    filePath: string,
    profile: string,
    customConfig?: IntelligentDetectionConfig
  ) => {
    setIsDetecting(true)
    setError(null)
    try {
      const customConfigJson = customConfig ? JSON.stringify(customConfig) : undefined
      const resultJson = await invoke<string>('detect_sample_boundaries', {
        filePath,
        profile,
        customConfig: customConfigJson
      })
      const result: IntelligentDetectionResult = JSON.parse(resultJson)
      return result
    } catch (err) {
      setError(err as string)
      throw err
    } finally {
      setIsDetecting(false)
    }
  }, [])

  const applyProfessionalTrimming = useCallback(async (
    filePath: string,
    detectionResult: IntelligentDetectionResult,
    outputPath?: string
  ) => {
    setIsTrimming(true)
    setError(null)
    try {
      const detectionJson = JSON.stringify(detectionResult)
      const trimmedPath = await invoke<string>('apply_professional_trimming', {
        filePath,
        detectionResult: detectionJson,
        outputPath
      })
      return trimmedPath
    } catch (err) {
      setError(err as string)
      throw err
    } finally {
      setIsTrimming(false)
    }
  }, [])

  return {
    isDetecting,
    isTrimming,
    error,
    getSynthesizerProfiles,
    getDetectionConfig,
    detectSampleBoundaries,
    applyProfessionalTrimming
  }
}

