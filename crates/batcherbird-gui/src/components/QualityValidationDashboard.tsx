import { useState, useCallback } from 'react'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import { Progress } from '@/components/ui/progress'
import { 
  AlertTriangle, 
  CheckCircle2, 
  Info, 
  TrendingUp, 
  Volume2, 
  Clock,
  FileText,
  Zap
} from 'lucide-react'
import { useFileSystem } from '@/hooks/useTauri'

interface QualityMetrics {
  overall_score: number
  dynamic_range: number
  peak_level: number
  rms_level: number
  thd_plus_noise: number
  signal_to_noise_ratio: number
  frequency_response_flatness: number
  stereo_imaging: number
  processing_time_ms: number
}

interface QualityRecommendation {
  category: 'critical' | 'warning' | 'suggestion' | 'info'
  title: string
  description: string
  fix_action?: string
  expected_improvement?: number
}

interface QualityValidationResult {
  file_path: string
  file_size_bytes: number
  duration_ms: number
  sample_rate: number
  bit_depth: number
  channels: number
  metrics: QualityMetrics
  recommendations: QualityRecommendation[]
  processing_timestamp: string
  validation_version: string
}

interface QualityValidationDashboardProps {
  /** Path to the audio file to analyze */
  audioFilePath: string | null
  /** Called when validation is completed */
  onValidationComplete?: (result: QualityValidationResult) => void
  /** Whether the component is visible */
  isVisible?: boolean
  className?: string
}

export function QualityValidationDashboard({
  audioFilePath,
  onValidationComplete,
  isVisible = true,
  className = ''
}: QualityValidationDashboardProps) {
  const [validationResult, setValidationResult] = useState<QualityValidationResult | null>(null)
  const [isValidating, setIsValidating] = useState(false)
  const [error, setError] = useState<string | null>(null)

  const { selectAudioFile } = useFileSystem()

  const validateQuality = useCallback(async (filePath: string) => {
    setIsValidating(true)
    setError(null)
    
    try {
      // This would be a Tauri command - for now we'll simulate the response
      // In a real implementation, this would call: await invoke('validate_audio_quality', { filePath })
      
      // Simulate processing time
      await new Promise(resolve => setTimeout(resolve, 2000))
      
      // Mock validation result
      const mockResult: QualityValidationResult = {
        file_path: filePath,
        file_size_bytes: 1024000,
        duration_ms: 3500,
        sample_rate: 44100,
        bit_depth: 16,
        channels: 2,
        metrics: {
          overall_score: 8.7,
          dynamic_range: 18.5,
          peak_level: -3.2,
          rms_level: -18.7,
          thd_plus_noise: 0.0032,
          signal_to_noise_ratio: 96.3,
          frequency_response_flatness: 0.8,
          stereo_imaging: 0.92,
          processing_time_ms: 1847
        },
        recommendations: [
          {
            category: 'warning',
            title: 'Peak Level Too High',
            description: 'Peak level at -3.2dB may cause clipping in some systems',
            fix_action: 'Reduce gain by 3-6dB',
            expected_improvement: 1.2
          },
          {
            category: 'suggestion',
            title: 'Dynamic Range Optimization',
            description: 'Dynamic range could be improved for professional use',
            fix_action: 'Consider multiband compression',
            expected_improvement: 0.8
          },
          {
            category: 'info',
            title: 'Excellent SNR',
            description: 'Signal-to-noise ratio is excellent for this format',
          },
          {
            category: 'critical',
            title: 'Frequency Response',
            description: 'Significant frequency response deviation detected above 8kHz',
            fix_action: 'Apply high-frequency EQ correction',
            expected_improvement: 2.1
          }
        ],
        processing_timestamp: new Date().toISOString(),
        validation_version: '1.0.0'
      }
      
      setValidationResult(mockResult)
      onValidationComplete?.(mockResult)
      
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Validation failed')
    } finally {
      setIsValidating(false)
    }
  }, [onValidationComplete])

  const handleValidateFile = useCallback(async () => {
    if (!audioFilePath) {
      try {
        const selectedFile = await selectAudioFile()
        await validateQuality(selectedFile)
      } catch (err) {
        // File selection cancelled
      }
    } else {
      await validateQuality(audioFilePath)
    }
  }, [audioFilePath, validateQuality, selectAudioFile])

  const getScoreColor = (score: number): string => {
    if (score >= 9.0) return 'text-green-400'
    if (score >= 7.0) return 'text-yellow-400'
    if (score >= 5.0) return 'text-orange-400'
    return 'text-red-400'
  }

  const getRecommendationIcon = (category: QualityRecommendation['category']) => {
    switch (category) {
      case 'critical': return <AlertTriangle className="w-4 h-4 text-red-400" />
      case 'warning': return <AlertTriangle className="w-4 h-4 text-yellow-400" />
      case 'suggestion': return <Info className="w-4 h-4 text-blue-400" />
      case 'info': return <CheckCircle2 className="w-4 h-4 text-green-400" />
    }
  }

  const getRecommendationBadgeVariant = (category: QualityRecommendation['category']): 'default' | 'secondary' | 'destructive' | 'outline' => {
    switch (category) {
      case 'critical': return 'destructive'
      case 'warning': return 'secondary'
      case 'suggestion': return 'outline'
      case 'info': return 'default'
    }
  }

  if (!isVisible) {
    return null
  }

  return (
    <div className={`space-y-4 ${className}`}>
      {/* Validation Control */}
      <Card className="bg-gray-900 border-gray-700">
        <CardHeader className="pb-3">
          <CardTitle className="text-lg text-gray-100 flex items-center gap-2">
            <TrendingUp className="w-5 h-5 text-purple-400" />
            Quality Validation
          </CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="flex items-center justify-between">
            <div>
              <div className="flex items-center space-x-2">
                {audioFilePath ? (
                  <Badge variant="outline" className="text-green-400 border-green-400">
                    File ready
                  </Badge>
                ) : (
                  <Badge variant="outline" className="text-gray-400 border-gray-400">
                    No file selected
                  </Badge>
                )}
              </div>
              {audioFilePath && (
                <p className="text-xs text-gray-400 mt-1 truncate">
                  {audioFilePath.split('/').pop()}
                </p>
              )}
            </div>
            
            <Button
              onClick={handleValidateFile}
              disabled={isValidating}
              className="bg-purple-600 hover:bg-purple-700 text-white"
            >
              {isValidating ? (
                <>
                  <div className="animate-spin rounded-full h-4 w-4 border-b-2 border-white mr-2" />
                  Analyzing...
                </>
              ) : (
                <>
                  <Zap className="w-4 h-4 mr-2" />
                  Validate Quality
                </>
              )}
            </Button>
          </div>
        </CardContent>
      </Card>

      {/* Validation Results */}
      {validationResult && (
        <div className="space-y-4">
          {/* Overall Score */}
          <Card className="bg-gray-900 border-gray-700">
            <CardHeader className="pb-3">
              <CardTitle className="text-lg text-gray-100 flex items-center gap-2">
                <CheckCircle2 className="w-5 h-5 text-green-400" />
                Quality Score
              </CardTitle>
            </CardHeader>
            <CardContent>
              <div className="text-center">
                <div className={`text-4xl font-bold ${getScoreColor(validationResult.metrics.overall_score)}`}>
                  {validationResult.metrics.overall_score.toFixed(1)}
                </div>
                <div className="text-gray-400 text-sm">out of 10.0</div>
                <Progress 
                  value={validationResult.metrics.overall_score * 10} 
                  className="mt-2 h-2"
                />
              </div>
            </CardContent>
          </Card>

          {/* Technical Metrics */}
          <Card className="bg-gray-900 border-gray-700">
            <CardHeader className="pb-3">
              <CardTitle className="text-lg text-gray-100 flex items-center gap-2">
                <Volume2 className="w-5 h-5 text-blue-400" />
                Technical Analysis
              </CardTitle>
            </CardHeader>
            <CardContent>
              <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
                <div className="bg-gray-800 rounded p-3 text-center">
                  <div className="text-lg font-mono text-blue-400">
                    {validationResult.metrics.dynamic_range.toFixed(1)} dB
                  </div>
                  <div className="text-xs text-gray-400">Dynamic Range</div>
                </div>
                
                <div className="bg-gray-800 rounded p-3 text-center">
                  <div className="text-lg font-mono text-green-400">
                    {validationResult.metrics.peak_level.toFixed(1)} dBFS
                  </div>
                  <div className="text-xs text-gray-400">Peak Level</div>
                </div>
                
                <div className="bg-gray-800 rounded p-3 text-center">
                  <div className="text-lg font-mono text-purple-400">
                    {validationResult.metrics.signal_to_noise_ratio.toFixed(1)} dB
                  </div>
                  <div className="text-xs text-gray-400">SNR</div>
                </div>
                
                <div className="bg-gray-800 rounded p-3 text-center">
                  <div className="text-lg font-mono text-yellow-400">
                    {(validationResult.metrics.thd_plus_noise * 100).toFixed(3)}%
                  </div>
                  <div className="text-xs text-gray-400">THD+N</div>
                </div>
              </div>

              <div className="mt-4 grid grid-cols-1 md:grid-cols-2 gap-4">
                <div className="bg-gray-800 rounded p-3">
                  <div className="flex items-center justify-between mb-2">
                    <span className="text-sm text-gray-200">Frequency Response</span>
                    <span className="text-sm font-mono text-gray-300">
                      {validationResult.metrics.frequency_response_flatness.toFixed(2)}
                    </span>
                  </div>
                  <Progress 
                    value={validationResult.metrics.frequency_response_flatness * 100} 
                    className="h-1"
                  />
                </div>
                
                <div className="bg-gray-800 rounded p-3">
                  <div className="flex items-center justify-between mb-2">
                    <span className="text-sm text-gray-200">Stereo Imaging</span>
                    <span className="text-sm font-mono text-gray-300">
                      {validationResult.metrics.stereo_imaging.toFixed(2)}
                    </span>
                  </div>
                  <Progress 
                    value={validationResult.metrics.stereo_imaging * 100} 
                    className="h-1"
                  />
                </div>
              </div>
            </CardContent>
          </Card>

          {/* File Information */}
          <Card className="bg-gray-900 border-gray-700">
            <CardHeader className="pb-3">
              <CardTitle className="text-lg text-gray-100 flex items-center gap-2">
                <FileText className="w-5 h-5 text-gray-400" />
                File Information
              </CardTitle>
            </CardHeader>
            <CardContent>
              <div className="grid grid-cols-1 md:grid-cols-3 gap-4 text-sm">
                <div>
                  <div className="text-gray-400">Duration</div>
                  <div className="text-gray-200 font-mono">
                    {(validationResult.duration_ms / 1000).toFixed(1)}s
                  </div>
                </div>
                <div>
                  <div className="text-gray-400">Sample Rate</div>
                  <div className="text-gray-200 font-mono">
                    {validationResult.sample_rate} Hz
                  </div>
                </div>
                <div>
                  <div className="text-gray-400">Bit Depth</div>
                  <div className="text-gray-200 font-mono">
                    {validationResult.bit_depth} bit
                  </div>
                </div>
                <div>
                  <div className="text-gray-400">Channels</div>
                  <div className="text-gray-200 font-mono">
                    {validationResult.channels === 2 ? 'Stereo' : validationResult.channels === 1 ? 'Mono' : `${validationResult.channels}ch`}
                  </div>
                </div>
                <div>
                  <div className="text-gray-400">File Size</div>
                  <div className="text-gray-200 font-mono">
                    {(validationResult.file_size_bytes / 1024 / 1024).toFixed(1)} MB
                  </div>
                </div>
                <div>
                  <div className="text-gray-400">Analysis Time</div>
                  <div className="text-gray-200 font-mono">
                    {validationResult.metrics.processing_time_ms.toFixed(0)}ms
                  </div>
                </div>
              </div>
            </CardContent>
          </Card>

          {/* Recommendations */}
          {validationResult.recommendations.length > 0 && (
            <Card className="bg-gray-900 border-gray-700">
              <CardHeader className="pb-3">
                <CardTitle className="text-lg text-gray-100 flex items-center gap-2">
                  <Clock className="w-5 h-5 text-orange-400" />
                  Recommendations
                  <Badge variant="outline" className="ml-2">
                    {validationResult.recommendations.length}
                  </Badge>
                </CardTitle>
              </CardHeader>
              <CardContent className="space-y-3">
                {validationResult.recommendations.map((rec, index) => (
                  <div key={index} className="bg-gray-800 rounded p-3">
                    <div className="flex items-start space-x-3">
                      {getRecommendationIcon(rec.category)}
                      <div className="flex-1">
                        <div className="flex items-center justify-between mb-1">
                          <h4 className="text-sm font-semibold text-gray-200">{rec.title}</h4>
                          <Badge variant={getRecommendationBadgeVariant(rec.category)} className="text-xs">
                            {rec.category}
                          </Badge>
                        </div>
                        <p className="text-xs text-gray-400 mb-2">{rec.description}</p>
                        {rec.fix_action && (
                          <div className="text-xs text-blue-400">
                            💡 {rec.fix_action}
                            {rec.expected_improvement && (
                              <span className="text-green-400 ml-2">
                                (+{rec.expected_improvement.toFixed(1)} score)
                              </span>
                            )}
                          </div>
                        )}
                      </div>
                    </div>
                  </div>
                ))}
              </CardContent>
            </Card>
          )}
        </div>
      )}

      {/* Error Display */}
      {error && (
        <Card className="bg-red-900/20 border-red-600/30">
          <CardContent className="pt-4">
            <div className="flex items-center space-x-2 text-red-400">
              <AlertTriangle className="w-4 h-4" />
              <span className="text-sm">{error}</span>
            </div>
          </CardContent>
        </Card>
      )}
    </div>
  )
}