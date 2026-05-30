import { useState, useRef, useEffect } from 'react'
import { Send, Database, BarChart3, MessageSquare } from 'lucide-react'
import { useChatStore } from '@/stores/chatStore'
import { useConnectionStore } from '@/stores/connectionStore'
import type { Message, NlConvertResponse } from '@/types/api'

export default function ChatWorkspacePage() {
  const { messages, addMessage, isLoading, setLoading } = useChatStore()
  const { currentConnectionId } = useConnectionStore()

  const [input, setInput] = useState('')
  const messagesEndRef = useRef<HTMLDivElement>(null)

  const scrollToBottom = () => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' })
  }

  useEffect(() => {
    scrollToBottom()
  }, [messages])

  const handleSend = async () => {
    if (!input.trim() || !currentConnectionId) return

    const userMessage: Message = {
      id: Date.now().toString(),
      conversation_id: '',
      role: 'user',
      content: input,
      created_at: new Date().toISOString(),
    }

    addMessage(userMessage)
    setInput('')
    setLoading(true)

    try {
      const response = await fetch('/api/v1/nl/convert', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          Authorization: `Bearer ${localStorage.getItem('smartquery-auth')}`,
        },
        body: JSON.stringify({
          connection_id: currentConnectionId,
          question: input,
        }),
      })

      const data = await response.json()

      if (data.code === 0) {
        const result: NlConvertResponse = data.data

        const assistantMessage: Message = {
          id: (Date.now() + 1).toString(),
          conversation_id: '',
          role: 'assistant',
          content: result.explanation,
          generated_sql: result.sql,
          created_at: new Date().toISOString(),
        }

        addMessage(assistantMessage)
      } else {
        const errorMessage: Message = {
          id: (Date.now() + 1).toString(),
          conversation_id: '',
          role: 'assistant',
          content: data.message || '抱歉，发生了错误，请重试。',
          created_at: new Date().toISOString(),
        }
        addMessage(errorMessage)
      }
    } catch {
      const errorMessage: Message = {
        id: (Date.now() + 1).toString(),
        conversation_id: '',
        role: 'assistant',
        content: '网络错误，请检查连接后重试。',
        created_at: new Date().toISOString(),
      }
      addMessage(errorMessage)
    } finally {
      setLoading(false)
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
        {!currentConnectionId && (
          <span className="text-sm text-amber-600">
            请先在 SQL 模式选择数据库连接
          </span>
        )}
      </div>

      {/* 消息列表 */}
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
                        <button className="mt-3 btn-primary text-sm py-1.5">
                          执行查询
                        </button>
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
