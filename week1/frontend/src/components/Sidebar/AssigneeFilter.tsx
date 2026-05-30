import { useAssignees } from '@/hooks/useAssignees'

interface AssigneeFilterProps {
  value: string | undefined
  onChange: (name: string | undefined) => void
}

export default function AssigneeFilter({ value, onChange }: AssigneeFilterProps) {
  const { data, loading } = useAssignees()
  return (
    <fieldset>
      <legend className="mb-2 font-medium" style={{ color: '#6C757D' }}>负责人</legend>
      <select
        value={value ?? ''}
        onChange={(e) => onChange(e.target.value || undefined)}
        disabled={loading}
        className="w-full rounded-lg border border-gray-200 bg-white px-2 py-1.5 text-sm transition-colors duration-200 focus:border-primary focus:outline-none focus:ring-2 focus:ring-primary/20 disabled:bg-gray-50"
        style={{ color: '#1A1F26' }}
      >
        <option value="">全部</option>
        {data.map((name) => (
          <option key={name} value={name}>
            {name}
          </option>
        ))}
      </select>
    </fieldset>
  )
}
