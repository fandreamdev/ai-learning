/**
 * 集中测试 src/api 下的封装：
 * - tickets.* 各方法把入参拼成正确的 URL & body
 * - aggregations.* 同上
 *
 * 实现策略：mock '@/api/request' 的 axios 实例，断言其被调用的方法 + 路径 + 参数。
 */
import { afterEach, describe, expect, it, vi } from 'vitest'

// 在导入业务模块之前 mock '@/api/request'
vi.mock('@/api/request', () => {
  return {
    request: {
      get: vi.fn().mockResolvedValue({}),
      post: vi.fn().mockResolvedValue({}),
      put: vi.fn().mockResolvedValue({}),
      patch: vi.fn().mockResolvedValue({}),
      delete: vi.fn().mockResolvedValue(undefined),
    },
  }
})

import { request } from '@/api/request'
import { listAssignees, listTags } from '@/api/aggregations'
import {
  createTicket,
  deleteTicket,
  getTicket,
  listTickets,
  updateTicket,
  updateTicketStatus,
} from '@/api/tickets'

describe('api/aggregations', () => {
  afterEach(() => {
    vi.mocked(request.get).mockClear()
  })

  it('listTags calls GET /tags', async () => {
    await listTags()
    expect(request.get).toHaveBeenCalledWith('/tags', undefined)
  })

  it('listAssignees calls GET /assignees', async () => {
    await listAssignees()
    expect(request.get).toHaveBeenCalledWith('/assignees', undefined)
  })
})

describe('api/tickets', () => {
  afterEach(() => {
    vi.mocked(request.get).mockClear()
    vi.mocked(request.post).mockClear()
    vi.mocked(request.put).mockClear()
    vi.mocked(request.patch).mockClear()
    vi.mocked(request.delete).mockClear()
  })

  it('listTickets serializes multi-value status/priority as csv', async () => {
    await listTickets({
      status: ['open', 'in_progress'],
      priority: ['high'],
      assignee: 'zhang',
      tag: 'bug',
      keyword: 'login',
      sort_by: 'updated_at',
      sort_order: 'asc',
      page: 2,
      page_size: 50,
    })
    const params = vi.mocked(request.get).mock.calls[0]?.[1]?.params as Record<string, unknown>
    expect(params).toEqual({
      status: 'open,in_progress',
      priority: 'high',
      assignee: 'zhang',
      tag: 'bug',
      keyword: 'login',
      sort_by: 'updated_at',
      sort_order: 'asc',
      page: 2,
      page_size: 50,
    })
  })

  it('listTickets without args sends empty params', async () => {
    await listTickets()
    const params = vi.mocked(request.get).mock.calls[0]?.[1]?.params as Record<string, unknown>
    expect(params).toEqual({})
  })

  it('getTicket builds correct URL', async () => {
    await getTicket(42)
    expect(request.get).toHaveBeenCalledWith('/tickets/42')
  })

  it('createTicket POSTs body verbatim', async () => {
    await createTicket({ title: 'x', priority: 'high', tags: ['bug'] })
    expect(request.post).toHaveBeenCalledWith('/tickets', {
      title: 'x',
      priority: 'high',
      tags: ['bug'],
    })
  })

  it('updateTicket PUTs to /tickets/:id', async () => {
    await updateTicket(7, { title: 'new' })
    expect(request.put).toHaveBeenCalledWith('/tickets/7', { title: 'new' })
  })

  it('updateTicketStatus PATCHes status payload', async () => {
    await updateTicketStatus(3, 'closed')
    expect(request.patch).toHaveBeenCalledWith('/tickets/3/status', { status: 'closed' })
  })

  it('deleteTicket DELETEs to /tickets/:id', async () => {
    await deleteTicket(9)
    expect(request.delete).toHaveBeenCalledWith('/tickets/9')
  })
})
