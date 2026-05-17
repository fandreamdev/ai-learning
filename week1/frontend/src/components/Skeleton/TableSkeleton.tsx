/** 表格骨架屏，结构与 TicketTable 列对齐。 */
export default function TableSkeleton({ rows = 5 }: { rows?: number }) {
  return (
    <div className="overflow-hidden rounded-lg border border-gray-200 bg-white shadow-sm">
      <div className="border-b border-gray-200 bg-gray-50 px-4 py-3">
        <div className="h-3 w-24 animate-pulse rounded bg-gray-200" />
      </div>
      <ul className="divide-y divide-gray-100">
        {Array.from({ length: rows }).map((_, i) => (
          <li key={i} className="flex items-center gap-4 px-4 py-4">
            <div className="h-3 w-10 animate-pulse rounded bg-gray-200" />
            <div className="h-3 flex-1 animate-pulse rounded bg-gray-200" />
            <div className="h-3 w-16 animate-pulse rounded bg-gray-200" />
            <div className="h-3 w-16 animate-pulse rounded bg-gray-200" />
            <div className="h-3 w-20 animate-pulse rounded bg-gray-200" />
            <div className="h-3 w-32 animate-pulse rounded bg-gray-200" />
          </li>
        ))}
      </ul>
    </div>
  )
}
