import { fireEvent, render, screen } from '@testing-library/react'
import { describe, expect, it, vi } from 'vitest'

import Pagination from '@/components/TicketTable/Pagination'

describe('Pagination', () => {
  it('renders nothing when total is 0', () => {
    const { container } = render(
      <Pagination page={1} pageSize={20} total={0} onPageChange={() => {}} onPageSizeChange={() => {}} />,
    )
    expect(container).toBeEmptyDOMElement()
  })

  it('disables previous on the first page', () => {
    render(
      <Pagination page={1} pageSize={20} total={100} onPageChange={() => {}} onPageSizeChange={() => {}} />,
    )
    expect(screen.getByText('上一页')).toBeDisabled()
    expect(screen.getByText('下一页')).toBeEnabled()
  })

  it('disables next on the last page', () => {
    render(
      <Pagination page={5} pageSize={20} total={100} onPageChange={() => {}} onPageSizeChange={() => {}} />,
    )
    expect(screen.getByText('上一页')).toBeEnabled()
    expect(screen.getByText('下一页')).toBeDisabled()
  })

  it('calls onPageChange when next is clicked', () => {
    const onPage = vi.fn()
    render(
      <Pagination page={2} pageSize={20} total={100} onPageChange={onPage} onPageSizeChange={() => {}} />,
    )
    fireEvent.click(screen.getByText('下一页'))
    expect(onPage).toHaveBeenCalledWith(3)
  })

  it('calls onPageSizeChange via select', () => {
    const onSize = vi.fn()
    render(
      <Pagination page={1} pageSize={20} total={100} onPageChange={() => {}} onPageSizeChange={onSize} />,
    )
    fireEvent.change(screen.getByRole('combobox'), { target: { value: '50' } })
    expect(onSize).toHaveBeenCalledWith(50)
  })
})
