interface ErrorToastProps {
  message: string
  onRetry?: () => void
}

export default function ErrorToast({ message, onRetry }: ErrorToastProps) {
  return (
    <div className="flex items-center justify-between gap-3 rounded-lg border px-4 py-3 text-sm" style={{ borderColor: '#FF4D4F', backgroundColor: '#FFF1F0', color: '#FF4D4F' }}>
      <span>⚠️ {message}</span>
      {onRetry && (
        <button
          type="button"
          onClick={onRetry}
          className="rounded-lg border px-3 py-1 text-xs transition-colors duration-200 hover:opacity-80"
          style={{ borderColor: '#FF4D4F', color: '#FF4D4F', backgroundColor: 'transparent' }}
        >
          重试
        </button>
      )}
    </div>
  )
}
