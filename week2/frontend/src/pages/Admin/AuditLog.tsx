import { useEffect, useState } from 'react'
import { Activity, RefreshCw, Search } from 'lucide-react'
import { api } from '@/api/client'

interface AuditEntry {
  id: string
  user_id?: string | null
  username?: string | null
  action: string
  resource_type?: string | null
  resource_id?: string | null
  details?: unknown
  ip_address?: string | null
  created_at: string
}

export function AuditLog() {
  const [query, setQuery] = useState('')
  const [logs, setLogs] = useState<AuditEntry[]>([])
  const [loading, setLoading] = useState(false)

  useEffect(() => {
    const timer = window.setTimeout(() => {
      fetchLogs(query)
    }, 250)

    return () => window.clearTimeout(timer)
  }, [query])

  const fetchLogs = async (keyword: string) => {
    setLoading(true)
    try {
      const response = await api.get('/audit-logs', {
        params: {
          page: 1,
          page_size: 100,
          query: keyword.trim() || undefined,
        },
      })
      setLogs(response.data?.data?.items ?? [])
    } finally {
      setLoading(false)
    }
  }

  const getActor = (log: AuditEntry) => log.username || log.user_id || 'system'

  const getTarget = (log: AuditEntry) =>
    [log.resource_type, log.resource_id].filter(Boolean).join(':') || '-'

  return (
    <div className="p-6">
      <div className="mb-6">
        <h1 className="text-2xl font-bold text-gray-800">审计日志</h1>
        <p className="mt-1 text-sm text-gray-500">查看用户操作、查询执行和系统事件</p>
      </div>

      <div className="mb-4 rounded-lg border border-gray-200 bg-white p-4">
        <div className="relative">
          <Search size={18} className="absolute left-3 top-1/2 -translate-y-1/2 text-gray-400" />
          <input
            value={query}
            onChange={(event) => setQuery(event.target.value)}
            className="w-full rounded-lg border border-gray-300 py-2 pl-10 pr-3 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
            placeholder="搜索操作者、动作或对象"
          />
        </div>
      </div>

      <div className="overflow-hidden rounded-lg border border-gray-200 bg-white">
        {loading ? (
          <div className="flex items-center justify-center gap-2 px-4 py-10 text-gray-500">
            <RefreshCw size={18} className="animate-spin" />
            <span>正在加载审计日志</span>
          </div>
        ) : (
          <table className="w-full text-sm">
          <thead className="bg-gray-50 text-left text-gray-600">
            <tr>
              <th className="px-4 py-3 font-medium">时间</th>
              <th className="px-4 py-3 font-medium">操作者</th>
              <th className="px-4 py-3 font-medium">动作</th>
              <th className="px-4 py-3 font-medium">对象</th>
              <th className="px-4 py-3 font-medium">IP</th>
            </tr>
          </thead>
          <tbody className="divide-y divide-gray-100">
            {logs.map((log) => (
              <tr key={log.id} className="hover:bg-gray-50">
                <td className="px-4 py-3 text-gray-500">{new Date(log.created_at).toLocaleString()}</td>
                <td className="px-4 py-3 text-gray-800">{getActor(log)}</td>
                <td className="px-4 py-3 text-gray-800">{log.action}</td>
                <td className="px-4 py-3 font-mono text-xs text-gray-600">{getTarget(log)}</td>
                <td className="px-4 py-3 text-gray-500">{log.ip_address || '-'}</td>
              </tr>
            ))}
          </tbody>
        </table>
        )}
        {!loading && logs.length === 0 && (
          <div className="flex items-center justify-center gap-2 px-4 py-10 text-gray-500">
            <Activity size={18} />
            <span>没有匹配的审计日志</span>
          </div>
        )}
      </div>
    </div>
  )
}

export default AuditLog
