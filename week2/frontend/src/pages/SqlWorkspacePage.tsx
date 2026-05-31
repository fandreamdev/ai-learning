import { useState, useCallback } from 'react'
import Editor from '@monaco-editor/react'
import { Play, Wand2, Database, Download } from 'lucide-react'
import { useConnectionStore } from '@/stores/connectionStore'
import { api } from '@/api/client'
import type { SqlExecuteResponse } from '@/types/api'

export default function SqlWorkspacePage() {
  const { connections, currentConnectionId, setCurrentConnection } = useConnectionStore()

  const [sql, setSql] = useState('SELECT * FROM users LIMIT 10;')
  const [result, setResult] = useState<SqlExecuteResponse | null>(null)
  const [isExecuting, setIsExecuting] = useState(false)
  const [error, setError] = useState<string | null>(null)

  const currentConnection = connections.find((c) => c.id === currentConnectionId)

  const handleExecute = useCallback(async () => {
    if (!currentConnectionId) {
      setError('请先选择数据库连接')
      return
    }

    setIsExecuting(true)
    setError(null)

    try {
      const response = await fetch('/api/v1/sql/execute', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          Authorization: `Bearer ${localStorage.getItem('smartquery-auth')}`,
        },
        body: JSON.stringify({
          connection_id: currentConnectionId,
          sql: sql,
        }),
      })

      const data = await response.json()

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
          <div className="space-y-2">
            <div className="text-xs text-gray-500">
              {currentConnection ? (
                <div className="space-y-2">
                  <div className="font-medium text-gray-700">
                    {currentConnection.name}
                  </div>
                  <div className="text-gray-400">
                    {currentConnection.database_name}
                  </div>
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
                  <button className="flex items-center gap-1 hover:text-primary-600">
                    <Download size={14} />
                    导出
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
