/**
 * Ticket 业务接口（spec §5.2）。
 *
 * 注意：``request`` 拦截器已拆封 ``{code, message, data}``，
 * 这里的返回类型直接是业务数据，使用 ``request.get<unknown, T>(...)`` 让 TS 推断正确。
 */
import { request } from '@/api/request'
import type { PageData } from '@/types/api'
import type {
  Ticket,
  TicketCreateInput,
  TicketListQuery,
  TicketStatus,
  TicketUpdateInput,
} from '@/types/ticket'

/** 把列表查询参数转换成 axios 接受的扁平对象，列表型字段用逗号分隔。 */
function serializeQuery(q: TicketListQuery): Record<string, string | number> {
  const params: Record<string, string | number> = {}
  if (q.status?.length) params.status = q.status.join(',')
  if (q.priority?.length) params.priority = q.priority.join(',')
  if (q.assignee) params.assignee = q.assignee
  if (q.tag) params.tag = q.tag
  if (q.keyword) params.keyword = q.keyword
  if (q.sort_by) params.sort_by = q.sort_by
  if (q.sort_order) params.sort_order = q.sort_order
  if (q.page) params.page = q.page
  if (q.page_size) params.page_size = q.page_size
  return params
}

export function listTickets(query: TicketListQuery = {}): Promise<PageData<Ticket>> {
  return request.get<unknown, PageData<Ticket>>('/tickets', {
    params: serializeQuery(query),
  })
}

export function getTicket(id: number): Promise<Ticket> {
  return request.get<unknown, Ticket>(`/tickets/${id}`)
}

export function createTicket(input: TicketCreateInput): Promise<Ticket> {
  return request.post<unknown, Ticket>('/tickets', input)
}

export function updateTicket(id: number, input: TicketUpdateInput): Promise<Ticket> {
  return request.put<unknown, Ticket>(`/tickets/${id}`, input)
}

export function updateTicketStatus(id: number, status: TicketStatus): Promise<Ticket> {
  return request.patch<unknown, Ticket>(`/tickets/${id}/status`, { status })
}

export function deleteTicket(id: number): Promise<void> {
  return request.delete<unknown, void>(`/tickets/${id}`)
}
