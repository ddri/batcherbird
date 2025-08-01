import { invoke } from '@tauri-apps/api/core'
import { useState, useEffect, useCallback } from 'react'
import type {
  SessionConfig,
  SessionState,
  ValidationReport,
  DeviceTestResult,
  WizardStep,
} from '@/types/session'
import {
  DEFAULT_AUDIO_CONFIG,
  DEFAULT_MIDI_CONFIG,
  DEFAULT_RECORDING_CONFIG,
  DEFAULT_EXPORT_CONFIG,
} from '@/types/session'
import { getCurrentTimestamp } from '@/types/session'

export function useSessionManager() {
  const [isInitialized, setIsInitialized] = useState(false)
  const [sessionState, setSessionState] = useState<SessionState>('Uninitialized')
  const [currentSession] = useState<SessionConfig | null>(null)
  const [error, setError] = useState<string | null>(null)

  // Initialize session manager on first use
  const initializeManager = useCallback(async () => {
    if (isInitialized) return

    try {
      await invoke<string>('initialize_session_manager')
      setIsInitialized(true)
      setError(null)
      console.log('✅ Session manager initialized')
    } catch (err) {
      setError(err as string)
      console.error('❌ Failed to initialize session manager:', err)
      throw err
    }
  }, [isInitialized])

  // Get current session state
  const refreshSessionState = useCallback(async () => {
    if (!isInitialized) return

    try {
      const state = await invoke<string>('get_session_state')
      setSessionState(state as SessionState)
    } catch (err) {
      console.error('Failed to get session state:', err)
    }
  }, [isInitialized])

  // Check if ready for recording
  const canRecord = useCallback(async (): Promise<boolean> => {
    if (!isInitialized) return false

    try {
      return await invoke<boolean>('can_record')
    } catch (err) {
      console.error('Failed to check recording state:', err)
      return false
    }
  }, [isInitialized])

  // Initialize on first render
  useEffect(() => {
    initializeManager().catch(console.error)
  }, [initializeManager])

  // Poll session state
  useEffect(() => {
    if (!isInitialized) return

    const interval = setInterval(refreshSessionState, 2000)
    return () => clearInterval(interval)
  }, [isInitialized, refreshSessionState])

  return {
    isInitialized,
    sessionState,
    currentSession,
    error,
    initializeManager,
    refreshSessionState,
    canRecord
  }
}

export function useSessionValidation() {
  const [isValidating, setIsValidating] = useState(false)
  const [validationReport, setValidationReport] = useState<ValidationReport | null>(null)

  const validateConfig = useCallback(async (config: SessionConfig): Promise<ValidationReport> => {
    setIsValidating(true)
    try {
      const report = await invoke<ValidationReport>('validate_session_config', { config })
      setValidationReport(report)
      return report
    } catch (err) {
      console.error('Validation failed:', err)
      throw err
    } finally {
      setIsValidating(false)
    }
  }, [])

  return {
    isValidating,
    validationReport,
    validateConfig
  }
}

export function useDeviceTesting() {
  const [isTesting, setIsTesting] = useState(false)
  const [testResults, setTestResults] = useState<DeviceTestResult | null>(null)

  const testDevices = useCallback(async (config: SessionConfig): Promise<DeviceTestResult> => {
    setIsTesting(true)
    try {
      const results = await invoke<DeviceTestResult>('test_device_connectivity', { config })
      setTestResults(results)
      return results
    } catch (err) {
      console.error('Device testing failed:', err)
      throw err
    } finally {
      setIsTesting(false)
    }
  }, [])

  return {
    isTesting,
    testResults,
    testDevices
  }
}

export function useSessionInitialization() {
  const [wizardStep, setWizardStep] = useState<WizardStep>('welcome')
  const [sessionConfig, setSessionConfig] = useState<SessionConfig>(() => getDefaultConfig())
  const [isInitializing, setIsInitializing] = useState(false)
  const [initializationError, setInitializationError] = useState<string | null>(null)

  const initializeSession = useCallback(async (): Promise<void> => {
    setIsInitializing(true)
    setInitializationError(null)
    
    try {
      const result = await invoke<string>('initialize_session', { config: sessionConfig })
      console.log('✅ Session initialized:', result)
      setWizardStep('ready')
    } catch (err) {
      setInitializationError(err as string)
      console.error('❌ Session initialization failed:', err)
      throw err
    } finally {
      setIsInitializing(false)
    }
  }, [sessionConfig])

  const nextStep = useCallback(() => {
    const steps: WizardStep[] = ['welcome', 'audio', 'midi', 'recording', 'export', 'validation', 'ready']
    const currentIndex = steps.indexOf(wizardStep)
    if (currentIndex < steps.length - 1) {
      setWizardStep(steps[currentIndex + 1])
    }
  }, [wizardStep])

  const previousStep = useCallback(() => {
    const steps: WizardStep[] = ['welcome', 'audio', 'midi', 'recording', 'export', 'validation', 'ready']
    const currentIndex = steps.indexOf(wizardStep)
    if (currentIndex > 0) {
      setWizardStep(steps[currentIndex - 1])
    }
  }, [wizardStep])

  const updateConfig = useCallback((updates: Partial<SessionConfig>) => {
    setSessionConfig(prev => ({ ...prev, ...updates }))
  }, [])

  const updateAudioConfig = useCallback((updates: Partial<SessionConfig['audio']>) => {
    setSessionConfig(prev => ({
      ...prev,
      audio: { ...prev.audio, ...updates }
    }))
  }, [])

  const updateMidiConfig = useCallback((updates: Partial<SessionConfig['midi']>) => {
    setSessionConfig(prev => ({
      ...prev,
      midi: { ...prev.midi, ...updates }
    }))
  }, [])

  const updateRecordingConfig = useCallback((updates: Partial<SessionConfig['recording']>) => {
    setSessionConfig(prev => ({
      ...prev,
      recording: { ...prev.recording, ...updates }
    }))
  }, [])

  const updateExportConfig = useCallback((updates: Partial<SessionConfig['export']>) => {
    setSessionConfig(prev => ({
      ...prev,
      export: { ...prev.export, ...updates }
    }))
  }, [])

  const resetToDefaults = useCallback(() => {
    setSessionConfig(getDefaultConfig())
    setWizardStep('welcome')
    setInitializationError(null)
  }, [])

  return {
    wizardStep,
    sessionConfig,
    isInitializing,
    initializationError,
    setWizardStep,
    nextStep,
    previousStep,
    updateConfig,
    updateAudioConfig,
    updateMidiConfig,
    updateRecordingConfig,
    updateExportConfig,
    initializeSession,
    resetToDefaults
  }
}

export function useSessionTemplates() {
  const [templates, setTemplates] = useState<string[]>([])
  const [isLoading, setIsLoading] = useState(false)

  const loadTemplates = useCallback(async () => {
    setIsLoading(true)
    try {
      const templateList = await invoke<string[]>('list_session_templates')
      setTemplates(templateList)
    } catch (err) {
      console.error('Failed to load templates:', err)
    } finally {
      setIsLoading(false)
    }
  }, [])

  const saveTemplate = useCallback(async (name: string, config: SessionConfig): Promise<void> => {
    try {
      await invoke<string>('save_session_template', { name, config })
      await loadTemplates() // Refresh list
    } catch (err) {
      console.error('Failed to save template:', err)
      throw err
    }
  }, [loadTemplates])

  const loadTemplate = useCallback(async (name: string): Promise<SessionConfig> => {
    try {
      return await invoke<SessionConfig>('load_session_template', { name })
    } catch (err) {
      console.error('Failed to load template:', err)
      throw err
    }
  }, [])

  // Load templates on mount
  useEffect(() => {
    loadTemplates()
  }, [loadTemplates])

  return {
    templates,
    isLoading,
    loadTemplates,
    saveTemplate,
    loadTemplate
  }
}

// Helper function to get default session configuration
function getDefaultConfig(): SessionConfig {
  const now = new Date()
  const projectName = `New Project ${now.getFullYear()}-${String(now.getMonth() + 1).padStart(2, '0')}-${String(now.getDate()).padStart(2, '0')} ${String(now.getHours()).padStart(2, '0')}-${String(now.getMinutes()).padStart(2, '0')}`
  
  return {
    project_name: projectName,
    project_directory: '', // Will be set to default Documents/BatcherBird Projects
    audio: { ...DEFAULT_AUDIO_CONFIG },
    midi: { ...DEFAULT_MIDI_CONFIG },
    recording: { ...DEFAULT_RECORDING_CONFIG },
    export: { ...DEFAULT_EXPORT_CONFIG },
    created_at: getCurrentTimestamp(),
  }
}

// Get default session config from backend
export async function getDefaultSessionConfig(): Promise<SessionConfig> {
  try {
    return await invoke<SessionConfig>('get_default_session_config')
  } catch (err) {
    console.error('Failed to get default config from backend:', err)
    return getDefaultConfig() // Fallback to frontend default
  }
}