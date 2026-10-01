import { invoke } from '@tauri-apps/api/core'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import {
  cancelTransfer,
  copyToRecovery,
  exportTransferReport,
  finishWatch,
  listenTransferEvents,
  listTransferRuns,
  loadTransferRun,
  prepareTransfer,
  pruneTransferRuns,
  retryTransfer,
  startTransfer,
  startWatch,
} from './transfer'
import type { TransferSettings } from '@/types/transfer'

vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn().mockResolvedValue(null) }))

const settings: TransferSettings = {
  source: 'D:\\Data',
  destination: 'E:\\Backup',
  mode: 'copy',
  include: [],
  exclude: ['*.tmp'],
  verify: 'hash',
  conflict: 'skip',
  includeHidden: true,
}

describe('transfer api', () => {
  beforeEach(() => {
    vi.mocked(invoke).mockClear()
  })

  it('passes each command its arguments', async () => {
    await prepareTransfer('req-1', settings)
    await startTransfer('run-1')
    await startWatch('run-1', { stableSeconds: 5, quietSeconds: 0 })
    await finishWatch('run-1')
    await cancelTransfer('run-1')
    await retryTransfer('run-1', { reason: 'fileLocked' })
    await copyToRecovery('run-1', { paths: ['a.txt'] }, null)
    await listTransferRuns()
    await loadTransferRun('run-1')
    await exportTransferReport('run-1', 'html', true)
    await pruneTransferRuns('run-1')

    expect(vi.mocked(invoke).mock.calls).toEqual([
      ['transfer_prepare', { requestId: 'req-1', settings }],
      ['transfer_start', { runId: 'run-1' }],
      ['transfer_watch', { runId: 'run-1', request: { stableSeconds: 5, quietSeconds: 0 } }],
      ['transfer_watch_finish', { runId: 'run-1' }],
      ['transfer_cancel', { runId: 'run-1' }],
      ['transfer_retry', { runId: 'run-1', selection: { reason: 'fileLocked' } }],
      [
        'transfer_copy_to_recovery',
        { runId: 'run-1', selection: { paths: ['a.txt'] }, folder: null },
      ],
      ['transfer_list_runs'],
      ['transfer_load_run', { runId: 'run-1' }],
      ['transfer_export_report', { runId: 'run-1', format: 'html', choosePath: true }],
      ['transfer_prune_runs', { keep: 'run-1' }],
    ])
  })

  it('does not listen for events outside the desktop app', async () => {
    const stop = await listenTransferEvents({
      progress: vi.fn(),
      items: vi.fn(),
      finished: vi.fn(),
    })

    expect(() => {
      stop()
    }).not.toThrow()
  })
})
