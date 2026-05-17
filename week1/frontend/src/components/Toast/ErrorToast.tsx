interface ErrorToastProps {
  message: string
  onRetry?: () => void
}

export default function ErrorToast({ message, onRetry }: ErrorToastProps) {
  return (
    <div className="flex items-center justify-between gap-3 rounded-md border border-red-200 bg-red-50 px-4 py-3 text-sm text-red-700">
      <span>⚠️ {message}</span>
      {onRetry && (
        <button
          type="button"
          onClick={onRetry}
          className="rounded-md border border-red-300 bg-white px-3 py-1 text-xs text-red-700 transition hover:bg-red-100"
        >
          重试
        </button>
      )}
    </div>
  )
}
