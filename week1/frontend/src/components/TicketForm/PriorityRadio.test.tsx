import { fireEvent, render, screen } from '@testing-library/react'
import { describe, expect, it, vi } from 'vitest'

import PriorityRadio from '@/components/TicketForm/PriorityRadio'

describe('PriorityRadio', () => {
  it('renders four radios with localized labels', () => {
    render(<PriorityRadio value="medium" onChange={() => {}} />)
    for (const label of ['低', '中', '高', '紧急']) {
      expect(screen.getByText(label)).toBeInTheDocument()
    }
  })

  it('marks the current value as checked', () => {
    render(<PriorityRadio value="high" onChange={() => {}} />)
    const high = screen.getByRole('radio', { name: /高/ }) as HTMLInputElement
    expect(high.checked).toBe(true)
  })

  it('calls onChange with selected value', () => {
    const onChange = vi.fn()
    render(<PriorityRadio value="medium" onChange={onChange} />)
    fireEvent.click(screen.getByRole('radio', { name: /紧急/ }))
    expect(onChange).toHaveBeenCalledWith('urgent')
  })
})
