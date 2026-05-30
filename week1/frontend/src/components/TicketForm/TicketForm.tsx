import { useEffect, useMemo, useState } from 'react'

import ConfirmDialog from '@/components/ConfirmDialog/ConfirmDialog'
import Modal from '@/components/Modal/Modal'
import PriorityRadio from '@/components/TicketForm/PriorityRadio'
import TagInput from '@/components/TicketForm/TagInput'
import { useToast } from '@/components/Toast/useToast'
import { useAssignees } from '@/hooks/useAssignees'
import { createTicket, updateTicket } from '@/api/tickets'
import { ApiError } from '@/types/api'
import type {
  Ticket,
  TicketCreateInput,
  TicketPriority,
  TicketUpdateInput,
} from '@/types/ticket'

interface TicketFormProps {
  open: boolean
  /** 提供则为编辑模式 */
  initial?: Ticket
  onClose: () => void
  onSubmitted: (saved: Ticket) => void
}

interface FormState {
  title: string
  description: string
  priority: TicketPriority
  assignee: string
  tags: string[]
}

const EMPTY: FormState = {
  title: '',
  description: '',
  priority: 'medium',
  assignee: '',
  tags: [],
}

function fromTicket(t: Ticket): FormState {
  return {
    title: t.title,
    description: t.description ?? '',
    priority: t.priority,
    assignee: t.assignee ?? '',
    tags: t.tags ?? [],
  }
}

/**
 * Ticket 新建/编辑弹窗（共用）。
 *
 * - 编辑模式不传 status；状态切换由详情页 PATCH /status 单独完成
 * - 关闭前若 dirty 则二次确认 "放弃修改？"
 * - 提交失败显示 Toast，弹窗保持开启
 */
export default function TicketForm({ open, initial, onClose, onSubmitted }: TicketFormProps) {
  const isEdit = !!initial
  const toast = useToast()
  const { data: assignees } = useAssignees()

  const initialState = useMemo<FormState>(() => (initial ? fromTicket(initial) : EMPTY), [initial])

  const [form, setForm] = useState<FormState>(initialState)
  const [titleErr, setTitleErr] = useState<string | null>(null)
  const [submitting, setSubmitting] = useState(false)
  const [askDiscard, setAskDiscard] = useState(false)

  // 重新打开时同步初始值
  useEffect(() => {
    if (open) {
      setForm(initialState)
      setTitleErr(null)
      setSubmitting(false)
      setAskDiscard(false)
    }
  }, [open, initialState])

  const dirty = useMemo(() => {
    if (form.title !== initialState.title) return true
    if (form.description !== initialState.description) return true
    if (form.priority !== initialState.priority) return true
    if (form.assignee !== initialState.assignee) return true
    if (form.tags.length !== initialState.tags.length) return true
    if (form.tags.some((t, i) => initialState.tags[i] !== t)) return true
    return false
  }, [form, initialState])

  const requestClose = () => {
    if (submitting) return
    if (dirty) {
      setAskDiscard(true)
    } else {
      onClose()
    }
  }

  const validate = (): boolean => {
    const trimmed = form.title.trim()
    if (!trimmed) {
      setTitleErr('标题不能为空')
      return false
    }
    if (trimmed.length > 200) {
      setTitleErr('标题最多 200 字符')
      return false
    }
    setTitleErr(null)
    return true
  }

  const handleSubmit = async () => {
    if (!validate()) return
    setSubmitting(true)
    try {
      let saved: Ticket
      if (isEdit && initial) {
        const payload: TicketUpdateInput = {
          title: form.title.trim(),
          description: form.description.trim() || null,
          priority: form.priority,
          assignee: form.assignee.trim() || null,
          tags: form.tags,
        }
        saved = await updateTicket(initial.id, payload)
        toast.success('已保存')
      } else {
        const payload: TicketCreateInput = {
          title: form.title.trim(),
          description: form.description.trim() || null,
          priority: form.priority,
          assignee: form.assignee.trim() || null,
          tags: form.tags,
        }
        saved = await createTicket(payload)
        toast.success('Ticket 已创建')
      }
      onSubmitted(saved)
      onClose()
    } catch (err) {
      const message = err instanceof ApiError ? err.message : '保存失败'
      toast.error(message)
    } finally {
      setSubmitting(false)
    }
  }

  return (
    <>
      <Modal
        open={open}
        onClose={requestClose}
        title={isEdit ? `编辑 Ticket #${initial.id}` : '新建 Ticket'}
        closeOnOverlayClick={!dirty && !submitting}
        size="md"
        footer={
          <>
            <button
              type="button"
              onClick={requestClose}
              disabled={submitting}
              className="rounded-lg border border-gray-200 bg-white px-4 py-1.5 text-sm transition-colors duration-200 hover:bg-gray-50 disabled:cursor-not-allowed disabled:opacity-60"
              style={{ color: '#6C757D' }}
            >
              取消
            </button>
            <button
              type="button"
              onClick={handleSubmit}
              disabled={submitting}
              className="rounded-lg px-4 py-1.5 text-sm font-medium text-white transition-colors duration-200 hover:opacity-90 disabled:cursor-not-allowed disabled:opacity-60"
              style={{ backgroundColor: '#0066FF' }}
            >
              {submitting ? '保存中...' : '保存'}
            </button>
          </>
        }
      >
        <form
          className="space-y-4 text-sm"
          onSubmit={(e) => {
            e.preventDefault()
            void handleSubmit()
          }}
        >
          <div>
            <label className="mb-1 block font-medium" style={{ color: '#6C757D' }}>
              标题 <span style={{ color: '#FF4D4F' }}>*</span>
            </label>
            <input
              type="text"
              value={form.title}
              onChange={(e) => {
                setForm((s) => ({ ...s, title: e.target.value }))
                if (titleErr) setTitleErr(null)
              }}
              maxLength={200}
              autoFocus
              className={`w-full rounded-lg border bg-white px-3 py-2 text-sm transition-colors duration-200 focus:outline-none focus:ring-2 focus:ring-primary/20 ${
                titleErr ? 'border-red-300' : 'border-gray-200'
              }`}
              style={{ color: '#1A1F26' }}
            />
            {titleErr && <p className="mt-1 text-xs" style={{ color: '#FF4D4F' }}>{titleErr}</p>}
          </div>

          <div>
            <label className="mb-1 block font-medium" style={{ color: '#6C757D' }}>描述</label>
            <textarea
              value={form.description}
              onChange={(e) => setForm((s) => ({ ...s, description: e.target.value }))}
              rows={4}
              className="w-full rounded-lg border border-gray-200 bg-white px-3 py-2 text-sm transition-colors duration-200 focus:border-primary focus:outline-none focus:ring-2 focus:ring-primary/20"
              style={{ color: '#1A1F26' }}
            />
          </div>

          <div>
            <label className="mb-1 block font-medium" style={{ color: '#6C757D' }}>优先级</label>
            <PriorityRadio
              value={form.priority}
              onChange={(p) => setForm((s) => ({ ...s, priority: p }))}
            />
          </div>

          <div>
            <label className="mb-1 block font-medium" style={{ color: '#6C757D' }}>负责人</label>
            <input
              type="text"
              list="assignee-options"
              value={form.assignee}
              maxLength={100}
              onChange={(e) => setForm((s) => ({ ...s, assignee: e.target.value }))}
              placeholder="可选；输入或从下拉选择"
              className="w-full rounded-lg border border-gray-200 bg-white px-3 py-2 text-sm transition-colors duration-200 focus:border-primary focus:outline-none focus:ring-2 focus:ring-primary/20"
              style={{ color: '#1A1F26' }}
            />
            <datalist id="assignee-options">
              {assignees.map((name) => (
                <option key={name} value={name} />
              ))}
            </datalist>
          </div>

          <div>
            <label className="mb-1 block font-medium text-gray-700">标签</label>
            <TagInput
              value={form.tags}
              onChange={(next) => setForm((s) => ({ ...s, tags: next }))}
              disabled={submitting}
            />
          </div>
        </form>
      </Modal>

      <ConfirmDialog
        open={askDiscard}
        title="放弃修改？"
        description="表单内容尚未保存，关闭后将丢失。"
        confirmText="放弃"
        destructive
        onCancel={() => setAskDiscard(false)}
        onConfirm={() => {
          setAskDiscard(false)
          onClose()
        }}
      />
    </>
  )
}
