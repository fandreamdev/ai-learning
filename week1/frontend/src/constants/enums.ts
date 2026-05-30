/**
 * 状态 / 优先级映射，对照 spec §3.3 与 §4.2。
 */
import type { TicketPriority, TicketStatus } from '@/types/ticket'

export const STATUS_LABEL: Record<TicketStatus, string> = {
  open: '待处理',
  in_progress: '处理中',
  done: '已完成',
  closed: '已关闭',
}

export const PRIORITY_LABEL: Record<TicketPriority, string> = {
  low: '低',
  medium: '中',
  high: '高',
  urgent: '紧急',
}

/** Priority colors — MotherDuck palette (spec §4.2) */
export const PRIORITY_COLOR: Record<TicketPriority, string> = {
  low: '#6C757D',     // gray-600: subdued
  medium: '#0066FF',  // primary: confident blue
  high: '#FA8C16',    // amber-orange: attention
  urgent: '#FF4D4F',  // warm red: urgent alert
}

/**
 * 状态流转规则（spec §3.3）：当前状态 → 允许的下一个状态集合。
 *
 * 同后端 ``app/core/constants.py`` 保持完全一致。
 */
export const STATUS_TRANSITIONS: Record<TicketStatus, TicketStatus[]> = {
  open: ['in_progress', 'closed'],
  in_progress: ['done', 'closed'],
  done: ['closed'],
  closed: ['open'],
}

/**
 * 排序字段下拉选项。
 */
export const SORT_OPTIONS = [
  { value: 'created_at:desc', label: '创建时间 ↓' },
  { value: 'created_at:asc', label: '创建时间 ↑' },
  { value: 'updated_at:desc', label: '更新时间 ↓' },
  { value: 'updated_at:asc', label: '更新时间 ↑' },
] as const

export const PAGE_SIZE_OPTIONS = [10, 20, 50, 100] as const
