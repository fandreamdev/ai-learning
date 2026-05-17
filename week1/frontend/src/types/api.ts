/**
 * 通用 API 响应信封（与后端 spec §5.1 对齐）。
 */
export interface ApiResponse<T> {
  code: number
  message: string
  data: T | null
}

/**
 * 列表分页响应（spec §5.1）。
 */
export interface PageData<T> {
  items: T[]
  total: number
  page: number
  page_size: number
}

/**
 * 业务错误。响应码非 0 或网络/超时都会以 ApiError 的形式抛出。
 */
export class ApiError extends Error {
  public readonly code: number

  constructor(code: number, message: string) {
    super(message)
    this.code = code
    this.name = 'ApiError'
  }
}
