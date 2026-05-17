import { useState } from 'react'

import { updateTicketStatus } from '@/api/tickets'
import { STATUS_LABEL, STATUS_TRANSITIONS } from '@/constants/enums'
import { useToast } from '@/components/Toast/useToast'
import { ApiError } from '@/types/api'
import type { Ticket, TicketStatus } from '@/types/ticket'

interface StatusSelectProps {
  ticket: Ticket
  onChanged: (next: Ticket) => void
}

/**
 * 详情页状态切换下拉。
 *
 * 仅展示 ``STATUS_TRANSITIONS`` 允许的目标，避免触发后端 40002。
 * 当前状态出现在第一项且 disabled，便于用户感知"当前 = X"。
 */
export default function StatusSelect({ ticket, onChanged }: StatusSelectProps) {
  const toast = useToast()
  const [pending, setPending] = useState(false)

  const allowed = STATUS_TRANSITIONS[ticket.status]

  const handleChange = async (e: React.ChangeEvent<HTMLSelectElement>) => {
    const next = e.target.value as TicketStatus
    if (next === ticket.status) return
    setPending(true)
    try {
      const updated = await updateTicketStatus(ticket.id, next)
      onChanged(updated)
      toast.success(`已切换到 ${STATUS_LABEL[next]}`)
    } catch (err) {
      const message = err instanceof ApiError ? err.message : '状态切换失败'
      toast.error(message)
    } finally {
      setPending(false)
    }
  }

  return (
    <select
      value={ticket.status}
      onChange={handleChange}
      disabled={pending}
      className="rounded-md border border-gray-300 bg-white px-2 py-1 text-sm focus:border-blue-500 focus:outline-none focus:ring-1 focus:ring-blue-500 disabled:cursor-not-allowed disabled:opacity-60"
      aria-label="切换状态"
    >
      <option value={ticket.status} disabled>
        {STATUS_LABEL[ticket.status]}（当前）
      </option>
      {allowed.map((s) => (
        <option key={s} value={s}>
          → {STATUS_LABEL[s]}
        </option>
      ))}
    </select>
  )
}
