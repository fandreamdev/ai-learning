import { Outlet, Link, useLocation } from 'react-router-dom'
import { Activity, BookOpen, Database, LogOut, MessageSquare, Shield, Users } from 'lucide-react'
import { useAuthStore } from '@/stores/authStore'

export default function MainLayout() {
  const location = useLocation()
  const { user, logout } = useAuthStore()

  const navItems = [
    { path: '/sql', label: 'SQL 模式', icon: Database },
    { path: '/chat', label: '对话模式', icon: MessageSquare },
  ]
  const adminItems = user?.role === 'admin'
    ? [
        { path: '/admin/users', label: '用户管理', icon: Users },
        { path: '/admin/roles', label: '角色管理', icon: Shield },
        { path: '/admin/semantics', label: '语义层', icon: BookOpen },
        { path: '/admin/audit', label: '审计日志', icon: Activity },
      ]
    : []
  const allNavItems = [...navItems, ...adminItems]

  const handleLogout = () => {
    logout()
    window.location.href = '/login'
  }

  return (
    <div className="h-screen overflow-hidden flex">
      {/* 侧边栏 */}
      <aside className="h-screen w-64 flex-shrink-0 bg-gray-900 text-white flex flex-col overflow-y-auto">
        <div className="p-4 border-b border-gray-700">
          <h1 className="text-xl font-bold">SmartQuery AI</h1>
          <p className="text-sm text-gray-400 mt-1">智能数据库查询</p>
        </div>

        <nav className="flex-1 p-4 space-y-2">
          {allNavItems.map((item) => {
            const Icon = item.icon
            const isActive = location.pathname === item.path

            return (
              <Link
                key={item.path}
                to={item.path}
                className={`flex items-center gap-3 px-4 py-3 rounded-lg transition-colors ${
                  isActive
                    ? 'bg-primary-600 text-white'
                    : 'text-gray-300 hover:bg-gray-800'
                }`}
              >
                <Icon size={20} />
                <span>{item.label}</span>
              </Link>
            )
          })}
        </nav>

        <div className="p-4 border-t border-gray-700">
          <div className="flex items-center gap-3 mb-4">
            <div className="w-10 h-10 rounded-full bg-primary-600 flex items-center justify-center">
              <span className="text-sm font-medium">
                {user?.username?.charAt(0).toUpperCase() || 'U'}
              </span>
            </div>
            <div className="flex-1 min-w-0">
              <p className="text-sm font-medium truncate">{user?.username}</p>
              <p className="text-xs text-gray-400 capitalize">{user?.role}</p>
            </div>
          </div>

          <button
            onClick={handleLogout}
            className="flex items-center gap-3 w-full px-4 py-2 text-gray-300 hover:bg-gray-800 rounded-lg transition-colors"
          >
            <LogOut size={18} />
            <span>退出登录</span>
          </button>
        </div>
      </aside>

      {/* 主内容区 */}
      <main className="h-screen min-w-0 flex-1 overflow-y-auto bg-gray-50">
        <Outlet />
      </main>
    </div>
  )
}
