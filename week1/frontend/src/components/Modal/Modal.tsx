import { useEffect, type ReactNode } from 'react'

import { cn } from '@/lib/cn'

interface ModalProps {
  open: boolean
  onClose: () => void
  title?: string
  size?: 'sm' | 'md' | 'lg'
  closeOnOverlayClick?: boolean
  closeOnEsc?: boolean
  children: ReactNode
  footer?: ReactNode
}

const SIZE_CLASS: Record<NonNullable<ModalProps['size']>, string> = {
  sm: 'max-w-md',
  md: 'max-w-xl',
  lg: 'max-w-3xl',
}

/**
 * 通用弹窗。
 *
 * - fixed 全屏蒙层 + 居中容器
 * - 打开时锁住 body 滚动，关闭时还原（cleanup 安全）
 * - 监听 keydown(Esc)；可由父级关闭
 */
export default function Modal({
  open,
  onClose,
  title,
  size = 'md',
  closeOnOverlayClick = true,
  closeOnEsc = true,
  children,
  footer,
}: ModalProps) {
  useEffect(() => {
    if (!open) return
    const previous = document.body.style.overflow
    document.body.style.overflow = 'hidden'
    return () => {
      document.body.style.overflow = previous
    }
  }, [open])

  useEffect(() => {
    if (!open || !closeOnEsc) return
    const onKey = (e: KeyboardEvent) => {
      if (e.key === 'Escape') {
        e.stopPropagation()
        onClose()
      }
    }
    window.addEventListener('keydown', onKey)
    return () => window.removeEventListener('keydown', onKey)
  }, [open, closeOnEsc, onClose])

  if (!open) return null

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/40 px-4 py-8"
      role="dialog"
      aria-modal="true"
      aria-labelledby={title ? 'modal-title' : undefined}
      onClick={() => {
        if (closeOnOverlayClick) onClose()
      }}
    >
      <div
        className={cn(
          'flex w-full flex-col overflow-hidden rounded-lg bg-white shadow-xl',
          SIZE_CLASS[size],
        )}
        onClick={(e) => e.stopPropagation()}
      >
        {title && (
          <header className="flex items-center justify-between border-b border-gray-200 px-5 py-3">
            <h2 id="modal-title" className="text-base font-semibold text-gray-900">
              {title}
            </h2>
            <button
              type="button"
              onClick={onClose}
              aria-label="关闭"
              className="text-gray-400 transition hover:text-gray-700"
            >
              ✕
            </button>
          </header>
        )}
        <div className="px-5 py-4">{children}</div>
        {footer && (
          <footer className="flex items-center justify-end gap-2 border-t border-gray-200 bg-gray-50 px-5 py-3">
            {footer}
          </footer>
        )}
      </div>
    </div>
  )
}
