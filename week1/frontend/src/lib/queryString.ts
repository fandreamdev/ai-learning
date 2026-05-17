/**
 * URL ↔ TicketListQuery 序列化/反序列化。
 *
 * 设计：
 * - 多值字段（status / priority）使用逗号分隔
 * - 默认值不写入 URL（page=1 / page_size=20 / sort_by=created_at / sort_order=desc）
 * - 反序列化对未知 / 非法值容错（直接丢弃）
 */
import type {
  SortBy,
  SortOrder,
  TicketListQuery,
  TicketPriority,
  TicketStatus,
} from '@/types/ticket'

const VALID_STATUSES: readonly TicketStatus[] = ['open', 'in_progress', 'done', 'closed']
const VALID_PRIORITIES: readonly TicketPriority[] = ['low', 'medium', 'high', 'urgent']
const VALID_SORT_BY: readonly SortBy[] = ['created_at', 'updated_at']
const VALID_SORT_ORDER: readonly SortOrder[] = ['asc', 'desc']

const DEFAULT_PAGE = 1
const DEFAULT_PAGE_SIZE = 20
const DEFAULT_SORT_BY: SortBy = 'created_at'
const DEFAULT_SORT_ORDER: SortOrder = 'desc'

function parseMulti<T extends string>(raw: string | null, valid: readonly T[]): T[] {
  if (!raw) return []
  const set = new Set<T>()
  const result: T[] = []
  for (const seg of raw.split(',')) {
    const trimmed = seg.trim()
    if (!trimmed) continue
    if ((valid as readonly string[]).includes(trimmed) && !set.has(trimmed as T)) {
      set.add(trimmed as T)
      result.push(trimmed as T)
    }
  }
  return result
}

function parsePositiveInt(raw: string | null, fallback: number, max?: number): number {
  if (!raw) return fallback
  const n = Number.parseInt(raw, 10)
  if (!Number.isFinite(n) || n < 1) return fallback
  if (max !== undefined && n > max) return max
  return n
}

/** URLSearchParams → TicketListQuery */
export function parseQueryFromSearch(search: URLSearchParams): TicketListQuery {
  const status = parseMulti(search.get('status'), VALID_STATUSES)
  const priority = parseMulti(search.get('priority'), VALID_PRIORITIES)
  const assignee = search.get('assignee')?.trim() || undefined
  const tag = search.get('tag')?.trim() || undefined
  const keyword = search.get('keyword')?.trim() || undefined

  const rawSortBy = search.get('sort_by')
  const sort_by: SortBy = (VALID_SORT_BY as readonly string[]).includes(rawSortBy ?? '')
    ? (rawSortBy as SortBy)
    : DEFAULT_SORT_BY

  const rawSortOrder = search.get('sort_order')
  const sort_order: SortOrder = (VALID_SORT_ORDER as readonly string[]).includes(rawSortOrder ?? '')
    ? (rawSortOrder as SortOrder)
    : DEFAULT_SORT_ORDER

  const page = parsePositiveInt(search.get('page'), DEFAULT_PAGE)
  const page_size = parsePositiveInt(search.get('page_size'), DEFAULT_PAGE_SIZE, 100)

  return {
    status: status.length ? status : undefined,
    priority: priority.length ? priority : undefined,
    assignee,
    tag,
    keyword,
    sort_by,
    sort_order,
    page,
    page_size,
  }
}

/** TicketListQuery → URLSearchParams（默认值省略） */
export function buildSearchFromQuery(q: TicketListQuery): URLSearchParams {
  const params = new URLSearchParams()
  if (q.status?.length) params.set('status', q.status.join(','))
  if (q.priority?.length) params.set('priority', q.priority.join(','))
  if (q.assignee) params.set('assignee', q.assignee)
  if (q.tag) params.set('tag', q.tag)
  if (q.keyword) params.set('keyword', q.keyword)
  if (q.sort_by && q.sort_by !== DEFAULT_SORT_BY) params.set('sort_by', q.sort_by)
  if (q.sort_order && q.sort_order !== DEFAULT_SORT_ORDER) params.set('sort_order', q.sort_order)
  if (q.page && q.page !== DEFAULT_PAGE) params.set('page', String(q.page))
  if (q.page_size && q.page_size !== DEFAULT_PAGE_SIZE) {
    params.set('page_size', String(q.page_size))
  }
  return params
}

export const DEFAULTS = {
  page: DEFAULT_PAGE,
  page_size: DEFAULT_PAGE_SIZE,
  sort_by: DEFAULT_SORT_BY,
  sort_order: DEFAULT_SORT_ORDER,
}
