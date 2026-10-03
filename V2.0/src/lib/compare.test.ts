import { beforeEach, describe, expect, it, vi } from 'vitest'

const core = vi.hoisted(() => ({
  invoke: vi.fn(),
  isTauri: vi.fn(() => true),
  channels: [] as { onmessage: (message: unknown) => void }[],
}))
const dialog = vi.hoisted(() => ({ open: vi.fn() }))

vi.mock('@tauri-apps/api/core', () => ({
  invoke: core.invoke,
  isTauri: core.isTauri,
  Channel: class {
    onmessage: (message: unknown) => void = () => undefined
    constructor() {
      core.channels.push(this)
    }
  },
}))
vi.mock('@tauri-apps/plugin-dialog', () => dialog)

import {
  cancelCompare,
  errorMessage,
  isCancelled,
  loadChildren,
  loadPaths,
  pickFolder,
  savePaths,
  startCompare,
} from './compare'

beforeEach(() => {
  vi.clearAllMocks()
  core.isTauri.mockReturnValue(true)
  core.channels.length = 0
  localStorage.clear()
})

describe('compare commands', () => {
  it('starts a compare and forwards progress events', async () => {
    core.invoke.mockResolvedValue({ rows: [] })
    const onProgress = vi.fn()
    await startCompare('C:\\a', 'D:\\b', onProgress)

    expect(core.invoke).toHaveBeenCalledWith('compare_start', {
      left: 'C:\\a',
      right: 'D:\\b',
      onEvent: core.channels[0],
    })
    const progress = { files: 1, dirs: 1, bytes: 5, current: 'C:\\a' }
    core.channels[0]?.onmessage({ kind: 'progress', left: progress, right: progress })
    expect(onProgress).toHaveBeenCalledWith({ left: progress, right: progress })
  })

  it('loads children and cancels', async () => {
    core.invoke.mockResolvedValue([])
    await loadChildren(7)
    expect(core.invoke).toHaveBeenCalledWith('compare_children', { id: 7 })
    await cancelCompare()
    expect(core.invoke).toHaveBeenCalledWith('compare_cancel')
  })

  it('recognises the cancel error and other messages', () => {
    expect(isCancelled('cancelled')).toBe(true)
    expect(isCancelled('Source: nope')).toBe(false)
    expect(errorMessage('Source: nope')).toBe('Source: nope')
    expect(errorMessage(new Error('boom'))).toBe('boom')
    expect(errorMessage(42)).toBe('Something went wrong.')
  })
})

describe('pickFolder', () => {
  it('returns the picked folder', async () => {
    dialog.open.mockResolvedValue('C:\\Data')
    await expect(pickFolder('Source')).resolves.toBe('C:\\Data')
    expect(dialog.open).toHaveBeenCalledWith(
      expect.objectContaining({ directory: true, multiple: false }),
    )
  })

  it('returns null when cancelled or outside the app', async () => {
    dialog.open.mockResolvedValue(null)
    await expect(pickFolder('Source')).resolves.toBeNull()
    core.isTauri.mockReturnValue(false)
    await expect(pickFolder('Source')).resolves.toBeNull()
    expect(dialog.open).toHaveBeenCalledTimes(1)
  })
})

describe('remembered paths', () => {
  it('round-trips through localStorage', () => {
    expect(loadPaths()).toEqual({ left: '', right: '' })
    savePaths({ left: 'C:\\a', right: 'D:\\b' })
    expect(loadPaths()).toEqual({ left: 'C:\\a', right: 'D:\\b' })
  })

  it('ignores corrupt values', () => {
    localStorage.setItem('deepserver2-compare-paths', '{nope')
    expect(loadPaths()).toEqual({ left: '', right: '' })
    localStorage.setItem('deepserver2-compare-paths', JSON.stringify({ left: 3, right: 'x' }))
    expect(loadPaths()).toEqual({ left: '', right: 'x' })
  })
})
