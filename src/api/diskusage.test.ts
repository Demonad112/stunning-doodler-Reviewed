import { beforeEach, describe, expect, it, vi } from 'vitest'
import {
  compareSnapshots,
  listDrives,
  listSnapshots,
  openDiskUsage,
  takeSnapshot,
} from './diskusage'
import { invoke } from '@tauri-apps/api/core'

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn().mockResolvedValue([]),
}))

describe('diskusage api', () => {
  beforeEach(() => {
    vi.mocked(invoke).mockClear()
  })

  it('lists drives and opens the treemap through Tauri commands', async () => {
    await listDrives()
    await openDiskUsage('D:\\Data')

    expect(invoke).toHaveBeenCalledWith('diskusage_list_drives')
    expect(invoke).toHaveBeenCalledWith('diskusage_open', { path: 'D:\\Data' })
  })

  it('lists and takes snapshots for a location', async () => {
    await listSnapshots('C:\\')
    await takeSnapshot('C:\\')

    expect(invoke).toHaveBeenCalledWith('diskusage_list_snapshots', { path: 'C:\\' })
    expect(invoke).toHaveBeenCalledWith('diskusage_snapshot', { path: 'C:\\' })
  })

  it('compares snapshots, showing only significant changes by default', async () => {
    await compareSnapshots('a.ledger.csv', 'b.ledger.csv')
    await compareSnapshots('a.ledger.csv', 'b.ledger.csv', true)

    expect(invoke).toHaveBeenNthCalledWith(1, 'diskusage_compare', {
      baseline: 'a.ledger.csv',
      current: 'b.ledger.csv',
      all: false,
    })
    expect(invoke).toHaveBeenNthCalledWith(2, 'diskusage_compare', {
      baseline: 'a.ledger.csv',
      current: 'b.ledger.csv',
      all: true,
    })
  })
})
