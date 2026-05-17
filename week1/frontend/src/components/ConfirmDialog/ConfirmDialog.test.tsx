import { fireEvent, render, screen } from '@testing-library/react'
import { describe, expect, it, vi } from 'vitest'

import ConfirmDialog from '@/components/ConfirmDialog/ConfirmDialog'

describe('ConfirmDialog', () => {
  it('calls onConfirm when 确认 is clicked', () => {
    const onConfirm = vi.fn()
    const onCancel = vi.fn()
    render(
      <ConfirmDialog
        open
        title="删除"
        description="确定吗？"
        confirmText="确认"
        cancelText="取消"
        onConfirm={onConfirm}
        onCancel={onCancel}
      />,
    )
    fireEvent.click(screen.getByText('确认'))
    expect(onConfirm).toHaveBeenCalled()
  })

  it('calls onCancel when 取消 is clicked', () => {
    const onCancel = vi.fn()
    render(
      <ConfirmDialog
        open
        title="t"
        confirmText="确认"
        cancelText="取消"
        onConfirm={() => {}}
        onCancel={onCancel}
      />,
    )
    fireEvent.click(screen.getByText('取消'))
    expect(onCancel).toHaveBeenCalled()
  })

  it('disables both buttons while loading', () => {
    render(
      <ConfirmDialog
        open
        title="t"
        confirmText="确认"
        cancelText="取消"
        loading
        onConfirm={() => {}}
        onCancel={() => {}}
      />,
    )
    expect(screen.getByText('取消')).toBeDisabled()
    expect(screen.getByText('处理中...')).toBeDisabled()
  })
})
