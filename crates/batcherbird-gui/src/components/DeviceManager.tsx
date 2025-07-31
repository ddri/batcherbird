import { useState, useEffect } from "react"
import { Button } from "@/components/ui/button"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select"
import { Badge } from "@/components/ui/badge"
import { Label } from "@/components/ui/label"
import { Switch } from "@/components/ui/switch"
import { Music, Mic, Settings, AlertCircle, CheckCircle } from "lucide-react"
import { useMidiDevices, useAudioInputDevices, useAudioOutputDevices, useDeviceConnection, useAudioMonitoring, useAudioDeviceInfo } from "@/hooks/useTauri"

interface DeviceManagerProps {
  onMidiPanic: () => void
  onOpenSetup: () => void
  onInputConfigChange?: (config: { mode: 'mono' | 'stereo', channels: number[] }) => void
}

export function DeviceManager({ onMidiPanic, onOpenSetup, onInputConfigChange }: DeviceManagerProps) {
  // Device hooks
  const { devices: midiDevices, loadDevices: loadMidiDevices, isLoading: midiLoading } = useMidiDevices()
  const { devices: audioInputDevices, loadDevices: loadAudioInputDevices } = useAudioInputDevices()
  const { devices: audioOutputDevices, loadDevices: loadAudioOutputDevices } = useAudioOutputDevices()
  
  // Connection hooks
  const { midiConnected, connectMidi, testMidiConnection, sendMidiPanic } = useDeviceConnection()
  const { levels } = useAudioMonitoring()
  const { deviceInfo, getDeviceInfo } = useAudioDeviceInfo()
  
  // Local state
  const [selectedMidiDevice, setSelectedMidiDevice] = useState<string>("")
  const [selectedAudioInput, setSelectedAudioInput] = useState<string>("")
  const [selectedAudioOutput, setSelectedAudioOutput] = useState<string>("")
  const [inputMode, setInputMode] = useState<'mono' | 'stereo'>('stereo')
  const [selectedInputChannels, setSelectedInputChannels] = useState<number[]>([0, 1])

  // Load devices on mount
  useEffect(() => {
    loadMidiDevices()
    loadAudioInputDevices()
    loadAudioOutputDevices()
  }, [loadMidiDevices, loadAudioInputDevices, loadAudioOutputDevices])
  
  // Notify parent when input configuration changes
  useEffect(() => {
    if (onInputConfigChange) {
      onInputConfigChange({
        mode: inputMode,
        channels: selectedInputChannels
      })
    }
  }, [inputMode, selectedInputChannels, onInputConfigChange])
  
  // Debug device lists
  useEffect(() => {
    if (midiDevices.length > 0) console.log("MIDI devices:", midiDevices)
    if (audioInputDevices.length > 0) console.log("Audio input devices:", audioInputDevices)
    if (audioOutputDevices.length > 0) console.log("Audio output devices:", audioOutputDevices)
  }, [midiDevices, audioInputDevices, audioOutputDevices])

  // Auto-select first devices when available
  useEffect(() => {
    if (midiDevices.length > 0 && !selectedMidiDevice) {
      setSelectedMidiDevice("0")
    }
  }, [midiDevices, selectedMidiDevice])

  useEffect(() => {
    if (audioInputDevices.length > 0 && !selectedAudioInput) {
      setSelectedAudioInput("0")
    }
  }, [audioInputDevices, selectedAudioInput])

  useEffect(() => {
    if (audioOutputDevices.length > 0 && !selectedAudioOutput) {
      setSelectedAudioOutput("0")
    }
  }, [audioOutputDevices, selectedAudioOutput])

  const handleMidiConnect = async () => {
    if (selectedMidiDevice) {
      try {
        await connectMidi(parseInt(selectedMidiDevice))
        console.log("Connected to MIDI device:", midiDevices[parseInt(selectedMidiDevice)])
      } catch (err) {
        console.error('Failed to connect MIDI device:', err)
      }
    }
  }
  
  // Auto-connect when device is selected
  const handleMidiDeviceChange = (value: string) => {
    setSelectedMidiDevice(value)
    if (value && !midiConnected) {
      connectMidi(parseInt(value))
    }
  }
  
  const handleAudioInputChange = async (value: string) => {
    setSelectedAudioInput(value)
    console.log("Selected audio input:", audioInputDevices[parseInt(value)])
    
    // Get device info to know how many channels it has
    try {
      const info = await getDeviceInfo(parseInt(value))
      console.log("Device info:", info)
      
      // Reset to stereo mode with first two channels if available
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
      onMidiPanic()
    } catch (err) {
      console.error('MIDI panic failed:', err)
    }
  }

  // Convert audio level to percentage for UI
  const audioLevelPercent = Math.max(0, Math.min(100, (levels.peak_db + 60) * (100 / 60)))

  return (
    <>
      {/* Title Bar Actions */}
      <div className="flex items-center justify-between px-6 py-3 bg-gray-900 border-b border-gray-700">
        <h1 className="text-lg font-semibold text-gray-100">BatcherBird</h1>
        <div className="flex items-center space-x-2">
          <Button
            onClick={onOpenSetup}
            variant="outline"
            size="sm"
            className="border-gray-600 text-gray-100 hover:bg-gray-800 bg-transparent"
          >
            <Settings className="w-4 h-4 mr-2" />
            Setup
          </Button>
          <Button 
            onClick={handleMidiPanic}
            variant="destructive" 
            size="sm"
            disabled={!midiConnected}
          >
            MIDI PANIC
          </Button>
        </div>
      </div>

      {/* Interface Selection */}
      <div className="grid md:grid-cols-2 gap-6 p-6">
        <Card className="bg-gray-900 border-gray-700">
          <CardHeader>
            <CardTitle className="flex items-center space-x-2 text-gray-100">
              <Music className="w-5 h-5 text-gray-300" />
              <span>MIDI Interface</span>
              {midiConnected ? (
                <Badge variant="secondary" className="bg-green-600 text-white">
                  <CheckCircle className="w-3 h-3 mr-1" />
                  Connected
                </Badge>
              ) : (
                <Badge variant="secondary" className="bg-gray-600 text-white">
                  <AlertCircle className="w-3 h-3 mr-1" />
                  Disconnected
                </Badge>
              )}
            </CardTitle>
          </CardHeader>
          <CardContent>
            <Select 
              value={selectedMidiDevice} 
              onValueChange={handleMidiDeviceChange}
              disabled={midiLoading}
            >
              <SelectTrigger className="bg-gray-800 border-gray-600 text-gray-100">
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
            
            <div className="flex items-center space-x-2 mt-3">
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
            
            {!midiConnected && selectedMidiDevice && (
              <Button 
                onClick={handleMidiConnect}
                className="w-full mt-2"
                size="sm"
              >
                Connect MIDI Device
              </Button>
            )}
            
            {midiConnected && (
              <Button 
                onClick={handleMidiTest}
                variant="outline"
                className="w-full mt-2 bg-transparent border-gray-600 text-gray-100 hover:bg-gray-800"
                size="sm"
              >
                Test Connection
              </Button>
            )}
          </CardContent>
        </Card>

        <Card className="bg-gray-900 border-gray-700">
          <CardHeader>
            <CardTitle className="flex items-center space-x-2 text-gray-100">
              <Mic className="w-5 h-5 text-gray-300" />
              <span>Audio Interface</span>
              <Badge variant="secondary" className="bg-green-600 text-white">
                <CheckCircle className="w-3 h-3 mr-1" />
                Ready
              </Badge>
            </CardTitle>
          </CardHeader>
          <CardContent>
            <Select 
              value={selectedAudioInput} 
              onValueChange={handleAudioInputChange}
            >
              <SelectTrigger className="bg-gray-800 border-gray-600 text-gray-100">
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
            
            <div className="flex items-center justify-between mt-3">
              <span className="text-sm text-gray-400">Input Level</span>
              <span className="text-sm text-gray-400">{levels.peak_db.toFixed(1)} dB</span>
            </div>
            <div className="w-full bg-gray-800 rounded-full h-2 mt-1">
              <div 
                className={`h-2 rounded-full transition-all duration-75 ${
                  levels.peak_db > -6 ? 'bg-red-400' : 
                  levels.peak_db > -12 ? 'bg-yellow-400' : 
                  'bg-green-400'
                }`}
                style={{ width: `${audioLevelPercent}%` }}
              />
            </div>
            
            {/* Input Mode Selection */}
            {deviceInfo && deviceInfo.total_channels > 1 && (
              <div className="mt-4 space-y-3">
                <div className="flex items-center justify-between">
                  <Label htmlFor="input-mode" className="text-sm text-gray-400">
                    Input Mode
                  </Label>
                  <div className="flex items-center space-x-2">
                    <Label htmlFor="input-mode" className="text-xs text-gray-500">
                      Mono
                    </Label>
                    <Switch
                      id="input-mode"
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
                    <Label htmlFor="input-mode" className="text-xs text-gray-500">
                      Stereo
                    </Label>
                  </div>
                </div>
                
                {/* Channel Selection */}
                <div className="space-y-2">
                  <Label className="text-sm text-gray-400">
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
                
                <div className="text-xs text-gray-500">
                  Device: {deviceInfo.total_channels} channels @ {deviceInfo.sample_rate} Hz
                </div>
              </div>
            )}
          </CardContent>
        </Card>
      </div>
    </>
  )
}