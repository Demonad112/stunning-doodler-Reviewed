import { createPinia, setActivePinia } from 'pinia'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import * as api from '@/api/transfer'
import { useJobsStore } from '@/stores/jobs'
import { useTransferStore } from '@/stores/transfer'
import type { ItemResult, RunDetails, RunSummary } from '@/types/transfer'

vi.mock('@/api/transfer', () => ({
  cancelTransfer: vi.fn().mockResolvedValue(true),
  copyToRecovery: vi.fn(),
  exportTransferReport: vi.fn(),
  finishWatch: vi.fn().mockResolvedValue(true),
  listenTransferEvents: vi.fn().mockResolvedValue(() => undefined),
  listTransferRuns: vi.fn().mockResolvedValue([]),
  loadTransferRun: vi.fn(),
  prepareTransfer: vi.fn(),
  pruneTransferRuns: vi.fn().mockResolvedValue({ removed: 3, freedBytes: 2048 }),
  retryTransfer: vi.fn().mockResolvedValue(undefined),
  startTransfer: vi.fn().mockResolvedValue(undefined),
  startWatch: vi.fn().mockResolvedValue(undefined),
}))

function summary(state: RunSummary['state'], notCopied = 0): RunSummary {
  return {
    id: 'run-1',
    settings: {
      source: 'D:\\Data',
      destination: 'E:\\Backup',
      mode: 'copy',
      include: [],
      exclude: [],
      verify: 'sizeAndTime',
      conflict: 'skip',
      includeHidden: true,
    },
    state,
    createdAtMs: 1,
    startedAtMs: null,
    finishedAtMs: null,
    machine: 'PC',
    user: 'me',
    totals: {
      plannedFiles: 3,
      plannedBytes: 30,
      folders: 0,
      excluded: 0,
      copied: 0,
      copiedBytes: 0,
      skippedIdentical: 0,
      notCopied,
      notCopiedBytes: 0,
    },
    error: null,
  }
}

function item(path: string, status: ItemResult['status']): ItemResult {
  return {
    relativePath: path,
    kind: 'file',
    size: 10,
    status,
    reason: status === 'notCopied' ? 'fileLocked' : undefined,
    attempts: 1,
    inferred: false,
    recoverable: false,
    atMs: 5,
  }
}

function details(state: RunSummary['state'], notCopied: ItemResult[] = []): RunDetails {
  return {
    summary: summary(state, notCopied.length),
    preflight: null,
    notCopied,
    recent: [],
    running: false,
    folder: 'C:\\runs\\run-1',
  }
}

describe('transfer store', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    localStorage.clear()
    vi.clearAllMocks()
  })

  it('checks first, then starts the copy and tracks it as a job', async () => {
    vi.mocked(api.prepareTransfer).mockResolvedValue({
      summary: summary('prepared'),
      preflight: {
        runId: 'run-1',
        files: 3,
        folders: 0,
        bytes: 30,
        excluded: 0,
        alreadyThere: 0,
        bytesNeeded: 30,
        freeBytes: 100,
        volume: 'E:\\',
        enoughSpace: true,
        writable: true,
        writeError: null,
        scanErrors: 0,
        blocked: 0,
        issueCount: 0,
        issues: [],
      },
    })
    vi.mocked(api.loadTransferRun).mockResolvedValue(details('prepared'))
    const store = useTransferStore()

    store.source = 'D:\\Data'
    store.destination = 'E:\\Backup'
    store.preferences.exclude = '*.tmp; node_modules'

    await store.prepare()

    expect(api.prepareTransfer).toHaveBeenCalledWith(
      expect.any(String),
      expect.objectContaining({ exclude: ['*.tmp', 'node_modules'], mode: 'copy' }),
    )
    expect(store.stage).toBe('prepared')

    await store.start()

    expect(api.startTransfer).toHaveBeenCalledWith('run-1')
    expect(store.stage).toBe('running')
    expect(useJobsStore().runningJobs.map((job) => job.id)).toEqual(['run-1'])
  })

  it('does not check without both folders', async () => {
    const store = useTransferStore()

    store.source = 'D:\\Data'

    await store.prepare()

    expect(api.prepareTransfer).not.toHaveBeenCalled()
    expect(store.stage).toBe('setup')
  })

  it('shows a failed check and stays on the setup', async () => {
    vi.mocked(api.prepareTransfer).mockRejectedValue({ message: 'The source is not a folder' })
    const store = useTransferStore()

    store.source = 'D:\\Missing'
    store.destination = 'E:\\Backup'

    await store.prepare()

    expect(store.errorMessage).toBe('The source is not a folder')
    expect(store.stage).toBe('setup')
  })

  it('moves a file out of Not copied once a retry copies it', async () => {
    vi.mocked(api.loadTransferRun).mockResolvedValue(details('running'))
    const store = useTransferStore()

    await store.loadRun('run-1')

    store.applyItems([item('a.txt', 'notCopied'), item('b.txt', 'notCopied')])
    expect(store.notCopiedCount).toBe(2)
    expect(store.groups.map((group) => group.reason)).toEqual(['fileLocked'])

    store.applyItems([item('a.txt', 'copied')])
    expect(store.notCopiedCount).toBe(1)
    expect(store.recent.map((row) => row.relativePath)).toEqual(['a.txt'])
  })

  it('reloads the run and marks the job done when the copy finishes', async () => {
    vi.mocked(api.loadTransferRun).mockResolvedValue(details('prepared'))
    const store = useTransferStore()

    await store.loadRun('run-1')
    await store.start()
    vi.mocked(api.loadTransferRun).mockResolvedValue(
      details('completed', [item('locked.txt', 'notCopied')]),
    )

    await store.onFinished({ runId: 'run-1', summary: summary('completed', 1), error: null })

    expect(store.stage).toBe('done')
    expect(store.notCopiedCount).toBe(1)
    expect(useJobsStore().jobs.at(0)?.status).toBe('completed')
    expect(api.listTransferRuns).toHaveBeenCalled()
  })

  it('ignores events from other runs', async () => {
    vi.mocked(api.loadTransferRun).mockResolvedValue(details('running'))
    const store = useTransferStore()

    await store.loadRun('run-1')

    store.onProgress({
      runId: 'other',
      phase: 'copying',
      filesDone: 1,
      filesTotal: 2,
      bytesDone: 1,
      bytesTotal: 2,
      copied: 1,
      skipped: 0,
      notCopied: 0,
      current: null,
    })
    await store.onFinished({ runId: 'other', summary: null, error: 'boom' })

    expect(store.progress).toBeNull()
    expect(store.errorMessage).toBe('')
  })

  it('retries a group and copies to the chosen recovery folder', async () => {
    vi.mocked(api.loadTransferRun).mockResolvedValue(details('completed'))
    vi.mocked(api.copyToRecovery).mockResolvedValue({
      folder: 'F:\\Rescue\\run-1',
      listFile: 'F:\\Rescue\\run-1\\NOT-COPIED.csv',
      copied: 1,
      skipped: [],
    })
    const store = useTransferStore()

    await store.loadRun('run-1')
    store.preferences.recoveryFolder = 'F:\\Rescue'

    await store.recover({ reason: 'diskFull' })
    expect(api.copyToRecovery).toHaveBeenCalledWith('run-1', { reason: 'diskFull' }, 'F:\\Rescue')
    expect(store.lastRecovery?.copied).toBe(1)

    await store.retry({ reason: 'fileLocked' })
    expect(api.retryTransfer).toHaveBeenCalledWith('run-1', { reason: 'fileLocked' })
    expect(store.stage).toBe('running')
  })

  it('shows recovery progress from events and cancels the recovery copy', async () => {
    vi.mocked(api.loadTransferRun).mockResolvedValue(details('completed'))
    let resolveRecovery: (value: Awaited<ReturnType<typeof api.copyToRecovery>>) => void = () =>
      undefined

    vi.mocked(api.copyToRecovery).mockReturnValue(
      new Promise((resolve) => {
        resolveRecovery = resolve
      }),
    )
    const store = useTransferStore()

    await store.loadRun('run-1')
    const handlers = vi.mocked(api.listenTransferEvents).mock.calls[0][0]
    const recovering = store.recover({})

    expect(store.recoveryProgress).toEqual({ done: 0, total: 0 })
    handlers.recovery?.({ runId: 'other', done: 9, total: 9 })
    handlers.recovery?.({ runId: 'run-1', done: 2, total: 5 })
    expect(store.recoveryProgress).toEqual({ done: 2, total: 5 })

    await store.cancelRecovery()
    expect(api.cancelTransfer).toHaveBeenCalledWith('run-1')

    resolveRecovery({ folder: 'E:\\Backup_NotCopied', listFile: '', copied: 2, skipped: [] })
    await recovering
    expect(store.recoveryProgress).toBeNull()
    expect(store.busyAction).toBeNull()
  })

  it('cleans up old runs and reloads the list', async () => {
    const store = useTransferStore()

    await store.pruneRuns()

    expect(api.pruneTransferRuns).toHaveBeenCalledWith(null)
    expect(api.listTransferRuns).toHaveBeenCalled()
    expect(store.lastPrune).toEqual({ removed: 3, freedBytes: 2048 })
  })
})
