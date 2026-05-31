import { useEffect, useMemo, useState } from 'react'
import { BookOpen, Plus, RefreshCw, Trash2 } from 'lucide-react'
import { api } from '@/api/client'
import { useConnectionStore } from '@/stores/connectionStore'

interface SemanticDefinition {
  id: string
  connection_id: string
  table_name: string
  column_name?: string | null
  business_name: string
  business_description?: string | null
  synonyms?: string[] | null
  is_active: boolean
  updated_at: string
}

export default function SemanticManagement() {
  const { connections, currentConnectionId, setCurrentConnection, fetchConnections } = useConnectionStore()
  const [items, setItems] = useState<SemanticDefinition[]>([])
  const [loading, setLoading] = useState(false)
  const [query, setQuery] = useState('')
  const [tableName, setTableName] = useState('')
  const [columnName, setColumnName] = useState('')
  const [businessName, setBusinessName] = useState('')
  const [businessDescription, setBusinessDescription] = useState('')
  const [synonyms, setSynonyms] = useState('')

  const selectedConnectionId = currentConnectionId || connections[0]?.id || ''

  useEffect(() => {
    if (connections.length === 0) {
      fetchConnections().catch(() => undefined)
    }
  }, [connections.length, fetchConnections])

  useEffect(() => {
    if (selectedConnectionId) {
      fetchSemantics(selectedConnectionId, query)
    }
  }, [selectedConnectionId, query])

  const filteredConnections = useMemo(() => connections, [connections])

  const fetchSemantics = async (connectionId: string, keyword: string) => {
    setLoading(true)
    try {
      const response = await api.get('/semantics', {
        params: {
          connection_id: connectionId,
          page: 1,
          page_size: 100,
          query: keyword.trim() || undefined,
        },
      })
      setItems(response.data?.data?.items ?? [])
    } finally {
      setLoading(false)
    }
  }

  const handleCreate = async () => {
    if (!selectedConnectionId || !tableName.trim() || !businessName.trim()) return

    await api.post('/semantics', {
      connection_id: selectedConnectionId,
      table_name: tableName.trim(),
      column_name: columnName.trim() || null,
      business_name: businessName.trim(),
      business_description: businessDescription.trim() || null,
      synonyms: synonyms
        .split(',')
        .map((item) => item.trim())
        .filter(Boolean),
    })

    setTableName('')
    setColumnName('')
    setBusinessName('')
    setBusinessDescription('')
    setSynonyms('')
    await fetchSemantics(selectedConnectionId, query)
  }

  const handleDelete = async (id: string) => {
    await api.delete(`/semantics/${id}`)
    await fetchSemantics(selectedConnectionId, query)
  }

  return (
    <div className="p-6">
      <div className="mb-6">
        <h1 className="text-2xl font-bold text-gray-800">语义层管理</h1>
        <p className="mt-1 text-sm text-gray-500">维护表、字段、业务名称和同义词映射</p>
      </div>

      <div className="mb-4 grid gap-3 rounded-lg border border-gray-200 bg-white p-4 md:grid-cols-3">
        <select
          value={selectedConnectionId}
          onChange={(event) => setCurrentConnection(event.target.value || null)}
          className="rounded-lg border border-gray-300 px-3 py-2 text-sm"
        >
          <option value="">选择连接</option>
          {filteredConnections.map((connection) => (
            <option key={connection.id} value={connection.id}>
              {connection.name}
            </option>
          ))}
        </select>
        <input
          value={query}
          onChange={(event) => setQuery(event.target.value)}
          className="rounded-lg border border-gray-300 px-3 py-2 text-sm"
          placeholder="搜索表、字段或业务名称"
        />
        <button
          onClick={() => selectedConnectionId && fetchSemantics(selectedConnectionId, query)}
          className="flex items-center justify-center gap-2 rounded-lg bg-gray-100 px-3 py-2 text-sm text-gray-700 hover:bg-gray-200"
        >
          <RefreshCw size={16} />
          刷新
        </button>
      </div>

      <div className="mb-4 grid gap-3 rounded-lg border border-gray-200 bg-white p-4 md:grid-cols-6">
        <input value={tableName} onChange={(event) => setTableName(event.target.value)} className="rounded-lg border border-gray-300 px-3 py-2 text-sm" placeholder="表名" />
        <input value={columnName} onChange={(event) => setColumnName(event.target.value)} className="rounded-lg border border-gray-300 px-3 py-2 text-sm" placeholder="字段名，可空" />
        <input value={businessName} onChange={(event) => setBusinessName(event.target.value)} className="rounded-lg border border-gray-300 px-3 py-2 text-sm" placeholder="业务名称" />
        <input value={businessDescription} onChange={(event) => setBusinessDescription(event.target.value)} className="rounded-lg border border-gray-300 px-3 py-2 text-sm" placeholder="业务描述" />
        <input value={synonyms} onChange={(event) => setSynonyms(event.target.value)} className="rounded-lg border border-gray-300 px-3 py-2 text-sm" placeholder="同义词，逗号分隔" />
        <button onClick={handleCreate} className="flex items-center justify-center gap-2 rounded-lg bg-blue-600 px-3 py-2 text-sm text-white hover:bg-blue-700">
          <Plus size={16} />
          添加
        </button>
      </div>

      <div className="overflow-hidden rounded-lg border border-gray-200 bg-white">
        {loading ? (
          <div className="flex items-center justify-center gap-2 px-4 py-10 text-gray-500">
            <RefreshCw size={18} className="animate-spin" />
            <span>正在加载语义定义</span>
          </div>
        ) : (
          <table className="w-full text-sm">
            <thead className="bg-gray-50 text-left text-gray-600">
              <tr>
                <th className="px-4 py-3 font-medium">对象</th>
                <th className="px-4 py-3 font-medium">业务名称</th>
                <th className="px-4 py-3 font-medium">同义词</th>
                <th className="px-4 py-3 font-medium">状态</th>
                <th className="px-4 py-3 text-right font-medium">操作</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-gray-100">
              {items.map((item) => (
                <tr key={item.id} className="hover:bg-gray-50">
                  <td className="px-4 py-3 font-mono text-xs text-gray-700">{item.column_name ? `${item.table_name}.${item.column_name}` : item.table_name}</td>
                  <td className="px-4 py-3 text-gray-800">
                    <div>{item.business_name}</div>
                    {item.business_description && <div className="mt-1 text-xs text-gray-500">{item.business_description}</div>}
                  </td>
                  <td className="px-4 py-3 text-gray-600">{Array.isArray(item.synonyms) ? item.synonyms.join(', ') : '-'}</td>
                  <td className="px-4 py-3">{item.is_active ? '启用' : '停用'}</td>
                  <td className="px-4 py-3 text-right">
                    <button onClick={() => handleDelete(item.id)} className="rounded p-1.5 text-red-500 hover:bg-red-50" title="删除">
                      <Trash2 size={16} />
                    </button>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        )}
        {!loading && items.length === 0 && (
          <div className="flex items-center justify-center gap-2 px-4 py-10 text-gray-500">
            <BookOpen size={18} />
            <span>暂无语义定义</span>
          </div>
        )}
      </div>
    </div>
  )
}
