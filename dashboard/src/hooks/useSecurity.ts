import useSWR from 'swr'
import { fetcher } from '@/lib/fetcher'
import type { SecurityResponse } from '@/lib/types'

export function useSecurity() {
  return useSWR<SecurityResponse>('/api/security', fetcher as () => Promise<SecurityResponse>, { refreshInterval: 5000 })
}
