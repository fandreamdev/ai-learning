import { useState, useMemo } from 'react';
import { ChevronDown, ChevronUp, Copy, Download, Search } from 'lucide-react';
import type { ColumnMetadata } from '@/types/api';

interface QueryResultProps {
  columns: ColumnMetadata[];
  rows: unknown[][];
  rowCount: number;
  durationMs: number;
  loading?: boolean;
  onExport?: (format: 'csv' | 'json') => void;
  onChart?: () => void;
}

export function QueryResult({
  columns,
  rows,
  rowCount,
  durationMs,
  loading = false,
  onChart,
}: QueryResultProps) {
  const [expandedRows, setExpandedRows] = useState<Set<number>>(new Set());
  const [sortColumn, setSortColumn] = useState<number | null>(null);
  const [sortDirection, setSortDirection] = useState<'asc' | 'desc'>('asc');
  const [filterText, setFilterText] = useState('');

  // 过滤和排序数据
  const processedData = useMemo(() => {
    let data = rows.map((row, index) => ({ row, index }));

    // 过滤
    if (filterText) {
      const lowerFilter = filterText.toLowerCase();
      data = data.filter(({ row }) =>
        row.some((cell) => String(cell).toLowerCase().includes(lowerFilter))
      );
    }

    // 排序
    if (sortColumn !== null) {
      data.sort((a, b) => {
        const aVal = a.row[sortColumn];
        const bVal = b.row[sortColumn];
        const aStr = String(aVal ?? '');
        const bStr = String(bVal ?? '');

        // 尝试数字排序
        const aNum = Number(aVal);
        const bNum = Number(bVal);
        if (!isNaN(aNum) && !isNaN(bNum)) {
          return sortDirection === 'asc' ? aNum - bNum : bNum - aNum;
        }

        return sortDirection === 'asc'
          ? aStr.localeCompare(bStr)
          : bStr.localeCompare(aStr);
      });
    }

    return data;
  }, [rows, filterText, sortColumn, sortDirection]);

  const toggleRow = (index: number) => {
    const newExpanded = new Set(expandedRows);
    if (newExpanded.has(index)) {
      newExpanded.delete(index);
    } else {
      newExpanded.add(index);
    }
    setExpandedRows(newExpanded);
  };

  const handleSort = (columnIndex: number) => {
    if (sortColumn === columnIndex) {
      setSortDirection((prev) => (prev === 'asc' ? 'desc' : 'asc'));
    } else {
      setSortColumn(columnIndex);
      setSortDirection('asc');
    }
  };

  const copyToClipboard = (text: string) => {
    navigator.clipboard.writeText(text);
  };

  const exportCsv = () => {
    const headers = columns.map((c) => c.name).join(',');
    const csvRows = processedData.map(({ row }) =>
      row.map((cell) => `"${String(cell ?? '').replace(/"/g, '""')}"`).join(',')
    );
    const csv = [headers, ...csvRows].join('\n');

    const blob = new Blob([csv], { type: 'text/csv' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `query_result_${Date.now()}.csv`;
    a.click();
    URL.revokeObjectURL(url);
  };

  const exportJson = () => {
    const data = processedData.map(({ row }) => {
      const obj: Record<string, unknown> = {};
      columns.forEach((col, i) => {
        obj[col.name] = row[i];
      });
      return obj;
    });

    const blob = new Blob([JSON.stringify(data, null, 2)], { type: 'application/json' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `query_result_${Date.now()}.json`;
    a.click();
    URL.revokeObjectURL(url);
  };

  const formatCellValue = (value: unknown): string => {
    if (value === null || value === undefined) return 'NULL';
    if (typeof value === 'object') return JSON.stringify(value);
    return String(value);
  };

  if (loading) {
    return (
      <div className="flex items-center justify-center h-64 bg-gray-50 rounded-lg">
        <div className="flex flex-col items-center gap-2">
          <div className="w-8 h-8 border-4 border-blue-500 border-t-transparent rounded-full animate-spin" />
          <span className="text-sm text-gray-500">加载中...</span>
        </div>
      </div>
    );
  }

  if (rows.length === 0) {
    return (
      <div className="flex items-center justify-center h-64 bg-gray-50 rounded-lg">
        <div className="text-center text-gray-500">
          <p className="text-lg mb-2">查询结果为空</p>
          <p className="text-sm">没有找到匹配的数据</p>
        </div>
      </div>
    );
  }

  return (
    <div className="bg-white rounded-lg border border-gray-200 overflow-hidden">
      {/* 头部信息 */}
      <div className="flex items-center justify-between px-4 py-3 bg-gray-50 border-b border-gray-200">
        <div className="flex items-center gap-4 text-sm text-gray-600">
          <span>
            共 <strong>{rowCount}</strong> 行
          </span>
          <span>
            <strong>{columns.length}</strong> 列
          </span>
          <span>
            查询耗时 <strong>{durationMs}ms</strong>
          </span>
          {filterText && (
            <span className="text-blue-600">
              筛选后 <strong>{processedData.length}</strong> 行
            </span>
          )}
        </div>
        <div className="flex items-center gap-2">
          <button
            onClick={onChart}
            className="px-3 py-1.5 text-sm bg-blue-500 text-white rounded hover:bg-blue-600 transition-colors"
          >
            生成图表
          </button>
          <button
            onClick={exportCsv}
            className="px-3 py-1.5 text-sm text-gray-700 bg-white border border-gray-300 rounded hover:bg-gray-50 transition-colors flex items-center gap-1"
          >
            <Download size={14} />
            CSV
          </button>
          <button
            onClick={exportJson}
            className="px-3 py-1.5 text-sm text-gray-700 bg-white border border-gray-300 rounded hover:bg-gray-50 transition-colors flex items-center gap-1"
          >
            <Download size={14} />
            JSON
          </button>
        </div>
      </div>

      {/* 搜索框 */}
      <div className="px-4 py-2 border-b border-gray-200">
        <div className="relative">
          <Search size={16} className="absolute left-3 top-1/2 -translate-y-1/2 text-gray-400" />
          <input
            type="text"
            placeholder="搜索..."
            value={filterText}
            onChange={(e) => setFilterText(e.target.value)}
            className="w-full pl-9 pr-4 py-2 border border-gray-300 rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
          />
        </div>
      </div>

      {/* 表格 */}
      <div className="overflow-auto max-h-96">
        <table className="w-full text-sm">
          <thead className="bg-gray-100 sticky top-0">
            <tr>
              <th className="px-4 py-2 text-left font-medium text-gray-600 w-10">
                <span className="sr-only">展开</span>
              </th>
              {columns.map((col, index) => (
                <th
                  key={col.name}
                  className="px-4 py-2 text-left font-medium text-gray-600 cursor-pointer hover:bg-gray-200 transition-colors"
                  onClick={() => handleSort(index)}
                >
                  <div className="flex items-center gap-1">
                    <span>{col.name}</span>
                    {sortColumn === index && (
                      sortDirection === 'asc' ? (
                        <ChevronUp size={14} />
                      ) : (
                        <ChevronDown size={14} />
                      )
                    )}
                  </div>
                  <div className="text-xs font-normal text-gray-400">{col.data_type}</div>
                </th>
              ))}
            </tr>
          </thead>
          <tbody>
            {processedData.slice(0, 1000).map(({ row, index }) => (
              <>
                <tr
                  key={index}
                  className="border-b border-gray-100 hover:bg-gray-50"
                >
                  <td className="px-4 py-2">
                    <button
                      onClick={() => toggleRow(index)}
                      className="p-1 hover:bg-gray-200 rounded"
                    >
                      {expandedRows.has(index) ? (
                        <ChevronUp size={14} />
                      ) : (
                        <ChevronDown size={14} />
                      )}
                    </button>
                  </td>
                  {row.map((cell, cellIndex) => (
                    <td
                      key={cellIndex}
                      className="px-4 py-2 max-w-xs truncate"
                      title={formatCellValue(cell)}
                    >
                      <div className="flex items-center gap-2">
                        <span>{formatCellValue(cell)}</span>
                        <button
                          onClick={() => copyToClipboard(formatCellValue(cell))}
                          className="opacity-0 group-hover:opacity-100 p-1 hover:bg-gray-200 rounded"
                        >
                          <Copy size={12} />
                        </button>
                      </div>
                    </td>
                  ))}
                </tr>
                {expandedRows.has(index) && (
                  <tr key={`${index}-expanded`} className="bg-gray-50">
                    <td colSpan={columns.length + 1} className="px-4 py-3">
                      <div className="text-sm">
                        <div className="font-medium text-gray-700 mb-2">完整数据</div>
                        <pre className="bg-white p-3 rounded border border-gray-200 overflow-auto">
                          {JSON.stringify(
                            Object.fromEntries(
                              columns.map((col, i) => [col.name, row[i]])
                            ),
                            null,
                            2
                          )}
                        </pre>
                      </div>
                    </td>
                  </tr>
                )}
              </>
            ))}
          </tbody>
        </table>
      </div>

      {processedData.length > 1000 && (
        <div className="px-4 py-2 bg-yellow-50 text-yellow-700 text-sm text-center">
          只显示前 1000 行，共 {processedData.length} 行
        </div>
      )}
    </div>
  );
}

export default QueryResult;
