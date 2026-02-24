import useSWR from 'swr'
import { fetcher } from '@/lib/fetcher'
import type { Alert } from '@/lib/types'

export function useAlerts() {
  return useSWR<Alert[]>('/api/alerts', fetcher as () => Promise<Alert[]>, { refreshInterval: 5000 })
}
