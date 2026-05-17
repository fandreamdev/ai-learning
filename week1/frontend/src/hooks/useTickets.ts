import { useEffect, useState } from 'react'

import { listTickets } from '@/api/tickets'
import { ApiError, type PageData } from '@/types/api'
import type { Ticket, TicketListQuery } from '@/types/ticket'

interface State {
  data: PageData<Ticket> | null
  loading: boolean
  error: ApiError | null
}

const INITIAL: State = { data: null, loading: true, error: null }

/**
 * 加载 ticket 列表。
 *
 * - 通过 AbortController 取消旧请求，避免快速切换筛选时的竞态
 * - 错误以 ``ApiError`` 形式返回，包含业务码或网络/超时码
 */
export function useTickets(query: TicketListQuery): State & { reloadKey: number; reload: () => void } {
  const [state, setState] = useState<State>(INITIAL)
  const [reloadKey, setReloadKey] = useState(0)
  const queryKey = JSON.stringify(query)

  useEffect(() => {
    const ctrl = new AbortController()
    setState((s) => ({ ...s, loading: true, error: null }))
    listTickets(query, { signal: ctrl.signal })
      .then((data) => {
        if (ctrl.signal.aborted) return
        setState({ data, loading: false, error: null })
      })
      .catch((err: unknown) => {
        if (ctrl.signal.aborted) return
        const apiErr = err instanceof ApiError ? err : new ApiError(0, String(err))
        setState({ data: null, loading: false, error: apiErr })
      })
    return () => ctrl.abort()
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [queryKey, reloadKey])

  return {
    ...state,
    reloadKey,
    reload: () => setReloadKey((n) => n + 1),
  }
}
