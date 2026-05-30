import { SORT_OPTIONS } from '@/constants/enums'
import type { SortBy, SortOrder } from '@/types/ticket'

interface SortBarProps {
  sortBy: SortBy
  sortOrder: SortOrder
  onChange: (sortBy: SortBy, sortOrder: SortOrder) => void
}

export default function SortBar({ sortBy, sortOrder, onChange }: SortBarProps) {
  const value = `${sortBy}:${sortOrder}`
  return (
    <label className="flex items-center gap-2 text-sm" style={{ color: '#6C757D' }}>
      排序：
      <select
        value={value}
        onChange={(e) => {
          const [by, order] = e.target.value.split(':') as [SortBy, SortOrder]
          onChange(by, order)
        }}
        className="rounded-lg border border-gray-200 bg-white px-2 py-1 text-sm transition-colors duration-200 focus:border-primary focus:outline-none focus:ring-2 focus:ring-primary/20"
        style={{ color: '#1A1F26' }}
      >
        {SORT_OPTIONS.map((opt) => (
          <option key={opt.value} value={opt.value}>
            {opt.label}
          </option>
        ))}
      </select>
    </label>
  )
}
