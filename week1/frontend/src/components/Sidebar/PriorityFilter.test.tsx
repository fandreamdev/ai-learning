import { fireEvent, render, screen } from '@testing-library/react'
import { describe, expect, it, vi } from 'vitest'

import PriorityFilter from '@/components/Sidebar/PriorityFilter'

describe('PriorityFilter', () => {
  it('renders four priorities in spec order', () => {
    render(<PriorityFilter selected={[]} onToggle={() => {}} />)
    const labels = screen.getAllByRole('checkbox').map((c) => c.parentElement?.textContent ?? '')
    expect(labels).toHaveLength(4)
  })

  it('calls onToggle with the clicked priority', () => {
    const onToggle = vi.fn()
    render(<PriorityFilter selected={[]} onToggle={onToggle} />)
    fireEvent.click(screen.getByRole('checkbox', { name: /紧急/ }))
    expect(onToggle).toHaveBeenCalledWith('urgent')
  })

  it('marks selected priorities as checked', () => {
    render(<PriorityFilter selected={['high', 'urgent']} onToggle={() => {}} />)
    const high = screen.getByRole('checkbox', { name: /高/ })
    const urgent = screen.getByRole('checkbox', { name: /紧急/ })
    expect(high).toBeChecked()
    expect(urgent).toBeChecked()
  })
})
