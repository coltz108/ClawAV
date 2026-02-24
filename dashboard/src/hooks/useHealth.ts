import useSWR from 'swr'
import { fetcher } from '@/lib/fetcher'
import type { HealthResponse } from '@/lib/types'

export function useHealth() {
  return useSWR<HealthResponse>('/api/health', fetcher as () => Promise<HealthResponse>, { refreshInterval: 5000 })
}
