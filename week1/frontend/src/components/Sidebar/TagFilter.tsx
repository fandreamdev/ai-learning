import { useTags } from '@/hooks/useTags'

interface TagFilterProps {
  value: string | undefined
  onChange: (tag: string | undefined) => void
}

export default function TagFilter({ value, onChange }: TagFilterProps) {
  const { data, loading } = useTags()
  return (
    <fieldset>
      <legend className="mb-2 font-medium text-gray-700">标签</legend>
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
                className="flex cursor-pointer items-center gap-2 text-gray-700 hover:text-gray-900"
              >
                <input
                  type="radio"
                  name="tag-filter"
                  checked={checked}
                  onChange={() => onChange(tag)}
                  className="h-4 w-4 border-gray-300 text-blue-600 focus:ring-blue-500"
                />
                <span>{tag}</span>
              </label>
            )
          })}
          {value !== undefined && (
            <button
              type="button"
              onClick={() => onChange(undefined)}
              className="text-xs text-blue-600 hover:underline"
            >
              清除
            </button>
          )}
        </div>
      )}
    </fieldset>
  )
}
