import { STATUS_LABEL } from '@/constants/enums'
import type { TicketStatus } from '@/types/ticket'

const STATUS_BG: Record<TicketStatus, string> = {
  open: 'bg-gray-100 text-gray-700',
  in_progress: 'bg-blue-50 text-primary',
  done: 'bg-emerald-50 text-emerald-700',
  closed: 'bg-gray-200 text-gray-500',
}

export default function StatusBadge({ status }: { status: TicketStatus }) {
  return (
    <span
      className={`inline-flex items-center rounded-full px-2 py-0.5 text-xs font-medium ${STATUS_BG[status]}`}
    >
      {STATUS_LABEL[status]}
    </span>
  )
}
