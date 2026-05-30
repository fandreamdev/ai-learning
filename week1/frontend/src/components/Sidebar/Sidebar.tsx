import AssigneeFilter from '@/components/Sidebar/AssigneeFilter'
import PriorityFilter from '@/components/Sidebar/PriorityFilter'
import StatusFilter from '@/components/Sidebar/StatusFilter'
import TagFilter from '@/components/Sidebar/TagFilter'
import type {
  TicketListQuery,
  TicketPriority,
  TicketStatus,
} from '@/types/ticket'

interface SidebarProps {
  query: TicketListQuery
  onToggleStatus: (s: TicketStatus) => void
  onTogglePriority: (p: TicketPriority) => void
  onChangeAssignee: (name: string | undefined) => void
  onChangeTag: (tag: string | undefined) => void
  onClearAll: () => void
}

export default function Sidebar({
  query,
  onToggleStatus,
  onTogglePriority,
  onChangeAssignee,
  onChangeTag,
  onClearAll,
}: SidebarProps) {
  const hasAnyFilter =
    !!query.status?.length ||
    !!query.priority?.length ||
    !!query.assignee ||
    !!query.tag ||
    !!query.keyword

  return (
    <div className="flex h-full flex-col gap-6 overflow-y-auto p-4 text-sm">
      <div className="flex items-center justify-between">
        <h2 className="font-semibold" style={{ color: '#1A1F26' }}>筛选条件</h2>
        {hasAnyFilter && (
          <button
            type="button"
            onClick={onClearAll}
            className="text-xs transition-colors duration-200 hover:underline"
            style={{ color: '#0066FF' }}
          >
            清除全部
          </button>
        )}
      </div>

      <StatusFilter selected={query.status ?? []} onToggle={onToggleStatus} />
      <PriorityFilter selected={query.priority ?? []} onToggle={onTogglePriority} />
      <AssigneeFilter value={query.assignee} onChange={onChangeAssignee} />
      <TagFilter value={query.tag} onChange={onChangeTag} />
    </div>
  )
}
