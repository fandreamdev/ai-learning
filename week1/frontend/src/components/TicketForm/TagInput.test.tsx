import { fireEvent, render, screen } from '@testing-library/react'
import { describe, expect, it, vi } from 'vitest'

import TagInput from '@/components/TicketForm/TagInput'

// 屏蔽 useTags 内的 axios 请求
vi.mock('@/api/aggregations', () => ({
  listTags: vi.fn().mockResolvedValue([]),
  listAssignees: vi.fn().mockResolvedValue([]),
}))

describe('TagInput', () => {
  it('renders existing tags as chips', () => {
    render(<TagInput value={['bug', 'feat']} onChange={() => {}} />)
    expect(screen.getByText('bug')).toBeInTheDocument()
    expect(screen.getByText('feat')).toBeInTheDocument()
  })

  it('appends a tag on Enter, normalizing case and trimming', () => {
    const onChange = vi.fn()
    render(<TagInput value={[]} onChange={onChange} />)
    const input = screen.getByPlaceholderText(/输入标签后回车/)
    fireEvent.change(input, { target: { value: '  Bug  ' } })
    fireEvent.keyDown(input, { key: 'Enter' })
    expect(onChange).toHaveBeenCalledWith(['bug'])
  })

  it('appends a tag on comma key', () => {
    const onChange = vi.fn()
    render(<TagInput value={[]} onChange={onChange} />)
    const input = screen.getByPlaceholderText(/输入标签后回车/)
    fireEvent.change(input, { target: { value: 'feat' } })
    fireEvent.keyDown(input, { key: ',' })
    expect(onChange).toHaveBeenCalledWith(['feat'])
  })

  it('rejects duplicate (case-insensitive) without calling onChange', () => {
    const onChange = vi.fn()
    render(<TagInput value={['bug']} onChange={onChange} />)
    const input = screen.getByPlaceholderText('') as HTMLInputElement
    fireEvent.change(input, { target: { value: 'BUG' } })
    fireEvent.keyDown(input, { key: 'Enter' })
    expect(onChange).not.toHaveBeenCalled()
  })

  it('rejects tag exceeding maxLen', () => {
    const onChange = vi.fn()
    render(<TagInput value={[]} onChange={onChange} maxLen={5} />)
    const input = screen.getByPlaceholderText(/输入标签后回车/)
    fireEvent.change(input, { target: { value: 'too-long-tag' } })
    fireEvent.keyDown(input, { key: 'Enter' })
    expect(onChange).not.toHaveBeenCalled()
    expect(screen.getByText(/标签长度需在/)).toBeInTheDocument()
  })

  it('rejects tag when count exceeds maxCount', () => {
    const onChange = vi.fn()
    const value = ['a', 'b', 'c']
    render(<TagInput value={value} onChange={onChange} maxCount={3} />)
    // 输入框被禁用，无法添加新标签
    const input = screen.queryByPlaceholderText(/输入标签后回车/)
    expect(input).toBeNull()
  })

  it('removes tag when × is clicked', () => {
    const onChange = vi.fn()
    render(<TagInput value={['bug', 'feat']} onChange={onChange} />)
    fireEvent.click(screen.getByLabelText('移除 bug'))
    expect(onChange).toHaveBeenCalledWith(['feat'])
  })

  it('removes last tag on Backspace when input empty', () => {
    const onChange = vi.fn()
    render(<TagInput value={['bug', 'feat']} onChange={onChange} />)
    const input = screen.getByPlaceholderText('') as HTMLInputElement
    fireEvent.keyDown(input, { key: 'Backspace' })
    expect(onChange).toHaveBeenCalledWith(['bug'])
  })
})
