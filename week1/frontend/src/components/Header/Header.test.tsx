import { fireEvent, render, screen } from '@testing-library/react'
import { describe, expect, it, vi } from 'vitest'

import Header from '@/components/Header/Header'

describe('Header', () => {
  it('shows the new-ticket button as enabled when callback is provided', () => {
    const onNew = vi.fn()
    render(<Header keyword={undefined} onKeywordChange={() => {}} onNewTicket={onNew} />)
    const btn = screen.getByText('+ 新建 Ticket')
    expect(btn).toBeEnabled()
    fireEvent.click(btn)
    expect(onNew).toHaveBeenCalled()
  })

  it('disables the new-ticket button when no callback', () => {
    render(<Header keyword={undefined} onKeywordChange={() => {}} />)
    expect(screen.getByText('+ 新建 Ticket')).toBeDisabled()
  })

  it('renders ProjectAlpha logo and title', () => {
    render(<Header keyword={undefined} onKeywordChange={() => {}} />)
    expect(screen.getByText('ProjectAlpha')).toBeInTheDocument()
  })
})
