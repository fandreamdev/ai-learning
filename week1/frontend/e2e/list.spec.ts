/**
 * E2E #1~3：列表加载 / 优先级过滤 / 关键字搜索
 */
import { expect, test } from '@playwright/test'

import { seedFixtures, teardownFixtures } from './fixtures'

test.beforeEach(async ({ request }) => {
  await seedFixtures(request)
})

test.afterAll(async ({ request }) => {
  await teardownFixtures(request)
})

test('1) list page loads with seeded tickets', async ({ page }) => {
  await page.goto('/')
  await expect(page.getByRole('heading', { name: 'Ticket 列表' })).toBeVisible()
  await expect(page.getByText('fix login captcha')).toBeVisible()
  await expect(page.getByText('add csv export')).toBeVisible()
  await expect(page.getByText('docs typo')).toBeVisible()
})

test('2) priority filter narrows the list', async ({ page }) => {
  await page.goto('/')
  await expect(page.getByText('fix login captcha')).toBeVisible()

  // 限定在 sidebar (complementary) 内点击 "高" label
  const sidebar = page.getByRole('complementary')
  await sidebar.getByText('高', { exact: true }).click()

  await expect(page.getByText('fix login captcha')).toBeVisible()
  await expect(page.getByText('add csv export')).not.toBeVisible()
  await expect(page.getByText('docs typo')).not.toBeVisible()

  // 取消后恢复
  await sidebar.getByText('高', { exact: true }).click()
  await expect(page.getByText('add csv export')).toBeVisible()
})

test('3) keyword search hits description', async ({ page }) => {
  await page.goto('/')

  await page.getByPlaceholder(/搜索 Ticket/).fill('csv')
  // 防抖 300ms
  await expect(page.getByText('add csv export')).toBeVisible()
  await expect(page.getByText('fix login captcha')).not.toBeVisible()

  // 清空恢复
  await page.getByPlaceholder(/搜索 Ticket/).fill('')
  await expect(page.getByText('fix login captcha')).toBeVisible()
})
