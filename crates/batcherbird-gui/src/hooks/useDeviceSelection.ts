import { useState, useEffect } from "react"
import { useMidiDevices, useAudioInputDevices, useAudioOutputDevices, useDeviceConnection } from "@/hooks/useTauri"

export function useDeviceSelection() {
  // Device hooks
  const { devices: midiDevices, loadDevices: loadMidiDevices, isLoading: midiLoading } = useMidiDevices()
  const { devices: audioInputDevices, loadDevices: loadAudioInputDevices } = useAudioInputDevices()
  const { devices: audioOutputDevices, loadDevices: loadAudioOutputDevices } = useAudioOutputDevices()
  const { midiConnected, connectMidi } = useDeviceConnection()

  // Selection state
  const [selectedMidiDevice, setSelectedMidiDevice] = useState<string>("")
  const [selectedAudioInput, setSelectedAudioInput] = useState<string>("")
  const [selectedAudioOutput, setSelectedAudioOutput] = useState<string>("")

  // Load devices on mount
  useEffect(() => {
    loadMidiDevices()
    loadAudioInputDevices()
    loadAudioOutputDevices()
  }, [loadMidiDevices, loadAudioInputDevices, loadAudioOutputDevices])

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

  // MIDI device change handler with auto-connect
  const handleMidiDeviceChange = (value: string) => {
    setSelectedMidiDevice(value)
    if (value && !midiConnected) {
      connectMidi(parseInt(value))
    }
  }

  // Device name getters
  const getMidiDeviceName = () => {
    if (!selectedMidiDevice || midiDevices.length === 0) return "No device"
    return midiDevices[parseInt(selectedMidiDevice)] || "Unknown device"
  }

  const getAudioInputDeviceName = () => {
    if (!selectedAudioInput || audioInputDevices.length === 0) return "No device"
    return audioInputDevices[parseInt(selectedAudioInput)] || "Unknown device"
  }

  const getAudioOutputDeviceName = () => {
    if (!selectedAudioOutput || audioOutputDevices.length === 0) return "No device"
    return audioOutputDevices[parseInt(selectedAudioOutput)] || "Unknown device"
  }

  return {
    // Device lists
    midiDevices,
    audioInputDevices,
    audioOutputDevices,
    midiLoading,

    // Selection state
    selectedMidiDevice,
    selectedAudioInput,
    selectedAudioOutput,

    // Selection setters
    setSelectedMidiDevice,
    setSelectedAudioInput,
    setSelectedAudioOutput,
    handleMidiDeviceChange,

    // Connection state
    midiConnected,

    // Helper functions
    getMidiDeviceName,
    getAudioInputDeviceName,
    getAudioOutputDeviceName,

    // Device loading functions
    loadMidiDevices,
    loadAudioInputDevices,
    loadAudioOutputDevices,
  }
}