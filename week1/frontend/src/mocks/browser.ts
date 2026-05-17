/**
 * MSW 浏览器 worker（阶段 7 联调或离线开发时启用）。
 *
 * 启用步骤：
 *   1. ``npx msw init public/`` 生成 ``public/mockServiceWorker.js``
 *   2. 在 ``.env`` 中设置 ``VITE_ENABLE_MOCK=true``
 */
import { setupWorker } from 'msw/browser'

import { handlers } from '@/mocks/handlers'

export const worker = setupWorker(...handlers)
