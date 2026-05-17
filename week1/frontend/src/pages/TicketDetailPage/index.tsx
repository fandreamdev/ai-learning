import { useEffect, useState } from 'react'
import { useNavigate, useParams } from 'react-router-dom'

import ConfirmDialog from '@/components/ConfirmDialog/ConfirmDialog'
import StatusSelect from '@/components/TicketDetail/StatusSelect'
import TicketForm from '@/components/TicketForm/TicketForm'
import PriorityBadge from '@/components/TicketTable/PriorityBadge'
import { useToast } from '@/components/Toast/useToast'
import { useTicket } from '@/hooks/useTicket'
import { deleteTicket } from '@/api/tickets'
import { formatDateTime } from '@/lib/format'
import { ApiError } from '@/types/api'

/**
 * Ticket 详情页（spec §6.4）。
 *
 * - 加载中显示骨架；找不到时 toast + 跳回列表
 * - 状态切换通过 PATCH /status；编辑通过 PUT；删除前二次确认
 * - 返回列表使用 navigate(-1) 保留筛选状态
 */
export default function TicketDetailPage() {
  const params = useParams<{ id: string }>()
  const navigate = useNavigate()
  const toast = useToast()

  const id = params.id ? Number.parseInt(params.id, 10) : undefined
  const { data: ticket, loading, error, setData } = useTicket(id)

  const [editOpen, setEditOpen] = useState(false)
  const [confirmDeleteOpen, setConfirmDeleteOpen] = useState(false)
  const [deleting, setDeleting] = useState(false)

  // 加载失败：toast 后跳回首页
  useEffect(() => {
    if (!error) return
    const message = error.code === 40401 ? 'Ticket 不存在' : error.message || '加载失败'
    toast.error(message)
    navigate('/', { replace: true })
  }, [error, toast, navigate])

  const handleDelete = async () => {
    if (!ticket) return
    setDeleting(true)
    try {
      await deleteTicket(ticket.id)
      toast.success(`已删除 Ticket #${ticket.id}`)
      navigate('/', { replace: true })
    } catch (err) {
      const message = err instanceof ApiError ? err.message : '删除失败'
      toast.error(message)
      setDeleting(false)
      setConfirmDeleteOpen(false)
    }
  }

  if (loading) {
    return (
      <main className="mx-auto max-w-3xl p-8">
        <div className="space-y-4">
          <div className="h-4 w-24 animate-pulse rounded bg-gray-200" />
          <div className="h-8 w-48 animate-pulse rounded bg-gray-200" />
          <div className="space-y-3 rounded-lg border border-gray-200 bg-white p-6">
            {Array.from({ length: 6 }).map((_, i) => (
              <div key={i} className="h-3 w-full animate-pulse rounded bg-gray-100" />
            ))}
          </div>
        </div>
      </main>
    )
  }

  if (!ticket) return null

  return (
    <main className="mx-auto max-w-3xl p-8">
      <button
        type="button"
        onClick={() => navigate(-1)}
        className="mb-4 text-sm text-blue-600 hover:underline"
      >
        ← 返回列表
      </button>

      <div className="rounded-lg border border-gray-200 bg-white p-6 shadow-sm">
        <header className="mb-4 border-b border-gray-100 pb-3">
          <p className="text-xs text-gray-500">Ticket #{ticket.id}</p>
          <h1 className="mt-1 text-2xl font-semibold text-gray-900">{ticket.title}</h1>
        </header>

        <dl className="grid grid-cols-[5rem_1fr] gap-y-3 text-sm text-gray-700">
          <dt className="text-gray-500">状态</dt>
          <dd>
            <StatusSelect ticket={ticket} onChanged={setData} />
          </dd>

          <dt className="text-gray-500">优先级</dt>
          <dd>
            <PriorityBadge priority={ticket.priority} />
          </dd>

          <dt className="text-gray-500">负责人</dt>
          <dd>{ticket.assignee || '-'}</dd>

          <dt className="text-gray-500">标签</dt>
          <dd className="flex flex-wrap gap-1.5">
            {ticket.tags?.length ? (
              ticket.tags.map((t) => (
                <span
                  key={t}
                  className="inline-flex items-center rounded-md bg-gray-100 px-1.5 py-0.5 text-xs text-gray-700"
                >
                  {t}
                </span>
              ))
            ) : (
              <span className="text-gray-400">-</span>
            )}
          </dd>

          <dt className="text-gray-500">创建时间</dt>
          <dd className="text-xs text-gray-500">{formatDateTime(ticket.created_at)}</dd>

          <dt className="text-gray-500">更新时间</dt>
          <dd className="text-xs text-gray-500">{formatDateTime(ticket.updated_at)}</dd>
        </dl>

        <section className="mt-5 border-t border-gray-100 pt-4">
          <h2 className="mb-2 text-sm font-medium text-gray-700">描述</h2>
          {ticket.description ? (
            <p className="whitespace-pre-wrap text-sm leading-relaxed text-gray-700">
              {ticket.description}
            </p>
          ) : (
            <p className="text-sm text-gray-400">（无）</p>
          )}
        </section>

        <footer className="mt-6 flex justify-end gap-2 border-t border-gray-100 pt-4">
          <button
            type="button"
            onClick={() => setEditOpen(true)}
            className="rounded-md border border-gray-300 bg-white px-4 py-1.5 text-sm text-gray-700 transition hover:bg-gray-50"
          >
            编辑
          </button>
          <button
            type="button"
            onClick={() => setConfirmDeleteOpen(true)}
            className="rounded-md border border-red-300 bg-white px-4 py-1.5 text-sm text-red-600 transition hover:bg-red-50"
          >
            删除
          </button>
        </footer>
      </div>

      <TicketForm
        open={editOpen}
        initial={ticket}
        onClose={() => setEditOpen(false)}
        onSubmitted={(saved) => setData(saved)}
      />

      <ConfirmDialog
        open={confirmDeleteOpen}
        title="确认删除"
        description={`确定要删除 Ticket #${ticket.id} 吗？此操作不可撤销。`}
        confirmText="确认删除"
        destructive
        loading={deleting}
        onCancel={() => setConfirmDeleteOpen(false)}
        onConfirm={handleDelete}
      />
    </main>
  )
}
