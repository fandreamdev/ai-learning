interface EmptyStateProps {
  title?: string
  description?: string
}

export default function EmptyState({
  title = '暂无 Ticket',
  description = '尝试调整筛选条件，或点击右上角按钮创建第一个 Ticket。',
}: EmptyStateProps) {
  return (
    <div className="flex flex-col items-center justify-center rounded-lg border border-dashed border-gray-200 bg-white p-12 text-center transition-colors duration-200">
      <div className="mb-3 grid h-12 w-12 place-items-center rounded-full" style={{ backgroundColor: '#F8F9FA' }}>
        <span className="text-2xl" style={{ color: '#6C757D' }}>📋</span>
      </div>
      <p className="font-medium" style={{ color: '#6C757D' }}>{title}</p>
      <p className="mt-1 text-sm" style={{ color: '#6C757D', opacity: 0.7 }}>{description}</p>
    </div>
  )
}
