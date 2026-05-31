import { Shield, Users, Database, MessageSquare } from 'lucide-react'
import type { UserRole } from '@/types/api'

const roles: Array<{
  role: UserRole
  label: string
  description: string
  permissions: string[]
}> = [
  {
    role: 'admin',
    label: '管理员',
    description: '管理用户、连接、查询和系统配置',
    permissions: ['用户管理', '连接管理', 'SQL 模式', '对话模式'],
  },
  {
    role: 'analyst',
    label: '分析师',
    description: '使用 SQL 和对话模式完成数据分析',
    permissions: ['连接查看', 'SQL 模式', '对话模式'],
  },
  {
    role: 'developer',
    label: '开发者',
    description: '调试 SQL 查询并维护数据连接',
    permissions: ['连接管理', 'SQL 模式'],
  },
  {
    role: 'business',
    label: '业务用户',
    description: '通过自然语言完成常规数据查询',
    permissions: ['对话模式'],
  },
]

const icons = [Shield, Users, Database, MessageSquare]

export function RoleManagement() {
  return (
    <div className="p-6">
      <div className="mb-6">
        <h1 className="text-2xl font-bold text-gray-800">角色管理</h1>
        <p className="mt-1 text-sm text-gray-500">查看系统内置角色和权限范围</p>
      </div>

      <div className="grid gap-4 md:grid-cols-2">
        {roles.map((item, index) => {
          const Icon = icons[index]
          return (
            <section key={item.role} className="rounded-lg border border-gray-200 bg-white p-4">
              <div className="flex items-start gap-3">
                <div className="rounded-md bg-blue-50 p-2 text-blue-600">
                  <Icon size={20} />
                </div>
                <div>
                  <h2 className="font-semibold text-gray-900">{item.label}</h2>
                  <p className="mt-1 text-sm text-gray-500">{item.description}</p>
                </div>
              </div>
              <div className="mt-4 flex flex-wrap gap-2">
                {item.permissions.map((permission) => (
                  <span key={permission} className="rounded bg-gray-100 px-2 py-1 text-xs text-gray-700">
                    {permission}
                  </span>
                ))}
              </div>
            </section>
          )
        })}
      </div>
    </div>
  )
}

export default RoleManagement
