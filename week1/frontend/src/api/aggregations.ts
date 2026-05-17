/**
 * 聚合接口：标签 / 负责人（spec §5.3）。
 */
import type { AxiosRequestConfig } from 'axios'

import { request } from '@/api/request'

export function listTags(config?: AxiosRequestConfig): Promise<string[]> {
  return request.get<unknown, string[]>('/tags', config)
}

export function listAssignees(config?: AxiosRequestConfig): Promise<string[]> {
  return request.get<unknown, string[]>('/assignees', config)
}
