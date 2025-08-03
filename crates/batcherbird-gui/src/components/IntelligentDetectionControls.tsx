import { useState, useEffect, useCallback } from 'react'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import { Label } from '@/components/ui/label'
import { Slider } from '@/components/ui/slider'
import { Switch } from '@/components/ui/switch'
import { Badge } from '@/components/ui/badge'
import { AlertCircle, Zap, Scissors, CheckCircle2 } from 'lucide-react'
import { useIntelligentDetection, useFileSystem, IntelligentDetectionConfig, IntelligentDetectionResult } from '@/hooks/useTauri'

interface IntelligentDetectionControlsProps {
  /** Path to the audio file to analyze */
  audioFilePath: string | null
  /** Called when trimming is completed */
  onTrimmingComplete?: (trimmedFilePath: string) => void
  /** Whether the component is visible */
  isVisible?: boolean
  className?: string
}

export function IntelligentDetectionControls({
  audioFilePath,
  onTrimmingComplete,
  isVisible = true,
  className = ''
}: IntelligentDetectionControlsProps) {
  const [profiles, setProfiles] = useState<string[]>([])
  const [selectedProfile, setSelectedProfile] = useState<string>('General')
  const [detectionConfig, setDetectionConfig] = useState<IntelligentDetectionConfig | null>(null)
  const [detectionResult, setDetectionResult] = useState<IntelligentDetectionResult | null>(null)
  const [enableAdvancedSettings, setEnableAdvancedSettings] = useState(false)

  const { selectAudioFile } = useFileSystem()
  const {
    isDetecting,
    isTrimming,
    error,
    getSynthesizerProfiles,
    getDetectionConfig,
    detectSampleBoundaries,
    applyProfessionalTrimming
  } = useIntelligentDetection()

  // Load synthesizer profiles on mount
  useEffect(() => {
    const loadProfiles = async () => {
      try {
        const profileList = await getSynthesizerProfiles()
        setProfiles(profileList)
        if (profileList.length > 0 && !profileList.includes(selectedProfile)) {
          setSelectedProfile(profileList[0])
        }
      } catch (err) {
        console.error('Failed to load profiles:', err)
      }
    }
    loadProfiles()
  }, [getSynthesizerProfiles])

  // Load detection config when profile changes
  useEffect(() => {
    if (selectedProfile) {
      const loadConfig = async () => {
        try {
          const config = await getDetectionConfig(selectedProfile)
          setDetectionConfig(config)
        } catch (err) {
          console.error('Failed to load detection config:', err)
        }
      }
      loadConfig()
    }
  }, [selectedProfile, getDetectionConfig])

  const handleDetectBoundaries = useCallback(async () => {
    if (!audioFilePath) {
      // If no file path provided, let user select one
      try {
        const selectedFile = await selectAudioFile()
        const result = await detectSampleBoundaries(selectedFile, selectedProfile, detectionConfig || undefined)
        setDetectionResult(result)
      } catch (err) {
        console.error('Detection failed:', err)
      }
    } else {
      try {
        const result = await detectSampleBoundaries(audioFilePath, selectedProfile, detectionConfig || undefined)
        setDetectionResult(result)
      } catch (err) {
        console.error('Detection failed:', err)
      }
    }
  }, [audioFilePath, selectedProfile, detectionConfig, detectSampleBoundaries, selectAudioFile])

  const handleApplyTrimming = useCallback(async () => {
    if (!detectionResult || !audioFilePath) return

    try {
      const trimmedPath = await applyProfessionalTrimming(audioFilePath, detectionResult)
      console.log('✅ Trimming completed:', trimmedPath)
      onTrimmingComplete?.(trimmedPath)
    } catch (err) {
      console.error('Trimming failed:', err)
    }
  }, [detectionResult, audioFilePath, applyProfessionalTrimming, onTrimmingComplete])

  const updateConfigValue = (key: keyof IntelligentDetectionConfig, value: any) => {
    if (detectionConfig) {
      setDetectionConfig({
        ...detectionConfig,
        [key]: value
      })
    }
  }

  const getConfidenceColor = (confidence: number): string => {
    if (confidence >= 0.8) return 'text-green-400'
    if (confidence >= 0.6) return 'text-yellow-400'
    return 'text-orange-400'
  }

  const getConfidenceBadgeVariant = (confidence: number): 'default' | 'secondary' | 'destructive' | 'outline' => {
    if (confidence >= 0.8) return 'default'
    if (confidence >= 0.6) return 'secondary'
    return 'destructive'
  }

  if (!isVisible) {
    return null
  }

  return (
    <div className={`space-y-4 ${className}`}>
      {/* Profile Selection */}
      <Card className="bg-gray-900 border-gray-700">
        <CardHeader className="pb-3">
          <CardTitle className="text-lg text-gray-100 flex items-center gap-2">
            <Zap className="w-5 h-5 text-blue-400" />
            Intelligent Detection
          </CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
            {/* Synthesizer Profile */}
            <div>
              <Label className="text-gray-200 mb-2 block">Synthesizer Profile</Label>
              <Select value={selectedProfile} onValueChange={setSelectedProfile}>
                <SelectTrigger className="bg-gray-800 border-gray-600 text-gray-100">
                  <SelectValue placeholder="Select profile" />
                </SelectTrigger>
                <SelectContent className="bg-gray-800 border-gray-600">
                  {profiles.map(profile => (
                    <SelectItem key={profile} value={profile} className="text-gray-100">
                      {profile}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
              <p className="text-xs text-gray-400 mt-1">
                Optimized settings for different synthesizer types
              </p>
            </div>

            {/* File Status */}
            <div>
              <Label className="text-gray-200 mb-2 block">Audio File</Label>
              <div className="flex items-center space-x-2">
                {audioFilePath ? (
                  <Badge variant="outline" className="text-green-400 border-green-400">
                    File loaded
                  </Badge>
                ) : (
                  <Badge variant="outline" className="text-gray-400 border-gray-400">
                    No file selected
                  </Badge>
                )}
                {!audioFilePath && (
                  <Button
                    variant="outline"
                    size="sm"
                    onClick={async () => {
                      try {
                        await selectAudioFile()
                      } catch (err) {
                        console.log('File selection cancelled')
                      }
                    }}
                    className="text-xs"
                  >
                    Select File
                  </Button>
                )}
              </div>
              {audioFilePath && (
                <p className="text-xs text-gray-400 mt-1 truncate">
                  {audioFilePath.split('/').pop()}
                </p>
              )}
            </div>
          </div>

          {/* Advanced Settings Toggle */}
          <div className="flex items-center space-x-2 pt-2">
            <Switch
              id="advanced-settings"
              checked={enableAdvancedSettings}
              onCheckedChange={setEnableAdvancedSettings}
            />
            <Label htmlFor="advanced-settings" className="text-gray-200">
              Show Advanced Settings
            </Label>
          </div>

          {/* Advanced Configuration */}
          {enableAdvancedSettings && detectionConfig && (
            <div className="space-y-3 p-3 bg-gray-800 rounded border border-gray-600">
              <h4 className="text-sm font-semibold text-gray-200">Advanced Detection Parameters</h4>
              
              <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
                {/* RMS Threshold */}
                <div>
                  <div className="flex items-center justify-between mb-1">
                    <Label className="text-gray-200 text-xs">RMS Threshold</Label>
                    <span className="text-xs font-mono text-gray-300">{detectionConfig.rms_threshold.toFixed(3)}</span>
                  </div>
                  <Slider
                    value={[detectionConfig.rms_threshold]}
                    onValueChange={([value]) => updateConfigValue('rms_threshold', value)}
                    min={0.001}
                    max={0.1}
                    step={0.001}
                    className="w-full"
                  />
                </div>

                {/* Pre-attack Time */}
                <div>
                  <div className="flex items-center justify-between mb-1">
                    <Label className="text-gray-200 text-xs">Pre-attack (ms)</Label>
                    <span className="text-xs font-mono text-gray-300">{detectionConfig.pre_attack_ms}</span>
                  </div>
                  <Slider
                    value={[detectionConfig.pre_attack_ms]}
                    onValueChange={([value]) => updateConfigValue('pre_attack_ms', value)}
                    min={0}
                    max={100}
                    step={1}
                    className="w-full"
                  />
                </div>

                {/* Post-release Time */}
                <div>
                  <div className="flex items-center justify-between mb-1">
                    <Label className="text-gray-200 text-xs">Post-release (ms)</Label>
                    <span className="text-xs font-mono text-gray-300">{detectionConfig.post_release_ms}</span>
                  </div>
                  <Slider
                    value={[detectionConfig.post_release_ms]}
                    onValueChange={([value]) => updateConfigValue('post_release_ms', value)}
                    min={0}
                    max={500}
                    step={10}
                    className="w-full"
                  />
                </div>

                {/* Minimum Length */}
                <div>
                  <div className="flex items-center justify-between mb-1">
                    <Label className="text-gray-200 text-xs">Min Length (ms)</Label>
                    <span className="text-xs font-mono text-gray-300">{detectionConfig.min_length_ms}</span>
                  </div>
                  <Slider
                    value={[detectionConfig.min_length_ms]}
                    onValueChange={([value]) => updateConfigValue('min_length_ms', value)}
                    min={10}
                    max={1000}
                    step={10}
                    className="w-full"
                  />
                </div>
              </div>
            </div>
          )}

          {/* Detection Action */}
          <Button
            onClick={handleDetectBoundaries}
            disabled={isDetecting || !selectedProfile}
            className="w-full bg-blue-600 hover:bg-blue-700 text-white"
          >
            {isDetecting ? (
              <>
                <div className="animate-spin rounded-full h-4 w-4 border-b-2 border-white mr-2" />
                Analyzing Audio...
              </>
            ) : (
              <>
                <Zap className="w-4 h-4 mr-2" />
                Detect Sample Boundaries
              </>
            )}
          </Button>
        </CardContent>
      </Card>

      {/* Detection Results */}
      {detectionResult && (
        <Card className="bg-gray-900 border-gray-700">
          <CardHeader className="pb-3">
            <CardTitle className="text-lg text-gray-100 flex items-center gap-2">
              <CheckCircle2 className="w-5 h-5 text-green-400" />
              Detection Results
            </CardTitle>
          </CardHeader>
          <CardContent className="space-y-4">
            {/* Overall Results */}
            <div className="grid grid-cols-1 md:grid-cols-3 gap-4 text-center">
              <div className="bg-gray-800 rounded p-3">
                <div className="text-lg font-mono text-green-400">
                  {(detectionResult.start_sample / 44100 * 1000).toFixed(1)}ms
                </div>
                <div className="text-xs text-gray-400">Start Time</div>
              </div>
              <div className="bg-gray-800 rounded p-3">
                <div className="text-lg font-mono text-blue-400">
                  {((detectionResult.end_sample - detectionResult.start_sample) / 44100 * 1000).toFixed(1)}ms
                </div>
                <div className="text-xs text-gray-400">Duration</div>
              </div>
              <div className="bg-gray-800 rounded p-3">
                <div className={`text-lg font-mono ${getConfidenceColor(detectionResult.confidence_score)}`}>
                  {(detectionResult.confidence_score * 100).toFixed(1)}%
                </div>
                <div className="text-xs text-gray-400">Confidence</div>
              </div>
            </div>

            {/* Algorithm Results */}
            <div>
              <Label className="text-gray-200 mb-2 block">Algorithm Analysis</Label>
              <div className="space-y-2">
                {detectionResult.algorithm_results.map((result, index) => (
                  <div key={index} className="flex items-center justify-between bg-gray-800 rounded p-2">
                    <span className="text-sm text-gray-200">{result.algorithm}</span>
                    <Badge variant={getConfidenceBadgeVariant(result.confidence)} className="text-xs">
                      {(result.confidence * 100).toFixed(0)}%
                    </Badge>
                  </div>
                ))}
              </div>
            </div>

            {/* Processing Stats */}
            <div className="text-xs text-gray-400 text-center">
              Processed using {detectionResult.profile_used} profile in {detectionResult.processing_time_ms.toFixed(1)}ms
            </div>

            {/* Apply Trimming */}
            <Button
              onClick={handleApplyTrimming}
              disabled={isTrimming || !audioFilePath}
              className="w-full bg-green-600 hover:bg-green-700 text-white"
            >
              {isTrimming ? (
                <>
                  <div className="animate-spin rounded-full h-4 w-4 border-b-2 border-white mr-2" />
                  Applying Trimming...
                </>
              ) : (
                <>
                  <Scissors className="w-4 h-4 mr-2" />
                  Apply Professional Trimming
                </>
              )}
            </Button>
          </CardContent>
        </Card>
      )}

      {/* Error Display */}
      {error && (
        <Card className="bg-red-900/20 border-red-600/30">
          <CardContent className="pt-4">
            <div className="flex items-center space-x-2 text-red-400">
              <AlertCircle className="w-4 h-4" />
              <span className="text-sm">{error}</span>
            </div>
          </CardContent>
        </Card>
      )}
    </div>
  )
}