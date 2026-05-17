import { renderHook, waitFor } from '@testing-library/react'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'

import * as ticketsApi from '@/api/tickets'
import { useTickets } from '@/hooks/useTickets'
import { ApiError, type PageData } from '@/types/api'
import type { Ticket, TicketListQuery } from '@/types/ticket'

vi.mock('@/api/tickets')

const fakePage: PageData<Ticket> = {
  items: [
    {
      id: 1,
      title: 'a',
      description: null,
      status: 'open',
      priority: 'medium',
      assignee: null,
      tags: [],
      created_at: '2026-05-17T00:00:00',
      updated_at: '2026-05-17T00:00:00',
    },
  ],
  total: 1,
  page: 1,
  page_size: 20,
}

const baseQuery: TicketListQuery = { page: 1, page_size: 20 }

describe('useTickets', () => {
  beforeEach(() => {
    vi.mocked(ticketsApi.listTickets).mockReset()
  })
  afterEach(() => {
    vi.restoreAllMocks()
  })

  it('returns loading then data on success', async () => {
    vi.mocked(ticketsApi.listTickets).mockResolvedValue(fakePage)
    const { result } = renderHook(() => useTickets(baseQuery))
    expect(result.current.loading).toBe(true)
    await waitFor(() => expect(result.current.loading).toBe(false))
    expect(result.current.data?.total).toBe(1)
    expect(result.current.error).toBeNull()
  })

  it('exposes ApiError on failure', async () => {
    vi.mocked(ticketsApi.listTickets).mockRejectedValue(new ApiError(50001, 'boom'))
    const { result } = renderHook(() => useTickets(baseQuery))
    await waitFor(() => expect(result.current.loading).toBe(false))
    expect(result.current.error?.code).toBe(50001)
    expect(result.current.error?.message).toBe('boom')
  })

  it('refetches when reload is called', async () => {
    vi.mocked(ticketsApi.listTickets).mockResolvedValue(fakePage)
    const { result } = renderHook(() => useTickets(baseQuery))
    await waitFor(() => expect(result.current.loading).toBe(false))
    expect(ticketsApi.listTickets).toHaveBeenCalledTimes(1)
    result.current.reload()
    await waitFor(() => expect(ticketsApi.listTickets).toHaveBeenCalledTimes(2))
  })

  it('refetches when query changes', async () => {
    vi.mocked(ticketsApi.listTickets).mockResolvedValue(fakePage)
    const { rerender } = renderHook(({ q }) => useTickets(q), {
      initialProps: { q: baseQuery },
    })
    await waitFor(() => expect(ticketsApi.listTickets).toHaveBeenCalledTimes(1))
    rerender({ q: { ...baseQuery, page: 2 } })
    await waitFor(() => expect(ticketsApi.listTickets).toHaveBeenCalledTimes(2))
  })
})
