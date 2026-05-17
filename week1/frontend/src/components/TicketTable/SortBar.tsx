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
    <label className="flex items-center gap-2 text-sm text-gray-600">
      排序：
      <select
        value={value}
        onChange={(e) => {
          const [by, order] = e.target.value.split(':') as [SortBy, SortOrder]
          onChange(by, order)
        }}
        className="rounded-md border border-gray-300 bg-white px-2 py-1 text-sm focus:border-blue-500 focus:outline-none focus:ring-1 focus:ring-blue-500"
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
