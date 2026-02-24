import useSWR from 'swr'
import { fetcher } from '@/lib/fetcher'
import type { PendingAction } from '@/lib/types'

export function usePending() {
  return useSWR<PendingAction[]>('/api/pending', fetcher as () => Promise<PendingAction[]>, { refreshInterval: 5000 })
}
