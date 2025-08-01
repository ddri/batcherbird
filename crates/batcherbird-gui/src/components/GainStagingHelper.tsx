import { useAudioMonitoring } from "@/hooks/useTauri"
import { AlertTriangle, CheckCircle, Info, TrendingUp, TrendingDown } from "lucide-react"
import { ReactElement } from "react"

interface GainStagingHelperProps {
  isVisible: boolean
  className?: string
}

interface GainAnalysis {
  currentRMS: number
  targetRMS: number
  gainAdjustment: number
  status: 'optimal' | 'slightly-hot' | 'too-hot' | 'too-quiet' | 'very-quiet'
  recommendation: string
  icon: ReactElement
  color: string
}

export function GainStagingHelper({ isVisible, className = "" }: GainStagingHelperProps) {
  const { levels, isMonitoring } = useAudioMonitoring()

  if (!isVisible || !isMonitoring) {
    return null
  }

  const analyzeGainStaging = (): GainAnalysis => {
    const targetRMS = -18.0 // Professional standard
    const currentRMS = levels.rms_db
    const gainAdjustment = targetRMS - currentRMS

    // Professional broadcast limits and optimal ranges
    if (levels.peak_db > -10) {
      return {
        currentRMS,
        targetRMS,
        gainAdjustment,
        status: 'too-hot',
        recommendation: `Reduce input gain by ${Math.abs(gainAdjustment).toFixed(1)}dB. Exceeding broadcast limit (-10dBFS).`,
        icon: <AlertTriangle className="w-4 h-4" />,
        color: 'text-red-400'
      }
    }

    if (currentRMS > -15) {
      return {
        currentRMS,
        targetRMS,
        gainAdjustment,
        status: 'slightly-hot',
        recommendation: `Slightly hot. Consider reducing gain by ${Math.abs(gainAdjustment).toFixed(1)}dB for optimal headroom.`,
        icon: <TrendingDown className="w-4 h-4" />,
        color: 'text-yellow-400'
      }
    }

    if (currentRMS < -30) {
      return {
        currentRMS,
        targetRMS,
        gainAdjustment,
        status: 'very-quiet',
        recommendation: `Very quiet signal. Increase gain by ${gainAdjustment.toFixed(1)}dB. Check your synth output level.`,
        icon: <TrendingUp className="w-4 h-4" />,
        color: 'text-yellow-400'
      }
    }

    if (currentRMS < -23) {
      return {
        currentRMS,
        targetRMS,
        gainAdjustment,
        status: 'too-quiet',
        recommendation: `Increase gain by ${gainAdjustment.toFixed(1)}dB to reach professional target (-18dBFS RMS).`,
        icon: <TrendingUp className="w-4 h-4" />,
        color: 'text-blue-400'
      }
    }

    // Optimal range: -21dB to -15dB RMS
    if (Math.abs(gainAdjustment) < 3) {
      return {
        currentRMS,
        targetRMS,
        gainAdjustment,
        status: 'optimal',
        recommendation: 'Excellent gain staging! Perfect for professional recording and plugin performance.',
        icon: <CheckCircle className="w-4 h-4" />,
        color: 'text-green-400'
      }
    }

    return {
      currentRMS,
      targetRMS,
      gainAdjustment,
      status: 'optimal',
      recommendation: 'Good gain staging. Ready for professional recording.',
      icon: <Info className="w-4 h-4" />,
      color: 'text-green-400'
    }
  }

  const analysis = analyzeGainStaging()

  const getBackgroundColor = (): string => {
    switch (analysis.status) {
      case 'too-hot':
        return 'bg-red-900/20 border-red-600/30'
      case 'slightly-hot':
        return 'bg-yellow-900/20 border-yellow-600/30'
      case 'too-quiet':
      case 'very-quiet':
        return 'bg-blue-900/20 border-blue-600/30'
      case 'optimal':
      default:
        return 'bg-green-900/20 border-green-600/30'
    }
  }

  return (
    <div className={`${getBackgroundColor()} border rounded-lg p-4 ${className}`}>
      <div className="flex items-start space-x-3">
        <div className={`${analysis.color} flex-shrink-0 mt-0.5`}>
          {analysis.icon}
        </div>
        
        <div className="flex-1 min-w-0">
          <div className="flex items-center justify-between mb-2">
            <h4 className="text-sm font-semibold text-gray-100">
              Gain Staging Analysis
            </h4>
            <div className="text-xs text-gray-400 font-mono">
              Current: {analysis.currentRMS.toFixed(1)}dB | Target: {analysis.targetRMS}dB
            </div>
          </div>
          
          <p className={`text-sm ${analysis.color} leading-relaxed`}>
            {analysis.recommendation}
          </p>
          
          {/* Professional context */}
          <div className="mt-3 pt-2 border-t border-gray-700/50">
            <div className="flex items-center justify-between text-xs text-gray-500">
              <span>Professional Standard: -18dBFS RMS</span>
              <span>Broadcast Limit: -10dBFS Peak</span>
            </div>
          </div>
        </div>
      </div>
    </div>
  )
}