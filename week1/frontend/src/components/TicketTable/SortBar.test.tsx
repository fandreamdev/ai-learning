import { fireEvent, render, screen } from '@testing-library/react'
import { describe, expect, it, vi } from 'vitest'

import SortBar from '@/components/TicketTable/SortBar'

describe('SortBar', () => {
  it('renders all four sort options', () => {
    render(<SortBar sortBy="created_at" sortOrder="desc" onChange={() => {}} />)
    const select = screen.getByRole('combobox')
    expect(select.children.length).toBe(4)
  })

  it('reflects the current sort state', () => {
    render(<SortBar sortBy="updated_at" sortOrder="asc" onChange={() => {}} />)
    const select = screen.getByRole('combobox') as HTMLSelectElement
    expect(select.value).toBe('updated_at:asc')
  })

  it('parses combined value back into pair on change', () => {
    const onChange = vi.fn()
    render(<SortBar sortBy="created_at" sortOrder="desc" onChange={onChange} />)
    fireEvent.change(screen.getByRole('combobox'), { target: { value: 'updated_at:asc' } })
    expect(onChange).toHaveBeenCalledWith('updated_at', 'asc')
  })
})
