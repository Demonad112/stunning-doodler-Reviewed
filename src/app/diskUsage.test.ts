import { describe, expect, it } from 'vitest'
import {
  addRecentLocation,
  diskUsageLocationsStorageKey,
  folderPathInLocation,
  formatByteChange,
  formatBytes,
  isNetworkPath,
  loadRecentLocations,
  saveRecentLocations,
  snapshotLocalLabel,
} from './diskUsage'

function memoryStorage(initial: Record<string, string> = {}): Storage {
  const values = new Map(Object.entries(initial))

  return {
    get length() {
      return values.size
    },
    clear: () => values.clear(),
    getItem: (key) => values.get(key) ?? null,
    key: (index) => [...values.keys()][index] ?? null,
    removeItem: (key) => void values.delete(key),
    setItem: (key, value) => void values.set(key, value),
  }
}

describe('disk usage helpers', () => {
  it('keeps recent locations newest first, without case-only duplicates, capped at ten', () => {
    let recent: string[] = []

    for (let index = 0; index < 12; index += 1) {
      recent = addRecentLocation(recent, `D:\\Data\\${String(index)}`)
    }
    recent = addRecentLocation(recent, 'd:\\data\\5\\')

    expect(recent).toHaveLength(10)
    expect(recent[0]).toBe('d:\\data\\5\\')
    expect(recent.filter((path) => path.toLowerCase().startsWith('d:\\data\\5'))).toHaveLength(1)
    expect(addRecentLocation(recent, '   ')).toBe(recent)
  })

  it('loads and saves recent locations, ignoring unreadable storage', () => {
    const storage = memoryStorage()

    saveRecentLocations(['\\\\server\\share', 'C:\\'], storage)
    expect(loadRecentLocations(storage)).toEqual(['\\\\server\\share', 'C:\\'])

    storage.setItem(diskUsageLocationsStorageKey, '{not json')
    expect(loadRecentLocations(storage)).toEqual([])
    storage.setItem(diskUsageLocationsStorageKey, JSON.stringify(['ok', 3, null]))
    expect(loadRecentLocations(storage)).toEqual(['ok'])
  })

  it('formats sizes and signed size changes', () => {
    expect(formatBytes(null)).toBe('')
    expect(formatBytes(512)).toBe('512 B')
    expect(formatBytes(1536)).toBe('1.5 KB')
    expect(formatBytes(5 * 1024 ** 3)).toBe('5.0 GB')
    expect(formatByteChange(2 * 1024 ** 2)).toBe('+2.0 MB')
    expect(formatByteChange(-4096)).toBe('-4.0 KB')
    expect(formatByteChange(0)).toBe('0 B')
  })

  it('labels snapshots in local time from their UTC stamp', () => {
    const label = snapshotLocalLabel('2026-09-21 14:13:20')
    const expected = new Date(Date.UTC(2026, 8, 21, 14, 13, 20)).toLocaleString()

    expect(label).toBe(expected)
    expect(snapshotLocalLabel('garbage')).toBe('garbage')
  })

  it('builds absolute folder paths from compare rows', () => {
    expect(folderPathInLocation('D:\\Data', '.')).toBe('D:\\Data')
    expect(folderPathInLocation('D:\\Data\\', 'a\\b')).toBe('D:\\Data\\a\\b')
    expect(folderPathInLocation('C:\\', 'Windows')).toBe('C:\\Windows')
  })

  it('recognizes network shares', () => {
    expect(isNetworkPath('\\\\server\\share')).toBe(true)
    expect(isNetworkPath('//server/share')).toBe(true)
    expect(isNetworkPath('C:\\')).toBe(false)
  })
})
