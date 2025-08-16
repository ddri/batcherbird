import { Dialog, DialogContent, DialogHeader, DialogTitle } from '@/components/ui/dialog'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { CheckCircle, Play, FolderOpen } from 'lucide-react'
import { useState, useEffect, useRef } from 'react'
import { useFileSystem } from '@/hooks/useTauri'
import { desktopDir } from '@tauri-apps/api/path'

interface SessionInitializationWizardProps {
  isOpen: boolean
  onClose: () => void
  onComplete: (sessionData: { projectName: string; outputDirectory: string }) => void
}

export function SessionInitializationWizard({ isOpen, onClose, onComplete }: SessionInitializationWizardProps) {
  const [projectName, setProjectName] = useState('')
  const [outputDirectory, setOutputDirectory] = useState('')
  const projectNameRef = useRef<HTMLInputElement>(null)
  const { selectOutputDirectory } = useFileSystem()

  // Initialize default output directory
  useEffect(() => {
    const initOutputDirectory = async () => {
      if (!outputDirectory) {
        try {
          const desktop = await desktopDir()
          setOutputDirectory(desktop)
        } catch (error) {
          console.error('Failed to get desktop directory:', error)
          setOutputDirectory('') // Will show as empty, user can select
        }
      }
    }
    initOutputDirectory()
  }, [outputDirectory])

  // Auto-focus project name input when wizard opens
  useEffect(() => {
    if (isOpen && projectNameRef.current) {
      // Small delay to ensure dialog is fully rendered
      setTimeout(() => {
        projectNameRef.current?.focus()
      }, 100)
    }
  }, [isOpen])

  const handleSelectDirectory = async () => {
    try {
      const directory = await selectOutputDirectory()
      setOutputDirectory(directory)
      console.log('Selected directory:', directory)
    } catch (error) {
      console.error('Directory selection failed:', error)
    }
  }

  const handleComplete = () => {
    const sessionData = { projectName: projectName.trim(), outputDirectory }
    console.log('Session initialized with:', sessionData)
    onComplete(sessionData)
    onClose()
  }

  return (
    <Dialog open={isOpen} onOpenChange={() => {}}>
      <DialogContent className="max-w-2xl bg-gray-900 border-gray-700">
        <DialogHeader>
          <DialogTitle className="text-xl text-gray-100 flex items-center space-x-2">
            <Play className="w-6 h-6 text-blue-400" />
            <span>Session Initialization</span>
          </DialogTitle>
        </DialogHeader>

        <div className="space-y-6 p-6">
          {/* Header */}
          <div className="text-center space-y-4">
            <div className="w-16 h-16 bg-blue-600 rounded-full flex items-center justify-center mx-auto">
              <CheckCircle className="w-8 h-8 text-white" />
            </div>
            <div>
              <h2 className="text-2xl font-bold text-gray-100 mb-2">Welcome to BatcherBird</h2>
              <p className="text-gray-400 max-w-md mx-auto">
                Let's set up your professional sampling session with just the essentials.
              </p>
            </div>
          </div>

          {/* Project Configuration */}
          <div className="space-y-4 max-w-md mx-auto">
            <div className="space-y-2">
              <Label htmlFor="project-name" className="text-gray-200 text-base font-medium">
                Project Name *
              </Label>
              <Input
                ref={projectNameRef}
                id="project-name"
                value={projectName}
                onChange={(e) => setProjectName(e.target.value)}
                placeholder="My Synth Samples"
                className="bg-gray-800 border-gray-600 text-gray-100 focus:border-blue-500 focus:ring-1 focus:ring-blue-500"
              />
              <p className="text-xs text-gray-500">
                Choose a descriptive name for your sampling project
              </p>
            </div>

            <div className="space-y-2">
              <Label className="text-gray-200 text-base font-medium">
                Output Directory
              </Label>
              <div className="flex space-x-2">
                <Input
                  value={outputDirectory}
                  onChange={(e) => setOutputDirectory(e.target.value)}
                  className="bg-gray-800 border-gray-600 text-gray-100 flex-1 focus:border-blue-500 focus:ring-1 focus:ring-blue-500"
                />
                <Button
                  onClick={handleSelectDirectory}
                  variant="outline"
                  className="bg-transparent border-gray-600 text-gray-100 hover:bg-gray-800 hover:border-blue-500"
                >
                  <FolderOpen className="w-4 h-4" />
                </Button>
              </div>
              <p className="text-xs text-gray-500">
                Directory where recorded samples will be saved - click folder icon to browse
              </p>
            </div>
          </div>

          {/* Action Buttons */}
          <div className="flex justify-between items-center pt-4 border-t border-gray-700">
            <Button
              variant="outline"
              onClick={onClose}
              className="bg-transparent border-gray-600 text-gray-100 hover:bg-gray-800"
            >
              Cancel
            </Button>

            <Button
              onClick={handleComplete}
              disabled={!projectName.trim()}
              className="bg-green-600 hover:bg-green-700 text-white"
            >
              Start Recording Session
            </Button>
          </div>
        </div>
      </DialogContent>
    </Dialog>
  )
}