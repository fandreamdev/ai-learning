import { render, screen } from '@testing-library/react'
import { describe, expect, it } from 'vitest'

import StatusSelect from '@/components/TicketDetail/StatusSelect'
import ToastProvider from '@/components/Toast/ToastProvider'
import type { Ticket } from '@/types/ticket'

const baseTicket: Ticket = {
  id: 1,
  title: 't',
  description: null,
  status: 'open',
  priority: 'medium',
  assignee: null,
  tags: [],
  created_at: '2026-05-17T00:00:00',
  updated_at: '2026-05-17T00:00:00',
}

const wrap = (ui: React.ReactNode) => <ToastProvider>{ui}</ToastProvider>

describe('StatusSelect', () => {
  it('shows current status as a disabled current option', () => {
    render(wrap(<StatusSelect ticket={baseTicket} onChanged={() => {}} />))
    const current = screen.getByRole('option', { name: /待处理（当前）/ }) as HTMLOptionElement
    expect(current.disabled).toBe(true)
  })

  it('only shows allowed transitions for open: in_progress + closed', () => {
    render(wrap(<StatusSelect ticket={baseTicket} onChanged={() => {}} />))
    expect(screen.queryByRole('option', { name: /处理中/ })).toBeInTheDocument()
    expect(screen.queryByRole('option', { name: /已关闭/ })).toBeInTheDocument()
    expect(screen.queryByRole('option', { name: /^→ 已完成/ })).not.toBeInTheDocument()
  })

  it('only shows 已关闭 for done', () => {
    render(wrap(<StatusSelect ticket={{ ...baseTicket, status: 'done' }} onChanged={() => {}} />))
    expect(screen.queryByRole('option', { name: /^→ 已关闭/ })).toBeInTheDocument()
    expect(screen.queryByRole('option', { name: /^→ 处理中/ })).not.toBeInTheDocument()
  })
})
