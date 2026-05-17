import { useCallback, useMemo } from 'react'
import { useSearchParams } from 'react-router-dom'

import { buildSearchFromQuery, DEFAULTS, parseQueryFromSearch } from '@/lib/queryString'
import type {
  SortBy,
  SortOrder,
  TicketListQuery,
  TicketPriority,
  TicketStatus,
} from '@/types/ticket'

/**
 * 列表查询条件 ↔ URL 双向绑定。
 *
 * 任何对查询条件的修改都通过 ``setQuery`` 写回 URL；
 * 浏览器后退/前进/分享 URL 都能完整恢复状态。
 */
export function useTicketListUrlState() {
  const [params, setParams] = useSearchParams()

  const query = useMemo<TicketListQuery>(() => parseQueryFromSearch(params), [params])

  const setQuery = useCallback(
    (next: TicketListQuery) => {
      setParams(buildSearchFromQuery(next), { replace: false })
    },
    [setParams],
  )

  /** 改变筛选/搜索类条件时重置 page=1。 */
  const updateFilter = useCallback(
    (patch: Partial<TicketListQuery>) => {
      setQuery({ ...query, ...patch, page: 1 })
    },
    [query, setQuery],
  )

  const toggleStatus = useCallback(
    (s: TicketStatus) => {
      const current = query.status ?? []
      const next = current.includes(s) ? current.filter((x) => x !== s) : [...current, s]
      updateFilter({ status: next.length ? next : undefined })
    },
    [query.status, updateFilter],
  )

  const togglePriority = useCallback(
    (p: TicketPriority) => {
      const current = query.priority ?? []
      const next = current.includes(p) ? current.filter((x) => x !== p) : [...current, p]
      updateFilter({ priority: next.length ? next : undefined })
    },
    [query.priority, updateFilter],
  )

  const setAssignee = useCallback(
    (name: string | undefined) => updateFilter({ assignee: name || undefined }),
    [updateFilter],
  )
  const setTag = useCallback(
    (tag: string | undefined) => updateFilter({ tag: tag || undefined }),
    [updateFilter],
  )
  const setKeyword = useCallback(
    (kw: string | undefined) => updateFilter({ keyword: kw || undefined }),
    [updateFilter],
  )

  const setSort = useCallback(
    (sort_by: SortBy, sort_order: SortOrder) => {
      setQuery({ ...query, sort_by, sort_order, page: 1 })
    },
    [query, setQuery],
  )

  const setPage = useCallback(
    (page: number) => setQuery({ ...query, page }),
    [query, setQuery],
  )

  const setPageSize = useCallback(
    (page_size: number) => setQuery({ ...query, page_size, page: 1 }),
    [query, setQuery],
  )

  const clearAll = useCallback(() => {
    setQuery({
      sort_by: DEFAULTS.sort_by,
      sort_order: DEFAULTS.sort_order,
      page: DEFAULTS.page,
      page_size: DEFAULTS.page_size,
    })
  }, [setQuery])

  return {
    query,
    setQuery,
    toggleStatus,
    togglePriority,
    setAssignee,
    setTag,
    setKeyword,
    setSort,
    setPage,
    setPageSize,
    clearAll,
  }
}
