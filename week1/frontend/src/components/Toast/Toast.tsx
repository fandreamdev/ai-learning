import { useEffect } from 'react'

import { cn } from '@/lib/cn'
import type { ToastItem } from '@/components/Toast/ToastContext'

const KIND_STYLE: Record<ToastItem['kind'], string> = {
  success: 'border-green-200 bg-green-50 text-green-800',
  error: 'border-red-200 bg-red-50 text-red-800',
  info: 'border-gray-200 bg-white text-gray-800',
}

const KIND_ICON: Record<ToastItem['kind'], string> = {
  success: '✓',
  error: '⚠',
  info: 'ℹ',
}

interface ToastProps {
  toast: ToastItem
  onDismiss: (id: string) => void
}

export default function Toast({ toast, onDismiss }: ToastProps) {
  const ttl = toast.kind === 'error' ? 5000 : 3000

  useEffect(() => {
    const timer = setTimeout(() => onDismiss(toast.id), ttl)
    return () => clearTimeout(timer)
  }, [toast.id, ttl, onDismiss])

  return (
    <div
      role="status"
      className={cn(
        'flex w-80 items-start gap-2 rounded-md border px-3 py-2 text-sm shadow-md',
        KIND_STYLE[toast.kind],
      )}
    >
      <span aria-hidden className="mt-0.5 select-none">
        {KIND_ICON[toast.kind]}
      </span>
      <span className="flex-1 leading-snug">{toast.message}</span>
      <button
        type="button"
        onClick={() => onDismiss(toast.id)}
        aria-label="关闭"
        className="text-current opacity-50 transition hover:opacity-100"
      >
        ✕
      </button>
    </div>
  )
}
