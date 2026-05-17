import { useEffect, useState } from 'react'

import { getTicket } from '@/api/tickets'
import { ApiError } from '@/types/api'
import type { Ticket } from '@/types/ticket'

interface State {
  data: Ticket | null
  loading: boolean
  error: ApiError | null
}

const INITIAL: State = { data: null, loading: true, error: null }

/**
 * 加载单个 Ticket 详情。
 *
 * - 通过 AbortController 取消旧请求
 * - 错误以 ``ApiError`` 形式返回
 * - 提供 ``setData`` 让父级在编辑/状态切换后无需重新请求即可更新本地视图
 */
export function useTicket(id: number | undefined): State & {
  setData: (next: Ticket | null) => void
  reload: () => void
} {
  const [state, setState] = useState<State>(INITIAL)
  const [reloadKey, setReloadKey] = useState(0)

  useEffect(() => {
    if (id === undefined || Number.isNaN(id)) {
      setState({ data: null, loading: false, error: new ApiError(40400, '非法 Ticket ID') })
      return
    }
    const ctrl = new AbortController()
    setState((s) => ({ ...s, loading: true, error: null }))
    getTicket(id)
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
  }, [id, reloadKey])

  return {
    ...state,
    setData: (next) => setState((s) => ({ ...s, data: next })),
    reload: () => setReloadKey((n) => n + 1),
  }
}
