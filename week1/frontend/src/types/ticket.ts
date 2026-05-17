/**
 * Ticket 领域类型（与后端 schemas/ticket.py 1:1 对齐）。
 */

export type TicketStatus = 'open' | 'in_progress' | 'done' | 'closed'
export type TicketPriority = 'low' | 'medium' | 'high' | 'urgent'

export interface Ticket {
  id: number
  title: string
  description: string | null
  status: TicketStatus
  priority: TicketPriority
  assignee: string | null
  tags: string[]
  /** ISO 8601 字符串 */
  created_at: string
  updated_at: string
}

export interface TicketCreateInput {
  title: string
  description?: string | null
  priority?: TicketPriority
  assignee?: string | null
  tags?: string[]
}

export interface TicketUpdateInput {
  title?: string
  description?: string | null
  priority?: TicketPriority
  assignee?: string | null
  tags?: string[]
  status?: TicketStatus
}

export type SortBy = 'created_at' | 'updated_at'
export type SortOrder = 'asc' | 'desc'

export interface TicketListQuery {
  status?: TicketStatus[]
  priority?: TicketPriority[]
  assignee?: string
  tag?: string
  keyword?: string
  sort_by?: SortBy
  sort_order?: SortOrder
  page?: number
  page_size?: number
}
