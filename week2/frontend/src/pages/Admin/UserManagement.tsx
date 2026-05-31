import { useState, useEffect } from 'react';
import { Search, Plus, Edit2, Trash2, Shield, RefreshCw } from 'lucide-react';
import toast from 'react-hot-toast';
import type { UserPublic, UserRole } from '@/types/api';

interface UserManagementProps {
  currentUserId?: string;
}

export function UserManagement({ currentUserId }: UserManagementProps) {
  const [users, setUsers] = useState<UserPublic[]>([]);
  const [loading, setLoading] = useState(false);
  const [searchQuery, setSearchQuery] = useState('');
  const [showModal, setShowModal] = useState(false);
  const [editingUser, setEditingUser] = useState<UserPublic | null>(null);

  useEffect(() => {
    fetchUsers();
  }, []);

  const fetchUsers = async () => {
    setLoading(true);
    try {
      const response = await fetch('/api/v1/users', {
        headers: {
          Authorization: `Bearer ${localStorage.getItem('access_token')}`,
        },
      });
      const data = await response.json();
      if (data.code === 0) {
        setUsers(data.data.items || []);
      }
    } catch {
      toast.error('获取用户列表失败');
    } finally {
      setLoading(false);
    }
  };

  const handleDelete = async (userId: string) => {
    if (userId === currentUserId) {
      toast.error('不能删除当前登录用户');
      return;
    }
    if (!confirm('确定要删除此用户吗？')) return;

    try {
      const response = await fetch(`/api/v1/users/${userId}`, {
        method: 'DELETE',
        headers: {
          Authorization: `Bearer ${localStorage.getItem('access_token')}`,
        },
      });
      const result = await response.json();
      if (result.code === 0) {
        toast.success('用户已删除');
        fetchUsers();
      } else {
        toast.error(result.message || '删除失败');
      }
    } catch {
      toast.error('删除失败');
    }
  };

  const filteredUsers = users.filter(
    (user) =>
      user.username.toLowerCase().includes(searchQuery.toLowerCase()) ||
      user.email.toLowerCase().includes(searchQuery.toLowerCase())
  );

  const getRoleBadgeColor = (role: UserRole) => {
    switch (role) {
      case 'admin':
        return 'bg-red-100 text-red-700';
      case 'analyst':
        return 'bg-blue-100 text-blue-700';
      case 'developer':
        return 'bg-green-100 text-green-700';
      default:
        return 'bg-gray-100 text-gray-700';
    }
  };

  const getRoleLabel = (role: UserRole) => {
    switch (role) {
      case 'admin':
        return '管理员';
      case 'analyst':
        return '分析师';
      case 'developer':
        return '开发者';
      default:
        return '普通用户';
    }
  };

  return (
    <div className="p-6">
      <div className="flex items-center justify-between mb-6">
        <div>
          <h1 className="text-2xl font-bold text-gray-800">用户管理</h1>
          <p className="text-sm text-gray-500 mt-1">管理系统用户和权限</p>
        </div>
        <button
          onClick={() => {
            setEditingUser(null);
            setShowModal(true);
          }}
          className="flex items-center gap-2 px-4 py-2 bg-blue-500 text-white rounded-lg hover:bg-blue-600 transition-colors"
        >
          <Plus size={18} />
          添加用户
        </button>
      </div>

      <div className="mb-4">
        <div className="relative">
          <Search size={18} className="absolute left-3 top-1/2 -translate-y-1/2 text-gray-400" />
          <input
            type="text"
            placeholder="搜索用户名或邮箱..."
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            className="w-full pl-10 pr-4 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
          />
        </div>
      </div>

      <div className="bg-white rounded-lg border border-gray-200 overflow-hidden">
        {loading ? (
          <div className="flex items-center justify-center h-64">
            <RefreshCw size={24} className="animate-spin text-gray-400" />
          </div>
        ) : filteredUsers.length === 0 ? (
          <div className="flex flex-col items-center justify-center h-64 text-gray-500">
            <Shield size={48} className="mb-3 text-gray-300" />
            <p>没有找到匹配的用户</p>
          </div>
        ) : (
          <table className="w-full">
            <thead className="bg-gray-50">
              <tr>
                <th className="px-4 py-3 text-left text-sm font-medium text-gray-600">用户名</th>
                <th className="px-4 py-3 text-left text-sm font-medium text-gray-600">邮箱</th>
                <th className="px-4 py-3 text-left text-sm font-medium text-gray-600">角色</th>
                <th className="px-4 py-3 text-left text-sm font-medium text-gray-600">状态</th>
                <th className="px-4 py-3 text-left text-sm font-medium text-gray-600">创建时间</th>
                <th className="px-4 py-3 text-right text-sm font-medium text-gray-600">操作</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-gray-100">
              {filteredUsers.map((user) => (
                <tr key={user.id} className="hover:bg-gray-50">
                  <td className="px-4 py-3">
                    <div className="flex items-center gap-3">
                      <div className="w-8 h-8 bg-blue-100 rounded-full flex items-center justify-center text-blue-600 font-medium">
                        {user.username[0].toUpperCase()}
                      </div>
                      <span className="font-medium text-gray-800">{user.username}</span>
                    </div>
                  </td>
                  <td className="px-4 py-3 text-gray-600">{user.email}</td>
                  <td className="px-4 py-3">
                    <span className={`px-2 py-1 text-xs rounded-full ${getRoleBadgeColor(user.role)}`}>
                      {getRoleLabel(user.role)}
                    </span>
                  </td>
                  <td className="px-4 py-3">
                    <span className={`px-2 py-1 text-xs rounded-full ${user.is_active ? 'bg-green-100 text-green-700' : 'bg-gray-100 text-gray-600'}`}>
                      {user.is_active ? '启用' : '禁用'}
                    </span>
                  </td>
                  <td className="px-4 py-3 text-gray-500 text-sm">
                    {new Date(user.created_at).toLocaleDateString()}
                  </td>
                  <td className="px-4 py-3">
                    <div className="flex items-center justify-end gap-2">
                      <button
                        onClick={() => {
                          setEditingUser(user);
                          setShowModal(true);
                        }}
                        className="p-1.5 text-gray-500 hover:bg-gray-100 rounded"
                        title="编辑"
                      >
                        <Edit2 size={16} />
                      </button>
                      <button
                        onClick={() => handleDelete(user.id)}
                        className="p-1.5 text-red-500 hover:bg-red-50 rounded"
                        title="删除"
                        disabled={user.id === currentUserId}
                      >
                        <Trash2 size={16} />
                      </button>
                    </div>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        )}
      </div>
    </div>
  );
}

export default UserManagement;
