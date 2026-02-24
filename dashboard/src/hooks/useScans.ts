import useSWR from 'swr'
import { fetcher } from '@/lib/fetcher'
import type { ScanResult } from '@/lib/types'

export function useScans() {
  return useSWR<ScanResult[]>('/api/scans', fetcher as () => Promise<ScanResult[]>, { refreshInterval: 5000 })
}
