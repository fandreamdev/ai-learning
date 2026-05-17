import { useCallback, useMemo, useState, type ReactNode } from 'react'

import Toast from '@/components/Toast/Toast'
import {
  ToastContext,
  type ToastContextValue,
  type ToastItem,
  type ToastKind,
} from '@/components/Toast/ToastContext'

/**
 * 全局 Toast Provider。
 *
 * 顶部右侧浮层；多条堆叠；error 5s 自动消失，其它 3s。
 */
export default function ToastProvider({ children }: { children: ReactNode }) {
  const [toasts, setToasts] = useState<ToastItem[]>([])

  const dismiss = useCallback((id: string) => {
    setToasts((prev) => prev.filter((t) => t.id !== id))
  }, [])

  const show = useCallback((kind: ToastKind, message: string) => {
    const id =
      typeof crypto !== 'undefined' && 'randomUUID' in crypto
        ? crypto.randomUUID()
        : Math.random().toString(36).slice(2)
    setToasts((prev) => [...prev, { id, kind, message }])
  }, [])

  const value = useMemo<ToastContextValue>(
    () => ({
      show,
      success: (m) => show('success', m),
      error: (m) => show('error', m),
      info: (m) => show('info', m),
    }),
    [show],
  )

  return (
    <ToastContext.Provider value={value}>
      {children}
      <div className="pointer-events-none fixed right-4 top-4 z-[100] flex flex-col gap-2">
        {toasts.map((t) => (
          <div key={t.id} className="pointer-events-auto">
            <Toast toast={t} onDismiss={dismiss} />
          </div>
        ))}
      </div>
    </ToastContext.Provider>
  )
}
