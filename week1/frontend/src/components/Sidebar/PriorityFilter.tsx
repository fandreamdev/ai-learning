import { PRIORITY_COLOR, PRIORITY_LABEL } from '@/constants/enums'
import type { TicketPriority } from '@/types/ticket'

interface PriorityFilterProps {
  selected: TicketPriority[]
  onToggle: (priority: TicketPriority) => void
}

const ORDER: TicketPriority[] = ['low', 'medium', 'high', 'urgent']

export default function PriorityFilter({ selected, onToggle }: PriorityFilterProps) {
  return (
    <fieldset>
      <legend className="mb-2 font-medium" style={{ color: '#6C757D' }}>优先级</legend>
      <div className="space-y-2">
        {ORDER.map((p) => {
          const checked = selected.includes(p)
          return (
            <label
              key={p}
              className="flex cursor-pointer items-center gap-2 transition-colors duration-200 hover:text-gray-900"
              style={{ color: '#6C757D' }}
            >
              <input
                type="checkbox"
                checked={checked}
                onChange={() => onToggle(p)}
                className="h-4 w-4 rounded border-gray-300 transition-colors duration-200 focus:ring-2 focus:ring-primary focus:ring-offset-2"
                style={{ accentColor: '#0066FF' }}
              />
              <span
                className="inline-block h-2 w-2 rounded-full"
                style={{ backgroundColor: PRIORITY_COLOR[p] }}
                aria-hidden
              />
              <span>{PRIORITY_LABEL[p]}</span>
            </label>
          )
        })}
      </div>
    </fieldset>
  )
}
