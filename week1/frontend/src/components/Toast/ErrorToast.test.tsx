import { fireEvent, render, screen } from '@testing-library/react'
import { describe, expect, it, vi } from 'vitest'

import ErrorToast from '@/components/Toast/ErrorToast'

describe('ErrorToast', () => {
  it('renders the message', () => {
    render(<ErrorToast message="boom" />)
    expect(screen.getByText(/boom/)).toBeInTheDocument()
  })

  it('shows retry button when onRetry is provided', () => {
    const onRetry = vi.fn()
    render(<ErrorToast message="boom" onRetry={onRetry} />)
    fireEvent.click(screen.getByText('重试'))
    expect(onRetry).toHaveBeenCalled()
  })

  it('hides retry when onRetry is omitted', () => {
    render(<ErrorToast message="boom" />)
    expect(screen.queryByText('重试')).not.toBeInTheDocument()
  })
})
