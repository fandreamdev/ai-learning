interface EmptyStateProps {
  title?: string
  description?: string
}

export default function EmptyState({
  title = '暂无 Ticket',
  description = '尝试调整筛选条件，或点击右上角按钮创建第一个 Ticket。',
}: EmptyStateProps) {
  return (
    <div className="flex flex-col items-center justify-center rounded-lg border border-dashed border-gray-300 bg-white p-12 text-center">
      <div className="mb-3 grid h-12 w-12 place-items-center rounded-full bg-gray-100 text-2xl text-gray-400">
        📋
      </div>
      <p className="font-medium text-gray-700">{title}</p>
      <p className="mt-1 text-sm text-gray-500">{description}</p>
    </div>
  )
}
