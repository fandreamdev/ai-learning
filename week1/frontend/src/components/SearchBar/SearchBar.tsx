import { useEffect, useState } from 'react'

import { useDebouncedValue } from '@/hooks/useDebouncedValue'

interface SearchBarProps {
  value: string | undefined
  onChange: (next: string | undefined) => void
  /** 防抖时间，默认 300ms */
  delayMs?: number
  /** 最小触发字符数，默认 2 */
  minLength?: number
}

/**
 * 关键字搜索框（spec §3.2 F15、§6.1）。
 *
 * - 受控 input，外部 ``value`` 变化时同步内部 input
 * - 输入防抖 300ms
 * - 字符数 < minLength 时视为清空（外部传 undefined）
 */
export default function SearchBar({
  value,
  onChange,
  delayMs = 300,
  minLength = 2,
}: SearchBarProps) {
  const [input, setInput] = useState(value ?? '')

  // 外部 URL 变化时同步内部 state
  useEffect(() => {
    setInput(value ?? '')
  }, [value])

  const debounced = useDebouncedValue(input, delayMs)

  useEffect(() => {
    const trimmed = debounced.trim()
    const current = value ?? ''
    if (trimmed === current) return
    if (trimmed.length === 0) {
      onChange(undefined)
    } else if (trimmed.length >= minLength) {
      onChange(trimmed)
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [debounced])

  return (
    <input
      type="search"
      value={input}
      onChange={(e) => setInput(e.target.value)}
      placeholder="搜索 Ticket 标题或描述..."
      className="w-full rounded-md border border-gray-200 bg-gray-50 px-3 py-2 text-sm focus:border-blue-500 focus:bg-white focus:outline-none focus:ring-1 focus:ring-blue-500"
      aria-label="搜索 Ticket"
    />
  )
}
