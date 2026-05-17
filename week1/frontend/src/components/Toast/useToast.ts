import { useContext } from 'react'

import { ToastContext, type ToastContextValue } from '@/components/Toast/ToastContext'

export function useToast(): ToastContextValue {
  const ctx = useContext(ToastContext)
  if (!ctx) {
    throw new Error('useToast 必须在 <ToastProvider> 内部使用')
  }
  return ctx
}
