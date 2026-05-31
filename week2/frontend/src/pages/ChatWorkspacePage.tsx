import { useState, useRef, useEffect } from 'react'
import { Download, Plus, Send, Database, MessageSquare } from 'lucide-react'
import { useChatStore } from '@/stores/chatStore'
import { useConnectionStore } from '@/stores/connectionStore'
import { api } from '@/api/client'
import type { Conversation, Message } from '@/types/api'

interface NlExecuteResponse {
  columns: Array<{ name: string; data_type: string; ordinal: number }>
  rows: unknown[][]
  row_count: number
  duration_ms: number
  chart_config?: unknown
  data_insight?: string
}

export default function ChatWorkspacePage() {
  const {
    conversations,
    currentConversationId,
    messages,
    setConversations,
    setCurrentConversation,
    setMessages,
    addMessage,
    updateMessage,
    isLoading,
    setLoading,
  } = useChatStore()
  const { connections, currentConnectionId } = useConnectionStore()

  const [input, setInput] = useState('')
  const [executingMessageId, setExecutingMessageId] = useState<string | null>(null)
  const messagesEndRef = useRef<HTMLDivElement>(null)

  const scrollToBottom = () => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' })
  }

  useEffect(() => {
    scrollToBottom()
  }, [messages])

  useEffect(() => {
    fetchConversations()
  }, [])

  useEffect(() => {
    if (currentConversationId) {
      fetchMessages(currentConversationId)
    }
  }, [currentConversationId])

  const fetchConversations = async () => {
    const response = await api.get('/conversations')
    const items: Conversation[] = response.data?.data?.items ?? []
    setConversations(items)
    if (!currentConversationId && items.length > 0) {
      setCurrentConversation(items[0].id)
    }
  }

  const fetchMessages = async (conversationId: string) => {
    const response = await api.get(`/conversations/${conversationId}/messages`)
    setMessages(response.data?.data?.items ?? [])
  }

  const createConversation = async () => {
    const response = await api.post('/conversations', {
      title: input.trim() ? input.trim().slice(0, 40) : '新对话',
    })
    const conversation: Conversation | undefined = response.data?.data
    if (conversation) {
      setConversations([conversation, ...conversations])
      setCurrentConversation(conversation.id)
      return conversation.id
    }
    return null
  }

  const handleSend = async () => {
    if (!input.trim() || !currentConnectionId) return
    const conversationId = currentConversationId || await createConversation()
    if (!conversationId) return

    const userMessage: Message = {
      id: Date.now().toString(),
      conversation_id: conversationId,
      role: 'user',
      content: input,
      created_at: new Date().toISOString(),
    }

    addMessage(userMessage)
    setInput('')
    setLoading(true)

    try {
      const currentConnection = connections.find((conn) => conn.id === currentConnectionId)
      const response = await api.post(`/conversations/${conversationId}/messages`, {
        content: input,
        connection_id: currentConnectionId,
        dialect: currentConnection?.db_type || 'mysql',
      })

      const data = response.data

      if (data.code === 0) {
        const assistantMessage: Message | undefined = data.data?.assistant_message
        if (assistantMessage) {
          addMessage(assistantMessage)
        }
        fetchConversations()
      } else {
        const errorMessage: Message = {
          id: (Date.now() + 1).toString(),
          conversation_id: conversationId,
          role: 'assistant',
          content: data.message || '抱歉，发生了错误，请重试。',
          created_at: new Date().toISOString(),
        }
        addMessage(errorMessage)
      }
    } catch {
      const errorMessage: Message = {
        id: (Date.now() + 1).toString(),
        conversation_id: conversationId,
        role: 'assistant',
        content: '网络错误，请检查连接后重试。',
        created_at: new Date().toISOString(),
      }
      addMessage(errorMessage)
    } finally {
      setLoading(false)
    }
  }

  const handleExportConversation = () => {
    const conversation = conversations.find((item) => item.id === currentConversationId)
    const filename = `${conversation?.title || 'conversation'}-${new Date().toISOString().replace(/[:.]/g, '-')}.json`
    const blob = new Blob([
      JSON.stringify({ conversation, messages }, null, 2),
    ], { type: 'application/json;charset=utf-8' })
    const url = URL.createObjectURL(blob)
    const link = document.createElement('a')
    link.href = url
    link.download = filename
    document.body.appendChild(link)
    link.click()
    link.remove()
    URL.revokeObjectURL(url)
  }

  const handleExecuteGeneratedSql = async (message: Message) => {
    if (!currentConnectionId || !message.generated_sql) return

    setExecutingMessageId(message.id)
    try {
      const response = await api.post('/nl/execute', {
        connection_id: currentConnectionId,
        sql: message.generated_sql,
      })
      const data = response.data

      if (data.code === 0) {
        const result: NlExecuteResponse = data.data
        updateMessage(message.id, {
          execution_result: result,
          chart_config: result.chart_config,
          content: result.data_insight || `查询完成，返回 ${result.row_count} 行，耗时 ${result.duration_ms} ms。`,
        })
      } else {
        updateMessage(message.id, {
          content: data.message || '执行查询失败',
        })
      }
    } catch {
      updateMessage(message.id, {
        content: '执行查询失败，请检查连接和 SQL 后重试。',
      })
    } finally {
      setExecutingMessageId(null)
    }
  }

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault()
      handleSend()
    }
  }

  return (
    <div className="h-full flex flex-col">
      {/* 顶部 */}
      <div className="bg-white border-b px-4 py-3 flex items-center justify-between">
        <div className="flex items-center gap-3">
          <MessageSquare size={20} className="text-primary-600" />
          <span className="font-medium text-gray-900">对话模式</span>
        </div>
        <div className="flex items-center gap-2">
          {!currentConnectionId && (
            <span className="text-sm text-amber-600">
              请先在 SQL 模式选择数据库连接
            </span>
          )}
          <button onClick={createConversation} className="btn-secondary flex items-center gap-1 py-1.5 text-sm">
            <Plus size={16} />
            新对话
          </button>
          <button onClick={handleExportConversation} disabled={messages.length === 0} className="btn-secondary flex items-center gap-1 py-1.5 text-sm">
            <Download size={16} />
            导出
          </button>
        </div>
      </div>

      {/* 消息列表 */}
      <div className="flex border-b bg-white px-4 py-2 gap-2 overflow-x-auto">
        {conversations.map((conversation) => (
          <button
            key={conversation.id}
            onClick={() => setCurrentConversation(conversation.id)}
            className={`shrink-0 rounded px-3 py-1.5 text-sm ${
              conversation.id === currentConversationId
                ? 'bg-primary-600 text-white'
                : 'bg-gray-100 text-gray-700 hover:bg-gray-200'
            }`}
          >
            {conversation.title || '未命名对话'}
          </button>
        ))}
      </div>

      <div className="flex-1 overflow-auto p-4 space-y-4">
        {messages.length === 0 ? (
          <div className="h-full flex items-center justify-center">
            <div className="text-center text-gray-400">
              <MessageSquare size={48} className="mx-auto mb-4 opacity-50" />
              <p>输入自然语言问题，AI 将为您生成 SQL 查询</p>
              <p className="text-sm mt-2">
                例如：帮我查一下上个月销售额最高的产品
              </p>
            </div>
          </div>
        ) : (
          messages.map((message) => (
            <div
              key={message.id}
              className={`flex ${
                message.role === 'user' ? 'justify-end' : 'justify-start'
              }`}
            >
              <div
                className={`max-w-2xl rounded-lg p-4 ${
                  message.role === 'user'
                    ? 'bg-primary-600 text-white'
                    : 'bg-white border shadow-sm'
                }`}
              >
                {message.role === 'user' ? (
                  <p>{message.content}</p>
                ) : (
                  <div className="space-y-4">
                    <p className="text-gray-700">{message.content}</p>

                    {message.generated_sql && (
                      <div className="bg-gray-100 rounded-lg p-3">
                        <div className="flex items-center gap-2 text-xs text-gray-500 mb-2">
                          <Database size={14} />
                          <span>生成的 SQL</span>
                        </div>
                        <pre className="text-sm font-mono text-gray-800 overflow-x-auto">
                          {message.generated_sql}
                        </pre>
                        <button
                          className="mt-3 btn-primary text-sm py-1.5"
                          disabled={executingMessageId === message.id}
                          onClick={() => handleExecuteGeneratedSql(message)}
                        >
                          {executingMessageId === message.id ? '执行中...' : '执行查询'}
                        </button>
                        {Boolean(message.execution_result) && (
                          <div className="mt-3 text-xs text-gray-600">
                            查询完成：
                            {(message.execution_result as NlExecuteResponse).row_count} 行，
                            耗时 {(message.execution_result as NlExecuteResponse).duration_ms} ms
                          </div>
                        )}
                      </div>
                    )}
                  </div>
                )}
              </div>
            </div>
          ))
        )}

        {isLoading && (
          <div className="flex justify-start">
            <div className="bg-white border shadow-sm rounded-lg p-4">
              <div className="flex items-center gap-2 text-gray-500">
                <div className="w-4 h-4 border-2 border-primary-600 border-t-transparent rounded-full animate-spin" />
                <span>AI 正在思考...</span>
              </div>
            </div>
          </div>
        )}

        <div ref={messagesEndRef} />
      </div>

      {/* 输入区 */}
      <div className="bg-white border-t p-4">
        <div className="flex items-end gap-3">
          <div className="flex-1">
            <textarea
              value={input}
              onChange={(e) => setInput(e.target.value)}
              onKeyDown={handleKeyDown}
              placeholder={
                currentConnectionId
                  ? '输入您的数据查询问题...'
                  : '请先选择数据库连接'
              }
              disabled={!currentConnectionId || isLoading}
              className="input resize-none h-24"
            />
          </div>
          <button
            onClick={handleSend}
            disabled={!input.trim() || !currentConnectionId || isLoading}
            className="btn-primary p-3"
          >
            <Send size={20} />
          </button>
        </div>
      </div>
    </div>
  )
}
