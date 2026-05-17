import { fireEvent, render, screen } from '@testing-library/react'
import { describe, expect, it, vi } from 'vitest'

import StatusFilter from '@/components/Sidebar/StatusFilter'

describe('StatusFilter', () => {
  it('renders all four statuses with localized labels', () => {
    render(<StatusFilter selected={[]} onToggle={() => {}} />)
    for (const label of ['待处理', '处理中', '已完成', '已关闭']) {
      expect(screen.getByText(label)).toBeInTheDocument()
    }
  })

  it('marks selected statuses as checked', () => {
    render(<StatusFilter selected={['open', 'done']} onToggle={() => {}} />)
    const openCheckbox = screen.getByRole('checkbox', { name: /待处理/ })
    const doneCheckbox = screen.getByRole('checkbox', { name: /已完成/ })
    const closedCheckbox = screen.getByRole('checkbox', { name: /已关闭/ })
    expect(openCheckbox).toBeChecked()
    expect(doneCheckbox).toBeChecked()
    expect(closedCheckbox).not.toBeChecked()
  })

  it('calls onToggle with the clicked status', () => {
    const onToggle = vi.fn()
    render(<StatusFilter selected={[]} onToggle={onToggle} />)
    fireEvent.click(screen.getByRole('checkbox', { name: /处理中/ }))
    expect(onToggle).toHaveBeenCalledWith('in_progress')
  })
})
