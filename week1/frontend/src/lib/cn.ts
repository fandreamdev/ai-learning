import { clsx, type ClassValue } from 'clsx'
import { twMerge } from 'tailwind-merge'

/**
 * Tailwind 类名合并工具，与 Shadcn UI 默认 cn 一致。
 */
export function cn(...inputs: ClassValue[]): string {
  return twMerge(clsx(inputs))
}
