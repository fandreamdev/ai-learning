import { act, fireEvent, render, screen } from '@testing-library/react'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'

import ToastProvider from '@/components/Toast/ToastProvider'
import { useToast } from '@/components/Toast/useToast'

function Trigger({ message, kind }: { message: string; kind: 'success' | 'error' | 'info' }) {
  const t = useToast()
  return (
    <button type="button" onClick={() => t[kind](message)}>
      fire-{kind}
    </button>
  )
}

describe('ToastProvider', () => {
  beforeEach(() => {
    vi.useFakeTimers()
  })
  afterEach(() => {
    vi.useRealTimers()
  })

  it('shows a success toast and auto-dismisses after 3s', () => {
    render(
      <ToastProvider>
        <Trigger message="ok" kind="success" />
      </ToastProvider>,
    )
    fireEvent.click(screen.getByText('fire-success'))
    expect(screen.getByText('ok')).toBeInTheDocument()
    act(() => {
      vi.advanceTimersByTime(2999)
    })
    expect(screen.getByText('ok')).toBeInTheDocument()
    act(() => {
      vi.advanceTimersByTime(2)
    })
    expect(screen.queryByText('ok')).not.toBeInTheDocument()
  })

  it('error stays for 5s', () => {
    render(
      <ToastProvider>
        <Trigger message="boom" kind="error" />
      </ToastProvider>,
    )
    fireEvent.click(screen.getByText('fire-error'))
    expect(screen.getByText('boom')).toBeInTheDocument()
    act(() => {
      vi.advanceTimersByTime(3500)
    })
    expect(screen.getByText('boom')).toBeInTheDocument()
    act(() => {
      vi.advanceTimersByTime(1600)
    })
    expect(screen.queryByText('boom')).not.toBeInTheDocument()
  })
})
