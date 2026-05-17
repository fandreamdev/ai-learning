/**
 * Axios 实例与拦截器。
 *
 * 设计要点：
 * - 响应拦截器拆封 ``{code, message, data}``，业务函数直接拿 ``data``
 * - 错误统一转 ``ApiError``：业务错误用 ``code``，网络/超时填 0
 * - 默认超时 10 秒
 */
import axios, { AxiosError, type AxiosInstance } from 'axios'

import { ApiError, type ApiResponse } from '@/types/api'

const baseURL = import.meta.env.VITE_API_BASE_URL || '/api/v1'

export const request: AxiosInstance = axios.create({
  baseURL,
  timeout: 10_000,
  headers: { 'Content-Type': 'application/json' },
})

request.interceptors.response.use(
  (resp) => {
    // 204 No Content 等无 body
    if (resp.status === 204 || resp.data == null) {
      return undefined
    }
    const body = resp.data as ApiResponse<unknown>
    if (typeof body !== 'object' || !('code' in body)) {
      // 后端非标准响应（不应发生），直接返回原始 data
      return resp.data
    }
    if (body.code !== 0) {
      throw new ApiError(body.code, body.message ?? 'unknown error')
    }
    return body.data
  },
  (err: AxiosError<ApiResponse<unknown>>) => {
    if (err.response?.data && typeof err.response.data === 'object') {
      const body = err.response.data
      throw new ApiError(
        body.code ?? err.response.status,
        body.message ?? err.message ?? 'request failed',
      )
    }
    if (err.code === 'ECONNABORTED') {
      throw new ApiError(0, '请求超时，请稍后重试')
    }
    throw new ApiError(0, err.message || '网络错误')
  },
)
