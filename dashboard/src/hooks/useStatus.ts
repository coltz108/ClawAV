import useSWR from 'swr'
import { fetcher } from '@/lib/fetcher'
import type { StatusResponse } from '@/lib/types'

export function useStatus() {
  return useSWR<StatusResponse>('/api/status', fetcher as () => Promise<StatusResponse>, { refreshInterval: 5000 })
}
