import { useEffect, useState, useCallback } from 'react'
import Editor from '@monaco-editor/react'
import { Play, Wand2, Database, Download } from 'lucide-react'
import { useConnectionStore } from '@/stores/connectionStore'
import { api } from '@/api/client'
import type { SqlExecuteResponse } from '@/types/api'

interface SchemaTable {
  table_name: string
  table_schema: string
  table_type: string
  columns?: Array<{
    name: string
    data_type: string
    nullable: boolean
    comment?: string | null
  }>
}

export default function SqlWorkspacePage() {
  const { connections, currentConnectionId, setCurrentConnection, fetchConnections } = useConnectionStore()

  const [sql, setSql] = useState('SELECT * FROM users LIMIT 10;')
  const [result, setResult] = useState<SqlExecuteResponse | null>(null)
  const [schemaTables, setSchemaTables] = useState<SchemaTable[]>([])
  const [isSchemaLoading, setIsSchemaLoading] = useState(false)
  const [isExecuting, setIsExecuting] = useState(false)
  const [error, setError] = useState<string | null>(null)

  const currentConnection = connections.find((c) => c.id === currentConnectionId)

  useEffect(() => {
    if (connections.length === 0) {
      fetchConnections().catch(() => undefined)
    }
  }, [connections.length, fetchConnections])

  useEffect(() => {
    if (!currentConnectionId) {
      setSchemaTables([])
      return
    }

    setIsSchemaLoading(true)
    api
      .get(`/connections/${currentConnectionId}/schema`)
      .then((response) => {
        setSchemaTables(response.data?.data?.tables ?? [])
      })
      .catch(() => {
        setSchemaTables([])
      })
      .finally(() => setIsSchemaLoading(false))
  }, [currentConnectionId])

  const handleExecute = useCallback(async () => {
    if (!currentConnectionId) {
      setError('请先选择数据库连接')
      return
    }

    setIsExecuting(true)
    setError(null)

    try {
      const response = await api.post('/sql/execute', {
        connection_id: currentConnectionId,
        sql: sql,
      })

      const data = response.data

      if (data.code === 0) {
        setResult(data.data)
      } else {
        setError(data.message || '执行失败')
      }
    } catch (err) {
      setError('网络错误，请重试')
    } finally {
      setIsExecuting(false)
    }
  }, [currentConnectionId, sql])

  const handleFormat = useCallback(async () => {
    try {
      const response = await api.post('/sql/format', {
        sql,
        dialect: currentConnection?.db_type || 'postgresql',
      })
      const formatted = response.data?.data?.formatted_sql
      if (formatted) setSql(formatted)
    } catch {
      setError('SQL 格式化失败')
    }
  }, [currentConnection?.db_type, sql])

  const handleExport = useCallback((format: 'csv' | 'json') => {
    if (!result) return

    const filename = `query-result-${new Date().toISOString().replace(/[:.]/g, '-')}.${format}`
    const content =
      format === 'json'
        ? JSON.stringify({ columns: result.columns, rows: result.rows }, null, 2)
        : toCsv(result)
    const mime = format === 'json' ? 'application/json;charset=utf-8' : 'text/csv;charset=utf-8'
    const blob = new Blob([content], { type: mime })
    const url = URL.createObjectURL(blob)
    const link = document.createElement('a')
    link.href = url
    link.download = filename
    document.body.appendChild(link)
    link.click()
    link.remove()
    URL.revokeObjectURL(url)
  }, [result])

  return (
    <div className="h-full flex flex-col">
      {/* 顶部工具栏 */}
      <div className="bg-white border-b px-4 py-2 flex items-center justify-between">
        <div className="flex items-center gap-4">
          <div className="flex items-center gap-2">
            <Database size={18} className="text-gray-500" />
            <select
              value={currentConnectionId || ''}
              onChange={(e) => setCurrentConnection(e.target.value || null)}
              className="input py-1 px-2 w-48"
            >
              <option value="">选择连接...</option>
              {connections.map((conn) => (
                <option key={conn.id} value={conn.id}>
                  {conn.name}
                </option>
              ))}
            </select>
          </div>
        </div>

        <div className="flex items-center gap-2">
          <button
            onClick={handleFormat}
            className="btn-secondary flex items-center gap-1 py-1.5"
            title="格式化"
          >
            <Wand2 size={16} />
            <span className="text-sm">格式化</span>
          </button>
          <button
            onClick={handleExecute}
            disabled={isExecuting || !currentConnectionId}
            className="btn-primary flex items-center gap-1 py-1.5"
          >
            <Play size={16} />
            <span className="text-sm">{isExecuting ? '执行中...' : '执行'}</span>
          </button>
        </div>
      </div>

      {/* 编辑器和结果区 */}
      <div className="flex-1 flex">
        {/* 左侧面板 */}
        <div className="w-64 bg-white border-r p-4 overflow-auto">
          <h3 className="text-sm font-medium text-gray-700 mb-3">数据库结构</h3>
          <div className="space-y-3">
            <div className="text-xs text-gray-500">
              {currentConnection ? (
                <div className="space-y-2">
                  <div className="font-medium text-gray-700">
                    {currentConnection.name}
                  </div>
                  <div className="text-gray-400">
                    {currentConnection.database_name}
                  </div>
                  {isSchemaLoading ? (
                    <div className="text-gray-400">正在加载结构...</div>
                  ) : (
                    <div className="space-y-3">
                      {schemaTables.map((table) => (
                        <div key={`${table.table_schema}.${table.table_name}`} className="rounded border border-gray-100 p-2">
                          <button
                            className="font-mono text-xs text-gray-700 hover:text-primary-600"
                            onClick={() => setSql((value) => `${value}\nSELECT * FROM ${table.table_schema}.${table.table_name} LIMIT 100;`)}
                          >
                            {table.table_schema}.{table.table_name}
                          </button>
                          <div className="mt-1 space-y-1">
                            {(table.columns || []).slice(0, 8).map((column) => (
                              <div key={column.name} className="flex justify-between gap-2 text-[11px]">
                                <span className="truncate text-gray-600">{column.name}</span>
                                <span className="shrink-0 text-gray-400">{column.data_type}</span>
                              </div>
                            ))}
                          </div>
                        </div>
                      ))}
                      {schemaTables.length === 0 && (
                        <div className="text-gray-400">暂无表结构</div>
                      )}
                    </div>
                  )}
                </div>
              ) : (
                <p className="text-gray-400">请选择数据库连接</p>
              )}
            </div>
          </div>
        </div>

        {/* SQL 编辑器 */}
        <div className="flex-1 flex flex-col">
          <div className="h-1/2 border-b">
            <Editor
              height="100%"
              defaultLanguage="sql"
              value={sql}
              onChange={(value) => setSql(value || '')}
              theme="vs-light"
              options={{
                minimap: { enabled: false },
                fontSize: 14,
                lineNumbers: 'on',
                scrollBeyondLastLine: false,
                automaticLayout: true,
              }}
            />
          </div>

          {/* 结果区 */}
          <div className="h-1/2 flex flex-col bg-white">
            <div className="px-4 py-2 border-b flex items-center justify-between">
              <span className="text-sm font-medium text-gray-700">
                执行结果
              </span>
              {result && (
                <div className="flex items-center gap-4 text-xs text-gray-500">
                  <span>行数: {result.row_count}</span>
                  <span>耗时: {result.duration_ms}ms</span>
                  <button onClick={() => handleExport('csv')} className="flex items-center gap-1 hover:text-primary-600">
                    <Download size={14} />
                    CSV
                  </button>
                  <button onClick={() => handleExport('json')} className="flex items-center gap-1 hover:text-primary-600">
                    <Download size={14} />
                    JSON
                  </button>
                </div>
              )}
            </div>

            <div className="flex-1 overflow-auto p-4">
              {error ? (
                <div className="text-red-500 text-sm">{error}</div>
              ) : result ? (
                <table className="w-full text-sm">
                  <thead>
                    <tr className="border-b">
                      {result.columns.map((col) => (
                        <th
                          key={col.name}
                          className="text-left px-2 py-2 font-medium text-gray-600"
                        >
                          {col.name}
                        </th>
                      ))}
                    </tr>
                  </thead>
                  <tbody>
                    {result.rows.map((row, i) => (
                      <tr key={i} className="border-b hover:bg-gray-50">
                        {row.map((cell, j) => (
                          <td key={j} className="px-2 py-1.5 text-gray-800">
                            {String(cell ?? '')}
                          </td>
                        ))}
                      </tr>
                    ))}
                  </tbody>
                </table>
              ) : (
                <div className="text-gray-400 text-sm text-center py-8">
                  点击执行按钮运行查询
                </div>
              )}
            </div>
          </div>
        </div>
      </div>
    </div>
  )
}

function toCsv(result: SqlExecuteResponse) {
  const escapeCell = (value: unknown) => {
    const text = String(value ?? '')
    if (/[",\r\n]/.test(text)) {
      return `"${text.replace(/"/g, '""')}"`
    }
    return text
  }

  const header = result.columns.map((column) => escapeCell(column.name)).join(',')
  const rows = result.rows.map((row) => row.map(escapeCell).join(','))
  return [header, ...rows].join('\r\n')
}
