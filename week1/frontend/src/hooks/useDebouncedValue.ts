import { useEffect, useState } from 'react'

/** 通用防抖 hook：对 ``value`` 进行 ``delayMs`` 毫秒防抖。 */
export function useDebouncedValue<T>(value: T, delayMs: number): T {
  const [debounced, setDebounced] = useState(value)
  useEffect(() => {
    const timer = setTimeout(() => setDebounced(value), delayMs)
    return () => clearTimeout(timer)
  }, [value, delayMs])
  return debounced
}
