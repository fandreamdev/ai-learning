import { describe, expect, it } from 'vitest'

import { buildSearchFromQuery, parseQueryFromSearch } from '@/lib/queryString'

describe('queryString', () => {
  describe('parseQueryFromSearch', () => {
    it('returns defaults when empty', () => {
      const q = parseQueryFromSearch(new URLSearchParams())
      expect(q.page).toBe(1)
      expect(q.page_size).toBe(20)
      expect(q.sort_by).toBe('created_at')
      expect(q.sort_order).toBe('desc')
      expect(q.status).toBeUndefined()
      expect(q.priority).toBeUndefined()
    })

    it('parses multi values from csv', () => {
      const q = parseQueryFromSearch(
        new URLSearchParams('status=open,in_progress&priority=high,urgent'),
      )
      expect(q.status).toEqual(['open', 'in_progress'])
      expect(q.priority).toEqual(['high', 'urgent'])
    })

    it('drops invalid status values', () => {
      const q = parseQueryFromSearch(new URLSearchParams('status=open,foo,closed'))
      expect(q.status).toEqual(['open', 'closed'])
    })

    it('parses keyword and assignee with trim', () => {
      const q = parseQueryFromSearch(new URLSearchParams('keyword= login &assignee=zhang'))
      expect(q.keyword).toBe('login')
      expect(q.assignee).toBe('zhang')
    })

    it('clamps page_size at 100 and falls back on bad numbers', () => {
      const q = parseQueryFromSearch(
        new URLSearchParams('page=0&page_size=999&sort_by=title'),
      )
      expect(q.page).toBe(1)
      expect(q.page_size).toBe(100)
      expect(q.sort_by).toBe('created_at')
    })
  })

  describe('buildSearchFromQuery', () => {
    it('omits defaults', () => {
      const params = buildSearchFromQuery({
        page: 1,
        page_size: 20,
        sort_by: 'created_at',
        sort_order: 'desc',
      })
      expect(params.toString()).toBe('')
    })

    it('serializes multi values with comma', () => {
      const params = buildSearchFromQuery({
        status: ['open', 'in_progress'],
        priority: ['high'],
        page: 2,
      })
      expect(params.get('status')).toBe('open,in_progress')
      expect(params.get('priority')).toBe('high')
      expect(params.get('page')).toBe('2')
    })

    it('round-trips with parse', () => {
      const original = new URLSearchParams(
        'status=open,closed&keyword=login&sort_by=updated_at&page=3',
      )
      const parsed = parseQueryFromSearch(original)
      const rebuilt = buildSearchFromQuery(parsed)
      expect(rebuilt.get('status')).toBe('open,closed')
      expect(rebuilt.get('keyword')).toBe('login')
      expect(rebuilt.get('sort_by')).toBe('updated_at')
      expect(rebuilt.get('page')).toBe('3')
    })
  })
})
