import { PAGE_SIZE_OPTIONS } from '@/constants/enums'

interface PaginationProps {
  page: number
  pageSize: number
  total: number
  onPageChange: (page: number) => void
  onPageSizeChange: (pageSize: number) => void
}

export default function Pagination({
  page,
  pageSize,
  total,
  onPageChange,
  onPageSizeChange,
}: PaginationProps) {
  const totalPages = Math.max(1, Math.ceil(total / pageSize))
  const isFirst = page <= 1
  const isLast = page >= totalPages

  if (total === 0) return null

  return (
    <div className="flex items-center justify-between gap-4 text-sm" style={{ color: '#6C757D' }}>
      <span>
        共 <strong style={{ color: '#1A1F26' }}>{total}</strong> 条 · 第{' '}
        <strong style={{ color: '#1A1F26' }}>{page}</strong> / {totalPages} 页
      </span>

      <div className="flex items-center gap-2">
        <button
          type="button"
          onClick={() => onPageChange(page - 1)}
          disabled={isFirst}
          className="rounded-lg border border-gray-200 bg-white px-3 py-1 transition-colors duration-200 hover:bg-gray-50 disabled:cursor-not-allowed disabled:opacity-40"
        >
          上一页
        </button>
        <button
          type="button"
          onClick={() => onPageChange(page + 1)}
          disabled={isLast}
          className="rounded-lg border border-gray-200 bg-white px-3 py-1 transition-colors duration-200 hover:bg-gray-50 disabled:cursor-not-allowed disabled:opacity-40"
        >
          下一页
        </button>

        <label className="ml-2 flex items-center gap-1 text-xs">
          每页
          <select
            value={pageSize}
            onChange={(e) => onPageSizeChange(Number(e.target.value))}
            className="rounded-lg border border-gray-200 bg-white px-2 py-1 text-xs transition-colors duration-200 focus:border-primary focus:outline-none focus:ring-2 focus:ring-primary/20"
          >
            {PAGE_SIZE_OPTIONS.map((n) => (
              <option key={n} value={n}>
                {n}
              </option>
            ))}
          </select>
          条
        </label>
      </div>
    </div>
  )
}
