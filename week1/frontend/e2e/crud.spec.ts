/**
 * E2E #4~6：新建 / 状态切换 / 删除
 */
import { expect, test } from '@playwright/test'

import { seedFixtures, teardownFixtures } from './fixtures'

test.beforeEach(async ({ request }) => {
  await seedFixtures(request)
})

test.afterAll(async ({ request }) => {
  await teardownFixtures(request)
})

test('4) create ticket -> list increases', async ({ page }) => {
  await page.goto('/')
  await expect(page.getByText('docs typo')).toBeVisible()

  await page.getByRole('button', { name: '+ 新建 Ticket' }).click()
  await expect(page.getByRole('dialog', { name: '新建 Ticket' })).toBeVisible()

  // 标题字段：弹窗内的第一个 textbox（autoFocus 已聚焦）
  const titleInput = page.getByRole('dialog').locator('input[type="text"]').first()
  await titleInput.fill('e2e created ticket')
  await page.getByRole('button', { name: '保存' }).click()

  // 弹窗关闭 + Toast 成功 + 列表出现新条目
  await expect(page.getByRole('dialog', { name: '新建 Ticket' })).toBeHidden()
  await expect(page.getByText('e2e created ticket')).toBeVisible()
})

test('5) detail page: status select switches and persists', async ({ page, request }) => {
  await page.goto('/')
  // 进入第一个 ticket 详情
  await page.getByText('fix login captcha').click()
  await expect(page.getByRole('heading', { name: 'fix login captcha' })).toBeVisible()

  // 当前状态 open，下拉选 "→ 处理中"
  const select = page.getByLabel('切换状态')
  await select.selectOption({ label: '→ 处理中' })

  // 通过 API 检查后端状态
  const list = await (await request.get('/api/v1/tickets?keyword=login')).json()
  const ticket = list.data.items.find((t: { title: string }) => t.title === 'fix login captcha')
  expect(ticket.status).toBe('in_progress')
})

test('6) delete with confirm dialog -> list decreases', async ({ page }) => {
  await page.goto('/')
  await page.getByText('docs typo').click()
  await expect(page.getByRole('heading', { name: 'docs typo' })).toBeVisible()

  await page.getByRole('button', { name: '删除' }).click()
  // 确认弹窗
  await expect(page.getByRole('dialog', { name: '确认删除' })).toBeVisible()
  await page.getByRole('button', { name: '确认删除' }).click()

  // 跳回列表，且不再有 docs typo
  await expect(page).toHaveURL('/')
  await expect(page.getByText('docs typo')).not.toBeVisible()
  await expect(page.getByText('fix login captcha')).toBeVisible()
})
