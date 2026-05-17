import { render, screen } from '@testing-library/react'
import { MemoryRouter } from 'react-router-dom'
import { describe, expect, it } from 'vitest'

import TicketTable from '@/components/TicketTable/TicketTable'
import type { Ticket } from '@/types/ticket'

const fixture = (overrides: Partial<Ticket> = {}): Ticket => ({
  id: 1,
  title: 'fix login captcha',
  description: 'captcha not refreshing',
  status: 'open',
  priority: 'high',
  assignee: 'zhang',
  tags: ['bug', 'frontend'],
  created_at: '2026-05-17T09:30:00',
  updated_at: '2026-05-17T10:30:00',
  ...overrides,
})

const wrap = (ui: React.ReactNode) => <MemoryRouter>{ui}</MemoryRouter>

describe('TicketTable', () => {
  it('shows skeleton when loading', () => {
    render(wrap(<TicketTable items={[]} loading={true} />))
    // 骨架屏使用了 animate-pulse，而 EmptyState 不会出现
    expect(screen.queryByText('暂无 Ticket')).not.toBeInTheDocument()
  })

  it('shows empty state when items are empty and not loading', () => {
    render(wrap(<TicketTable items={[]} loading={false} />))
    expect(screen.getByText('暂无 Ticket')).toBeInTheDocument()
  })

  it('renders rows for items', () => {
    render(
      wrap(
        <TicketTable
          items={[fixture(), fixture({ id: 2, title: 'add export', priority: 'medium' })]}
          loading={false}
        />,
      ),
    )
    expect(screen.getByText('fix login captcha')).toBeInTheDocument()
    expect(screen.getByText('add export')).toBeInTheDocument()
    expect(screen.getByText('#1')).toBeInTheDocument()
    expect(screen.getByText('#2')).toBeInTheDocument()
  })
})
