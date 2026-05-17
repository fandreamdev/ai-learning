/**
 * 聚合接口：标签 / 负责人（spec §5.3）。
 */
import { request } from '@/api/request'

export function listTags(): Promise<string[]> {
  return request.get<unknown, string[]>('/tags')
}

export function listAssignees(): Promise<string[]> {
  return request.get<unknown, string[]>('/assignees')
}
