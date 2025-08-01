import { useState, useCallback, useRef } from 'react'

export interface Notification {
  id: string
  type: 'error' | 'warning' | 'info' | 'success'
  title: string
  message: string
  duration?: number
}

export function useNotifications() {
  const [notifications, setNotifications] = useState<Notification[]>([])
  const idCounterRef = useRef(0)

  const addNotification = useCallback((
    type: Notification['type'],
    title: string,
    message: string,
    duration: number = 5000
  ) => {
    const id = `notification-${idCounterRef.current++}`
    
    const notification: Notification = {
      id,
      type,
      title,
      message,
      duration
    }
    
    setNotifications(prev => [...prev, notification])
    
    // Auto-remove after duration
    if (duration > 0) {
      setTimeout(() => {
        setNotifications(prev => prev.filter(n => n.id !== id))
      }, duration)
    }
    
    return id
  }, [])

  const removeNotification = useCallback((id: string) => {
    setNotifications(prev => prev.filter(n => n.id !== id))
  }, [])

  const clearAllNotifications = useCallback(() => {
    setNotifications([])
  }, [])

  // Convenience methods following professional audio app patterns
  const showError = useCallback((title: string, message: string) => {
    return addNotification('error', title, message, 7000) // Errors stay longer
  }, [addNotification])

  const showWarning = useCallback((title: string, message: string) => {
    return addNotification('warning', title, message, 5000)
  }, [addNotification])

  const showInfo = useCallback((title: string, message: string) => {
    return addNotification('info', title, message, 4000)
  }, [addNotification])

  const showSuccess = useCallback((title: string, message: string) => {
    return addNotification('success', title, message, 3000) // Success fades quickly
  }, [addNotification])

  return {
    notifications,
    addNotification,
    removeNotification,
    clearAllNotifications,
    showError,
    showWarning,
    showInfo,
    showSuccess
  }
}