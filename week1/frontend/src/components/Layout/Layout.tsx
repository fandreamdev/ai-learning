import type { ReactNode } from 'react'

interface LayoutProps {
  header: ReactNode
  sidebar: ReactNode
  children: ReactNode
}

/**
 * 三栏布局：上 Header，下 (左 Sidebar + 右 Main)。
 *
 * 对照 spec §6.1。Header 高 56px，Sidebar 固定 240px。
 */
export default function Layout({ header, sidebar, children }: LayoutProps) {
  return (
    <div className="flex min-h-screen flex-col bg-gray-50">
      <header className="sticky top-0 z-20 h-14 border-b border-gray-200 bg-white shadow-sm">
        {header}
      </header>
      <div className="flex flex-1 overflow-hidden">
        <aside className="hidden w-60 shrink-0 border-r border-gray-200 bg-white md:block">
          {sidebar}
        </aside>
        <main className="flex-1 overflow-y-auto p-6">{children}</main>
      </div>
    </div>
  )
}
