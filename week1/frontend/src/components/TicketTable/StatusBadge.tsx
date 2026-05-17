import { STATUS_LABEL } from '@/constants/enums'
import type { TicketStatus } from '@/types/ticket'

const STATUS_BG: Record<TicketStatus, string> = {
  open: 'bg-gray-100 text-gray-700',
  in_progress: 'bg-blue-100 text-blue-700',
  done: 'bg-green-100 text-green-700',
  closed: 'bg-gray-300 text-gray-600',
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
