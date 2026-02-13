import { useState, useEffect } from "react"
import { listen, Event } from "@tauri-apps/api/event"

// Type for recording progress events from Tauri backend
interface RecordingProgressPayload {
  current: number
  total: number
  percent: number
  note?: number
  velocity?: number
  layer?: number
  totalLayers?: number
  noteName?: string
}
import { desktopDir } from "@tauri-apps/api/path"
import { Button } from "@/components/ui/button"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select"
import { Slider } from "@/components/ui/slider"
import { Switch } from "@/components/ui/switch"
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs"
import { DeviceStatusBar } from "@/components/DeviceStatusBar"
import { SetupModal } from "@/components/SetupModal"
import { WaveformDisplay } from "@/components/WaveformDisplay"
import { ProfessionalMeters } from "@/components/ProfessionalMeters"
import { RealtimeMeters } from "@/components/RealtimeMeters"
import { IntelligentDetectionControls } from "@/components/IntelligentDetectionControls"
// Hidden for v0.1.0 - uses mock data, backend not implemented
// import { QualityValidationDashboard } from "@/components/QualityValidationDashboard"
import { useRecording, useFileSystem, useWaveform, useLoopDetection, useAudioPlayback, useMidiDevices, useAudioInputDevices, useAudioOutputDevices, useDeviceConnection, useRealTimeVisualization } from "@/hooks/useTauri"
import { useRecordingState } from "@/hooks/useRecordingState"
import { useRecordingCountdown } from "@/hooks/useRecordingCountdown"
import { useNotifications } from "@/hooks/useNotifications"
// import { useSessionManager } from "@/hooks/useSession"
import { RecordingCountdown } from "@/components/RecordingCountdown"
import { NotificationDisplay } from "@/components/NotificationDisplay"
import { SessionInitializationWizard } from "@/components/SessionInitializationWizard-Simple"
import {
  Play,
  Square,
  Clock,
  Layers,
  FolderOpen,
  Volume2,
  X
} from "lucide-react"


export default function App() {
  // Professional session management - temporarily disabled until types are fixed
  // const { 
  //   isInitialized: sessionInitialized, 
  //   sessionState, 
  //   // currentSession, 
  //   // canRecord: sessionCanRecord,
  //   // error: sessionError 
  // } = useSessionManager()
  
  // Session initialization wizard state
  const [showSessionWizard, setShowSessionWizard] = useState(false)
  const [sessionInitialized, setSessionInitialized] = useState(false)
  
  // Recording state
  const [isRecording, setIsRecording] = useState(false)
  const [velocityLayers, setVelocityLayers] = useState([127, 100, 80, 60, 40, 20])
  
  // Form state
  const [selectedNote, setSelectedNote] = useState("60") // C4
  const [selectedVelocity, setSelectedVelocity] = useState([127]) // Default to max velocity
  const [selectedDuration, setSelectedDuration] = useState([2420]) // 2.42s in ms
  const [autoDetectSilence, setAutoDetectSilence] = useState(true)
  const [detectionThreshold, setDetectionThreshold] = useState([-35])
  const [sampleName, setSampleName] = useState("Roland-EM1018")
  const [outputDirectory, setOutputDirectory] = useState("")
  const [exportFormat, setExportFormat] = useState("wav16")
  const [creatorName, setCreatorName] = useState("")
  const [instrumentDescription, setInstrumentDescription] = useState("")
  const [recordingMode, setRecordingMode] = useState("single")
  const [startNote, setStartNote] = useState("36") // C2
  const [endNote, setEndNote] = useState("84") // C6
  const [noteToNoteDelay, setNoteToNoteDelay] = useState(200) // ms between notes
  const [layerToLayerDelay, setLayerToLayerDelay] = useState(500) // ms between velocity layers
  
  // Recording progress state for Epic 4
  const [recordingProgress, setRecordingProgress] = useState<{
    current: number
    total: number
    percent: number
    note?: number
    velocity?: number
    layer?: number
    totalLayers?: number
    noteName?: string
  } | null>(null)
  
  // Modal state
  const [setupModalOpen, setSetupModalOpen] = useState(false)
  const [setupModalTab, setSetupModalTab] = useState("midi")
  
  // Tauri hooks
  const { recordSample, recordRange, recordRangeWithVelocityLayers, previewNote, cancelRecording, isRecording: backendRecording } = useRecording()
  const { selectOutputDirectory, selectAudioFile, createDirectory } = useFileSystem()
  
  // Device hooks
  const { devices: midiDevices, loadDevices: loadMidiDevices, isLoading: midiLoading } = useMidiDevices()
  const { devices: audioInputDevices, loadDevices: loadAudioInputDevices } = useAudioInputDevices()
  const { devices: audioOutputDevices, loadDevices: loadAudioOutputDevices } = useAudioOutputDevices()
  const { midiConnected, connectMidi } = useDeviceConnection()
  
  // Device selection state (managed directly in App)
  const [selectedMidiDevice, setSelectedMidiDevice] = useState<string>("")
  const [selectedAudioInput, setSelectedAudioInput] = useState<string>("")
  const [selectedAudioOutput, setSelectedAudioOutput] = useState<string>("")
  
  // Load devices on mount
  useEffect(() => {
    loadMidiDevices()
    loadAudioInputDevices()
    loadAudioOutputDevices()
  }, [loadMidiDevices, loadAudioInputDevices, loadAudioOutputDevices])
  
  // Initialize default output directory
  useEffect(() => {
    const initOutputDirectory = async () => {
      if (!outputDirectory) {
        try {
          const desktop = await desktopDir()
          setOutputDirectory(`${desktop}Batcherbird Samples`)
        } catch (error) {
          setOutputDirectory('Batcherbird Samples') // Fallback
        }
      }
    }
    initOutputDirectory()
  }, [outputDirectory])
  
  // Listen for recording progress events (Epic 4)
  useEffect(() => {
    const unlisten = listen<RecordingProgressPayload>('recording_progress', (event: Event<RecordingProgressPayload>) => {
      setRecordingProgress(event.payload)
    })
    
    return () => {
      unlisten.then(fn => fn())
    }
  }, [])

  // Check if session initialization is needed
  useEffect(() => {
    if (!sessionInitialized && !showSessionWizard) {
      setShowSessionWizard(true)
    }
  }, [sessionInitialized, showSessionWizard])

  // Load device preferences from localStorage and auto-select + auto-connect
  useEffect(() => {
    if (midiDevices.length > 0 && !selectedMidiDevice) {
      const savedDevice = localStorage.getItem('selectedMidiDevice')
      const deviceIndex = savedDevice && parseInt(savedDevice) < midiDevices.length ? savedDevice : "0"
      setSelectedMidiDevice(deviceIndex)
      
      // Auto-connect like manual selection does (following professional audio app pattern)
      if (deviceIndex && !midiConnected) {
        connectMidi(parseInt(deviceIndex))
      }
    }
  }, [midiDevices, selectedMidiDevice, midiConnected, connectMidi])

  useEffect(() => {
    if (audioInputDevices.length > 0 && !selectedAudioInput) {
      const savedDevice = localStorage.getItem('selectedAudioInput')
      const deviceIndex = savedDevice && parseInt(savedDevice) < audioInputDevices.length ? savedDevice : "0"
      setSelectedAudioInput(deviceIndex)
    }
  }, [audioInputDevices, selectedAudioInput])

  useEffect(() => {
    if (audioOutputDevices.length > 0 && !selectedAudioOutput) {
      const savedDevice = localStorage.getItem('selectedAudioOutput')
      const deviceIndex = savedDevice && parseInt(savedDevice) < audioOutputDevices.length ? savedDevice : "0"
      setSelectedAudioOutput(deviceIndex)
    }
  }, [audioOutputDevices, selectedAudioOutput])

  // Device name getters
  const getMidiDeviceName = () => {
    if (!selectedMidiDevice || midiDevices.length === 0) return "No MIDI Device"
    return midiDevices[parseInt(selectedMidiDevice)] || "Unknown device"
  }

  const getAudioInputDeviceName = () => {
    if (!selectedAudioInput || audioInputDevices.length === 0) return "No Audio Device"
    return audioInputDevices[parseInt(selectedAudioInput)] || "Unknown device"
  }

  // MIDI device change handler with auto-connect and persistence
  const handleMidiDeviceChange = (value: string) => {
    setSelectedMidiDevice(value)
    localStorage.setItem('selectedMidiDevice', value)
    if (value && !midiConnected) {
      connectMidi(parseInt(value))
    }
  }

  // Audio device change handlers with persistence
  const handleAudioInputChange = (value: string) => {
    setSelectedAudioInput(value)
    localStorage.setItem('selectedAudioInput', value)
  }

  const handleAudioOutputChange = (value: string) => {
    setSelectedAudioOutput(value)
    localStorage.setItem('selectedAudioOutput', value)
  }

  // Recording state management
  const {
    state: recordingState,
    canRecord,
    canArm,
    canDisarm,
    isCountingDown,
    showLevelMeter,
    arm,
    disarm,
    startCountdown,
    startRecording,
    cancelCountdown,
    startPreview,
    stopPreview,
    getStateDisplayText,
    getStateColor
  } = useRecordingState()

  // Professional countdown system
  const {
    countdownState,
    startCountdown: startCountdownTimer,
    cancelCountdown: cancelCountdownTimer
  } = useRecordingCountdown()

  // Professional notification system
  const {
    notifications,
    removeNotification,
    showError,
    showSuccess
  } = useNotifications()
  const { waveformData, isLoading: waveformLoading, isTransitioning: waveformTransitioning, error: waveformError, loadWaveform, clearWaveform, transitionToFilePlayback } = useWaveform()
  const { getLastRecordedSamplePath } = useLoopDetection()
  const { 
    isPlaying, 
    playbackPosition, 
    loadAudioFile, 
    togglePlayPause, 
    seek 
  } = useAudioPlayback()
  
  // Real-time visualization using Tauri channels
  const {
    isRecording: isRealTimeRecording,
    vizChunks: realtimeVizChunks,
    startRecording: startRealTimeVisualization,
    stopRecording: stopRealTimeVisualization
  } = useRealTimeVisualization()
  
  // Track the last recorded file
  const [lastRecordedFile, setLastRecordedFile] = useState<string | null>(null)

  // Keyboard shortcuts for professional workflow
  useEffect(() => {
    const handleKeyDown = (event: KeyboardEvent) => {
      // Prevent shortcuts when typing in inputs
      if (event.target instanceof HTMLInputElement || event.target instanceof HTMLTextAreaElement) {
        return
      }

      // ESC cancels countdown
      if (event.key === 'Escape' && isCountingDown) {
        event.preventDefault()
        cancelCountdownTimer()
        cancelCountdown()
        return
      }

      // Spacebar for play/pause (universal professional standard)
      if (event.key === ' ') {
        event.preventDefault()
        if (lastRecordedFile) {
          togglePlayPause()
        }
        return
      }
    }

    window.addEventListener('keydown', handleKeyDown)
    return () => window.removeEventListener('keydown', handleKeyDown)
  }, [isCountingDown, cancelCountdownTimer, cancelCountdown, lastRecordedFile, togglePlayPause])

  // Handlers

  
  const handleCloseSetup = () => {
    setSetupModalOpen(false)
  }

  const handleRecord = async () => {
    if (!canRecord) {
      return
    }

    // Professional countdown before recording (2 seconds like Pro Tools)
    startCountdown()

    try {
      await startCountdownTimer(2000) // 2 second countdown
    } catch (error) {
      cancelCountdown()
      return
    }

    // Capture state values to avoid race conditions during recording
    const capturedOutputDirectory = outputDirectory
    const capturedSampleName = sampleName
    const capturedExportFormat = exportFormat
    const capturedCreatorName = creatorName
    const capturedInstrumentDescription = instrumentDescription
    
    if (recordingMode === "single") {
      try {
        startRecording() // Update state to recording
        setIsRecording(true)

        // Clear old waveform data
        clearWaveform()

        // Start real-time waveform visualization via Tauri channels
        await startRealTimeVisualization()

        await recordSample(
          parseInt(selectedNote),
          selectedVelocity[0],
          selectedDuration[0],
          capturedOutputDirectory,
          capturedSampleName,
          capturedExportFormat,
          capturedCreatorName,
          capturedInstrumentDescription
        )

        // Stop real-time visualization immediately after recording completes
        stopRealTimeVisualization()

        // Transition from recording to file playback
        try {
          const lastSamplePath = await getLastRecordedSamplePath(capturedOutputDirectory, capturedSampleName)

          // Add small delay to ensure file is fully written
          await new Promise(resolve => setTimeout(resolve, 500))

          setLastRecordedFile(lastSamplePath)

          try {
            await transitionToFilePlayback(lastSamplePath)
          } catch (waveformErr) {
            throw waveformErr
          }

          // Also load the file for playback
          try {
            await loadAudioFile(lastSamplePath)
          } catch (audioErr) {
            // Don't throw - waveform is more important than playback
          }
          
          // Show success notification
          showSuccess(
            "Recording Complete",
            `Sample recorded successfully. Press spacebar to play.`
          )
          
          // Recording completed successfully - update state
          setIsRecording(false)
          await disarm() // Return to idle state after successful recording

        } catch (waveformError) {
          showError(
            "Waveform Loading Failed",
            `Could not load waveform visualization: ${waveformError}. Recording saved successfully.`
          )

          // Even if waveform loading failed, recording was successful
          setIsRecording(false)
          await disarm() // Return to idle state after successful recording
        }
      } catch (error) {
        showError(
          "Recording Failed",
          `Could not complete recording: ${error}. Check your audio interface and try again.`
        )
        
        // Only reset state on actual recording failure
        setIsRecording(false)
        await disarm() // Return to idle state on recording failure
      } finally {
        // Ensure real-time visualization is stopped (safety cleanup)
        stopRealTimeVisualization()
      }
    } else {
      // Range recording
      try {
        startRecording() // Update state to recording
        setIsRecording(true)
        
        // Clear old waveform data
        clearWaveform()
        
        // Start real-time waveform visualization via Tauri channels
        await startRealTimeVisualization()
        
        // Clear any previous progress
        setRecordingProgress(null)
        
        // Check if velocity layers are enabled (more than 1 velocity)
        const useVelocityLayers = velocityLayers.length > 1

        if (useVelocityLayers) {
          // Use the new Epic 4 command for velocity layers
          await recordRangeWithVelocityLayers(
            parseInt(startNote),
            parseInt(endNote),
            velocityLayers,  // Pass all velocity layers
            selectedDuration[0],
            capturedOutputDirectory,
            capturedSampleName,
            capturedExportFormat,
            capturedCreatorName,
            capturedInstrumentDescription,
            noteToNoteDelay,
            layerToLayerDelay
          )
        } else {
          // Use the original command for single velocity
          await recordRange(
            parseInt(startNote),
            parseInt(endNote),
            velocityLayers[0] || selectedVelocity[0],  // Use first velocity layer if set
            selectedDuration[0],
            capturedOutputDirectory,
            capturedSampleName,
            capturedExportFormat,
            capturedCreatorName,
            capturedInstrumentDescription
          )
        }

        // Stop real-time visualization immediately after range recording completes
        stopRealTimeVisualization()
        
        // Show success notification
        showSuccess(
          "Range Recording Complete",
          `All samples recorded successfully.`
        )
        
        // Recording completed successfully - update state
        setIsRecording(false)
        await disarm() // Return to idle state after successful recording

      } catch (error) {
        showError(
          "Range Recording Failed",
          `Could not complete range recording: ${error}. Check your audio interface and try again.`
        )
        
        // Only reset state on actual recording failure
        setIsRecording(false)
        await disarm() // Return to idle state on recording failure
      } finally {
        // Ensure real-time visualization is stopped (safety cleanup)
        stopRealTimeVisualization()
      }
    }
  }

  const handleStopAndPlay = async () => {
    // Capture state values immediately to avoid race conditions
    const capturedSampleName = sampleName
    const capturedOutputDirectory = outputDirectory

    try {
      // Stop recording immediately
      setIsRecording(false)
      // Note: Don't disarm here - we want to stay in playback mode

      // Stop real-time visualization
      stopRealTimeVisualization()

      // Get the recorded file path immediately
      let lastSamplePath: string
      try {
        lastSamplePath = await getLastRecordedSamplePath(capturedOutputDirectory, capturedSampleName)
      } catch (pathError) {
        showError(
          "File Not Found",
          `Could not locate the recorded sample. Check your output directory: ${capturedOutputDirectory}`
        )
        return
      }

      // Load for playback FIRST (immediate audio feedback)
      await loadAudioFile(lastSamplePath)

      // Start playback immediately (Ableton-style)
      await togglePlayPause()

      // Transition waveform in background (visual feedback)
      setLastRecordedFile(lastSamplePath)
      await transitionToFilePlayback(lastSamplePath)

    } catch (error) {
      // Fallback to regular stop behavior
      setIsRecording(false)
      await disarm() // Return to idle state on failure
      stopRealTimeVisualization()
    }
  }

  const handlePreview = async () => {
    try {
      // Temporarily arm for preview
      await startPreview()

      await previewNote(
        parseInt(selectedNote),
        selectedVelocity[0],
        selectedDuration[0]
      )

      // Stop preview after a delay
      setTimeout(async () => {
        await stopPreview()
      }, selectedDuration[0] + 1000) // Duration + 1 second buffer

    } catch (error) {
      await stopPreview() // Cleanup on error
      showError(
        "Preview Failed",
        `Could not preview note: ${error}. Check your MIDI and audio connections.`
      )
    }
  }

  const handleSelectDirectory = async () => {
    try {
      const directory = await selectOutputDirectory()
      setOutputDirectory(directory)
    } catch (error) {
      // User cancelled directory selection
    }
  }

  const handleLoadFile = async () => {
    try {
      const filePath = await selectAudioFile()

      // Stop any current playback first
      if (isPlaying) {
        await togglePlayPause()
      }

      // Load the file for playback
      await loadAudioFile(filePath)

      // Load the waveform
      await loadWaveform(filePath)

      // Update state to show the loaded file
      setLastRecordedFile(filePath)

      showSuccess("File Loaded", `Successfully loaded: ${filePath.split('/').pop()}`)

    } catch (error) {
      showError("Load Failed", `Could not load audio file: ${error}`)
    }
  }


  const handleCustomizeLayers = () => {
    // For now, just cycle through preset layer configurations
    const presets = [
      [127, 100, 80, 60, 40, 20],      // 6 layers
      [127, 96, 64, 32],               // 4 layers
      [127, 80, 40],                   // 3 layers
      [127],                           // 1 layer
      [127, 100, 80, 60, 40, 20, 10], // 7 layers
    ]
    const currentIndex = presets.findIndex(preset => 
      preset.length === velocityLayers.length && 
      preset.every((val, idx) => val === velocityLayers[idx])
    )
    const nextIndex = (currentIndex + 1) % presets.length
    setVelocityLayers(presets[nextIndex])
    setSelectedVelocity([presets[nextIndex][0]]) // Reset to first velocity
  }

  const handleLoadTemplate = () => {
    // TODO: Implement template loading
  }

  const handleSaveTemplate = () => {
    // TODO: Implement template saving
  }


  // Update recording state based on backend
  const actuallyRecording = isRecording || backendRecording

  return (
    <div className="h-screen bg-gray-950 text-gray-100 flex flex-col">
      {/* Device Status Bar */}
      <DeviceStatusBar 
        onOpenSetup={() => {
          setSetupModalTab("midi")
          setSetupModalOpen(true)
        }}
        onOpenAudioSetup={() => {
          setSetupModalTab("audio-input")
          setSetupModalOpen(true)
        }}
        getMidiDeviceName={getMidiDeviceName}
        getAudioInputDeviceName={getAudioInputDeviceName}
      />
      

      <div className="flex flex-1 overflow-hidden">
        {/* Main Content */}
        <div className="flex-1 flex flex-col">
          {/* Session Status Indicator */}
          {sessionInitialized && (
            <div className="bg-green-900/20 border-b border-green-600/30 px-8 py-2">
              <div className="flex items-center space-x-2 text-sm">
                <div className="w-2 h-2 bg-green-400 rounded-full"></div>
                <span className="text-green-200">Session: {sampleName}</span>
                <span className="text-gray-400">•</span>
                <span className="text-green-200">Output: {outputDirectory}</span>
              </div>
            </div>
          )}
          
          {/* Main Recording Interface */}
          {sessionInitialized ? (
            <div className="flex-1 p-8 overflow-auto">
            <div className="max-w-5xl mx-auto space-y-8">

              {/* Recording Mode Selection */}
              <Card className="bg-gray-900 border-gray-700">
                <CardHeader>
                  <CardTitle className="flex items-center space-x-2 text-gray-100 text-xl">
                    <Layers className="w-6 h-6 text-gray-300" />
                    <span>Recording Mode</span>
                  </CardTitle>
                </CardHeader>
                <CardContent>
                  <Tabs value={recordingMode} onValueChange={setRecordingMode} className="w-full">
                    <TabsList className="grid w-full grid-cols-2 bg-gray-800">
                      <TabsTrigger value="single" className="data-[state=active]:bg-gray-700 text-gray-100">
                        Single Note
                      </TabsTrigger>
                      <TabsTrigger value="range" className="data-[state=active]:bg-gray-700 text-gray-100">
                        Range Recording
                      </TabsTrigger>
                    </TabsList>
                    <TabsContent value="single" className="mt-4">
                      <div className="space-y-4">
                        <div>
                          <Label className="text-gray-200">MIDI Note</Label>
                          <Select value={selectedNote} onValueChange={setSelectedNote}>
                            <SelectTrigger className="mt-1 bg-gray-800 border-gray-600 text-gray-100">
                              <SelectValue />
                            </SelectTrigger>
                            <SelectContent className="bg-gray-800 border-gray-600">
                              <SelectItem value="60" className="text-gray-100 hover:bg-gray-700">
                                C4 (60)
                              </SelectItem>
                              <SelectItem value="61" className="text-gray-100 hover:bg-gray-700">
                                C#4 (61)
                              </SelectItem>
                              <SelectItem value="62" className="text-gray-100 hover:bg-gray-700">
                                D4 (62)
                              </SelectItem>
                              <SelectItem value="63" className="text-gray-100 hover:bg-gray-700">
                                D#4 (63)
                              </SelectItem>
                              <SelectItem value="64" className="text-gray-100 hover:bg-gray-700">
                                E4 (64)
                              </SelectItem>
                              <SelectItem value="65" className="text-gray-100 hover:bg-gray-700">
                                F4 (65)
                              </SelectItem>
                              <SelectItem value="66" className="text-gray-100 hover:bg-gray-700">
                                F#4 (66)
                              </SelectItem>
                              <SelectItem value="67" className="text-gray-100 hover:bg-gray-700">
                                G4 (67)
                              </SelectItem>
                              <SelectItem value="68" className="text-gray-100 hover:bg-gray-700">
                                G#4 (68)
                              </SelectItem>
                              <SelectItem value="69" className="text-gray-100 hover:bg-gray-700">
                                A4 (69)
                              </SelectItem>
                              <SelectItem value="70" className="text-gray-100 hover:bg-gray-700">
                                A#4 (70)
                              </SelectItem>
                              <SelectItem value="71" className="text-gray-100 hover:bg-gray-700">
                                B4 (71)
                              </SelectItem>
                              <SelectItem value="72" className="text-gray-100 hover:bg-gray-700">
                                C5 (72)
                              </SelectItem>
                            </SelectContent>
                          </Select>
                        </div>
                      </div>
                    </TabsContent>
                    <TabsContent value="range" className="mt-4">
                      <div className="grid grid-cols-2 gap-4">
                        <div>
                          <Label className="text-gray-200">Start Note</Label>
                          <Select value={startNote} onValueChange={setStartNote}>
                            <SelectTrigger className="mt-1 bg-gray-800 border-gray-600 text-gray-100">
                              <SelectValue />
                            </SelectTrigger>
                            <SelectContent className="bg-gray-800 border-gray-600">
                              <SelectItem value="36" className="text-gray-100 hover:bg-gray-700">
                                C2 (36)
                              </SelectItem>
                              <SelectItem value="48" className="text-gray-100 hover:bg-gray-700">
                                C3 (48)
                              </SelectItem>
                              <SelectItem value="60" className="text-gray-100 hover:bg-gray-700">
                                C4 (60)
                              </SelectItem>
                            </SelectContent>
                          </Select>
                        </div>
                        <div>
                          <Label className="text-gray-200">End Note</Label>
                          <Select value={endNote} onValueChange={setEndNote}>
                            <SelectTrigger className="mt-1 bg-gray-800 border-gray-600 text-gray-100">
                              <SelectValue />
                            </SelectTrigger>
                            <SelectContent className="bg-gray-800 border-gray-600">
                              <SelectItem value="72" className="text-gray-100 hover:bg-gray-700">
                                C5 (72)
                              </SelectItem>
                              <SelectItem value="84" className="text-gray-100 hover:bg-gray-700">
                                C6 (84)
                              </SelectItem>
                              <SelectItem value="96" className="text-gray-100 hover:bg-gray-700">
                                C7 (96)
                              </SelectItem>
                            </SelectContent>
                          </Select>
                        </div>
                      </div>
                    </TabsContent>
                  </Tabs>
                </CardContent>
              </Card>

              {/* Duration and Velocity */}
              <div className="grid md:grid-cols-2 gap-6">
                <Card className="bg-gray-900 border-gray-700">
                  <CardHeader>
                    <CardTitle className="flex items-center space-x-2 text-gray-100 text-lg">
                      <Clock className="w-5 h-5 text-gray-300" />
                      <span>Duration</span>
                    </CardTitle>
                  </CardHeader>
                  <CardContent>
                    <div className="space-y-4">
                      <div>
                        <Label className="text-gray-200">Sample Duration (seconds)</Label>
                        <div className="flex items-center space-x-4 mt-2">
                          <Slider 
                            value={[selectedDuration[0] / 1000]} 
                            onValueChange={(value) => setSelectedDuration([value[0] * 1000])}
                            max={10} 
                            min={0.5} 
                            step={0.1} 
                            className="flex-1" 
                          />
                          <span className="text-sm font-mono w-12 text-gray-300">
                            {(selectedDuration[0] / 1000).toFixed(1)}s
                          </span>
                        </div>
                      </div>
                      <div className="flex items-center space-x-2">
                        <Switch 
                          id="auto-detect" 
                          checked={autoDetectSilence}
                          onCheckedChange={setAutoDetectSilence}
                        />
                        <Label htmlFor="auto-detect" className="text-sm text-gray-200">
                          Auto-detect silence
                        </Label>
                      </div>
                    </div>
                  </CardContent>
                </Card>

                <Card className="bg-gray-900 border-gray-700">
                  <CardHeader>
                    <CardTitle className="flex items-center space-x-2 text-gray-100 text-lg">
                      <Volume2 className="w-5 h-5 text-gray-300" />
                      <span>Velocity Layers</span>
                    </CardTitle>
                  </CardHeader>
                  <CardContent>
                    <div className="space-y-3">
                      <div className="flex items-center justify-between">
                        <Label className="text-gray-200">Number of Layers</Label>
                        <span className="text-sm font-mono text-gray-300">{velocityLayers.length}</span>
                      </div>
                      <div className="grid grid-cols-6 gap-2">
                        {velocityLayers.map((velocity, index) => (
                          <div key={index} className="text-center">
                            <button
                              onClick={() => {
                                setSelectedVelocity([velocity])
                              }}
                              className={`text-xs py-1 px-2 rounded cursor-pointer transition-colors ${
                                selectedVelocity[0] === velocity 
                                  ? "bg-blue-500 text-white" 
                                  : "bg-gray-200 text-gray-900 hover:bg-gray-300"
                              }`}
                            >
                              {velocity}
                            </button>
                          </div>
                        ))}
                      </div>
                      <Button
                        onClick={handleCustomizeLayers}
                        variant="outline"
                        size="sm"
                        className="w-full bg-transparent border-gray-600 text-gray-100 hover:bg-gray-800"
                      >
                        Customize Layers
                      </Button>
                    </div>
                  </CardContent>
                </Card>

                {/* Recording Delays Configuration (Epic 4) */}
                {recordingMode === "range" && velocityLayers.length > 1 && (
                  <Card className="bg-gray-900 border-gray-700">
                    <CardHeader>
                      <CardTitle className="flex items-center space-x-2 text-gray-100 text-lg">
                        <Clock className="w-5 h-5 text-gray-300" />
                        <span>Recording Delays</span>
                      </CardTitle>
                    </CardHeader>
                    <CardContent>
                      <div className="space-y-4">
                        <div>
                          <div className="flex items-center justify-between mb-2">
                            <Label className="text-gray-200 text-sm">Note-to-Note Delay</Label>
                            <span className="text-sm font-mono text-gray-300">{noteToNoteDelay}ms</span>
                          </div>
                          <Slider
                            value={[noteToNoteDelay]}
                            onValueChange={(value) => setNoteToNoteDelay(value[0])}
                            min={50}
                            max={1000}
                            step={50}
                            className="w-full"
                          />
                          <p className="text-xs text-gray-400 mt-1">
                            Delay between recording each note
                          </p>
                        </div>
                        <div>
                          <div className="flex items-center justify-between mb-2">
                            <Label className="text-gray-200 text-sm">Layer-to-Layer Delay</Label>
                            <span className="text-sm font-mono text-gray-300">{layerToLayerDelay}ms</span>
                          </div>
                          <Slider
                            value={[layerToLayerDelay]}
                            onValueChange={(value) => setLayerToLayerDelay(value[0])}
                            min={100}
                            max={2000}
                            step={100}
                            className="w-full"
                          />
                          <p className="text-xs text-gray-400 mt-1">
                            Delay between velocity layers
                          </p>
                        </div>
                      </div>
                    </CardContent>
                  </Card>
                )}
              </div>

              {/* Waveform Display */}
              <Card className="bg-gray-900 border-gray-700 min-h-[320px]">
                <CardHeader>
                  <CardTitle className="flex items-center justify-between text-gray-100 text-xl">
                    <span>Sample Waveform</span>
                    <div className="flex items-center space-x-3">
                      <Button
                        onClick={handleLoadFile}
                        variant="outline"
                        size="sm"
                        className="border-gray-600 text-gray-100 hover:bg-gray-800 bg-transparent"
                      >
                        <FolderOpen className="w-4 h-4 mr-2" />
                        Load File
                      </Button>
                      {waveformData && lastRecordedFile && (
                        <div className="flex items-center space-x-2 text-sm text-gray-400">
                          <span>Duration: {waveformData.duration.toFixed(2)}s</span>
                          <span>•</span>
                          <span>{lastRecordedFile.split('/').pop()}</span>
                        </div>
                      )}
                    </div>
                  </CardTitle>
                </CardHeader>
                <CardContent>
                  <WaveformDisplay
                    waveformData={waveformData}
                    isLoading={waveformLoading}
                    isTransitioning={waveformTransitioning}
                    error={waveformError}
                    fileName={lastRecordedFile?.split('/').pop()}
                    duration={waveformData ? `${waveformData.duration.toFixed(2)}s` : undefined}
                    isPlaying={isPlaying}
                    playbackPosition={playbackPosition}
                    onPlayPause={togglePlayPause}
                    onSeek={seek}
                    realtimeVizChunks={realtimeVizChunks}
                    isRecording={isRealTimeRecording}
                  />
                </CardContent>
              </Card>

              {/* Intelligent Detection Controls */}
              {lastRecordedFile && (
                <IntelligentDetectionControls
                  audioFilePath={lastRecordedFile}
                  onTrimmingComplete={(trimmedPath) => {
                    // Optionally load the trimmed file into the waveform display
                    transitionToFilePlayback(trimmedPath, 500)
                  }}
                  isVisible={!!lastRecordedFile}
                />
              )}

              {/* Quality Validation Dashboard - Hidden for v0.1.0 (uses mock data, backend not implemented) */}
              {/* {lastRecordedFile && (
                <QualityValidationDashboard
                  audioFilePath={lastRecordedFile}
                  onValidationComplete={(result) => {
                    const scoreMessage = `Quality score: ${result.metrics.overall_score.toFixed(1)}/10.0`
                    if (result.metrics.overall_score >= 7.0) {
                      showSuccess('Quality Analysis Complete', scoreMessage)
                    } else {
                      showError('Quality Analysis Complete', scoreMessage)
                    }
                  }}
                  isVisible={!!lastRecordedFile}
                />
              )} */}

              {/* Recording Controls */}
              <Card className="bg-gray-900 border-gray-700">
                <CardContent className="pt-8">
                  {/* Professional Audio Monitoring */}
                  {showLevelMeter && (
                    <div className="mb-6">
                      <div className="p-4 bg-gray-800 rounded-lg border border-blue-600/30">
                        <div className="mb-2">
                          <h3 className="text-sm font-semibold text-blue-200">PROFESSIONAL AUDIO MONITORING</h3>
                          <p className="text-xs text-gray-400">Industry-standard VU, PPM, and Digital Peak meters with professional gain staging</p>
                        </div>
                        <ProfessionalMeters isVisible={showLevelMeter} />
                      </div>
                    </div>
                  )}

                  {/* Real-time Lock-Free Meters */}
                  {(showLevelMeter || isRecording) && (
                    <div className="mt-4">
                      <div className="mb-2">
                        <h3 className="text-sm font-semibold">Real-time Level Meters (Lock-Free)</h3>
                        <p className="text-xs text-gray-400">Professional 60fps meters with zero audio dropouts</p>
                      </div>
                      <RealtimeMeters isActive={showLevelMeter || isRecording} />
                    </div>
                  )}

                  <div className="flex items-center justify-between">
                    <div className="flex items-center space-x-4">
                      <div className="flex items-center space-x-4">
                        {/* ARM/DISARM Button */}
                        {canArm && (
                          <Button
                            onClick={arm}
                            variant="outline"
                            size="lg"
                            className="bg-yellow-900/50 border-yellow-600 text-yellow-200 hover:bg-yellow-800/50"
                          >
                            🎤 ARM TO RECORD
                          </Button>
                        )}

                        {canDisarm && (
                          <Button
                            onClick={disarm}
                            variant="outline"
                            size="lg"
                            className="bg-yellow-900/50 border-yellow-600 text-yellow-200 hover:bg-yellow-800/50"
                          >
                            DISARM
                          </Button>
                        )}

                        {/* Preview Button */}
                        <Button
                          onClick={handlePreview}
                          variant="outline"
                          size="lg"
                          className="bg-gray-900 border-gray-600 text-gray-100 hover:bg-gray-800"
                          disabled={actuallyRecording}
                        >
                          <Play className="w-5 h-5 mr-2" />
                          Preview
                        </Button>

                        {/* Record Button - Ableton-style seamless record/stop */}
                        <Button
                          onClick={actuallyRecording ? handleStopAndPlay : handleRecord}
                          size="lg"
                          className={`${actuallyRecording ? "bg-red-600 hover:bg-red-700 text-white" : "bg-gray-200 hover:bg-gray-300 text-gray-900"}`}
                          disabled={!canRecord && !actuallyRecording}
                        >
                          {actuallyRecording ? (
                            <>
                              <Square className="w-5 h-5 mr-2" />
                              {recordingMode === "range" ? "Stop & Export" : "Stop & Play"}
                            </>
                          ) : (
                            <>
                              <Play className="w-5 h-5 mr-2" />
                              {recordingMode === "range" ? "Start Range Recording" : "Start Recording"}
                            </>
                          )}
                        </Button>
                        {actuallyRecording && (
                          <div className="flex flex-col space-y-2">
                            <div className="flex items-center justify-between">
                              <div className="flex items-center space-x-2">
                                <div className="w-2 h-2 bg-red-500 rounded-full animate-pulse"></div>
                                <span className="text-sm text-gray-400">
                                  {recordingMode === "range" ? "Recording range..." : "Recording..."}
                                </span>
                              </div>
                              {recordingMode === "range" && (
                                <Button
                                  onClick={async () => {
                                    try {
                                      await cancelRecording()
                                      setRecordingProgress(null)
                                      showSuccess('Recording Cancelled', 'The range recording has been cancelled')
                                    } catch (err) {
                                      showError('Cancellation Failed', 'Unable to cancel the recording')
                                    }
                                  }}
                                  variant="ghost"
                                  size="sm"
                                  className="text-red-400 hover:text-red-300 hover:bg-red-950"
                                >
                                  <X className="w-4 h-4 mr-1" />
                                  Cancel
                                </Button>
                              )}
                            </div>
                            {recordingProgress && recordingMode === "range" && (
                              <div className="space-y-1">
                                <div className="flex justify-between text-xs text-gray-400">
                                  <span>
                                    Note {recordingProgress.noteName || recordingProgress.note} 
                                    {recordingProgress.velocity && ` • Velocity ${recordingProgress.velocity}`}
                                    {recordingProgress.layer && ` • Layer ${recordingProgress.layer}/${recordingProgress.totalLayers}`}
                                  </span>
                                  <span>{recordingProgress.current}/{recordingProgress.total} samples</span>
                                </div>
                                <div className="w-full bg-gray-700 rounded-full h-2">
                                  <div 
                                    className="bg-blue-500 h-2 rounded-full transition-all duration-300"
                                    style={{ width: `${recordingProgress.percent}%` }}
                                  />
                                </div>
                                <div className="text-xs text-gray-500 text-center">
                                  {recordingProgress.percent.toFixed(1)}% complete
                                </div>
                              </div>
                            )}
                          </div>
                        )}
                      </div>
                    </div>
                    <div className="text-right">
                      <div className={`text-sm ${getStateColor()}`}>{getStateDisplayText()}</div>
                      <div className="text-xs text-gray-300">
                        {recordingMode === 'single' 
                          ? 'Single note' 
                          : `${velocityLayers.length} velocity layers • Range recording`
                        }
                        {!canRecord && recordingState === 'idle' && ' • ARM to record'}
                      </div>
                    </div>
                  </div>
                </CardContent>
              </Card>
            </div>
            </div>
          ) : (
            <div className="flex-1 flex items-center justify-center">
              <div className="text-center space-y-4">
                <div className="w-16 h-16 bg-gray-600 rounded-full flex items-center justify-center mx-auto">
                  <Play className="w-8 h-8 text-gray-400" />
                </div>
                <div>
                  <h3 className="text-xl font-semibold text-gray-300 mb-2">Session Initialization Required</h3>
                  <p className="text-gray-500">Please complete the setup wizard to start recording.</p>
                </div>
              </div>
            </div>
          )}
        </div>

        {/* Right Sidebar */}
        <div className="w-80 bg-gray-900 border-l border-gray-700 p-6 overflow-auto">
          <div className="space-y-6">
            <div>
              <h3 className="text-lg font-semibold mb-4 text-gray-100">Output Settings</h3>
              <div className="space-y-4">
                <div>
                  <Label className="text-gray-200">Sample Name</Label>
                  <Input 
                    value={sampleName}
                    onChange={(e) => setSampleName(e.target.value)}
                    className="mt-1 bg-gray-800 border-gray-600 text-gray-100" 
                  />
                  <p className="text-xs text-gray-400 mt-1">Example: Roland_JP8000_C4_60_vel127.wav</p>
                </div>

                <div>
                  <Label className="text-gray-200">Output Directory</Label>
                  <div className="flex mt-1">
                    <Input
                      value={outputDirectory}
                      onChange={(e) => setOutputDirectory(e.target.value)}
                      className="bg-gray-800 border-gray-600 text-gray-100 rounded-r-none"
                    />
                    <Button
                      onClick={handleSelectDirectory}
                      variant="outline"
                      className="rounded-l-none bg-transparent border-gray-600 text-gray-100 hover:bg-gray-800"
                    >
                      <FolderOpen className="w-4 h-4" />
                    </Button>
                  </div>
                </div>

                <div>
                  <Label className="text-gray-200">Export Format</Label>
                  <Select value={exportFormat} onValueChange={setExportFormat}>
                    <SelectTrigger className="mt-1 bg-gray-800 border-gray-600 text-gray-100">
                      <SelectValue />
                    </SelectTrigger>
                    <SelectContent className="bg-gray-800 border-gray-600">
                      <SelectItem value="dspreset" className="text-gray-100 hover:bg-gray-700">
                        Decent Sampler (.dspreset)
                      </SelectItem>
                      <SelectItem value="wav24" className="text-gray-100 hover:bg-gray-700">
                        WAV 24-bit
                      </SelectItem>
                      <SelectItem value="wav16" className="text-gray-100 hover:bg-gray-700">
                        WAV 16-bit
                      </SelectItem>
                      <SelectItem value="wav32" className="text-gray-100 hover:bg-gray-700">
                        WAV 32-bit Float
                      </SelectItem>
                      <SelectItem value="sfz" className="text-gray-100 hover:bg-gray-700">
                        SFZ (.sfz)
                      </SelectItem>
                      <SelectItem value="all" className="text-gray-100 hover:bg-gray-700">
                        All Formats
                      </SelectItem>
                    </SelectContent>
                  </Select>
                </div>

                <div>
                  <Label className="text-gray-200">Creator Name</Label>
                  <Input
                    value={creatorName}
                    onChange={(e) => setCreatorName(e.target.value)}
                    placeholder="Your name"
                    className="mt-1 bg-gray-800 border-gray-600 text-gray-100 placeholder:text-gray-500"
                  />
                </div>

                <div>
                  <Label className="text-gray-200">Instrument Description</Label>
                  <Input
                    value={instrumentDescription}
                    onChange={(e) => setInstrumentDescription(e.target.value)}
                    placeholder="Brief description"
                    className="mt-1 bg-gray-800 border-gray-600 text-gray-100 placeholder:text-gray-500"
                  />
                  <p className="text-xs text-gray-400 mt-1">
                    Metadata will be embedded in the generated instrument file.
                  </p>
                </div>
              </div>
            </div>

            <div>
              <h3 className="text-lg font-semibold mb-4 text-gray-100">Sample Detection</h3>
              <div className="space-y-4">
                <div className="flex items-center space-x-2">
                  <Switch 
                    id="auto-detection" 
                    checked={autoDetectSilence}
                    onCheckedChange={setAutoDetectSilence}
                  />
                  <Label htmlFor="auto-detection" className="text-gray-200">
                    Enable Auto-Detection
                  </Label>
                </div>

                <div>
                  <div className="flex items-center justify-between mb-2">
                    <Label className="text-gray-200">Detection Threshold</Label>
                    <span className="text-sm font-mono text-gray-300">{detectionThreshold[0]} dB</span>
                  </div>
                  <Slider 
                    value={detectionThreshold} 
                    onValueChange={setDetectionThreshold}
                    max={-10} 
                    min={-60} 
                    step={1} 
                    className="w-full" 
                  />
                </div>
              </div>
            </div>

            <div>
              <h3 className="text-lg font-semibold mb-4 text-gray-100">Quick Actions</h3>
              <div className="space-y-2">
                <Button
                  onClick={handleLoadTemplate}
                  variant="outline"
                  className="w-full justify-start bg-transparent border-gray-600 text-gray-100 hover:bg-gray-800"
                >
                  Load Template
                </Button>
                <Button
                  onClick={handleSaveTemplate}
                  variant="outline"
                  className="w-full justify-start bg-transparent border-gray-600 text-gray-100 hover:bg-gray-800"
                >
                  Save as Template
                </Button>
              </div>
            </div>
          </div>
        </div>
      </div>
      
      {/* Setup Modal */}
      <SetupModal
        isOpen={setupModalOpen}
        onClose={handleCloseSetup}
        initialTab={setupModalTab}
        midiDevices={midiDevices}
        audioInputDevices={audioInputDevices}
        audioOutputDevices={audioOutputDevices}
        midiLoading={midiLoading}
        selectedMidiDevice={selectedMidiDevice}
        selectedAudioInput={selectedAudioInput}
        selectedAudioOutput={selectedAudioOutput}
        setSelectedAudioInput={handleAudioInputChange}
        setSelectedAudioOutput={handleAudioOutputChange}
        handleMidiDeviceChange={handleMidiDeviceChange}
        midiConnected={midiConnected}
      />
      
      {/* Professional Notification System */}
      <NotificationDisplay
        notifications={notifications}
        onRemove={removeNotification}
      />
      
      {/* Professional Recording Countdown Overlay */}
      <RecordingCountdown
        isCountingDown={countdownState.isCountingDown}
        countdownValue={countdownState.countdownValue}
        totalDuration={countdownState.totalDuration}
      />
      
      {/* Professional Session Initialization Wizard */}
      <SessionInitializationWizard
        isOpen={showSessionWizard}
        onClose={() => setShowSessionWizard(false)}
        onComplete={async (sessionData) => {
          // Create full project path: outputDirectory/projectName
          const fullProjectPath = `${sessionData.outputDirectory}/${sessionData.projectName}`

          try {
            // Create the project directory
            await createDirectory(fullProjectPath)
          } catch (error) {
            // Continue anyway - the recording system will try to create it
          }

          // Apply session data to the recording interface
          setSampleName(sessionData.projectName)
          setOutputDirectory(fullProjectPath)

          // Set professional defaults for session-based recording
          setExportFormat("all") // Default to "All Formats" for professional workflow

          // Session is now initialized
          setSessionInitialized(true)
          setShowSessionWizard(false)
        }}
      />
    </div>
  )
}