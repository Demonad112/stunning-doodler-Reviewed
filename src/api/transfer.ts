import { invoke } from '@tauri-apps/api/core'
import { isTauriRuntime } from '@/app/desktopDrop'
import type {
  FinishedEvent,
  ItemsEvent,
  PrepareResponse,
  PruneResult,
  RecoveryEvent,
  RecoveryResult,
  ReportExport,
  RunDetails,
  RunSummary,
  Selection,
  TransferProgress,
  TransferReportFormat,
  TransferSettings,
  WatchRequest,
} from '@/types/transfer'

export const transferEvents = {
  progress: 'transfer://progress',
  items: 'transfer://items',
  finished: 'transfer://finished',
  recovery: 'transfer://recovery',
} as const

/** Lists the source and runs the pre-flight check. Cancel it with `cancelTransfer(requestId)`. */
export function prepareTransfer(
  requestId: string,
  settings: TransferSettings,
): Promise<PrepareResponse> {
  return invoke<PrepareResponse>('transfer_prepare', { requestId, settings })
}

/** Starts copying a prepared run; progress and the result arrive as events. */
export function startTransfer(runId: string): Promise<void> {
  return invoke('transfer_start', { runId })
}

export function startWatch(runId: string, request: WatchRequest): Promise<void> {
  return invoke('transfer_watch', { runId, request })
}

/** Watch mode: stop watching and check every file that has not arrived. */
export function finishWatch(runId: string): Promise<boolean> {
  return invoke<boolean>('transfer_watch_finish', { runId })
}

export function cancelTransfer(runId: string): Promise<boolean> {
  return invoke<boolean>('transfer_cancel', { runId })
}

export function retryTransfer(runId: string, selection: Selection): Promise<void> {
  return invoke('transfer_retry', { runId, selection })
}

/** `folder` null: `<destination>_NotCopied\<run id>`. */
export function copyToRecovery(
  runId: string,
  selection: Selection,
  folder: string | null,
): Promise<RecoveryResult> {
  return invoke<RecoveryResult>('transfer_copy_to_recovery', { runId, selection, folder })
}

/**
 * Removes runs older than 90 days and all but the newest 200. Running ones and `keep` (the run
 * shown in the window) are kept.
 */
export function pruneTransferRuns(keep: string | null): Promise<PruneResult> {
  return invoke<PruneResult>('transfer_prune_runs', { keep })
}

export function listTransferRuns(): Promise<RunSummary[]> {
  return invoke<RunSummary[]>('transfer_list_runs')
}

export function loadTransferRun(runId: string): Promise<RunDetails> {
  return invoke<RunDetails>('transfer_load_run', { runId })
}

/** With `choosePath` a save dialog asks where; null when it is cancelled. */
export function exportTransferReport(
  runId: string,
  format: TransferReportFormat,
  choosePath: boolean,
): Promise<ReportExport | null> {
  return invoke<ReportExport | null>('transfer_export_report', { runId, format, choosePath })
}

export interface TransferEventHandlers {
  progress: (progress: TransferProgress) => void
  items: (event: ItemsEvent) => void
  finished: (event: FinishedEvent) => void
  recovery?: (event: RecoveryEvent) => void
}

/** Subscribes to the transfer events; resolves to a function that unsubscribes. */
export async function listenTransferEvents(handlers: TransferEventHandlers): Promise<() => void> {
  if (!isTauriRuntime()) {
    return () => undefined
  }

  const { listen } = await import('@tauri-apps/api/event')
  const stoppers = await Promise.all([
    listen<TransferProgress>(transferEvents.progress, (event) => {
      handlers.progress(event.payload)
    }),
    listen<ItemsEvent>(transferEvents.items, (event) => {
      handlers.items(event.payload)
    }),
    listen<FinishedEvent>(transferEvents.finished, (event) => {
      handlers.finished(event.payload)
    }),
    listen<RecoveryEvent>(transferEvents.recovery, (event) => {
      handlers.recovery?.(event.payload)
    }),
  ])

  return () => {
    stoppers.forEach((stop) => {
      stop()
    })
  }
}
