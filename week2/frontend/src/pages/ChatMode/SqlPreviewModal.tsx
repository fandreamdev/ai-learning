import { useState } from 'react';
import { X, Copy, Check, Play, AlertCircle } from 'lucide-react';

interface SqlPreviewModalProps {
  sql: string;
  explanation?: string;
  confidence?: number;
  isOpen: boolean;
  onClose: () => void;
  onExecute?: (sql: string) => void;
  onEdit?: (sql: string) => void;
}

export function SqlPreviewModal({
  sql,
  explanation,
  confidence,
  isOpen,
  onClose,
  onExecute,
  onEdit,
}: SqlPreviewModalProps) {
  const [editedSql, setEditedSql] = useState(sql);
  const [copied, setCopied] = useState(false);
  const [isEditing, setIsEditing] = useState(false);

  if (!isOpen) return null;

  const handleCopy = async () => {
    await navigator.clipboard.writeText(editedSql);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  const handleExecute = () => {
    onExecute?.(editedSql);
  };

  const handleEdit = () => {
    if (isEditing && onEdit) {
      onEdit(editedSql);
    }
    setIsEditing(!isEditing);
  };

  const getConfidenceColor = (conf: number) => {
    if (conf >= 0.9) return 'text-green-600 bg-green-50';
    if (conf >= 0.7) return 'text-yellow-600 bg-yellow-50';
    return 'text-red-600 bg-red-50';
  };

  return (
    <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
      <div className="bg-white rounded-xl shadow-xl w-full max-w-2xl mx-4 max-h-[80vh] flex flex-col">
        {/* 头部 */}
        <div className="flex items-center justify-between p-4 border-b">
          <div className="flex items-center gap-3">
            <h3 className="text-lg font-semibold text-gray-800">SQL 预览</h3>
            {confidence !== undefined && (
              <span
                className={`px-2 py-0.5 text-xs rounded-full font-medium ${getConfidenceColor(confidence)}`}
              >
                置信度: {(confidence * 100).toFixed(0)}%
              </span>
            )}
          </div>
          <button
            onClick={onClose}
            className="p-1 hover:bg-gray-100 rounded"
          >
            <X size={20} />
          </button>
        </div>

        {/* 解释 */}
        {explanation && (
          <div className="px-4 py-3 bg-blue-50 border-b">
            <div className="flex items-start gap-2">
              <AlertCircle size={16} className="text-blue-500 mt-0.5 flex-shrink-0" />
              <p className="text-sm text-blue-800">{explanation}</p>
            </div>
          </div>
        )}

        {/* SQL 内容 */}
        <div className="flex-1 overflow-auto p-4">
          <div className="flex items-center justify-between mb-2">
            <span className="text-sm font-medium text-gray-600">SQL 语句</span>
            <div className="flex items-center gap-2">
              <button
                onClick={handleEdit}
                className={`px-2 py-1 text-xs rounded ${isEditing ? 'bg-blue-100 text-blue-700' : 'bg-gray-100 text-gray-600 hover:bg-gray-200'}`}
              >
                {isEditing ? '保存编辑' : '编辑'}
              </button>
              <button
                onClick={handleCopy}
                className="flex items-center gap-1 px-2 py-1 text-xs bg-gray-100 text-gray-600 hover:bg-gray-200 rounded"
              >
                {copied ? <Check size={12} /> : <Copy size={12} />}
                {copied ? '已复制' : '复制'}
              </button>
            </div>
          </div>

          {isEditing ? (
            <textarea
              value={editedSql}
              onChange={(e) => setEditedSql(e.target.value)}
              className="w-full h-48 p-3 font-mono text-sm border border-gray-300 rounded-lg resize-none focus:outline-none focus:ring-2 focus:ring-blue-500"
              placeholder="输入 SQL 语句..."
            />
          ) : (
            <pre className="w-full h-48 p-3 bg-gray-50 border border-gray-200 rounded-lg overflow-auto font-mono text-sm whitespace-pre-wrap">
              {editedSql}
            </pre>
          )}

          {isEditing && (
            <p className="mt-2 text-xs text-gray-500">
              * 提示：编辑后的 SQL 将直接执行，不再经过置信度检查
            </p>
          )}
        </div>

        {/* 底部按钮 */}
        <div className="flex items-center justify-end gap-3 p-4 border-t bg-gray-50 rounded-b-xl">
          <button
            onClick={onClose}
            className="px-4 py-2 text-gray-600 hover:bg-gray-200 rounded-lg transition-colors"
          >
            取消
          </button>
          <button
            onClick={handleExecute}
            className="flex items-center gap-2 px-4 py-2 bg-blue-500 text-white rounded-lg hover:bg-blue-600 transition-colors"
          >
            <Play size={16} />
            执行查询
          </button>
        </div>
      </div>
    </div>
  );
}

export default SqlPreviewModal;
