import { useState } from "react"
import { Button } from "@/components/ui/button"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select"
import { Slider } from "@/components/ui/slider"
import { Switch } from "@/components/ui/switch"
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs"
import { Progress } from "@/components/ui/progress"
import { DeviceManager } from "@/components/DeviceManager"
import { WaveformDisplay } from "@/components/WaveformDisplay"
import { useRecording, useFileSystem, useDeviceConnection, useWaveform, useLoopDetection } from "@/hooks/useTauri"
import {
  Play,
  Square,
  Clock,
  Layers,
  FolderOpen,
  Volume2,
} from "lucide-react"


export default function App() {
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
  const [exportFormat, setExportFormat] = useState("dspreset")
  const [creatorName, setCreatorName] = useState("")
  const [instrumentDescription, setInstrumentDescription] = useState("")
  const [recordingMode, setRecordingMode] = useState("single")
  const [startNote, setStartNote] = useState("36") // C2
  const [endNote, setEndNote] = useState("84") // C6
  
  // Modal state
  const [setupModalOpen, setSetupModalOpen] = useState(false)
  
  // Tauri hooks
  const { recordSample, recordRange, previewNote, isRecording: backendRecording } = useRecording()
  const { selectOutputDirectory } = useFileSystem()
  const { testMidiConnection } = useDeviceConnection()
  const { waveformData, isLoading: waveformLoading, error: waveformError, loadWaveform } = useWaveform()
  const { getLastRecordedSamplePath } = useLoopDetection()
  
  // Track the last recorded file
  const [lastRecordedFile, setLastRecordedFile] = useState<string | null>(null)

  // Handlers
  const handleMidiPanic = () => {
    console.log("MIDI Panic triggered")
  }

  const handleOpenSetup = () => {
    setSetupModalOpen(true)
    console.log("Opening setup modal")
  }
  
  const handleCloseSetup = () => {
    setSetupModalOpen(false)
  }

  const handleRecord = async () => {
    if (recordingMode === "single") {
      try {
        setIsRecording(true)
        const result = await recordSample(
          parseInt(selectedNote),
          selectedVelocity[0],
          selectedDuration[0],
          outputDirectory,
          sampleName,
          exportFormat,
          creatorName,
          instrumentDescription
        )
        console.log("Recording complete:", result)
        
        // Load waveform after successful recording
        try {
          const lastSamplePath = await getLastRecordedSamplePath(outputDirectory, sampleName)
          console.log("Loading waveform for:", lastSamplePath)
          setLastRecordedFile(lastSamplePath)
          await loadWaveform(lastSamplePath)
        } catch (waveformError) {
          console.error("Failed to load waveform:", waveformError)
        }
      } catch (error) {
        console.error("Recording failed:", error)
        // TODO: Show error message to user
      } finally {
        setIsRecording(false)
      }
    } else {
      // Range recording
      try {
        setIsRecording(true)
        const result = await recordRange(
          parseInt(startNote),
          parseInt(endNote),
          selectedVelocity[0],
          selectedDuration[0],
          outputDirectory,
          sampleName,
          exportFormat,
          creatorName,
          instrumentDescription
        )
        console.log("Range recording complete:", result)
        // TODO: Show success message to user
      } catch (error) {
        console.error("Range recording failed:", error)
        // TODO: Show error message to user
      } finally {
        setIsRecording(false)
      }
    }
  }

  const handlePreview = async () => {
    try {
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
    } catch (error) {
      console.error("Preview failed:", error)
      alert(`Preview failed: ${error}`)
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

  const handleTestMidiConnection = async () => {
    try {
      const result = await testMidiConnection()
      console.log("Test MIDI result:", result)
    } catch (error) {
      console.error("Test MIDI failed:", error)
    }
  }

  // Update recording state based on backend
  const actuallyRecording = isRecording || backendRecording

  return (
    <div className="h-screen bg-gray-950 text-gray-100 flex flex-col">
      {/* Device Manager handles title bar and device connections */}
      <DeviceManager onMidiPanic={handleMidiPanic} onOpenSetup={handleOpenSetup} />
      

      <div className="flex flex-1 overflow-hidden">
        {/* Main Content */}
        <div className="flex-1 flex flex-col">
          {/* Step Content */}
          <div className="flex-1 p-6 overflow-auto">
            <div className="max-w-4xl mx-auto space-y-6">
              {/* Device interfaces are now handled by DeviceManager component above */}

              {/* Sample Type Selection */}
              <Card className="bg-gray-900 border-gray-700">
                <CardHeader>
                  <CardTitle className="flex items-center space-x-2 text-gray-100">
                    <Layers className="w-5 h-5 text-gray-300" />
                    <span>Sample Type</span>
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
                    <CardTitle className="flex items-center space-x-2 text-gray-100">
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
                    <CardTitle className="flex items-center space-x-2 text-gray-100">
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
              <Card className="bg-gray-900 border-gray-700">
                <CardHeader>
                  <CardTitle className="flex items-center justify-between text-gray-100">
                    <span>Sample Waveform</span>
                    {waveformData && lastRecordedFile && (
                      <div className="flex items-center space-x-2 text-sm text-gray-400">
                        <span>Duration: {waveformData.duration.toFixed(2)}s</span>
                        <span>•</span>
                        <span>{lastRecordedFile.split('/').pop()}</span>
                      </div>
                    )}
                  </CardTitle>
                </CardHeader>
                <CardContent>
                  <WaveformDisplay
                    waveformData={waveformData}
                    isLoading={waveformLoading}
                    error={waveformError}
                    fileName={lastRecordedFile?.split('/').pop()}
                    duration={waveformData ? `${waveformData.duration.toFixed(2)}s` : undefined}
                    onSeek={(position) => {
                      console.log("Seek to position:", position)
                      // TODO: Implement actual seek functionality
                    }}
                  />
                </CardContent>
              </Card>

              {/* Recording Controls */}
              <Card className="bg-gray-900 border-gray-700">
                <CardContent className="pt-6">
                  <div className="flex items-center justify-between">
                    <div className="flex items-center space-x-4">
                      <div className="flex items-center space-x-4">
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
                        <Button
                          onClick={handleRecord}
                          size="lg"
                          className={`${actuallyRecording ? "bg-red-600 hover:bg-red-700 text-white" : "bg-gray-200 hover:bg-gray-300 text-gray-900"}`}
                          disabled={actuallyRecording && recordingMode === "range"} // Can't stop range recording from UI
                        >
                          {actuallyRecording ? (
                            <>
                              <Square className="w-5 h-5 mr-2" />
                              {recordingMode === "range" ? "Recording Range..." : "Stop Recording"}
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
                      <div className="text-sm text-gray-400">Ready to record</div>
                      <div className="text-xs text-gray-300">6 velocity layers • Single note</div>
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
                      <SelectItem value="decentsampler" className="text-gray-100 hover:bg-gray-700">
                        Decent Sampler (.dspreset)
                      </SelectItem>
                      <SelectItem value="wav" className="text-gray-100 hover:bg-gray-700">
                        WAV Files Only
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
                <Button
                  onClick={handleTestMidiConnection}
                  variant="outline"
                  className="w-full justify-start bg-transparent border-gray-600 text-gray-100 hover:bg-gray-800"
                >
                  Test MIDI Connection
                </Button>
              </div>
            </div>
          </div>
        </div>
      </div>
      
      {/* Setup Modal */}
      {setupModalOpen && (
        <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
          <div className="bg-gray-900 border border-gray-700 rounded-lg p-6 max-w-md w-full">
            <h2 className="text-xl font-semibold text-gray-100 mb-4">Setup</h2>
            <p className="text-gray-300 mb-4">Configure your MIDI and audio settings here.</p>
            <Button
              onClick={handleCloseSetup}
              className="w-full"
            >
              Close
            </Button>
          </div>
        </div>
      )}
    </div>
  )
}