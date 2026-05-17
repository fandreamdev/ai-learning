/**
 * MSW 请求处理器。
 *
 * 当前为空数组：阶段 4 仅安装 MSW 依赖，不主动 Mock 任何请求。
 * 后续阶段需要离线开发时，可在此添加 ``http.get('/api/v1/tickets', ...)`` 等处理器。
 */
import type { HttpHandler } from 'msw'

export const handlers: HttpHandler[] = []
