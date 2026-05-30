import { useState, type KeyboardEvent } from 'react'

import { useTags } from '@/hooks/useTags'

interface TagInputProps {
  value: string[]
  onChange: (next: string[]) => void
  /** 单项最大字符数 */
  maxLen?: number
  /** 总数上限 */
  maxCount?: number
  disabled?: boolean
}

/**
 * 标签输入：chip 风格 + 回车追加。
 *
 * - 自动 trim + lower + 去重，与后端 _normalize_tag_list 行为一致
 * - 输入空格/逗号也可分隔
 * - 推荐区显示已存在的标签（来自 useTags），点击追加
 */
export default function TagInput({
  value,
  onChange,
  maxLen = 20,
  maxCount = 10,
  disabled = false,
}: TagInputProps) {
  const [draft, setDraft] = useState('')
  const [error, setError] = useState<string | null>(null)
  const { data: suggestions } = useTags()

  const tryAdd = (raw: string) => {
    const item = raw.trim().toLowerCase()
    if (!item) return
    if (item.length > maxLen) {
      setError(`标签长度需在 1~${maxLen} 之间`)
      return
    }
    if (value.length >= maxCount) {
      setError(`最多 ${maxCount} 个标签`)
      return
    }
    if (value.includes(item)) {
      setDraft('')
      return
    }
    onChange([...value, item])
    setDraft('')
    setError(null)
  }

  const handleKeyDown = (e: KeyboardEvent<HTMLInputElement>) => {
    if (e.key === 'Enter' || e.key === ',') {
      e.preventDefault()
      tryAdd(draft)
    } else if (e.key === 'Backspace' && draft === '' && value.length > 0) {
      onChange(value.slice(0, -1))
    }
  }

  const remove = (tag: string) => {
    onChange(value.filter((t) => t !== tag))
    setError(null)
  }

  const remainingSuggestions = suggestions.filter((s) => !value.includes(s))

  return (
    <div className="space-y-2">
      <div
        className={`flex min-h-9 flex-wrap items-center gap-1.5 rounded-lg border bg-white px-2 py-1.5 transition-colors duration-200 focus-within:border-primary focus-within:ring-2 focus-within:ring-primary/20 ${
          error ? 'border-red-300' : 'border-gray-200'
        }`}
      >
        {value.map((tag) => (
          <span
            key={tag}
            className="inline-flex items-center gap-1 rounded-md px-2 py-0.5 text-xs font-medium"
            style={{ backgroundColor: '#F0F4FF', color: '#0066FF' }}
          >
            {tag}
            <button
              type="button"
              onClick={() => remove(tag)}
              disabled={disabled}
              aria-label={`移除 ${tag}`}
              className="transition-colors duration-200 hover:opacity-70 disabled:cursor-not-allowed"
              style={{ color: '#0066FF' }}
            >
              ✕
            </button>
          </span>
        ))}
        <input
          type="text"
          value={draft}
          onChange={(e) => {
            setDraft(e.target.value)
            setError(null)
          }}
          onKeyDown={handleKeyDown}
          onBlur={() => draft && tryAdd(draft)}
          disabled={disabled || value.length >= maxCount}
          placeholder={value.length === 0 ? '输入标签后回车...' : ''}
          className="flex-1 min-w-[8rem] border-0 bg-transparent text-sm placeholder:text-gray-400 focus:outline-none focus:ring-0 disabled:cursor-not-allowed"
          style={{ color: '#1A1F26' }}
        />
      </div>

      {error && <p className="text-xs" style={{ color: '#FF4D4F' }}>{error}</p>}

      {remainingSuggestions.length > 0 && (
        <div className="flex flex-wrap items-center gap-1.5 text-xs">
          <span className="text-gray-400">推荐：</span>
          {remainingSuggestions.slice(0, 8).map((s) => (
            <button
              key={s}
              type="button"
              onClick={() => tryAdd(s)}
              disabled={disabled || value.length >= maxCount}
              className="rounded-lg border border-gray-200 bg-white px-2 py-0.5 text-xs transition-colors duration-200 hover:border-primary hover:text-primary disabled:cursor-not-allowed disabled:opacity-50"
              style={{ color: '#6C757D' }}
            >
              + {s}
            </button>
          ))}
        </div>
      )}

      <p className="text-xs text-gray-400">
        {value.length} / {maxCount} 个标签，每个 1~{maxLen} 字符
      </p>
    </div>
  )
}
