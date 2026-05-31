import { useMemo, useState } from 'react'
import { Activity, Search } from 'lucide-react'

interface AuditEntry {
  id: string
  actor: string
  action: string
  target: string
  status: 'success' | 'failed'
  createdAt: string
}

const sampleLogs: AuditEntry[] = [
  {
    id: '1',
    actor: 'admin',
    action: '用户登录',
    target: 'SmartQuery AI',
    status: 'success',
    createdAt: new Date().toISOString(),
  },
  {
    id: '2',
    actor: 'analyst',
    action: '执行 SQL',
    target: 'sales.orders',
    status: 'success',
    createdAt: new Date(Date.now() - 3600_000).toISOString(),
  },
]

export function AuditLog() {
  const [query, setQuery] = useState('')

  const logs = useMemo(() => {
    const keyword = query.trim().toLowerCase()
    if (!keyword) return sampleLogs
    return sampleLogs.filter((log) =>
      [log.actor, log.action, log.target, log.status].some((value) =>
        value.toLowerCase().includes(keyword)
      )
    )
  }, [query])

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
        <table className="w-full text-sm">
          <thead className="bg-gray-50 text-left text-gray-600">
            <tr>
              <th className="px-4 py-3 font-medium">时间</th>
              <th className="px-4 py-3 font-medium">操作者</th>
              <th className="px-4 py-3 font-medium">动作</th>
              <th className="px-4 py-3 font-medium">对象</th>
              <th className="px-4 py-3 font-medium">状态</th>
            </tr>
          </thead>
          <tbody className="divide-y divide-gray-100">
            {logs.map((log) => (
              <tr key={log.id} className="hover:bg-gray-50">
                <td className="px-4 py-3 text-gray-500">{new Date(log.createdAt).toLocaleString()}</td>
                <td className="px-4 py-3 text-gray-800">{log.actor}</td>
                <td className="px-4 py-3 text-gray-800">{log.action}</td>
                <td className="px-4 py-3 font-mono text-xs text-gray-600">{log.target}</td>
                <td className="px-4 py-3">
                  <span
                    className={
                      log.status === 'success'
                        ? 'rounded bg-green-100 px-2 py-1 text-xs text-green-700'
                        : 'rounded bg-red-100 px-2 py-1 text-xs text-red-700'
                    }
                  >
                    {log.status === 'success' ? '成功' : '失败'}
                  </span>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
        {logs.length === 0 && (
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
