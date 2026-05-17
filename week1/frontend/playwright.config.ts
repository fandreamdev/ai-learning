import { defineConfig, devices } from '@playwright/test'

export default defineConfig({
  testDir: './e2e',
  // 关键：避免与 vitest 的 src/__tests__/*.test.tsx 冲突
  testMatch: ['**/*.spec.ts'],
  fullyParallel: false, // 共用业务库，避免 fixture 互相干扰
  workers: 1,
  retries: 0,
  reporter: [['list']],
  use: {
    baseURL: 'http://localhost:5173',
    trace: 'retain-on-failure',
    screenshot: 'only-on-failure',
  },
  projects: [
    {
      name: 'chromium',
      use: { ...devices['Desktop Chrome'] },
    },
  ],
})
