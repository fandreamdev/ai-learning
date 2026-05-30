/** 表格骨架屏，结构与 TicketTable 列对齐。 */
export default function TableSkeleton({ rows = 5 }: { rows?: number }) {
  return (
    <div className="overflow-hidden rounded-lg border border-gray-200 bg-white">
      <div className="border-b border-gray-100 px-4 py-3" style={{ backgroundColor: '#F8F9FA' }}>
        <div className="h-3 w-24 animate-pulse rounded" style={{ backgroundColor: '#E9ECEF' }} />
      </div>
      <ul className="divide-y divide-gray-100">
        {Array.from({ length: rows }).map((_, i) => (
          <li key={i} className="flex items-center gap-4 px-4 py-4">
            <div className="h-3 w-10 animate-pulse rounded" style={{ backgroundColor: '#E9ECEF' }} />
            <div className="h-3 flex-1 animate-pulse rounded" style={{ backgroundColor: '#E9ECEF' }} />
            <div className="h-3 w-16 animate-pulse rounded" style={{ backgroundColor: '#E9ECEF' }} />
            <div className="h-3 w-16 animate-pulse rounded" style={{ backgroundColor: '#E9ECEF' }} />
            <div className="h-3 w-20 animate-pulse rounded" style={{ backgroundColor: '#E9ECEF' }} />
            <div className="h-3 w-32 animate-pulse rounded" style={{ backgroundColor: '#E9ECEF' }} />
          </li>
        ))}
      </ul>
    </div>
  )
}
