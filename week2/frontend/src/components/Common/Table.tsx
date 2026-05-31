import type { ReactNode } from 'react'

export interface TableColumn<T> {
  key: string
  title: ReactNode
  render: (row: T) => ReactNode
  className?: string
}

interface TableProps<T> {
  columns: TableColumn<T>[]
  data: T[]
  rowKey: (row: T) => string
  emptyText?: string
}

export function Table<T>({ columns, data, rowKey, emptyText = '暂无数据' }: TableProps<T>) {
  return (
    <div className="overflow-auto rounded-lg border border-gray-200 bg-white">
      <table className="w-full text-sm">
        <thead className="bg-gray-50">
          <tr>
            {columns.map((column) => (
              <th key={column.key} className={column.className ?? 'px-4 py-3 text-left font-medium text-gray-600'}>
                {column.title}
              </th>
            ))}
          </tr>
        </thead>
        <tbody className="divide-y divide-gray-100">
          {data.length === 0 ? (
            <tr>
              <td colSpan={columns.length} className="px-4 py-8 text-center text-gray-500">
                {emptyText}
              </td>
            </tr>
          ) : (
            data.map((row) => (
              <tr key={rowKey(row)} className="hover:bg-gray-50">
                {columns.map((column) => (
                  <td key={column.key} className={column.className ?? 'px-4 py-3 text-gray-700'}>
                    {column.render(row)}
                  </td>
                ))}
              </tr>
            ))
          )}
        </tbody>
      </table>
    </div>
  )
}

export default Table
