import { PRIORITY_COLOR, PRIORITY_LABEL } from '@/constants/enums'
import type { TicketPriority } from '@/types/ticket'

interface PriorityRadioProps {
  value: TicketPriority
  onChange: (next: TicketPriority) => void
  disabled?: boolean
}

const ORDER: TicketPriority[] = ['low', 'medium', 'high', 'urgent']

export default function PriorityRadio({ value, onChange, disabled }: PriorityRadioProps) {
  return (
    <div className="flex items-center gap-4" role="radiogroup" aria-label="优先级">
      {ORDER.map((p) => {
        const checked = value === p
        return (
          <label
            key={p}
            className="flex cursor-pointer items-center gap-1.5 text-sm transition-colors duration-200"
            style={{ color: '#6C757D' }}
          >
            <input
              type="radio"
              name="priority"
              value={p}
              checked={checked}
              onChange={() => onChange(p)}
              disabled={disabled}
              className="h-4 w-4 border-gray-300 transition-colors duration-200 focus:ring-2 focus:ring-primary focus:ring-offset-2"
              style={{ accentColor: '#0066FF' }}
            />
            <span
              aria-hidden
              className="inline-block h-2 w-2 rounded-full"
              style={{ backgroundColor: PRIORITY_COLOR[p] }}
            />
            {PRIORITY_LABEL[p]}
          </label>
        )
      })}
    </div>
  )
}
