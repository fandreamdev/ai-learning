import { Link } from 'react-router-dom'

/**
 * 阶段 5 将实现完整列表页（Header + Sidebar + TicketTable + 分页）。
 * 当前为占位组件，仅用于验证脚手架可启动。
 */
export default function TicketListPage() {
  return (
    <main className="min-h-screen p-8">
      <header className="mb-6 flex items-center justify-between">
        <h1 className="text-2xl font-semibold">ProjectAlpha</h1>
        <span className="text-sm text-gray-500">阶段 4 · 前端基础框架</span>
      </header>

      <section className="rounded-lg border border-gray-200 bg-white p-6 shadow-sm">
        <p className="text-gray-700">
          ✅ 前端骨架已就绪：Vite + React 19 + TypeScript + Tailwind v4 + React Router 7。
        </p>
        <p className="mt-2 text-sm text-gray-500">
          列表与筛选功能将在阶段 5 实现。新建 / 详情 / 删除 等交互将在阶段 6 实现。
        </p>
        <p className="mt-4 text-sm">
          路由测试：
          <Link to="/tickets/1" className="ml-2 text-blue-600 underline">
            /tickets/1（详情页占位）
          </Link>
          <Link to="/no-such-route" className="ml-4 text-blue-600 underline">
            /no-such-route（404）
          </Link>
        </p>
      </section>
    </main>
  )
}
