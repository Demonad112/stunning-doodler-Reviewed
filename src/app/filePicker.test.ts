import { describe, expect, it, vi } from 'vitest'
import { pickRecentPath } from './filePicker'

vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }))

describe('pickRecentPath', () => {
  it('opens a file picker and records the selected file path', async () => {
    const open = vi.fn().mockResolvedValue('C:/work/left.txt')

    await expect(
      pickRecentPath({
        kind: 'file',
        history: [],
        open,
      }),
    ).resolves.toEqual({
      selected: { path: 'C:/work/left.txt', kind: 'file' },
      history: [{ path: 'C:/work/left.txt', kind: 'file' }],
    })

    expect(open).toHaveBeenCalledWith({ directory: false })
  })

  it('opens a folder picker and records the selected folder path', async () => {
    const open = vi.fn().mockResolvedValue('C:/work/project')

    await expect(
      pickRecentPath({
        kind: 'folder',
        history: [],
        open,
      }),
    ).resolves.toMatchObject({
      selected: { path: 'C:/work/project', kind: 'folder' },
    })

    expect(open).toHaveBeenCalledWith({ directory: true })
  })

  it('keeps history unchanged when selection is cancelled', async () => {
    const history = [{ path: 'C:/work/existing.txt', kind: 'file' as const }]
    const open = vi.fn().mockResolvedValue(null)

    await expect(
      pickRecentPath({
        kind: 'file',
        history,
        open,
      }),
    ).resolves.toEqual({
      selected: null,
      history,
    })
  })
})

describe('pickNativePath', () => {
  it('treats anything but a non-empty path as cancelled', async () => {
    vi.stubGlobal('__TAURI_INTERNALS__', {})
    const { invoke } = await import('@tauri-apps/api/core')
    const { pickNativePath } = await import('./filePicker')
    const mockedInvoke = vi.mocked(invoke)

    mockedInvoke.mockResolvedValueOnce('D:/data')
    await expect(pickNativePath({ directory: true })).resolves.toBe('D:/data')
    mockedInvoke.mockResolvedValueOnce({})
    await expect(pickNativePath({ directory: true })).resolves.toBeNull()
    mockedInvoke.mockResolvedValueOnce('')
    await expect(pickNativePath({ directory: false })).resolves.toBeNull()
    mockedInvoke.mockRejectedValueOnce(new Error('no dialog'))
    await expect(pickNativePath({ directory: false })).resolves.toBeNull()

    vi.unstubAllGlobals()
  })
})
