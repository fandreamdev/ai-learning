import { useState } from 'react';
import { ChevronDown, ChevronRight, Copy, Check } from 'lucide-react';

interface ExecutionPlanProps {
  planType?: string;
  estimatedCost?: number | null;
  estimatedRows?: number | null;
  actualRows?: number | null;
  details?: Record<string, unknown>;
  raw?: string;
  loading?: boolean;
}

interface PlanNode {
  nodeType: string;
  relationName?: string;
  alias?: string;
  startupCost?: number;
  totalCost?: number;
  rows?: number;
  loops?: number;
  children?: PlanNode[];
}

export function ExecutionPlan({
  planType = 'SELECT',
  estimatedCost,
  estimatedRows,
  actualRows,
  details = {},
  raw,
  loading = false,
}: ExecutionPlanProps) {
  const [expandedNodes, setExpandedNodes] = useState<Set<string>>(new Set(['root']));
  const [copied, setCopied] = useState(false);

  const toggleNode = (nodeId: string) => {
    const newExpanded = new Set(expandedNodes);
    if (newExpanded.has(nodeId)) {
      newExpanded.delete(nodeId);
    } else {
      newExpanded.add(nodeId);
    }
    setExpandedNodes(newExpanded);
  };

  const copyToClipboard = () => {
    const text = raw || JSON.stringify(details, null, 2);
    navigator.clipboard.writeText(text);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  const parsePlanNode = (node: Record<string, unknown>, parentId: string, depth: number): PlanNode => {
    const nodeId = `${parentId}-${depth}`;
    const planNode: PlanNode = {
      nodeType: (node['node_type'] || node['Node Type'] || 'Unknown') as string,
      relationName: (node['relation_name'] || node['Relation Name']) as string | undefined,
      alias: (node['alias'] || node['Alias']) as string | undefined,
      startupCost: (node['startup_cost'] || node['Startup Cost']) as number | undefined,
      totalCost: (node['total_cost'] || node['Total Cost']) as number | undefined,
      rows: (node['rows_removed_by_filter'] || node['Rows Removed by Filter'] || node['Plan Rows']) as number | undefined,
      loops: (node['loops'] || node['Loops']) as number | undefined,
    };

    // 解析子节点
    const plans = node['Plans'] || node['children'] || node['Plan'];
    if (plans && Array.isArray(plans)) {
      planNode.children = plans.map((child: Record<string, unknown>, index: number) =>
        parsePlanNode(child, nodeId, index)
      );
    }

    return planNode;
  };

  const renderPlanNode = (node: PlanNode, nodeId: string, depth: number = 0) => {
    const isExpanded = expandedNodes.has(nodeId);
    const hasChildren = node.children && node.children.length > 0;

    const nodeColors: Record<string, string> = {
      'Seq Scan': 'bg-blue-100 border-blue-300',
      'Index Scan': 'bg-green-100 border-green-300',
      'Index Only Scan': 'bg-green-100 border-green-300',
      'Bitmap Heap Scan': 'bg-green-100 border-green-300',
      'Nested Loop': 'bg-yellow-100 border-yellow-300',
      'Hash Join': 'bg-orange-100 border-orange-300',
      'Merge Join': 'bg-orange-100 border-orange-300',
      'Hash': 'bg-purple-100 border-purple-300',
      'Sort': 'bg-pink-100 border-pink-300',
      'Aggregate': 'bg-indigo-100 border-indigo-300',
      'Limit': 'bg-gray-100 border-gray-300',
    };

    const colorClass = nodeColors[node.nodeType] || 'bg-gray-100 border-gray-300';

    return (
      <div key={nodeId} className="ml-4">
        <div
          className={`flex items-center gap-2 p-2 mb-1 border rounded ${colorClass} cursor-pointer hover:shadow-sm transition-shadow`}
          onClick={() => hasChildren && toggleNode(nodeId)}
          style={{ marginLeft: depth * 16 }}
        >
          {hasChildren ? (
            isExpanded ? (
              <ChevronDown size={16} className="flex-shrink-0" />
            ) : (
              <ChevronRight size={16} className="flex-shrink-0" />
            )
          ) : (
            <span className="w-4" />
          )}

          <span className="font-medium text-gray-800">{node.nodeType}</span>

          {node.relationName && (
            <>
              <span className="text-gray-400">on</span>
              <span className="text-blue-700 font-mono text-sm">{node.relationName}</span>
            </>
          )}

          {node.alias && (
            <span className="text-gray-500 text-sm">({node.alias})</span>
          )}

          <div className="flex-1" />

          {node.totalCost !== undefined && (
            <span className="text-xs bg-white px-2 py-0.5 rounded border border-gray-200">
              cost={node.startupCost?.toFixed(2) || '0'}..{node.totalCost.toFixed(2)}
            </span>
          )}

          {node.rows !== undefined && (
            <span className="text-xs bg-white px-2 py-0.5 rounded border border-gray-200">
              rows={node.rows}
            </span>
          )}

          {node.loops !== undefined && node.loops > 1 && (
            <span className="text-xs bg-yellow-100 px-2 py-0.5 rounded border border-yellow-300">
              x{node.loops}
            </span>
          )}
        </div>

        {isExpanded && hasChildren && (
          <div className="border-l-2 border-gray-300 ml-2">
            {node.children!.map((child, index) =>
              renderPlanNode(child, `${nodeId}-${index}`, 0)
            )}
          </div>
        )}
      </div>
    );
  };

  if (loading) {
    return (
      <div className="flex items-center justify-center h-64 bg-gray-50 rounded-lg">
        <div className="flex flex-col items-center gap-2">
          <div className="w-8 h-8 border-4 border-blue-500 border-t-transparent rounded-full animate-spin" />
          <span className="text-sm text-gray-500">分析执行计划中...</span>
        </div>
      </div>
    );
  }

  // 尝试解析 details 中的执行计划
  let planTree: PlanNode | null = null;
  try {
    if (details && Object.keys(details).length > 0) {
      planTree = parsePlanNode(details as Record<string, unknown>, 'root', 0);
    }
  } catch {
    // 解析失败
  }

  return (
    <div className="bg-white rounded-lg border border-gray-200 overflow-hidden">
      {/* 头部 */}
      <div className="flex items-center justify-between px-4 py-3 bg-gray-50 border-b border-gray-200">
        <div className="flex items-center gap-4">
          <h3 className="font-medium text-gray-800">执行计划</h3>
          <span className="px-2 py-0.5 bg-blue-100 text-blue-700 text-xs rounded">
            {planType}
          </span>
        </div>
        <button
          onClick={copyToClipboard}
          className="flex items-center gap-1 px-3 py-1.5 text-sm text-gray-600 hover:bg-gray-200 rounded transition-colors"
        >
          {copied ? <Check size={14} /> : <Copy size={14} />}
          {copied ? '已复制' : '复制'}
        </button>
      </div>

      {/* 统计信息 */}
      <div className="px-4 py-3 border-b border-gray-200 bg-blue-50">
        <div className="grid grid-cols-3 gap-4 text-sm">
          {estimatedCost !== undefined && estimatedCost !== null && (
            <div>
              <span className="text-gray-500">预估代价:</span>
              <span className="ml-2 font-medium text-gray-800">{estimatedCost.toFixed(2)}</span>
            </div>
          )}
          {estimatedRows !== undefined && estimatedRows !== null && (
            <div>
              <span className="text-gray-500">预估行数:</span>
              <span className="ml-2 font-medium text-gray-800">{estimatedRows.toLocaleString()}</span>
            </div>
          )}
          {actualRows !== undefined && actualRows !== null && (
            <div>
              <span className="text-gray-500">实际行数:</span>
              <span className="ml-2 font-medium text-gray-800">{actualRows.toLocaleString()}</span>
            </div>
          )}
        </div>
      </div>

      {/* 执行计划树 */}
      {planTree ? (
        <div className="p-4 overflow-auto max-h-96">
          {renderPlanNode(planTree, 'root')}
        </div>
      ) : raw ? (
        <div className="p-4">
          <pre className="text-xs font-mono bg-gray-50 p-3 rounded border border-gray-200 overflow-auto">
            {raw}
          </pre>
        </div>
      ) : (
        <div className="flex items-center justify-center h-32 text-gray-500">
          <p>暂无执行计划信息</p>
        </div>
      )}

      {/* 图例 */}
      <div className="px-4 py-3 border-t border-gray-200 bg-gray-50">
        <div className="text-xs text-gray-500">
          <span className="font-medium">图例:</span>
          <div className="flex flex-wrap gap-3 mt-2">
            <span className="flex items-center gap-1">
              <span className="w-3 h-3 bg-blue-100 border border-blue-300 rounded" />
              <span>扫描操作</span>
            </span>
            <span className="flex items-center gap-1">
              <span className="w-3 h-3 bg-green-100 border border-green-300 rounded" />
              <span>索引操作</span>
            </span>
            <span className="flex items-center gap-1">
              <span className="w-3 h-3 bg-orange-100 border border-orange-300 rounded" />
              <span>连接操作</span>
            </span>
            <span className="flex items-center gap-1">
              <span className="w-3 h-3 bg-yellow-100 border border-yellow-300 rounded" />
              <span>嵌套循环</span>
            </span>
          </div>
        </div>
      </div>
    </div>
  );
}

export default ExecutionPlan;
