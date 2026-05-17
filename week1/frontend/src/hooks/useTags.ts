import { useEffect, useState } from 'react'

import { listTags } from '@/api/aggregations'
import { ApiError } from '@/types/api'

/** 加载所有已使用的标签（spec §5.3.1）。简单一次性加载。 */
export function useTags(): {
  data: string[]
  loading: boolean
  error: ApiError | null
} {
  const [data, setData] = useState<string[]>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<ApiError | null>(null)

  useEffect(() => {
    const ctrl = new AbortController()
    listTags({ signal: ctrl.signal })
      .then((tags) => {
        if (ctrl.signal.aborted) return
        setData(tags)
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
