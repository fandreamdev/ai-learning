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
          <div className="h-4 w-24 animate-pulse rounded" style={{ backgroundColor: '#E9ECEF' }} />
          <div className="h-8 w-48 animate-pulse rounded" style={{ backgroundColor: '#E9ECEF' }} />
          <div className="space-y-3 rounded-lg border border-gray-200 bg-white p-6">
            {Array.from({ length: 6 }).map((_, i) => (
              <div key={i} className="h-3 w-full animate-pulse rounded" style={{ backgroundColor: '#F0F0F0' }} />
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
        className="mb-4 text-sm transition-colors duration-200 hover:underline"
        style={{ color: '#0066FF' }}
      >
        ← 返回列表
      </button>

      <div className="rounded-lg border border-gray-200 bg-white">
        <header className="mb-4 border-b border-gray-100 pb-3 px-6 pt-6">
          <p className="text-xs" style={{ color: '#6C757D' }}>Ticket #{ticket.id}</p>
          <h1 className="mt-1 text-2xl font-semibold" style={{ color: '#1A1F26' }}>{ticket.title}</h1>
        </header>

        <dl className="grid grid-cols-[5rem_1fr] gap-y-3 px-6 pb-6 text-sm">
          <dt className="font-medium" style={{ color: '#6C757D' }}>状态</dt>
          <dd>
            <StatusSelect ticket={ticket} onChanged={setData} />
          </dd>

          <dt className="font-medium" style={{ color: '#6C757D' }}>优先级</dt>
          <dd>
            <PriorityBadge priority={ticket.priority} />
          </dd>

          <dt className="font-medium" style={{ color: '#6C757D' }}>负责人</dt>
          <dd style={{ color: '#6C757D' }}>{ticket.assignee || '-'}</dd>

          <dt className="font-medium" style={{ color: '#6C757D' }}>标签</dt>
          <dd className="flex flex-wrap gap-1.5">
            {ticket.tags?.length ? (
              ticket.tags.map((t) => (
                <span
                  key={t}
                  className="inline-flex items-center rounded-md px-1.5 py-0.5 text-xs font-medium"
                  style={{ backgroundColor: '#F0F4FF', color: '#0066FF' }}
                >
                  {t}
                </span>
              ))
            ) : (
              <span className="text-gray-400">-</span>
            )}
          </dd>

          <dt className="font-medium" style={{ color: '#6C757D' }}>创建时间</dt>
          <dd className="text-xs" style={{ color: '#6C757D' }}>{formatDateTime(ticket.created_at)}</dd>

          <dt className="font-medium" style={{ color: '#6C757D' }}>更新时间</dt>
          <dd className="text-xs" style={{ color: '#6C757D' }}>{formatDateTime(ticket.updated_at)}</dd>
        </dl>

        <section className="border-t border-gray-100 px-6 pb-6 pt-4">
          <h2 className="mb-2 text-sm font-medium" style={{ color: '#6C757D' }}>描述</h2>
          {ticket.description ? (
            <p className="whitespace-pre-wrap text-sm leading-relaxed" style={{ color: '#1A1F26' }}>
              {ticket.description}
            </p>
          ) : (
            <p className="text-sm text-gray-400">（无）</p>
          )}
        </section>

        <footer className="flex justify-end gap-2 border-t border-gray-100 px-6 pb-6 pt-4">
          <button
            type="button"
            onClick={() => setEditOpen(true)}
            className="rounded-lg border border-gray-200 bg-white px-4 py-1.5 text-sm transition-colors duration-200 hover:bg-gray-50"
            style={{ color: '#6C757D' }}
          >
            编辑
          </button>
          <button
            type="button"
            onClick={() => setConfirmDeleteOpen(true)}
            className="rounded-lg border px-4 py-1.5 text-sm transition-colors duration-200 hover:opacity-90"
            style={{ borderColor: '#FF4D4F', color: '#FF4D4F', backgroundColor: 'transparent' }}
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
