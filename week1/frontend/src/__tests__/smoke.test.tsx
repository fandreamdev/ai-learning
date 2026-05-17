import { render, screen } from '@testing-library/react'
import { MemoryRouter, Route, Routes } from 'react-router-dom'
import { describe, expect, it } from 'vitest'

import NotFoundPage from '@/pages/NotFoundPage'
import TicketDetailPage from '@/pages/TicketDetailPage'

/**
 * 冒烟测试：仅测试无网络副作用的页面。
 * 列表页的渲染由更专注的组件测试覆盖，避免触发真实 fetch。
 */
describe('routing smoke', () => {
  it('renders 404 for unknown routes', () => {
    render(
      <MemoryRouter initialEntries={['/no-such-route']}>
        <Routes>
          <Route path="*" element={<NotFoundPage />} />
        </Routes>
      </MemoryRouter>,
    )
    expect(screen.getByText('404')).toBeInTheDocument()
  })

  it('renders detail placeholder with id from URL', () => {
    render(
      <MemoryRouter initialEntries={['/tickets/42']}>
        <Routes>
          <Route path="/tickets/:id" element={<TicketDetailPage />} />
        </Routes>
      </MemoryRouter>,
    )
    expect(screen.getByText(/Ticket #42/)).toBeInTheDocument()
  })
})
