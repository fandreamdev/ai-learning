/**
 * request 拦截器测试：
 * - 业务码非 0 抛 ApiError
 * - HTTP 错误转 ApiError
 * - 超时（ECONNABORTED）转 ApiError(0, '请求超时')
 *
 * 通过 mock axios.create 返回的实例，捕获注册的 fulfilled/error handlers 并直接调用。
 */
import { describe, expect, it, vi } from 'vitest'
import type { AxiosError, AxiosResponse } from 'axios'

import { ApiError } from '@/types/api'

type Fulfilled = (resp: AxiosResponse) => unknown
type Rejected = (err: AxiosError) => unknown

let onSuccess: Fulfilled | undefined
let onError: Rejected | undefined

vi.mock('axios', () => {
  return {
    default: {
      create: () => ({
        interceptors: {
          response: {
            use: (s: Fulfilled, e: Rejected) => {
              onSuccess = s
              onError = e
            },
          },
        },
      }),
    },
  }
})

// import after mock to register interceptors
await import('@/api/request')

describe('request interceptors', () => {
  it('unwraps {code: 0, data} to data', () => {
    const result = onSuccess!({
      status: 200,
      data: { code: 0, message: 'ok', data: { hello: 'world' } },
    } as AxiosResponse)
    expect(result).toEqual({ hello: 'world' })
  })

  it('throws ApiError for code != 0', () => {
    expect(() =>
      onSuccess!({
        status: 200,
        data: { code: 40401, message: 'not found', data: null },
      } as AxiosResponse),
    ).toThrow(ApiError)
  })

  it('returns undefined for 204 no content', () => {
    const result = onSuccess!({ status: 204, data: undefined } as unknown as AxiosResponse)
    expect(result).toBeUndefined()
  })

  it('translates axios error with response body to ApiError', () => {
    const err = {
      response: {
        status: 400,
        data: { code: 40001, message: 'bad input', data: null },
      },
      message: 'Request failed',
    } as AxiosError
    expect(() => onError!(err)).toThrow(ApiError)
    try {
      onError!(err)
    } catch (e) {
      const apiErr = e as ApiError
      expect(apiErr.code).toBe(40001)
      expect(apiErr.message).toBe('bad input')
    }
  })

  it('translates timeout (ECONNABORTED) to ApiError(0)', () => {
    const err = { code: 'ECONNABORTED', message: 'timeout' } as AxiosError
    try {
      onError!(err)
    } catch (e) {
      const apiErr = e as ApiError
      expect(apiErr.code).toBe(0)
      expect(apiErr.message).toContain('请求超时')
    }
  })

  it('translates generic network error to ApiError(0)', () => {
    const err = { message: 'Network Error', code: 'ERR_NETWORK' } as AxiosError
    try {
      onError!(err)
    } catch (e) {
      const apiErr = e as ApiError
      expect(apiErr.code).toBe(0)
    }
  })
})
