import { renderHook, waitFor } from '@testing-library/react'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'

import * as ticketsApi from '@/api/tickets'
import { useTicket } from '@/hooks/useTicket'
import { ApiError } from '@/types/api'
import type { Ticket } from '@/types/ticket'

vi.mock('@/api/tickets')

const fakeTicket: Ticket = {
  id: 1,
  title: 'a',
  description: null,
  status: 'open',
  priority: 'medium',
  assignee: null,
  tags: [],
  created_at: '2026-05-17T00:00:00',
  updated_at: '2026-05-17T00:00:00',
}

describe('useTicket', () => {
  beforeEach(() => {
    vi.mocked(ticketsApi.getTicket).mockReset()
  })
  afterEach(() => {
    vi.restoreAllMocks()
  })

  it('returns ticket on success', async () => {
    vi.mocked(ticketsApi.getTicket).mockResolvedValue(fakeTicket)
    const { result } = renderHook(() => useTicket(1))
    await waitFor(() => expect(result.current.loading).toBe(false))
    expect(result.current.data?.id).toBe(1)
  })

  it('returns ApiError 40401 on not found', async () => {
    vi.mocked(ticketsApi.getTicket).mockRejectedValue(new ApiError(40401, 'Ticket #1 不存在'))
    const { result } = renderHook(() => useTicket(1))
    await waitFor(() => expect(result.current.loading).toBe(false))
    expect(result.current.error?.code).toBe(40401)
  })

  it('immediately errors with invalid id', () => {
    const { result } = renderHook(() => useTicket(undefined))
    expect(result.current.loading).toBe(false)
    expect(result.current.error?.code).toBe(40400)
  })

  it('setData updates data without refetch', async () => {
    vi.mocked(ticketsApi.getTicket).mockResolvedValue(fakeTicket)
    const { result } = renderHook(() => useTicket(1))
    await waitFor(() => expect(result.current.loading).toBe(false))
    const calls = vi.mocked(ticketsApi.getTicket).mock.calls.length
    result.current.setData({ ...fakeTicket, title: 'updated' })
    await waitFor(() => expect(result.current.data?.title).toBe('updated'))
    expect(vi.mocked(ticketsApi.getTicket).mock.calls.length).toBe(calls)
  })
})
