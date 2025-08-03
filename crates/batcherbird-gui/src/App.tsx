import { useState, useEffect } from "react"
import { Button } from "@/components/ui/button"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select"
import { Slider } from "@/components/ui/slider"
import { Switch } from "@/components/ui/switch"
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs"
import { Progress } from "@/components/ui/progress"
import { DeviceStatusBar } from "@/components/DeviceStatusBar"
import { SetupModal } from "@/components/SetupModal"
import { WaveformDisplay } from "@/components/WaveformDisplay"
import { ProfessionalMeters } from "@/components/ProfessionalMeters"
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
  const [recordingProgress] = useState(0)
  const [velocityLayers, setVelocityLayers] = useState([127, 100, 80, 60, 40, 20])
  
  // Form state
  const [selectedNote, setSelectedNote] = useState("60") // C4
  const [selectedVelocity, setSelectedVelocity] = useState([127]) // Default to max velocity
  const [selectedDuration, setSelectedDuration] = useState([2420]) // 2.42s in ms
  const [autoDetectSilence, setAutoDetectSilence] = useState(true)
  const [detectionThreshold, setDetectionThreshold] = useState([-35])
  const [sampleName, setSampleName] = useState("Roland-EM1018")
  const [outputDirectory, setOutputDirectory] = useState("/Users/dryan/Desktop/Batch")
  const [exportFormat, setExportFormat] = useState("wav16")
  const [creatorName, setCreatorName] = useState("")
  const [instrumentDescription, setInstrumentDescription] = useState("")
  const [recordingMode, setRecordingMode] = useState("single")
  const [startNote, setStartNote] = useState("36") // C2
  const [endNote, setEndNote] = useState("84") // C6
  
  // Modal state
  const [setupModalOpen, setSetupModalOpen] = useState(false)
  const [setupModalTab, setSetupModalTab] = useState("midi")
  
  // Tauri hooks
  const { recordSample, recordRange, previewNote, isRecording: backendRecording } = useRecording()
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

  // Check if session initialization is needed
  useEffect(() => {
    if (!sessionInitialized && !showSessionWizard) {
      console.log('🎛️ Session not initialized, showing setup wizard')
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
        console.log('🔌 Auto-connecting to saved MIDI device:', midiDevices[parseInt(deviceIndex)])
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
          console.log('🎹 Spacebar: Toggle play/pause')
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
      console.warn("Cannot record - not armed")
      return
    }

    // Professional countdown before recording (2 seconds like Pro Tools)
    startCountdown()
    
    try {
      await startCountdownTimer(2000) // 2 second countdown
    } catch (error) {
      console.log("Countdown cancelled")
      cancelCountdown()
      return
    }

    // Capture state values to avoid race conditions during recording
    const capturedOutputDirectory = outputDirectory
    const capturedSampleName = sampleName
    const capturedExportFormat = exportFormat
    const capturedCreatorName = creatorName
    const capturedInstrumentDescription = instrumentDescription
    
    console.log('🎤 Recording with values:', {
      capturedOutputDirectory,
      capturedSampleName,
      sessionInitialized
    })

    if (recordingMode === "single") {
      console.log('📝 App.tsx: Entering single recording mode')
      try {
        console.log('📝 App.tsx: About to start recording state transition')
        startRecording() // Update state to recording
        setIsRecording(true)
        console.log('📝 App.tsx: Recording state set to true')
        
        // Clear old waveform data
        clearWaveform()
        console.log('📝 App.tsx: Waveform cleared')
        
        // Start real-time waveform visualization via Tauri channels
        console.log('🎤 Starting real-time visualization...')
        await startRealTimeVisualization()
        console.log('🎤 Real-time visualization started, isRealTimeRecording:', isRealTimeRecording)
        
        console.log('🚀 App.tsx: About to call recordSample hook function')
        console.log('🚀 App.tsx: recordSample function type:', typeof recordSample)
        
        const result = await recordSample(
          parseInt(selectedNote),
          selectedVelocity[0],
          selectedDuration[0],
          capturedOutputDirectory,
          capturedSampleName,
          capturedExportFormat,
          capturedCreatorName,
          capturedInstrumentDescription
        )
        console.log("🎉 App.tsx: Recording complete:", result)
        
        // Stop real-time visualization immediately after recording completes
        console.log("🔄 Transitioning from real-time to file-based waveform...")
        stopRealTimeVisualization()
        
        // Transition from recording to file playback
        try {
          console.log("🔍 Debug - Regular recording - About to call getLastRecordedSamplePath with:", { outputDirectory: capturedOutputDirectory, sampleName: capturedSampleName })
          const lastSamplePath = await getLastRecordedSamplePath(capturedOutputDirectory, capturedSampleName)
          console.log("🔄 Transitioning to file playback for:", lastSamplePath)
          console.log("Output directory:", capturedOutputDirectory)
          console.log("Sample name:", capturedSampleName)
          
          // Add small delay to ensure file is fully written
          console.log("⏱️ Adding delay to ensure file is fully written...")
          await new Promise(resolve => setTimeout(resolve, 500))
          
          setLastRecordedFile(lastSamplePath)
          
          console.log("🌊 Starting waveform transition...")
          try {
            await transitionToFilePlayback(lastSamplePath)
            console.log("✅ Waveform transition completed successfully")
          } catch (waveformErr) {
            console.error("❌ Waveform transition failed:", waveformErr)
            console.error("❌ Waveform transition error details:", JSON.stringify(waveformErr))
            throw waveformErr
          }
          
          // Also load the file for playback
          console.log("🎵 Loading audio file for playback...")
          try {
            await loadAudioFile(lastSamplePath)
            console.log("✅ Audio file loaded for playback")
          } catch (audioErr) {
            console.error("❌ Audio file loading failed:", audioErr)
            // Don't throw - waveform is more important than playback
          }
          
          console.log("✅ Smooth transition to playback mode complete")
          
          // Show success notification
          showSuccess(
            "Recording Complete",
            `Sample recorded successfully. Press spacebar to play.`
          )
          
          // Recording completed successfully - update state
          setIsRecording(false)
          await disarm() // Return to idle state after successful recording
          
        } catch (waveformError) {
          console.error("Failed to load waveform:", waveformError)
          showError(
            "Waveform Loading Failed",
            `Could not load waveform visualization: ${waveformError}. Recording saved successfully.`
          )
          
          // Even if waveform loading failed, recording was successful
          setIsRecording(false)
          await disarm() // Return to idle state after successful recording
        }
      } catch (error) {
        console.error("Recording failed:", error)
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
        
        const result = await recordRange(
          parseInt(startNote),
          parseInt(endNote),
          selectedVelocity[0],
          selectedDuration[0],
          capturedOutputDirectory,
          capturedSampleName,
          capturedExportFormat,
          capturedCreatorName,
          capturedInstrumentDescription
        )
        console.log("Range recording complete:", result)
        
        // Stop real-time visualization immediately after range recording completes
        console.log("🔄 Range recording complete, stopping visualization...")
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
        console.error("Range recording failed:", error)
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
    console.log("🎵 Stopping recording and starting playback (Ableton-style)")
    
    // Capture state values immediately to avoid race conditions
    const capturedSampleName = sampleName
    const capturedOutputDirectory = outputDirectory
    
    console.log("🔍 Debug - Captured sampleName:", capturedSampleName)
    console.log("🔍 Debug - Captured outputDirectory:", capturedOutputDirectory)
    console.log("🔍 Debug - Current sampleName:", sampleName)
    console.log("🔍 Debug - Current outputDirectory:", outputDirectory)
    
    try {
      // Stop recording immediately
      setIsRecording(false)
      // Note: Don't disarm here - we want to stay in playback mode
      
      // Stop real-time visualization
      console.log("🔄 Stopping real-time visualization...")
      stopRealTimeVisualization()
      
      // Get the recorded file path immediately
      let lastSamplePath: string
      try {
        console.log("🔍 Debug - About to call getLastRecordedSamplePath with:", { outputDirectory: capturedOutputDirectory, sampleName: capturedSampleName })
        lastSamplePath = await getLastRecordedSamplePath(capturedOutputDirectory, capturedSampleName)
        console.log("📁 Loading recorded sample:", lastSamplePath)
      } catch (pathError) {
        console.error("Failed to find recorded sample:", pathError)
        console.error("🔍 Debug - pathError details:", pathError)
        showError(
          "File Not Found",
          `Could not locate the recorded sample. Check your output directory: ${capturedOutputDirectory}`
        )
        return
      }
      
      // Load for playback FIRST (immediate audio feedback)
      await loadAudioFile(lastSamplePath)
      console.log("🎵 Audio loaded for playback")
      
      // Start playback immediately (Ableton-style)
      await togglePlayPause()
      console.log("▶️ Started immediate playback")
      
      // Transition waveform in background (visual feedback)
      setLastRecordedFile(lastSamplePath)
      await transitionToFilePlayback(lastSamplePath)
      console.log("🌊 Waveform transition completed")
      
      console.log("✅ Seamless stop→play transition complete")
      
    } catch (error) {
      console.error("Failed to stop and play:", error)
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
      
      console.log("Previewing note:", {
        note: parseInt(selectedNote),
        velocity: selectedVelocity[0],
        duration: selectedDuration[0]
      })
      
      const result = await previewNote(
        parseInt(selectedNote),
        selectedVelocity[0],
        selectedDuration[0]
      )
      console.log("Preview result:", result)
      
      // Stop preview after a delay
      setTimeout(async () => {
        await stopPreview()
      }, selectedDuration[0] + 1000) // Duration + 1 second buffer
      
    } catch (error) {
      console.error("Preview failed:", error)
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
      console.error("Directory selection failed:", error)
    }
  }

  const handleLoadFile = async () => {
    try {
      console.log("🎵 Opening file picker to load audio file...")
      const filePath = await selectAudioFile()
      console.log("📁 User selected file:", filePath)
      
      // Stop any current playback first
      if (isPlaying) {
        await togglePlayPause()
      }
      
      // Load the file for playback
      await loadAudioFile(filePath)
      console.log("🎵 Audio file loaded for playback")
      
      // Load the waveform
      await loadWaveform(filePath)
      console.log("🌊 Waveform loaded successfully")
      
      // Update state to show the loaded file
      setLastRecordedFile(filePath)
      
      showSuccess("File Loaded", `Successfully loaded: ${filePath.split('/').pop()}`)
      
    } catch (error) {
      console.error("Failed to load audio file:", error)
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
    console.log("Changed velocity layers to:", presets[nextIndex])
  }

  const handleLoadTemplate = () => {
    // TODO: Implement template loading
    console.log("Load template clicked")
  }

  const handleSaveTemplate = () => {
    // TODO: Implement template saving
    console.log("Save template clicked")
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
                          <div className="flex items-center space-x-2">
                            <div className="w-2 h-2 bg-red-500 rounded-full animate-pulse"></div>
                            <span className="text-sm text-gray-400">
                              {recordingMode === "range" ? "Recording range..." : "Recording..."}
                            </span>
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
                  {isRecording && (
                    <div className="mt-4">
                      <div className="flex items-center justify-between text-sm mb-2 text-gray-300">
                        <span>Progress</span>
                        <span>{recordingProgress}%</span>
                      </div>
                      <Progress value={recordingProgress} className="w-full" />
                    </div>
                  )}
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
          console.log('✅ Session initialization completed with data:', sessionData)
          
          // Create full project path: outputDirectory/projectName
          const fullProjectPath = `${sessionData.outputDirectory}/${sessionData.projectName}`
          
          try {
            // Create the project directory
            await createDirectory(fullProjectPath)
            console.log('📁 Project directory created:', fullProjectPath)
          } catch (error) {
            console.error('❌ Failed to create project directory:', error)
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
          
          console.log('🎤 Recording interface is now ready!')
          console.log('📁 Project directory:', fullProjectPath)
        }}
      />
    </div>
  )
}