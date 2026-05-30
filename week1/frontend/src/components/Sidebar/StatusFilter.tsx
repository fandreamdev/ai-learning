import { STATUS_LABEL } from '@/constants/enums'
import type { TicketStatus } from '@/types/ticket'

interface StatusFilterProps {
  selected: TicketStatus[]
  onToggle: (status: TicketStatus) => void
}

const ORDER: TicketStatus[] = ['open', 'in_progress', 'done', 'closed']

export default function StatusFilter({ selected, onToggle }: StatusFilterProps) {
  return (
    <fieldset>
      <legend className="mb-2 font-medium" style={{ color: '#6C757D' }}>状态</legend>
      <div className="space-y-2">
        {ORDER.map((status) => {
          const checked = selected.includes(status)
          return (
            <label
              key={status}
              className="flex cursor-pointer items-center gap-2 transition-colors duration-200 hover:text-gray-900"
              style={{ color: '#6C757D' }}
            >
              <input
                type="checkbox"
                checked={checked}
                onChange={() => onToggle(status)}
                className="h-4 w-4 rounded border-gray-300 transition-colors duration-200 focus:ring-2 focus:ring-primary focus:ring-offset-2"
                style={{ accentColor: '#0066FF' }}
              />
              <span>{STATUS_LABEL[status]}</span>
            </label>
          )
        })}
      </div>
    </fieldset>
  )
}
