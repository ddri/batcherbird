import { useState, useEffect } from "react"
import { Button } from "@/components/ui/button"
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select"
import { Label } from "@/components/ui/label"
import { Switch } from "@/components/ui/switch"
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs"
import { Slider } from "@/components/ui/slider"
import { Badge } from "@/components/ui/badge"
import { 
  Music, 
  Mic, 
  Volume2, 
  Settings, 
  X, 
  CheckCircle, 
  AlertCircle, 
  Zap 
} from "lucide-react"
import { 
  useDeviceConnection, 
  useAudioDeviceInfo 
} from "@/hooks/useTauri"

interface SetupModalProps {
  isOpen: boolean
  onClose: () => void
  initialTab: string
  onInputConfigChange?: (config: { mode: 'mono' | 'stereo', channels: number[] }) => void
  midiDevices: string[]
  audioInputDevices: string[]
  audioOutputDevices: string[]
  midiLoading: boolean
  selectedMidiDevice: string
  selectedAudioInput: string
  selectedAudioOutput: string
  setSelectedAudioInput: (value: string) => void
  setSelectedAudioOutput: (value: string) => void
  handleMidiDeviceChange: (value: string) => void
  midiConnected: boolean
}

export function SetupModal({ 
  isOpen, 
  onClose, 
  initialTab,
  onInputConfigChange,
  midiDevices,
  audioInputDevices,
  audioOutputDevices,
  midiLoading,
  selectedMidiDevice,
  selectedAudioInput,
  selectedAudioOutput,
  setSelectedAudioInput,
  setSelectedAudioOutput,
  handleMidiDeviceChange,
  midiConnected
}: SetupModalProps) {
  
  // Connection hooks
  const { connectMidi, testMidiConnection, sendMidiPanic } = useDeviceConnection()
  const { deviceInfo, getDeviceInfo } = useAudioDeviceInfo()
  
  // Local state (not shared)
  const [currentTab, setCurrentTab] = useState(initialTab)
  const [inputMode, setInputMode] = useState<'mono' | 'stereo'>('stereo')
  const [selectedInputChannels, setSelectedInputChannels] = useState<number[]>([0, 1])
  const [detectionThreshold, setDetectionThreshold] = useState([-35])
  const [autoDetectSilence, setAutoDetectSilence] = useState(true)

  // Update tab when initialTab changes
  useEffect(() => {
    setCurrentTab(initialTab)
  }, [initialTab])

  
  // Notify parent when input configuration changes
  useEffect(() => {
    if (onInputConfigChange) {
      onInputConfigChange({
        mode: inputMode,
        channels: selectedInputChannels
      })
    }
  }, [inputMode, selectedInputChannels, onInputConfigChange])

  const handleMidiConnect = async () => {
    if (selectedMidiDevice) {
      try {
        await connectMidi(parseInt(selectedMidiDevice))
      } catch (err) {
        console.error('Failed to connect MIDI device:', err)
      }
    }
  }
  
  const handleAudioInputChange = async (value: string) => {
    setSelectedAudioInput(value)
    
    try {
      const info = await getDeviceInfo(parseInt(value))
      
      if (info.total_channels >= 2) {
        setInputMode('stereo')
        setSelectedInputChannels([0, 1])
      } else {
        setInputMode('mono')
        setSelectedInputChannels([0])
      }
    } catch (err) {
      console.error('Failed to get device info:', err)
    }
  }

  const handleMidiTest = async () => {
    try {
      await testMidiConnection()
    } catch (err) {
      console.error('MIDI test failed:', err)
    }
  }

  const handleMidiPanic = async () => {
    try {
      await sendMidiPanic()
    } catch (err) {
      console.error('MIDI panic failed:', err)
    }
  }


  if (!isOpen) return null

  return (
    <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
      <div className="bg-gray-900 border border-gray-700 rounded-lg w-full max-w-2xl max-h-[80vh] overflow-hidden">
        {/* Header */}
        <div className="flex items-center justify-between p-6 border-b border-gray-700">
          <h2 className="text-xl font-semibold text-gray-100">Device Setup</h2>
          <Button
            onClick={onClose}
            variant="ghost"
            size="sm"
            className="text-gray-400 hover:text-gray-100"
          >
            <X className="w-4 h-4" />
          </Button>
        </div>

        {/* Content */}
        <div className="p-6 overflow-auto max-h-[calc(80vh-120px)] min-h-[350px]">
          <Tabs value={currentTab} onValueChange={setCurrentTab} className="w-full">
            <TabsList className="grid w-full grid-cols-4 bg-gray-800">
              <TabsTrigger value="midi" className="data-[state=active]:bg-gray-700 text-gray-100">
                <Music className="w-4 h-4 mr-2" />
                MIDI
              </TabsTrigger>
              <TabsTrigger value="audio-input" className="data-[state=active]:bg-gray-700 text-gray-100">
                <Mic className="w-4 h-4 mr-2" />
                Audio In
              </TabsTrigger>
              <TabsTrigger value="audio-output" className="data-[state=active]:bg-gray-700 text-gray-100">
                <Volume2 className="w-4 h-4 mr-2" />
                Audio Out
              </TabsTrigger>
              <TabsTrigger value="advanced" className="data-[state=active]:bg-gray-700 text-gray-100">
                <Settings className="w-4 h-4 mr-2" />
                Advanced
              </TabsTrigger>
            </TabsList>

            {/* MIDI Tab */}
            <TabsContent value="midi" className="mt-6 space-y-4">
              <div className="flex items-center justify-between">
                <h3 className="text-lg font-medium text-gray-100">MIDI Interface</h3>
                {midiConnected ? (
                  <Badge className="bg-green-600 text-white">
                    <CheckCircle className="w-3 h-3 mr-1" />
                    Connected
                  </Badge>
                ) : (
                  <Badge variant="secondary" className="bg-gray-600 text-white">
                    <AlertCircle className="w-3 h-3 mr-1" />
                    Disconnected
                  </Badge>
                )}
              </div>

              <div className="space-y-4">
                <div>
                  <Label className="text-gray-200">MIDI Output Device</Label>
                  <Select 
                    value={selectedMidiDevice} 
                    onValueChange={handleMidiDeviceChange}
                    disabled={midiLoading}
                  >
                    <SelectTrigger className="mt-1 bg-gray-800 border-gray-600 text-gray-100">
                      <SelectValue placeholder={midiLoading ? "Loading devices..." : "Select MIDI device"} />
                    </SelectTrigger>
                    <SelectContent className="bg-gray-800 border-gray-600">
                      {midiDevices.length === 0 ? (
                        <SelectItem value="none" disabled className="text-gray-400">
                          No MIDI devices found
                        </SelectItem>
                      ) : (
                        midiDevices.map((device, index) => (
                          <SelectItem 
                            key={index} 
                            value={index.toString()} 
                            className="text-gray-100 hover:bg-gray-700"
                          >
                            {device}
                          </SelectItem>
                        ))
                      )}
                    </SelectContent>
                  </Select>
                </div>

                <div className="flex items-center space-x-2">
                  {midiConnected ? (
                    <>
                      <div className="w-2 h-2 bg-green-400 rounded-full"></div>
                      <span className="text-sm text-gray-400">
                        {midiDevices[parseInt(selectedMidiDevice)] || "Connected"} - Active
                      </span>
                    </>
                  ) : (
                    <>
                      <div className="w-2 h-2 bg-gray-600 rounded-full"></div>
                      <span className="text-sm text-gray-400">No device connected</span>
                    </>
                  )}
                </div>

                <div className="flex space-x-2">
                  {!midiConnected && selectedMidiDevice && (
                    <Button onClick={handleMidiConnect} className="flex-1">
                      Connect MIDI Device
                    </Button>
                  )}
                  
                  {midiConnected && (
                    <>
                      <Button 
                        onClick={handleMidiTest}
                        variant="outline"
                        className="flex-1 bg-transparent border-gray-600 text-gray-100 hover:bg-gray-800"
                      >
                        Test Connection
                      </Button>
                      <Button 
                        onClick={handleMidiPanic}
                        variant="destructive"
                        className="flex items-center"
                      >
                        <Zap className="w-4 h-4 mr-2" />
                        MIDI Panic
                      </Button>
                    </>
                  )}
                </div>
              </div>
            </TabsContent>

            {/* Audio Input Tab */}
            <TabsContent value="audio-input" className="mt-6 space-y-4">
              <div className="flex items-center justify-between">
                <h3 className="text-lg font-medium text-gray-100">Audio Input</h3>
              </div>

              <div className="space-y-4">
                <div>
                  <Label className="text-gray-200">Audio Input Device</Label>
                  <Select 
                    value={selectedAudioInput} 
                    onValueChange={handleAudioInputChange}
                  >
                    <SelectTrigger className="mt-1 bg-gray-800 border-gray-600 text-gray-100">
                      <SelectValue placeholder="Select audio input" />
                    </SelectTrigger>
                    <SelectContent className="bg-gray-800 border-gray-600">
                      {audioInputDevices.length === 0 ? (
                        <SelectItem value="none" disabled className="text-gray-400">
                          No audio devices found
                        </SelectItem>
                      ) : (
                        audioInputDevices.map((device, index) => (
                          <SelectItem 
                            key={index} 
                            value={index.toString()} 
                            className="text-gray-100 hover:bg-gray-700"
                          >
                            {device}
                          </SelectItem>
                        ))
                      )}
                    </SelectContent>
                  </Select>
                </div>


                {/* Input Mode Selection */}
                {deviceInfo && deviceInfo.total_channels > 1 && (
                  <div className="space-y-3">
                    <div className="flex items-center justify-between">
                      <Label className="text-gray-200">Input Mode</Label>
                      <div className="flex items-center space-x-2">
                        <Label className="text-xs text-gray-500">Mono</Label>
                        <Switch
                          checked={inputMode === 'stereo'}
                          onCheckedChange={(checked) => {
                            setInputMode(checked ? 'stereo' : 'mono')
                            if (checked) {
                              setSelectedInputChannels([0, 1])
                            } else {
                              setSelectedInputChannels([selectedInputChannels[0] || 0])
                            }
                          }}
                          className="data-[state=checked]:bg-blue-600"
                        />
                        <Label className="text-xs text-gray-500">Stereo</Label>
                      </div>
                    </div>
                    
                    {/* Channel Selection */}
                    <div className="space-y-2">
                      <Label className="text-gray-200">
                        Input Channels ({inputMode === 'mono' ? 'Select 1' : 'Select 2'})
                      </Label>
                      <div className="grid grid-cols-2 gap-2">
                        {inputMode === 'mono' ? (
                          <Select
                            value={selectedInputChannels[0]?.toString() || '0'}
                            onValueChange={(value) => setSelectedInputChannels([parseInt(value)])}
                          >
                            <SelectTrigger className="bg-gray-800 border-gray-600 text-gray-100">
                              <SelectValue />
                            </SelectTrigger>
                            <SelectContent className="bg-gray-800 border-gray-600">
                              {Array.from({ length: deviceInfo.total_channels }, (_, i) => (
                                <SelectItem
                                  key={i}
                                  value={i.toString()}
                                  className="text-gray-100 hover:bg-gray-700"
                                >
                                  {deviceInfo.channel_names[i] || `Input ${i + 1}`}
                                </SelectItem>
                              ))}
                            </SelectContent>
                          </Select>
                        ) : (
                          <>
                            <Select
                              value={selectedInputChannels[0]?.toString() || '0'}
                              onValueChange={(value) => 
                                setSelectedInputChannels([parseInt(value), selectedInputChannels[1] || 1])
                              }
                            >
                              <SelectTrigger className="bg-gray-800 border-gray-600 text-gray-100">
                                <SelectValue />
                              </SelectTrigger>
                              <SelectContent className="bg-gray-800 border-gray-600">
                                {Array.from({ length: deviceInfo.total_channels }, (_, i) => (
                                  <SelectItem
                                    key={i}
                                    value={i.toString()}
                                    className="text-gray-100 hover:bg-gray-700"
                                  >
                                    L: {deviceInfo.channel_names[i] || `Input ${i + 1}`}
                                  </SelectItem>
                                ))}
                              </SelectContent>
                            </Select>
                            <Select
                              value={selectedInputChannels[1]?.toString() || '1'}
                              onValueChange={(value) => 
                                setSelectedInputChannels([selectedInputChannels[0] || 0, parseInt(value)])
                              }
                            >
                              <SelectTrigger className="bg-gray-800 border-gray-600 text-gray-100">
                                <SelectValue />
                              </SelectTrigger>
                              <SelectContent className="bg-gray-800 border-gray-600">
                                {Array.from({ length: deviceInfo.total_channels }, (_, i) => (
                                  <SelectItem
                                    key={i}
                                    value={i.toString()}
                                    className="text-gray-100 hover:bg-gray-700"
                                  >
                                    R: {deviceInfo.channel_names[i] || `Input ${i + 1}`}
                                  </SelectItem>
                                ))}
                              </SelectContent>
                            </Select>
                          </>
                        )}
                      </div>
                    </div>
                    
                    <div className="mt-4 p-3 bg-gray-800 rounded-lg">
                      <div className="text-sm text-gray-300 font-medium mb-2">Device Information</div>
                      <div className="space-y-1 text-xs text-gray-400">
                        <div className="flex justify-between">
                          <span>Channels:</span>
                          <span>{deviceInfo.total_channels}</span>
                        </div>
                        <div className="flex justify-between">
                          <span>Sample Rate:</span>
                          <span>{deviceInfo.sample_rate} Hz</span>
                        </div>
                        <div className="flex justify-between">
                          <span>Status:</span>
                          <span className="text-green-400">Ready</span>
                        </div>
                      </div>
                    </div>
                  </div>
                )}
              </div>
            </TabsContent>

            {/* Audio Output Tab */}
            <TabsContent value="audio-output" className="mt-6 space-y-4">
              <h3 className="text-lg font-medium text-gray-100">Audio Output</h3>
              
              <div className="space-y-4">
                <div>
                  <Label className="text-gray-200">Audio Output Device</Label>
                  <Select 
                    value={selectedAudioOutput} 
                    onValueChange={setSelectedAudioOutput}
                  >
                    <SelectTrigger className="mt-1 bg-gray-800 border-gray-600 text-gray-100">
                      <SelectValue placeholder="Select audio output" />
                    </SelectTrigger>
                    <SelectContent className="bg-gray-800 border-gray-600">
                      {audioOutputDevices.length === 0 ? (
                        <SelectItem value="none" disabled className="text-gray-400">
                          No audio output devices found
                        </SelectItem>
                      ) : (
                        audioOutputDevices.map((device, index) => (
                          <SelectItem 
                            key={index} 
                            value={index.toString()} 
                            className="text-gray-100 hover:bg-gray-700"
                          >
                            {device}
                          </SelectItem>
                        ))
                      )}
                    </SelectContent>
                  </Select>
                </div>
              </div>
            </TabsContent>

            {/* Advanced Tab */}
            <TabsContent value="advanced" className="mt-6 space-y-4">
              <h3 className="text-lg font-medium text-gray-100">Advanced Settings</h3>
              
              <div className="space-y-6">
                <div>
                  <h4 className="text-md font-medium text-gray-200 mb-4">Sample Detection</h4>
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
                      <p className="text-xs text-gray-500 mt-1">
                        Lower values detect quieter sounds. Higher values ignore background noise.
                      </p>
                    </div>
                  </div>
                </div>
              </div>
            </TabsContent>
          </Tabs>
        </div>

        {/* Footer */}
        <div className="flex justify-end p-4 border-t border-gray-700">
          <Button onClick={onClose} className="px-4 py-2">
            Done
          </Button>
        </div>
      </div>
    </div>
  )
}