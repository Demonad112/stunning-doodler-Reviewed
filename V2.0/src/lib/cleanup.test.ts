import { beforeEach, describe, expect, it, vi } from 'vitest'

const core = vi.hoisted(() => ({
  invoke: vi.fn(),
  channels: [] as { onmessage: (message: unknown) => void }[],
}))

vi.mock('@tauri-apps/api/core', () => ({
  invoke: core.invoke,
  isTauri: () => true,
  Channel: class {
    onmessage: (message: unknown) => void = () => undefined
    constructor() {
      core.channels.push(this)
    }
  },
}))
vi.mock('@tauri-apps/plugin-dialog', () => ({ open: vi.fn() }))

import {
  cleanupJob,
  driveFor,
  openFolder,
  openTrail,
  removeItem,
  runScan,
  scanPercent,
  canClean,
  runQuickClean,
  type QuickWin,
  type CleanupRow,
  type Drive,
  type Overview,
} from './cleanup'

function row(id: number, name: string, size: number): CleanupRow {
  return {
    id,
    name,
    kind: 'dir',
    size,
    files: 1,
    dirs: 0,
    cloudFiles: 0,
    cloudBytes: 0,
    errors: 0,
    error: null,
    modifiedMs: null,
    hasChildren: true,
  }
}

function overview(size: number): Overview {
  return { root: row(0, 'C:\\', size), largestFiles: [], fileTypes: [] }
}

const drive: Drive = { path: 'C:\\', label: 'Local Disk', total: 1000, free: 600, removable: false }

beforeEach(() => {
  vi.clearAllMocks()
  core.channels.length = 0
})

describe('cleanup scan', () => {
  it('streams progress and opens the scanned root', async () => {
    core.invoke.mockImplementationOnce(() => {
      core.channels[0]?.onmessage({ files: 3, dirs: 1, bytes: 50, current: 'C:\\x' })
      expect(cleanupJob.running).toBe(true)
      expect(cleanupJob.progress?.bytes).toBe(50)
      return Promise.resolve({
        path: 'C:\\',
        overview: overview(100),
        rows: [row(1, 'Users', 100)],
        elapsedMs: 5,
      })
    })
    await runScan('C:\\')

    expect(core.invoke).toHaveBeenCalledWith('cleanup_scan', {
      job: { client: '', ticket: '', technician: '' },
      path: 'C:\\',
      onProgress: core.channels[0],
    })
    expect(cleanupJob.running).toBe(false)
    expect(cleanupJob.trail.map((folder) => folder.id)).toEqual([0])
    expect(cleanupJob.rows.map((item) => item.name)).toEqual(['Users'])
  })

  it('stays quiet on cancel and shows other errors', async () => {
    core.invoke.mockRejectedValueOnce('cancelled')
    await runScan('C:\\')
    expect(cleanupJob.notice).toBe('Scan cancelled.')
    expect(cleanupJob.error).toBe('')

    core.invoke.mockRejectedValueOnce('D:\\ does not exist.')
    await runScan('D:\\')
    expect(cleanupJob.error).toBe('D:\\ does not exist.')
    expect(cleanupJob.overview).toBeNull()
  })

  it('walks into folders and back up the trail', async () => {
    cleanupJob.trail = [row(0, 'C:\\', 100)]
    core.invoke.mockResolvedValueOnce([row(2, 'Sam', 60)])
    await openFolder(row(1, 'Users', 100))
    expect(core.invoke).toHaveBeenLastCalledWith('cleanup_children', { id: 1 })
    expect(cleanupJob.trail.map((folder) => folder.name)).toEqual(['C:\\', 'Users'])

    core.invoke.mockResolvedValueOnce([row(1, 'Users', 100)])
    await openTrail(0)
    expect(cleanupJob.trail).toHaveLength(1)
    expect(cleanupJob.rows[0]?.name).toBe('Users')
  })

  it('refreshes totals and the shown folder after a delete', async () => {
    cleanupJob.trail = [row(0, 'C:\\', 100), row(1, 'Users', 100)]
    core.invoke.mockResolvedValueOnce(overview(40)).mockResolvedValueOnce([row(3, 'Public', 40)])
    await removeItem(2, false)

    expect(core.invoke).toHaveBeenNthCalledWith(1, 'cleanup_delete', { id: 2, permanent: false })
    expect(core.invoke).toHaveBeenNthCalledWith(2, 'cleanup_children', { id: 1 })
    expect(cleanupJob.overview?.root.size).toBe(40)
    expect(cleanupJob.trail.map((folder) => folder.name)).toEqual(['C:\\', 'Users'])
    expect(cleanupJob.rows.map((item) => item.name)).toEqual(['Public'])
  })
})

describe('quick cleanups', () => {
  const card: QuickWin = {
    id: 'windowsTemp',
    title: 'Windows temporary files',
    description: '',
    needsAdmin: true,
    size: 10,
    files: 1,
  }

  it('can be ticked only with something to free and the rights it needs', () => {
    expect(canClean(card, true)).toBe(true)
    expect(canClean(card, false)).toBe(false)
    expect(canClean({ ...card, needsAdmin: false }, false)).toBe(true)
    expect(canClean({ ...card, size: 0 }, true)).toBe(false)
  })

  it('sends the chosen cleanups with the job details', async () => {
    core.invoke.mockResolvedValueOnce({ report: { items: [] }, reportId: 'cleanup-1' })
    await runQuickClean(['userTemp', 'recycleBin'])
    expect(core.invoke).toHaveBeenCalledWith('cleanup_quick_clean', {
      ids: ['userTemp', 'recycleBin'],
      job: { client: '', ticket: '', technician: '' },
    })
  })
})

describe('drive helpers', () => {
  it('matches a typed path to its drive', () => {
    expect(driveFor('c:', [drive])).toBe(drive)
    expect(driveFor('C:/', [drive])).toBe(drive)
    expect(driveFor('C:\\Users', [drive])).toBeUndefined()
  })

  it('turns bytes found into a percentage of the used space', () => {
    expect(scanPercent(200, drive)).toBe(50)
    expect(scanPercent(5000, drive)).toBe(99)
    expect(scanPercent(200, undefined)).toBeNull()
  })
})
