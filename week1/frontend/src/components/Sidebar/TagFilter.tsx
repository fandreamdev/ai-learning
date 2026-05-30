import { useTags } from '@/hooks/useTags'

interface TagFilterProps {
  value: string | undefined
  onChange: (tag: string | undefined) => void
}

export default function TagFilter({ value, onChange }: TagFilterProps) {
  const { data, loading } = useTags()
  return (
    <fieldset>
      <legend className="mb-2 font-medium" style={{ color: '#6C757D' }}>标签</legend>
      {loading ? (
        <p className="text-xs text-gray-400">加载中...</p>
      ) : data.length === 0 ? (
        <p className="text-xs text-gray-400">暂无标签</p>
      ) : (
        <div className="space-y-2">
          {data.map((tag) => {
            const checked = value === tag
            return (
              <label
                key={tag}
                className="flex cursor-pointer items-center gap-2 transition-colors duration-200 hover:text-gray-900"
                style={{ color: '#6C757D' }}
              >
                <input
                  type="radio"
                  name="tag-filter"
                  checked={checked}
                  onChange={() => onChange(tag)}
                  className="h-4 w-4 border-gray-300 transition-colors duration-200 focus:ring-2 focus:ring-primary focus:ring-offset-2"
                  style={{ accentColor: '#0066FF' }}
                />
                <span>{tag}</span>
              </label>
            )
          })}
          {value !== undefined && (
            <button
              type="button"
              onClick={() => onChange(undefined)}
              className="text-xs transition-colors duration-200 hover:underline"
              style={{ color: '#0066FF' }}
            >
              清除
            </button>
          )}
        </div>
      )}
    </fieldset>
  )
}
