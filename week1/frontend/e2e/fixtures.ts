/**
 * E2E fixture：
 * - 通过 API 清空 tickets，再注入固定的 3 条数据
 * - 每个测试前 reset，避免互相干扰
 *
 * 通过前端 dev server 的 Vite 代理访问 (`baseURL` = http://localhost:5173/api/v1）
 * 而不是直连后端 8000，避免 Windows 上 IPv6 vs IPv4 的 localhost 解析差异。
 */
import { type APIRequestContext, expect } from '@playwright/test'

export interface SeedTicket {
  id: number
  title: string
  status: string
  priority: string
  assignee: string | null
  tags: string[]
  description: string | null
}

async function clearAll(request: APIRequestContext): Promise<void> {
  // page_size 上限为 100；E2E fixture 数据量很小，足够
  const resp = await request.get('/api/v1/tickets?page_size=100')
  if (!resp.ok()) {
    const body = await resp.text()
    throw new Error(
      `clearAll GET failed: status=${resp.status()} url=${resp.url()} body=${body.slice(0, 200)}`,
    )
  }
  const body = (await resp.json()) as { data: { items: { id: number }[] } }
  for (const t of body.data.items) {
    await request.delete(`/api/v1/tickets/${t.id}`)
  }
}

export async function seedFixtures(request: APIRequestContext): Promise<SeedTicket[]> {
  await clearAll(request)
  const inputs = [
    {
      title: 'fix login captcha',
      description: 'captcha not refreshing',
      priority: 'high',
      assignee: 'alice',
      tags: ['bug', 'frontend'],
    },
    {
      title: 'add csv export',
      description: 'export tickets to csv',
      priority: 'medium',
      assignee: 'bob',
      tags: ['feat'],
    },
    {
      title: 'docs typo',
      description: null,
      priority: 'low',
      assignee: null,
      tags: [],
    },
  ]
  const created: SeedTicket[] = []
  for (const payload of inputs) {
    const resp = await request.post('/api/v1/tickets', { data: payload })
    expect(resp.ok()).toBeTruthy()
    const body = (await resp.json()) as { data: SeedTicket }
    created.push(body.data)
  }
  return created
}

export async function teardownFixtures(request: APIRequestContext): Promise<void> {
  await clearAll(request)
}
