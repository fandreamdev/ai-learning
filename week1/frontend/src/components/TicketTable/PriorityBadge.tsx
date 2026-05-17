import { PRIORITY_COLOR, PRIORITY_LABEL } from '@/constants/enums'
import type { TicketPriority } from '@/types/ticket'

export default function PriorityBadge({ priority }: { priority: TicketPriority }) {
  return (
    <span className="inline-flex items-center gap-1.5 text-xs font-medium text-gray-700">
      <span
        className="inline-block h-2 w-2 rounded-full"
        style={{ backgroundColor: PRIORITY_COLOR[priority] }}
        aria-hidden
      />
      {PRIORITY_LABEL[priority]}
    </span>
  )
}
