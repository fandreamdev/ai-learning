import Modal from '@/components/Modal/Modal'

interface ConfirmDialogProps {
  open: boolean
  title: string
  description?: string
  confirmText?: string
  cancelText?: string
  destructive?: boolean
  loading?: boolean
  onConfirm: () => void
  onCancel: () => void
}

/**
 * 二次确认弹窗。
 *
 * - destructive=true 时确认按钮显示为红色
 * - loading 中禁用按钮防止重复点击
 */
export default function ConfirmDialog({
  open,
  title,
  description,
  confirmText = '确认',
  cancelText = '取消',
  destructive = false,
  loading = false,
  onConfirm,
  onCancel,
}: ConfirmDialogProps) {
  return (
    <Modal
      open={open}
      onClose={loading ? () => {} : onCancel}
      title={title}
      size="sm"
      closeOnOverlayClick={!loading}
      closeOnEsc={!loading}
      footer={
        <>
          <button
            type="button"
            onClick={onCancel}
            disabled={loading}
            className="rounded-lg border border-gray-200 bg-white px-4 py-1.5 text-sm transition-colors duration-200 hover:bg-gray-50 disabled:cursor-not-allowed disabled:opacity-60"
            style={{ color: '#6C757D' }}
          >
            {cancelText}
          </button>
          <button
            type="button"
            onClick={onConfirm}
            disabled={loading}
            className={
              destructive
                ? 'rounded-lg px-4 py-1.5 text-sm font-medium text-white transition-colors duration-200 hover:opacity-90 disabled:cursor-not-allowed disabled:opacity-60'
                : 'rounded-lg px-4 py-1.5 text-sm font-medium text-white transition-colors duration-200 hover:opacity-90 disabled:cursor-not-allowed disabled:opacity-60'
            }
            style={destructive ? { backgroundColor: '#FF4D4F' } : { backgroundColor: '#0066FF' }}
          >
            {loading ? '处理中...' : confirmText}
          </button>
        </>
      }
    >
      {description && <p className="text-sm leading-relaxed text-gray-700">{description}</p>}
    </Modal>
  )
}
