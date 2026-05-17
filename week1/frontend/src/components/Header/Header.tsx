import SearchBar from '@/components/SearchBar/SearchBar'

interface HeaderProps {
  keyword: string | undefined
  onKeywordChange: (kw: string | undefined) => void
  onNewTicket?: () => void
}

/**
 * 顶部栏：Logo + 关键字搜索 + 新建按钮（按钮在阶段 6 启用）。
 */
export default function Header({ keyword, onKeywordChange, onNewTicket }: HeaderProps) {
  return (
    <div className="mx-auto flex h-full w-full max-w-[1600px] items-center gap-6 px-6">
      <div className="flex items-center gap-2">
        <span className="grid h-8 w-8 place-items-center rounded-md bg-blue-600 font-bold text-white">
          P
        </span>
        <span className="text-lg font-semibold text-gray-900">ProjectAlpha</span>
      </div>

      <div className="flex-1 max-w-2xl">
        <SearchBar value={keyword} onChange={onKeywordChange} />
      </div>

      <button
        type="button"
        onClick={onNewTicket}
        disabled={!onNewTicket}
        className="rounded-md bg-blue-600 px-4 py-2 text-sm font-medium text-white shadow-sm transition hover:bg-blue-700 disabled:cursor-not-allowed disabled:opacity-50"
        title={onNewTicket ? '新建 Ticket' : '阶段 6 启用'}
      >
        + 新建 Ticket
      </button>
    </div>
  )
}
