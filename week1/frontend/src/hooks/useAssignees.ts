import { useEffect, useState } from 'react'

import { listAssignees } from '@/api/aggregations'
import { ApiError } from '@/types/api'

/** 加载所有负责人（spec §5.3.2）。 */
export function useAssignees(): {
  data: string[]
  loading: boolean
  error: ApiError | null
} {
  const [data, setData] = useState<string[]>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<ApiError | null>(null)

  useEffect(() => {
    const ctrl = new AbortController()
    listAssignees({ signal: ctrl.signal })
      .then((names) => {
        if (ctrl.signal.aborted) return
        setData(names)
        setLoading(false)
      })
      .catch((err: unknown) => {
        if (ctrl.signal.aborted) return
        setError(err instanceof ApiError ? err : new ApiError(0, String(err)))
        setLoading(false)
      })
    return () => ctrl.abort()
  }, [])

  return { data, loading, error }
}
