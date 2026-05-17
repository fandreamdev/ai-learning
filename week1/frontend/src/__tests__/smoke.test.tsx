import { render, screen } from '@testing-library/react'
import { MemoryRouter, Route, Routes } from 'react-router-dom'
import { describe, expect, it } from 'vitest'

import NotFoundPage from '@/pages/NotFoundPage'

/**
 * 冒烟测试：仅测试无网络副作用的页面。
 * 列表页与详情页因带 fetch 副作用，由更专注的组件测试覆盖。
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

  it('NotFoundPage offers a link back to home', () => {
    render(
      <MemoryRouter initialEntries={['/anything']}>
        <Routes>
          <Route path="*" element={<NotFoundPage />} />
        </Routes>
      </MemoryRouter>,
    )
    expect(screen.getByText('返回首页')).toBeInTheDocument()
  })
})
