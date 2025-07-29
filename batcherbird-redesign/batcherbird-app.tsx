"use client"

import { useState } from "react"
import { Button } from "@/components/ui/button"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select"
import { Slider } from "@/components/ui/slider"
import { Switch } from "@/components/ui/switch"
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs"
import { Badge } from "@/components/ui/badge"
import { Progress } from "@/components/ui/progress"
import {
  Play,
  Square,
  Settings,
  Mic,
  Music,
  Clock,
  Layers,
  FolderOpen,
  Volume2,
  ZoomIn,
  ZoomOut,
  RotateCcw,
} from "lucide-react"

export default function Component() {
  const [currentStep, setCurrentStep] = useState(1)
  const [isRecording, setIsRecording] = useState(false)
  const [recordingProgress, setRecordingProgress] = useState(0)
  const [velocityLayers] = useState([127, 100, 80, 60, 40, 20])

  return (
    <div className="h-screen bg-gray-950 text-gray-100 flex flex-col">
      {/* Title Bar */}
      <div className="flex items-center justify-between px-6 py-3 bg-gray-900 border-b border-gray-700">
        <div className="flex items-center space-x-3">
          <div className="flex space-x-2">
            <div className="w-3 h-3 bg-red-500 rounded-full"></div>
            <div className="w-3 h-3 bg-yellow-500 rounded-full"></div>
            <div className="w-3 h-3 bg-green-500 rounded-full"></div>
          </div>
          <h1 className="text-lg font-semibold text-gray-100">BatcherBird</h1>
        </div>
        <div className="flex items-center space-x-2">
          <Button
            variant="outline"
            size="sm"
            className="border-gray-600 text-gray-100 hover:bg-gray-800 bg-transparent"
          >
            <Settings className="w-4 h-4 mr-2" />
            Setup
          </Button>
          <Button variant="destructive" size="sm">
            MIDI PANIC
          </Button>
        </div>
      </div>

      <div className="flex flex-1 overflow-hidden">
        {/* Main Content */}
        <div className="flex-1 flex flex-col">
          {/* Step Content */}
          <div className="flex-1 p-6 overflow-auto">
            <div className="max-w-4xl mx-auto space-y-6">
              {/* Interface Selection */}
              <div className="grid md:grid-cols-2 gap-6">
                <Card className="bg-gray-900 border-gray-700">
                  <CardHeader>
                    <CardTitle className="flex items-center space-x-2 text-gray-100">
                      <Music className="w-5 h-5 text-gray-300" />
                      <span>MIDI Interface</span>
                      <Badge variant="secondary" className="bg-green-600 text-white">
                        Connected
                      </Badge>
                    </CardTitle>
                  </CardHeader>
                  <CardContent>
                    <Select defaultValue="minifuse2">
                      <SelectTrigger className="bg-gray-800 border-gray-600 text-gray-100">
                        <SelectValue />
                      </SelectTrigger>
                      <SelectContent className="bg-gray-800 border-gray-600">
                        <SelectItem value="minifuse2" className="text-gray-100 hover:bg-gray-700">
                          MiniFuse 2
                        </SelectItem>
                        <SelectItem value="other" className="text-gray-100 hover:bg-gray-700">
                          Other Interface
                        </SelectItem>
                      </SelectContent>
                    </Select>
                    <div className="flex items-center space-x-2 mt-3">
                      <div className="w-2 h-2 bg-green-400 rounded-full"></div>
                      <span className="text-sm text-gray-400">MiniFuse 2 - Active</span>
                    </div>
                  </CardContent>
                </Card>

                <Card className="bg-gray-900 border-gray-700">
                  <CardHeader>
                    <CardTitle className="flex items-center space-x-2 text-gray-100">
                      <Mic className="w-5 h-5 text-gray-300" />
                      <span>Audio Interface</span>
                      <Badge variant="secondary" className="bg-green-600 text-white">
                        Connected
                      </Badge>
                    </CardTitle>
                  </CardHeader>
                  <CardContent>
                    <Select defaultValue="minifuse2-audio">
                      <SelectTrigger className="bg-gray-800 border-gray-600 text-gray-100">
                        <SelectValue />
                      </SelectTrigger>
                      <SelectContent className="bg-gray-800 border-gray-600">
                        <SelectItem value="minifuse2-audio" className="text-gray-100 hover:bg-gray-700">
                          MiniFuse 2 Audio
                        </SelectItem>
                        <SelectItem value="other-audio" className="text-gray-100 hover:bg-gray-700">
                          Other Audio Interface
                        </SelectItem>
                      </SelectContent>
                    </Select>
                    <div className="flex items-center justify-between mt-3">
                      <span className="text-sm text-gray-400">Input Level</span>
                      <span className="text-sm text-gray-400">-12 dB</span>
                    </div>
                    <div className="w-full bg-gray-800 rounded-full h-2 mt-1">
                      <div className="bg-green-400 h-2 rounded-full" style={{ width: "65%" }}></div>
                    </div>
                  </CardContent>
                </Card>
              </div>

              {/* Sample Type Selection */}
              <Card className="bg-gray-900 border-gray-700">
                <CardHeader>
                  <CardTitle className="flex items-center space-x-2 text-gray-100">
                    <Layers className="w-5 h-5 text-gray-300" />
                    <span>Sample Type</span>
                  </CardTitle>
                </CardHeader>
                <CardContent>
                  <Tabs defaultValue="single" className="w-full">
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
                          <Select defaultValue="c4">
                            <SelectTrigger className="mt-1 bg-gray-800 border-gray-600 text-gray-100">
                              <SelectValue />
                            </SelectTrigger>
                            <SelectContent className="bg-gray-800 border-gray-600">
                              <SelectItem value="c4" className="text-gray-100 hover:bg-gray-700">
                                C4 (60)
                              </SelectItem>
                              <SelectItem value="c3" className="text-gray-100 hover:bg-gray-700">
                                C3 (48)
                              </SelectItem>
                              <SelectItem value="c5" className="text-gray-100 hover:bg-gray-700">
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
                          <Select defaultValue="c2">
                            <SelectTrigger className="mt-1 bg-gray-800 border-gray-600 text-gray-100">
                              <SelectValue />
                            </SelectTrigger>
                            <SelectContent className="bg-gray-800 border-gray-600">
                              <SelectItem value="c2" className="text-gray-100 hover:bg-gray-700">
                                C2 (36)
                              </SelectItem>
                              <SelectItem value="c3" className="text-gray-100 hover:bg-gray-700">
                                C3 (48)
                              </SelectItem>
                            </SelectContent>
                          </Select>
                        </div>
                        <div>
                          <Label className="text-gray-200">End Note</Label>
                          <Select defaultValue="c6">
                            <SelectTrigger className="mt-1 bg-gray-800 border-gray-600 text-gray-100">
                              <SelectValue />
                            </SelectTrigger>
                            <SelectContent className="bg-gray-800 border-gray-600">
                              <SelectItem value="c6" className="text-gray-100 hover:bg-gray-700">
                                C6 (84)
                              </SelectItem>
                              <SelectItem value="c7" className="text-gray-100 hover:bg-gray-700">
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
                          <Slider defaultValue={[2.42]} max={10} min={0.5} step={0.1} className="flex-1" />
                          <span className="text-sm font-mono w-12 text-gray-300">2.42s</span>
                        </div>
                      </div>
                      <div className="flex items-center space-x-2">
                        <Switch id="auto-detect" defaultChecked />
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
                            <div className="bg-gray-200 text-gray-900 text-xs py-1 px-2 rounded">{velocity}</div>
                          </div>
                        ))}
                      </div>
                      <Button
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
                    <div className="flex items-center space-x-2 text-sm text-gray-400">
                      <span>Duration: 2.42s</span>
                      <span>•</span>
                      <span>Roland-EM1018_C4_60_vel127.wav</span>
                    </div>
                  </CardTitle>
                </CardHeader>
                <CardContent>
                  <div className="bg-gray-950 rounded-lg p-4 h-48 flex items-center justify-center">
                    <svg width="100%" height="100%" viewBox="0 0 800 150" className="text-gray-300">
                      <path
                        d="M0,75 Q50,25 100,75 T200,75 Q250,125 300,75 T400,75 Q450,25 500,75 T600,75 Q650,125 700,75 T800,75"
                        stroke="#d1d5db"
                        strokeWidth="2"
                        fill="none"
                      />
                      <path
                        d="M0,75 Q50,125 100,75 T200,75 Q250,25 300,75 T400,75 Q450,125 500,75 T600,75 Q650,25 700,75 T800,75"
                        stroke="#d1d5db"
                        strokeWidth="2"
                        fill="none"
                        opacity="0.6"
                      />
                    </svg>
                  </div>
                  <div className="flex items-center justify-center space-x-2 mt-4">
                    <Button
                      variant="outline"
                      size="sm"
                      className="border-gray-600 text-gray-100 hover:bg-gray-800 bg-transparent"
                    >
                      <Play className="w-4 h-4 mr-2" />
                      Play
                    </Button>
                    <Button
                      variant="outline"
                      size="sm"
                      className="border-gray-600 text-gray-100 hover:bg-gray-800 bg-transparent"
                    >
                      <ZoomIn className="w-4 h-4" />
                    </Button>
                    <Button
                      variant="outline"
                      size="sm"
                      className="border-gray-600 text-gray-100 hover:bg-gray-800 bg-transparent"
                    >
                      <ZoomOut className="w-4 h-4" />
                    </Button>
                    <Button
                      variant="outline"
                      size="sm"
                      className="border-gray-600 text-gray-100 hover:bg-gray-800 bg-transparent"
                    >
                      <RotateCcw className="w-4 h-4" />
                    </Button>
                  </div>
                </CardContent>
              </Card>

              {/* Recording Controls */}
              <Card className="bg-gray-900 border-gray-700">
                <CardContent className="pt-6">
                  <div className="flex items-center justify-between">
                    <div className="flex items-center space-x-4">
                      <div className="flex items-center space-x-4">
                        <Button
                          variant="outline"
                          size="lg"
                          className="bg-gray-900 border-gray-600 text-gray-100 hover:bg-gray-800"
                        >
                          <Play className="w-5 h-5 mr-2" />
                          Preview
                        </Button>
                        <Button
                          size="lg"
                          className={`${isRecording ? "bg-red-600 hover:bg-red-700 text-white" : "bg-gray-200 hover:bg-gray-300 text-gray-900"}`}
                          onClick={() => setIsRecording(!isRecording)}
                        >
                          {isRecording ? (
                            <>
                              <Square className="w-5 h-5 mr-2" />
                              Stop Recording
                            </>
                          ) : (
                            <>
                              <Play className="w-5 h-5 mr-2" />
                              Start Recording
                            </>
                          )}
                        </Button>
                        {isRecording && (
                          <div className="flex items-center space-x-2">
                            <div className="w-2 h-2 bg-red-500 rounded-full animate-pulse"></div>
                            <span className="text-sm text-gray-400">Recording...</span>
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
                  <Input defaultValue="Roland-EM1018" className="mt-1 bg-gray-800 border-gray-600 text-gray-100" />
                  <p className="text-xs text-gray-400 mt-1">Example: Roland_JP8000_C4_60_vel127.wav</p>
                </div>

                <div>
                  <Label className="text-gray-200">Output Directory</Label>
                  <div className="flex mt-1">
                    <Input
                      defaultValue="/Users/dryan/Desktop/Batch"
                      className="bg-gray-800 border-gray-600 text-gray-100 rounded-r-none"
                    />
                    <Button
                      variant="outline"
                      className="rounded-l-none bg-transparent border-gray-600 text-gray-100 hover:bg-gray-800"
                    >
                      <FolderOpen className="w-4 h-4" />
                    </Button>
                  </div>
                </div>

                <div>
                  <Label className="text-gray-200">Export Format</Label>
                  <Select defaultValue="dspreset">
                    <SelectTrigger className="mt-1 bg-gray-800 border-gray-600 text-gray-100">
                      <SelectValue />
                    </SelectTrigger>
                    <SelectContent className="bg-gray-800 border-gray-600">
                      <SelectItem value="dspreset" className="text-gray-100 hover:bg-gray-700">
                        Decent Sampler (.dspreset)
                      </SelectItem>
                      <SelectItem value="wav" className="text-gray-100 hover:bg-gray-700">
                        WAV Files Only
                      </SelectItem>
                      <SelectItem value="kontakt" className="text-gray-100 hover:bg-gray-700">
                        Kontakt (.nki)
                      </SelectItem>
                    </SelectContent>
                  </Select>
                </div>

                <div>
                  <Label className="text-gray-200">Creator Name</Label>
                  <Input
                    placeholder="Your name"
                    className="mt-1 bg-gray-800 border-gray-600 text-gray-100 placeholder:text-gray-500"
                  />
                </div>

                <div>
                  <Label className="text-gray-200">Instrument Description</Label>
                  <Input
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
                  <Switch id="auto-detection" defaultChecked />
                  <Label htmlFor="auto-detection" className="text-gray-200">
                    Enable Auto-Detection
                  </Label>
                </div>

                <div>
                  <div className="flex items-center justify-between mb-2">
                    <Label className="text-gray-200">Detection Threshold</Label>
                    <span className="text-sm font-mono text-gray-300">-35 dB</span>
                  </div>
                  <Slider defaultValue={[-35]} max={-10} min={-60} step={1} className="w-full" />
                </div>
              </div>
            </div>

            <div>
              <h3 className="text-lg font-semibold mb-4 text-gray-100">Quick Actions</h3>
              <div className="space-y-2">
                <Button
                  variant="outline"
                  className="w-full justify-start bg-transparent border-gray-600 text-gray-100 hover:bg-gray-800"
                >
                  Load Template
                </Button>
                <Button
                  variant="outline"
                  className="w-full justify-start bg-transparent border-gray-600 text-gray-100 hover:bg-gray-800"
                >
                  Save as Template
                </Button>
                <Button
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
    </div>
  )
}
