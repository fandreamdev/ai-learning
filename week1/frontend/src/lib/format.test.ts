import { describe, expect, it } from 'vitest'

import { formatDate, formatDateTime } from '@/lib/format'

describe('formatDateTime', () => {
  it('formats ISO 8601 string to YYYY-MM-DD HH:mm:ss', () => {
    const out = formatDateTime('2026-05-17T09:30:00')
    expect(out).toMatch(/^2026-05-17 \d{2}:\d{2}:\d{2}$/)
  })

  it('returns - for null', () => {
    expect(formatDateTime(null)).toBe('-')
    expect(formatDateTime(undefined)).toBe('-')
  })

  it('returns - for invalid string', () => {
    expect(formatDateTime('not-a-date')).toBe('-')
  })
})

describe('formatDate', () => {
  it('formats only the date portion', () => {
    expect(formatDate('2026-05-17T09:30:00')).toBe('2026-05-17')
  })

  it('returns - for invalid input', () => {
    expect(formatDate(null)).toBe('-')
    expect(formatDate('xxx')).toBe('-')
  })
})
