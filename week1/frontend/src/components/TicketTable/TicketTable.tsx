import { Link } from 'react-router-dom'

import EmptyState from '@/components/EmptyState/EmptyState'
import TableSkeleton from '@/components/Skeleton/TableSkeleton'
import PriorityBadge from '@/components/TicketTable/PriorityBadge'
import StatusBadge from '@/components/TicketTable/StatusBadge'
import { formatDateTime } from '@/lib/format'
import type { Ticket } from '@/types/ticket'

interface TicketTableProps {
  items: Ticket[]
  loading: boolean
}

export default function TicketTable({ items }: TicketTableProps) {
  if (items.length === 0) return <EmptyState />

  return (
    <div className="overflow-hidden rounded-lg border border-gray-200 bg-white">
      <table className="min-w-full divide-y divide-gray-100">
        <thead className="text-left text-xs uppercase" style={{ color: '#6C757D', backgroundColor: '#F8F9FA' }}>
          <tr>
            <th className="w-16 px-4 py-3 font-medium">ID</th>
            <th className="px-4 py-3 font-medium">标题</th>
            <th className="w-24 px-4 py-3 font-medium">状态</th>
            <th className="w-24 px-4 py-3 font-medium">优先级</th>
            <th className="w-28 px-4 py-3 font-medium">负责人</th>
            <th className="w-44 px-4 py-3 font-medium">更新时间</th>
          </tr>
        </thead>
        <tbody className="divide-y divide-gray-100 text-sm" style={{ color: '#1A1F26' }}>
          {items.map((t) => (
            <tr key={t.id} className="transition-colors duration-200 hover:bg-gray-50/80">
              <td className="px-4 py-3" style={{ color: '#6C757D' }}>#{t.id}</td>
              <td className="px-4 py-3">
                <Link
                  to={`/tickets/${t.id}`}
                  className="font-medium transition-colors duration-200 hover:underline"
                  style={{ color: '#1A1F26' }}
                >
                  {t.title}
                </Link>
                {t.tags?.length ? (
                  <div className="mt-1 flex flex-wrap gap-1">
                    {t.tags.map((tag) => (
                      <span
                        key={tag}
                        className="inline-flex items-center rounded-md px-1.5 py-0.5 text-xs"
                        style={{ backgroundColor: '#F0F4FF', color: '#0066FF' }}
                      >
                        {tag}
                      </span>
                    ))}
                  </div>
                ) : null}
              </td>
              <td className="px-4 py-3">
                <StatusBadge status={t.status} />
              </td>
              <td className="px-4 py-3">
                <PriorityBadge priority={t.priority} />
              </td>
              <td className="px-4 py-3" style={{ color: '#6C757D' }}>{t.assignee || '-'}</td>
              <td className="px-4 py-3 text-xs" style={{ color: '#6C757D' }}>{formatDateTime(t.updated_at)}</td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  )
}
