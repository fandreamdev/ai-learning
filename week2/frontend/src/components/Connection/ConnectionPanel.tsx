import { useState, useEffect } from 'react';
import { Plus, Edit2, Trash2, CheckCircle, XCircle, Database, RefreshCw, Settings } from 'lucide-react';
import toast from 'react-hot-toast';
import { useConnectionStore } from '@/stores/connectionStore';
import type { Connection, CreateConnectionRequest, DatabaseType } from '@/types/api';

// 数据库类型选项
const DATABASE_TYPES: { value: DatabaseType; label: string; icon: string }[] = [
  { value: 'postgresql', label: 'PostgreSQL', icon: '🐘' },
  { value: 'mysql', label: 'MySQL', icon: '🐬' },
  { value: 'clickhouse', label: 'ClickHouse', icon: '🏠' },
  { value: 'sqlite', label: 'SQLite', icon: '📄' },
];

interface ConnectionPanelProps {
  onConnectionSelect?: (connectionId: string) => void;
  compact?: boolean;
}

export function ConnectionPanel({ onConnectionSelect, compact = false }: ConnectionPanelProps) {
  const {
    connections,
    loading,
    selectedConnectionId,
    fetchConnections,
    createConnection,
    updateConnection,
    deleteConnection,
    testConnection,
    setDefaultConnection,
  } = useConnectionStore();

  const [isModalOpen, setIsModalOpen] = useState(false);
  const [editingConnection, setEditingConnection] = useState<Connection | null>(null);
  const [testingId, setTestingId] = useState<string | null>(null);

  useEffect(() => {
    fetchConnections();
  }, [fetchConnections]);

  const handleCreate = () => {
    setEditingConnection(null);
    setIsModalOpen(true);
  };

  const handleEdit = (conn: Connection) => {
    setEditingConnection(conn);
    setIsModalOpen(true);
  };

  const handleDelete = async (id: string) => {
    if (!confirm('确定要删除此连接吗？')) return;
    try {
      await deleteConnection(id);
      toast.success('连接已删除');
    } catch {
      toast.error('删除失败');
    }
  };

  const handleTest = async (id: string) => {
    setTestingId(id);
    try {
      const result = await testConnection(id);
      if (result.success) {
        toast.success('连接测试成功');
      } else {
        toast.error(`连接失败: ${result.message}`);
      }
    } catch {
      toast.error('测试连接失败');
    } finally {
      setTestingId(null);
    }
  };

  const handleSetDefault = async (id: string) => {
    try {
      await setDefaultConnection(id);
      toast.success('已设为默认连接');
    } catch {
      toast.error('设置失败');
    }
  };

  const handleSelect = (conn: Connection) => {
    useConnectionStore.setState({ currentConnectionId: conn.id });
    onConnectionSelect?.(conn.id);
  };

  if (compact) {
    return (
      <div className="flex items-center gap-2">
        <select
          className="flex-1 px-3 py-2 border border-gray-300 rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
          value={selectedConnectionId || ''}
          onChange={(e) => {
            const conn = connections.find((c: Connection) => c.id === e.target.value);
            if (conn) handleSelect(conn);
          }}
        >
          <option value="">选择连接...</option>
          {connections.map((conn: Connection) => (
            <option key={conn.id} value={conn.id}>
              {conn.is_default ? '⭐ ' : ''}{conn.name}
            </option>
          ))}
        </select>
        <button
          onClick={fetchConnections}
          className="p-2 text-gray-500 hover:text-gray-700 rounded-lg hover:bg-gray-100"
          title="刷新"
        >
          <RefreshCw size={18} className={loading ? 'animate-spin' : ''} />
        </button>
      </div>
    );
  }

  return (
    <div className="p-4">
      <div className="flex items-center justify-between mb-4">
        <h2 className="text-lg font-semibold text-gray-800 flex items-center gap-2">
          <Database size={20} />
          数据库连接
        </h2>
        <button
          onClick={handleCreate}
          className="flex items-center gap-1 px-3 py-1.5 bg-blue-500 text-white rounded-lg text-sm hover:bg-blue-600 transition-colors"
        >
          <Plus size={16} />
          新建连接
        </button>
      </div>

      {loading && connections.length === 0 ? (
        <div className="flex items-center justify-center py-12">
          <RefreshCw size={24} className="animate-spin text-gray-400" />
        </div>
      ) : connections.length === 0 ? (
        <div className="text-center py-12 text-gray-500">
          <Database size={48} className="mx-auto mb-3 text-gray-300" />
          <p className="mb-4">暂无数据库连接</p>
          <button
            onClick={handleCreate}
            className="px-4 py-2 bg-blue-500 text-white rounded-lg hover:bg-blue-600"
          >
            创建第一个连接
          </button>
        </div>
      ) : (
        <div className="space-y-3">
          {connections.map((conn: Connection) => (
            <ConnectionCard
              key={conn.id}
              connection={conn}
              isSelected={selectedConnectionId === conn.id}
              isTesting={testingId === conn.id}
              onSelect={() => handleSelect(conn)}
              onEdit={() => handleEdit(conn)}
              onDelete={() => handleDelete(conn.id)}
              onTest={() => handleTest(conn.id)}
              onSetDefault={() => handleSetDefault(conn.id)}
            />
          ))}
        </div>
      )}

      {isModalOpen && (
        <ConnectionModal
          connection={editingConnection}
          onClose={() => setIsModalOpen(false)}
          onSave={async (data) => {
            try {
              if (editingConnection) {
                await updateConnection(editingConnection.id, data);
                toast.success('连接已更新');
              } else {
                await createConnection(data);
                toast.success('连接已创建');
              }
              setIsModalOpen(false);
            } catch (err) {
              toast.error(editingConnection ? '更新失败' : '创建失败');
            }
          }}
        />
      )}
    </div>
  );
}

// 连接卡片组件
interface ConnectionCardProps {
  connection: Connection;
  isSelected: boolean;
  isTesting: boolean;
  onSelect: () => void;
  onEdit: () => void;
  onDelete: () => void;
  onTest: () => void;
  onSetDefault: () => void;
}

function ConnectionCard({
  connection,
  isSelected,
  isTesting,
  onSelect,
  onEdit,
  onDelete,
  onTest,
  onSetDefault,
}: ConnectionCardProps) {
  const dbType = DATABASE_TYPES.find((t) => t.value === connection.db_type);

  return (
    <div
      className={`p-4 rounded-lg border-2 transition-all cursor-pointer ${
        isSelected
          ? 'border-blue-500 bg-blue-50'
          : 'border-gray-200 bg-white hover:border-gray-300'
      }`}
      onClick={onSelect}
    >
      <div className="flex items-start justify-between">
        <div className="flex items-center gap-3">
          <span className="text-2xl">{dbType?.icon || '🗄️'}</span>
          <div>
            <div className="flex items-center gap-2">
              <h3 className="font-medium text-gray-900">{connection.name}</h3>
              {connection.is_default && (
                <span className="px-1.5 py-0.5 bg-yellow-100 text-yellow-700 text-xs rounded">
                  默认
                </span>
              )}
            </div>
            <p className="text-sm text-gray-500">
              {dbType?.label || connection.db_type} • {connection.host}:{connection.port}
            </p>
          </div>
        </div>
        <div className="flex items-center gap-1">
          {connection.status ? (
            <CheckCircle size={18} className="text-green-500" />
          ) : (
            <XCircle size={18} className="text-gray-400" />
          )}
        </div>
      </div>

      <div className="mt-3 flex items-center gap-2" onClick={(e) => e.stopPropagation()}>
        <button
          onClick={onTest}
          disabled={isTesting}
          className="flex items-center gap-1 px-2 py-1 text-sm text-gray-600 hover:bg-gray-100 rounded"
        >
          <RefreshCw size={14} className={isTesting ? 'animate-spin' : ''} />
          {isTesting ? '测试中...' : '测试'}
        </button>
        <button
          onClick={onSetDefault}
          disabled={connection.is_default}
          className="flex items-center gap-1 px-2 py-1 text-sm text-gray-600 hover:bg-gray-100 rounded"
        >
          <Settings size={14} />
          设为默认
        </button>
        <button
          onClick={onEdit}
          className="flex items-center gap-1 px-2 py-1 text-sm text-gray-600 hover:bg-gray-100 rounded"
        >
          <Edit2 size={14} />
          编辑
        </button>
        <button
          onClick={onDelete}
          className="flex items-center gap-1 px-2 py-1 text-sm text-red-600 hover:bg-red-50 rounded"
        >
          <Trash2 size={14} />
          删除
        </button>
      </div>
    </div>
  );
}

// 连接编辑弹窗
interface ConnectionModalProps {
  connection: Connection | null;
  onClose: () => void;
  onSave: (data: CreateConnectionRequest) => Promise<void>;
}

function ConnectionModal({ connection, onClose, onSave }: ConnectionModalProps) {
  const [form, setForm] = useState<CreateConnectionRequest>({
    name: connection?.name || '',
    db_type: connection?.db_type || 'postgresql',
    host: connection?.host || 'localhost',
    port: connection?.port || 5432,
    database_name: connection?.database_name || '',
    username: connection?.username || '',
    password: '',
    is_default: connection?.is_default || false,
  });

  const [saving, setSaving] = useState(false);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setSaving(true);
    try {
      await onSave(form);
    } finally {
      setSaving(false);
    }
  };

  const handleTypeChange = (type: DatabaseType) => {
    const defaultPorts: Record<DatabaseType, number> = {
      postgresql: 5432,
      mysql: 3306,
      clickhouse: 8123,
      sqlite: 0,
    };
    setForm({ ...form, db_type: type, port: defaultPorts[type] });
  };

  return (
    <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
      <div className="bg-white rounded-xl shadow-xl w-full max-w-lg mx-4">
        <div className="flex items-center justify-between p-4 border-b">
          <h3 className="text-lg font-semibold">
            {connection ? '编辑连接' : '新建连接'}
          </h3>
          <button onClick={onClose} className="p-1 hover:bg-gray-100 rounded">
            <XCircle size={20} />
          </button>
        </div>

        <form onSubmit={handleSubmit} className="p-4 space-y-4">
          {/* 连接名称 */}
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-1">
              连接名称
            </label>
            <input
              type="text"
              value={form.name}
              onChange={(e) => setForm({ ...form, name: e.target.value })}
              className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
              placeholder="我的数据库"
              required
            />
          </div>

          {/* 数据库类型 */}
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-1">
              数据库类型
            </label>
            <div className="grid grid-cols-2 gap-2">
              {DATABASE_TYPES.map((type) => (
                <button
                  key={type.value}
                  type="button"
                  onClick={() => handleTypeChange(type.value)}
                  className={`p-3 rounded-lg border-2 text-left transition-all ${
                    form.db_type === type.value
                      ? 'border-blue-500 bg-blue-50'
                      : 'border-gray-200 hover:border-gray-300'
                  }`}
                >
                  <span className="text-xl mr-2">{type.icon}</span>
                  <span className="font-medium">{type.label}</span>
                </button>
              ))}
            </div>
          </div>

          {/* 连接信息 */}
          <div className="grid grid-cols-3 gap-3">
            <div className="col-span-2">
              <label className="block text-sm font-medium text-gray-700 mb-1">
                主机
              </label>
              <input
                type="text"
                value={form.host}
                onChange={(e) => setForm({ ...form, host: e.target.value })}
                className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
                placeholder="localhost"
                required
              />
            </div>
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">
                端口
              </label>
              <input
                type="number"
                value={form.port}
                onChange={(e) => setForm({ ...form, port: parseInt(e.target.value) || 0 })}
                className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
                required
              />
            </div>
          </div>

          {/* 数据库名称 */}
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-1">
              数据库名称
            </label>
            <input
              type="text"
              value={form.database_name}
              onChange={(e) => setForm({ ...form, database_name: e.target.value })}
              className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
              placeholder="mydb"
              required
            />
          </div>

          {/* 用户名密码 */}
          <div className="grid grid-cols-2 gap-3">
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">
                用户名
              </label>
              <input
                type="text"
                value={form.username}
                onChange={(e) => setForm({ ...form, username: e.target.value })}
                className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
                placeholder="postgres"
              />
            </div>
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">
                密码
              </label>
              <input
                type="password"
                value={form.password}
                onChange={(e) => setForm({ ...form, password: e.target.value })}
                className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
                placeholder={connection ? '不修改请留空' : ''}
              />
            </div>
          </div>

          {/* 设为默认 */}
          <label className="flex items-center gap-2 cursor-pointer">
            <input
              type="checkbox"
              checked={form.is_default}
              onChange={(e) => setForm({ ...form, is_default: e.target.checked })}
              className="w-4 h-4 text-blue-500 rounded border-gray-300 focus:ring-blue-500"
            />
            <span className="text-sm text-gray-700">设为默认连接</span>
          </label>

          {/* 按钮 */}
          <div className="flex justify-end gap-3 pt-4 border-t">
            <button
              type="button"
              onClick={onClose}
              className="px-4 py-2 text-gray-700 hover:bg-gray-100 rounded-lg"
            >
              取消
            </button>
            <button
              type="submit"
              disabled={saving}
              className="px-4 py-2 bg-blue-500 text-white rounded-lg hover:bg-blue-600 disabled:opacity-50"
            >
              {saving ? '保存中...' : '保存'}
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}

export default ConnectionPanel;
