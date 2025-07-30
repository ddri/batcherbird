import { useState, useEffect } from "react"
import { Button } from "@/components/ui/button"
import { Badge } from "@/components/ui/badge"
import { Music, Mic, Settings, CheckCircle, AlertCircle } from "lucide-react"
import { useMidiDevices, useAudioInputDevices, useDeviceConnection } from "@/hooks/useTauri"

interface DeviceStatusBarProps {
  onOpenSetup: () => void
}

export function DeviceStatusBar({ onOpenSetup }: DeviceStatusBarProps) {
  const { devices: midiDevices, loadDevices: loadMidiDevices } = useMidiDevices()
  const { devices: audioInputDevices, loadDevices: loadAudioInputDevices } = useAudioInputDevices()
  const { midiConnected, audioConnected } = useDeviceConnection()
  
  const [selectedMidiDevice, setSelectedMidiDevice] = useState<string>("")
  const [selectedAudioInput, setSelectedAudioInput] = useState<string>("")

  useEffect(() => {
    loadMidiDevices()
    loadAudioInputDevices()
  }, [loadMidiDevices, loadAudioInputDevices])

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

  const getMidiDeviceName = () => {
    if (!selectedMidiDevice || midiDevices.length === 0) return "No device"
    return midiDevices[parseInt(selectedMidiDevice)] || "Unknown device"
  }

  const getAudioDeviceName = () => {
    if (!selectedAudioInput || audioInputDevices.length === 0) return "No device"
    return audioInputDevices[parseInt(selectedAudioInput)] || "Unknown device"
  }

  return (
    <div className="flex items-center justify-between px-6 py-3 bg-gray-900 border-b border-gray-700">
      <h1 className="text-lg font-semibold text-gray-100">BatcherBird</h1>
      
      <div className="flex items-center space-x-4">
        {/* MIDI Status */}
        <button
          onClick={onOpenSetup}
          className="flex items-center space-x-2 px-3 py-1.5 rounded-lg bg-gray-800 hover:bg-gray-700 transition-colors"
        >
          <Music className="w-4 h-4 text-gray-300" />
          <span className="text-sm text-gray-200">{getMidiDeviceName()}</span>
          {midiConnected ? (
            <CheckCircle className="w-4 h-4 text-green-400" />
          ) : (
            <AlertCircle className="w-4 h-4 text-gray-500" />
          )}
        </button>

        {/* Audio Status */}
        <button
          onClick={onOpenSetup}
          className="flex items-center space-x-2 px-3 py-1.5 rounded-lg bg-gray-800 hover:bg-gray-700 transition-colors"
        >
          <Mic className="w-4 h-4 text-gray-300" />
          <span className="text-sm text-gray-200">{getAudioDeviceName()}</span>
          {audioConnected ? (
            <CheckCircle className="w-4 h-4 text-green-400" />
          ) : (
            <AlertCircle className="w-4 h-4 text-gray-500" />
          )}
        </button>

        {/* Setup Button */}
        <Button
          onClick={onOpenSetup}
          variant="outline"
          size="sm"
          className="border-gray-600 text-gray-100 hover:bg-gray-800 bg-transparent"
        >
          <Settings className="w-4 h-4 mr-2" />
          Setup
        </Button>
      </div>
    </div>
  )
}