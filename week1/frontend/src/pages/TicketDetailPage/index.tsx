import { Link, useParams } from 'react-router-dom'

/**
 * 阶段 6 将实现完整详情页（状态切换 / 编辑 / 删除）。
 * 当前为占位组件。
 */
export default function TicketDetailPage() {
  const { id } = useParams()
  return (
    <main className="min-h-screen p-8">
      <Link to="/" className="text-sm text-blue-600 underline">
        &lt; 返回列表
      </Link>
      <h1 className="mt-4 text-2xl font-semibold">Ticket #{id}</h1>
      <p className="mt-2 text-gray-500">详情页将在阶段 6 实现。</p>
    </main>
  )
}
