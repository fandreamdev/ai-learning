import { useAssignees } from '@/hooks/useAssignees'

interface AssigneeFilterProps {
  value: string | undefined
  onChange: (name: string | undefined) => void
}

export default function AssigneeFilter({ value, onChange }: AssigneeFilterProps) {
  const { data, loading } = useAssignees()
  return (
    <fieldset>
      <legend className="mb-2 font-medium text-gray-700">负责人</legend>
      <select
        value={value ?? ''}
        onChange={(e) => onChange(e.target.value || undefined)}
        disabled={loading}
        className="w-full rounded-md border border-gray-300 bg-white px-2 py-1.5 text-sm focus:border-blue-500 focus:outline-none focus:ring-1 focus:ring-blue-500 disabled:bg-gray-50"
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
