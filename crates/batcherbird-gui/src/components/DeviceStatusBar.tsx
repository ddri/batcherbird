import { Button } from "@/components/ui/button"
import { Badge } from "@/components/ui/badge"
import { Music, Mic, Settings } from "lucide-react"

interface DeviceStatusBarProps {
  onOpenSetup: () => void
  onOpenAudioSetup: () => void
  selectedMidiDevice: string
  selectedAudioInput: string
  midiConnected: boolean
  getMidiDeviceName: () => string
  getAudioInputDeviceName: () => string
}

export function DeviceStatusBar({ 
  onOpenSetup,
  onOpenAudioSetup,
  selectedMidiDevice, 
  selectedAudioInput, 
  midiConnected, 
  getMidiDeviceName, 
  getAudioInputDeviceName 
}: DeviceStatusBarProps) {


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
          <span className="text-sm text-gray-200">
            {getMidiDeviceName()}
          </span>
          <span className="text-sm font-mono text-green-400">
            ✓
          </span>
        </button>

        {/* Audio Status */}
        <button
          onClick={onOpenAudioSetup}
          className="flex items-center space-x-2 px-3 py-1.5 rounded-lg bg-gray-800 hover:bg-gray-700 transition-colors"
        >
          <Mic className="w-4 h-4 text-gray-300" />
          <span className="text-sm text-gray-200">
            {getAudioInputDeviceName()}
          </span>
          <span className="text-sm font-mono text-green-400">
            ✓
          </span>
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