/**
 * 首页 / 仪表盘
 */
import { Link } from 'react-router-dom'

const features = [
  {
    title: 'SQL 专业模式',
    description: '专业的 SQL 编辑器，支持语法高亮、自动补全、执行计划分析',
    icon: '💾',
    path: '/sql',
    color: 'bg-blue-50 text-blue-600',
  },
  {
    title: '自然语言对话',
    description: '用自然语言描述您的需求，AI 自动生成 SQL 并执行',
    icon: '💬',
    path: '/chat',
    color: 'bg-green-50 text-green-600',
  },
  {
    title: '智能图表',
    description: '根据数据特征自动推荐最佳可视化图表',
    icon: '📊',
    path: '/sql',
    color: 'bg-purple-50 text-purple-600',
  },
  {
    title: '语义层管理',
    description: '配置表字段的业务含义，提升自然语言理解准确度',
    icon: '📚',
    path: '/admin/connections',
    color: 'bg-orange-50 text-orange-600',
  },
]

export default function Dashboard() {
  return (
    <div className="space-y-8">
      {/* 欢迎区 */}
      <div className="bg-gradient-to-r from-primary-600 to-primary-700 rounded-2xl p-8 text-white">
        <h1 className="text-3xl font-bold mb-2">欢迎使用 SmartQuery AI</h1>
        <p className="text-primary-100 text-lg">
          智能双模数据库查询系统，让数据查询变得简单高效
        </p>
      </div>

      {/* 功能卡片 */}
      <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
        {features.map((feature) => (
          <Link
            key={feature.path}
            to={feature.path}
            className="bg-white rounded-xl p-6 shadow-sm border border-gray-100 hover:shadow-md hover:border-primary-200 transition-all"
          >
            <div className={`inline-flex items-center justify-center w-12 h-12 rounded-xl mb-4 ${feature.color}`}>
              <span className="text-2xl">{feature.icon}</span>
            </div>
            <h3 className="text-lg font-semibold text-gray-900 mb-2">
              {feature.title}
            </h3>
            <p className="text-gray-500 text-sm">
              {feature.description}
            </p>
          </Link>
        ))}
      </div>

      {/* 快捷操作 */}
      <div className="bg-white rounded-xl p-6 shadow-sm border border-gray-100">
        <h2 className="text-lg font-semibold text-gray-900 mb-4">快速开始</h2>
        <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
          <Link
            to="/sql"
            className="flex items-center gap-3 p-4 rounded-lg border border-gray-200 hover:border-primary-300 hover:bg-primary-50 transition-colors"
          >
            <span className="text-2xl">▶️</span>
            <div>
              <div className="font-medium text-gray-900">新建查询</div>
              <div className="text-xs text-gray-500">SQL 模式</div>
            </div>
          </Link>
          <Link
            to="/chat"
            className="flex items-center gap-3 p-4 rounded-lg border border-gray-200 hover:border-primary-300 hover:bg-primary-50 transition-colors"
          >
            <span className="text-2xl">💬</span>
            <div>
              <div className="font-medium text-gray-900">对话查询</div>
              <div className="text-xs text-gray-500">自然语言</div>
            </div>
          </Link>
          <Link
            to="/admin/connections"
            className="flex items-center gap-3 p-4 rounded-lg border border-gray-200 hover:border-primary-300 hover:bg-primary-50 transition-colors"
          >
            <span className="text-2xl">🔗</span>
            <div>
              <div className="font-medium text-gray-900">管理连接</div>
              <div className="text-xs text-gray-500">配置数据库</div>
            </div>
          </Link>
          <Link
            to="/admin/users"
            className="flex items-center gap-3 p-4 rounded-lg border border-gray-200 hover:border-primary-300 hover:bg-primary-50 transition-colors"
          >
            <span className="text-2xl">👥</span>
            <div>
              <div className="font-medium text-gray-900">用户管理</div>
              <div className="text-xs text-gray-500">权限配置</div>
            </div>
          </Link>
        </div>
      </div>
    </div>
  )
}
