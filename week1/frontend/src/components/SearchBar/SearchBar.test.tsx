import { act, fireEvent, render, screen } from '@testing-library/react'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'

import SearchBar from '@/components/SearchBar/SearchBar'

describe('SearchBar', () => {
  beforeEach(() => {
    vi.useFakeTimers()
  })
  afterEach(() => {
    vi.useRealTimers()
  })

  it('does not fire onChange when input is below minLength', () => {
    const onChange = vi.fn()
    render(<SearchBar value={undefined} onChange={onChange} delayMs={300} />)

    fireEvent.change(screen.getByRole('searchbox'), { target: { value: 'a' } })
    act(() => {
      vi.advanceTimersByTime(400)
    })
    expect(onChange).not.toHaveBeenCalled()
  })

  it('fires onChange with trimmed string when length >= minLength after debounce', () => {
    const onChange = vi.fn()
    render(<SearchBar value={undefined} onChange={onChange} delayMs={300} />)

    fireEvent.change(screen.getByRole('searchbox'), { target: { value: '  login ' } })
    act(() => {
      vi.advanceTimersByTime(299)
    })
    expect(onChange).not.toHaveBeenCalled()
    act(() => {
      vi.advanceTimersByTime(1)
    })
    expect(onChange).toHaveBeenCalledWith('login')
  })

  it('clears upstream value when input becomes empty', () => {
    const onChange = vi.fn()
    render(<SearchBar value="login" onChange={onChange} delayMs={300} />)
    fireEvent.change(screen.getByRole('searchbox'), { target: { value: '' } })
    act(() => {
      vi.advanceTimersByTime(300)
    })
    expect(onChange).toHaveBeenCalledWith(undefined)
  })
})
