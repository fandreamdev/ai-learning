import { useState } from 'react'

import Header from '@/components/Header/Header'
import Layout from '@/components/Layout/Layout'
import Sidebar from '@/components/Sidebar/Sidebar'
import TicketForm from '@/components/TicketForm/TicketForm'
import Pagination from '@/components/TicketTable/Pagination'
import SortBar from '@/components/TicketTable/SortBar'
import TicketTable from '@/components/TicketTable/TicketTable'
import ErrorToast from '@/components/Toast/ErrorToast'
import { useTicketListUrlState } from '@/hooks/useTicketListUrlState'
import { useTickets } from '@/hooks/useTickets'

/**
 * Ticket 列表页（spec §6.1）。
 *
 * - 所有筛选/搜索/排序/分页状态保存在 URL（``useTicketListUrlState``）
 * - 数据通过 ``useTickets`` 加载，AbortController 防止竞态
 * - 加载/空/错误三态分别由 TableSkeleton / EmptyState / ErrorToast 处理
 * - "新建" 按钮触发 TicketForm 弹窗，提交成功后刷新列表
 */
export default function TicketListPage() {
  const {
    query,
    toggleStatus,
    togglePriority,
    setAssignee,
    setTag,
    setKeyword,
    setSort,
    setPage,
    setPageSize,
    clearAll,
  } = useTicketListUrlState()

  const { data, loading, error, reload } = useTickets(query)
  const [createOpen, setCreateOpen] = useState(false)

  const items = data?.items ?? []
  const total = data?.total ?? 0
  const page = data?.page ?? query.page ?? 1
  const pageSize = data?.page_size ?? query.page_size ?? 20

  return (
    <>
      <Layout
        header={
          <Header
            keyword={query.keyword}
            onKeywordChange={setKeyword}
            onNewTicket={() => setCreateOpen(true)}
          />
        }
        sidebar={
          <Sidebar
            query={query}
            onToggleStatus={toggleStatus}
            onTogglePriority={togglePriority}
            onChangeAssignee={setAssignee}
            onChangeTag={setTag}
            onClearAll={clearAll}
          />
        }
      >
        <div className="mx-auto flex max-w-[1400px] flex-col gap-4">
          <div className="flex items-center justify-between">
            <h1 className="text-xl font-semibold text-gray-900">Ticket 列表</h1>
            <SortBar
              sortBy={query.sort_by ?? 'created_at'}
              sortOrder={query.sort_order ?? 'desc'}
              onChange={setSort}
            />
          </div>

          {error && <ErrorToast message={error.message} onRetry={reload} />}

          {!error && <TicketTable items={items} loading={loading} />}

          {!loading && !error && total > 0 && (
            <Pagination
              page={page}
              pageSize={pageSize}
              total={total}
              onPageChange={setPage}
              onPageSizeChange={setPageSize}
            />
          )}
        </div>
      </Layout>

      <TicketForm
        open={createOpen}
        onClose={() => setCreateOpen(false)}
        onSubmitted={() => {
          reload()
        }}
      />
    </>
  )
}
