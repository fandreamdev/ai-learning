import { act, renderHook } from '@testing-library/react'
import { MemoryRouter, useLocation } from 'react-router-dom'
import { describe, expect, it } from 'vitest'

import { useTicketListUrlState } from '@/hooks/useTicketListUrlState'

function makeWrapper(initialEntries: string[]) {
  return ({ children }: { children: React.ReactNode }) => (
    <MemoryRouter initialEntries={initialEntries}>{children}</MemoryRouter>
  )
}

describe('useTicketListUrlState', () => {
  it('parses defaults from empty URL', () => {
    const { result } = renderHook(() => useTicketListUrlState(), {
      wrapper: makeWrapper(['/']),
    })
    expect(result.current.query.page).toBe(1)
    expect(result.current.query.page_size).toBe(20)
    expect(result.current.query.sort_by).toBe('created_at')
    expect(result.current.query.sort_order).toBe('desc')
  })

  it('toggleStatus adds and removes from list', () => {
    const { result } = renderHook(() => useTicketListUrlState(), {
      wrapper: makeWrapper(['/']),
    })
    act(() => result.current.toggleStatus('open'))
    expect(result.current.query.status).toEqual(['open'])
    act(() => result.current.toggleStatus('in_progress'))
    expect(result.current.query.status).toEqual(['open', 'in_progress'])
    act(() => result.current.toggleStatus('open'))
    expect(result.current.query.status).toEqual(['in_progress'])
  })

  it('togglePriority works the same way', () => {
    const { result } = renderHook(() => useTicketListUrlState(), {
      wrapper: makeWrapper(['/']),
    })
    act(() => result.current.togglePriority('high'))
    expect(result.current.query.priority).toEqual(['high'])
    act(() => result.current.togglePriority('high'))
    expect(result.current.query.priority).toBeUndefined()
  })

  it('setKeyword resets page to 1', () => {
    const { result } = renderHook(() => useTicketListUrlState(), {
      wrapper: makeWrapper(['/?page=3']),
    })
    expect(result.current.query.page).toBe(3)
    act(() => result.current.setKeyword('login'))
    expect(result.current.query.keyword).toBe('login')
    expect(result.current.query.page).toBe(1)
  })

  it('clearAll removes all filters but keeps defaults', () => {
    const { result } = renderHook(() => useTicketListUrlState(), {
      wrapper: makeWrapper(['/?status=open&keyword=hi&page=2']),
    })
    act(() => result.current.clearAll())
    expect(result.current.query.status).toBeUndefined()
    expect(result.current.query.keyword).toBeUndefined()
    expect(result.current.query.page).toBe(1)
  })

  it('writes only non-default params back to URL', () => {
    function Probe(): null {
      const ctx = useTicketListUrlState()
      const loc = useLocation()
      ;(globalThis as Record<string, unknown>).__ctx = ctx
      ;(globalThis as Record<string, unknown>).__loc = loc
      return null
    }
    const { rerender } = renderHook(() => null, {
      wrapper: ({ children }) => (
        <MemoryRouter initialEntries={['/']}>
          <Probe />
          {children}
        </MemoryRouter>
      ),
    })
    rerender()
    const ctx = (globalThis as Record<string, unknown>).__ctx as ReturnType<
      typeof useTicketListUrlState
    >
    act(() => ctx.setKeyword('login'))
    const loc = (globalThis as Record<string, unknown>).__loc as ReturnType<typeof useLocation>
    expect(loc.search).toBe('?keyword=login')
  })
})
