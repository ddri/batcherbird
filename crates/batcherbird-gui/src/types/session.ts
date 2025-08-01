// TypeScript types matching the Rust session configuration

export interface SessionConfig {
  project_name: string
  project_directory: string
  audio: AudioSessionConfig
  midi: MidiSessionConfig
  recording: RecordingSessionConfig
  export: ExportSessionConfig
  created_at: string
}

export interface AudioSessionConfig {
  input_device?: string
  output_device?: string
  sample_rate: number        // 44100, 48000, 88200, 96000, etc.
  bit_depth: number          // 16, 24, 32
  buffer_size: number        // 128, 256, 512, 1024, 2048
  input_channels: number[]   // Selected input channels
  monitoring_enabled: boolean
  playthrough_enabled: boolean
}

export interface MidiSessionConfig {
  output_device?: string
  channel: number            // 0-15 (MIDI channels 1-16)
  velocity_curve: VelocityCurve
  program_change_delay_ms: number
}

export interface RecordingSessionConfig {
  note_duration_ms: number   // 100-30000ms
  release_time_ms: number    // 0-10000ms
  pre_delay_ms: number       // 0-1000ms
  post_delay_ms: number      // 0-1000ms
  auto_detect_silence: boolean
  detection_threshold_db: number // -60.0 to -6.0
}

export interface ExportSessionConfig {
  output_directory: string
  naming_pattern: string
  sample_format: AudioFormat
  normalize: boolean
  fade_in_ms: number
  fade_out_ms: number
  creator_name?: string
  project_description?: string
}

export type VelocityCurve = 'Linear' | 'Exponential' | 'Logarithmic' | { Custom: number[] }

export type AudioFormat = 'Wav16Bit' | 'Wav24Bit' | 'Wav32BitFloat' | 'DecentSampler' | 'SFZ' | 'All'

export type ExportTarget = 'LivePerformance' | 'StudioProduction' | 'Distribution' | 'Archival'

export type SessionState = 'Uninitialized' | 'Initializing' | 'Ready' | 'Recording' | { Error: string }

export interface ValidationError {
  field: string
  message: string
  severity: 'Error' | 'Warning' | 'Info'
}

export interface ValidationReport {
  errors: ValidationError[]
  warnings: ValidationError[]
  is_valid: boolean
}

export interface TestResult {
  success: boolean
  message: string
  details?: string
  latency_ms?: number
}

export interface DeviceTestResult {
  audio_input: TestResult
  audio_output: TestResult
  midi_output: TestResult
  overall_success: boolean
}

// Session initialization wizard steps
export type WizardStep = 
  | 'welcome'       // Project overview and name
  | 'audio'         // Audio device and settings
  | 'midi'          // MIDI device and settings
  | 'recording'     // Recording parameters
  | 'export'        // Export settings and directories
  | 'validation'    // Final validation and test
  | 'ready'         // Session ready confirmation

// Professional defaults for UI
export const PROFESSIONAL_SAMPLE_RATES = [44100, 48000, 88200, 96000, 176400, 192000]
export const PROFESSIONAL_BIT_DEPTHS = [16, 24, 32]
export const PROFESSIONAL_BUFFER_SIZES = [128, 256, 512, 1024, 2048, 4096]

export const DEFAULT_AUDIO_CONFIG: AudioSessionConfig = {
  sample_rate: 48000,      // Modern film/TV standard
  bit_depth: 24,           // Professional recording standard
  buffer_size: 512,        // Low latency balance
  input_channels: [0],     // First channel default
  monitoring_enabled: true,
  playthrough_enabled: false,
}

export const DEFAULT_MIDI_CONFIG: MidiSessionConfig = {
  channel: 0,              // MIDI channel 1 (0-indexed)
  velocity_curve: 'Linear',
  program_change_delay_ms: 50,
}

export const DEFAULT_RECORDING_CONFIG: RecordingSessionConfig = {
  note_duration_ms: 2500,  // 2.5s captures full decay
  release_time_ms: 1000,   // 1s professional standard
  pre_delay_ms: 100,       // Eliminates MIDI latency
  post_delay_ms: 100,      // Clean buffer flush
  auto_detect_silence: true,
  detection_threshold_db: -35.0, // Professional threshold
}

export const DEFAULT_EXPORT_CONFIG: ExportSessionConfig = {
  output_directory: '',
  naming_pattern: '{project_name}_{note_name}_{note}_{velocity}.wav',
  sample_format: 'Wav24Bit',
  normalize: false,
  fade_in_ms: 0.0,
  fade_out_ms: 10.0,       // Professional fade-out
}

// Export format recommendations
export const EXPORT_FORMAT_RECOMMENDATIONS: Record<ExportTarget, AudioFormat[]> = {
  LivePerformance: ['DecentSampler', 'Wav24Bit'],
  StudioProduction: ['Wav32BitFloat', 'DecentSampler', 'SFZ'],
  Distribution: ['DecentSampler', 'SFZ', 'Wav24Bit'],
  Archival: ['Wav32BitFloat'],
}

// Helper functions
export function formatSampleRate(rate: number): string {
  if (rate >= 1000) {
    return `${(rate / 1000).toFixed(1)}kHz`
  }
  return `${rate}Hz`
}

export function formatBitDepth(depth: number): string {
  return `${depth}-bit`
}

export function formatBufferSize(size: number): string {
  return `${size} samples`
}

export function formatMidiChannel(channel: number): string {
  return `Channel ${channel + 1}` // Convert 0-based to 1-based for display
}

export function formatAudioFormat(format: AudioFormat): string {
  switch (format) {
    case 'Wav16Bit': return 'WAV 16-bit'
    case 'Wav24Bit': return 'WAV 24-bit'
    case 'Wav32BitFloat': return 'WAV 32-bit Float'
    case 'DecentSampler': return 'Decent Sampler (.dspreset)'
    case 'SFZ': return 'SFZ (.sfz)'
    case 'All': return 'All Formats'
  }
}

export function getAudioFormatDescription(format: AudioFormat): string {
  switch (format) {
    case 'Wav16Bit': return 'Standard quality, smaller files'
    case 'Wav24Bit': return 'Professional quality, recommended'
    case 'Wav32BitFloat': return 'Maximum quality, future-proof'
    case 'DecentSampler': return 'Free sampler format, instant playback'
    case 'SFZ': return 'Professional sampler standard'
    case 'All': return 'Export in multiple formats'
  }
}

export function validateProjectName(name: string): string | null {
  if (!name || name.trim().length === 0) {
    return 'Project name is required'
  }
  if (name.trim().length > 255) {
    return 'Project name too long (max 255 characters)'
  }
  if (!/^[a-zA-Z0-9\s\-_()[\]{}]+$/.test(name.trim())) {
    return 'Project name contains invalid characters'
  }
  return null
}

export function getCurrentTimestamp(): string {
  return new Date().toISOString()
}

export function formatDuration(ms: number): string {
  if (ms >= 1000) {
    return `${(ms / 1000).toFixed(1)}s`
  }
  return `${ms}ms`
}

export function formatThreshold(db: number): string {
  return `${db.toFixed(1)}dB`
}