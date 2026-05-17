import { fireEvent, render, screen } from '@testing-library/react'
import { describe, expect, it, vi } from 'vitest'

import Modal from '@/components/Modal/Modal'

describe('Modal', () => {
  it('does not render when open=false', () => {
    render(
      <Modal open={false} onClose={() => {}} title="hello">
        <p>body</p>
      </Modal>,
    )
    expect(screen.queryByRole('dialog')).not.toBeInTheDocument()
  })

  it('renders title and body when open', () => {
    render(
      <Modal open onClose={() => {}} title="hello">
        <p>body</p>
      </Modal>,
    )
    expect(screen.getByRole('dialog')).toBeInTheDocument()
    expect(screen.getByText('hello')).toBeInTheDocument()
    expect(screen.getByText('body')).toBeInTheDocument()
  })

  it('calls onClose on Esc by default', () => {
    const onClose = vi.fn()
    render(
      <Modal open onClose={onClose} title="t">
        <p>body</p>
      </Modal>,
    )
    fireEvent.keyDown(window, { key: 'Escape' })
    expect(onClose).toHaveBeenCalled()
  })

  it('does not close on Esc when closeOnEsc=false', () => {
    const onClose = vi.fn()
    render(
      <Modal open onClose={onClose} title="t" closeOnEsc={false}>
        <p>body</p>
      </Modal>,
    )
    fireEvent.keyDown(window, { key: 'Escape' })
    expect(onClose).not.toHaveBeenCalled()
  })

  it('does not close when clicking inner content', () => {
    const onClose = vi.fn()
    render(
      <Modal open onClose={onClose} title="t">
        <p>body</p>
      </Modal>,
    )
    fireEvent.click(screen.getByText('body'))
    expect(onClose).not.toHaveBeenCalled()
  })
})
