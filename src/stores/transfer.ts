import { defineStore } from 'pinia'
import { computed, markRaw, ref, shallowRef } from 'vue'
import {
  cancelTransfer,
  copyToRecovery,
  exportTransferReport,
  finishWatch,
  listenTransferEvents,
  listTransferRuns,
  loadTransferRun,
  prepareTransfer,
  retryTransfer,
  startTransfer,
  startWatch,
} from '@/api/transfer'
import {
  groupNotCopied,
  loadTransferPreferences,
  parsePatterns,
  RateTracker,
  saveTransferPreferences,
  secondsLeft,
  type NotCopiedGroup,
} from '@/app/transferMonitor'
import { useJobsStore, type JobStatus } from '@/stores/jobs'
import type {
  FinishedEvent,
  ItemResult,
  ItemsEvent,
  PreflightReport,
  RecoveryResult,
  ReportExport,
  RunDetails,
  RunSummary,
  Selection,
  TransferProgress,
  TransferReportFormat,
  TransferSettings,
} from '@/types/transfer'

/** Where the screen is: filling in the setup, checking, ready, working, or showing a result. */
export type TransferStage = 'setup' | 'preparing' | 'prepared' | 'running' | 'watching' | 'done'

/** Copied files kept for the live feed. */
export const recentLimit = 200

function errorText(error: unknown): string {
  if (typeof error === 'object' && error !== null && 'message' in error) {
    return String(error.message)
  }

  return String(error)
}

function baseName(path: string): string {
  return (
    path
      .replace(/[\\/]+$/, '')
      .split(/[\\/]/)
      .pop() ?? path
  )
}

export const useTransferStore = defineStore('transfer', () => {
  const jobs = useJobsStore()
  const preferences = ref(loadTransferPreferences())
  const source = ref('')
  const destination = ref('')
  const stage = ref<TransferStage>('setup')
  const requestId = ref<string | null>(null)
  const summary = ref<RunSummary | null>(null)
  const preflight = ref<PreflightReport | null>(null)
  const progress = ref<TransferProgress | null>(null)
  const runFolder = ref('')
  const runs = ref<RunSummary[]>([])
  const errorMessage = ref('')
  const busyAction = ref<'recovery' | 'report' | null>(null)
  const lastRecovery = ref<RecoveryResult | null>(null)
  const lastReport = ref<ReportExport | null>(null)
  const bytesPerSecond = ref<number | null>(null)
  // A plain Map (can hold 100k rows); `notCopiedRevision` tells computeds it changed.
  const notCopied = markRaw(new Map<string, ItemResult>())
  const notCopiedRevision = ref(0)
  const recent = shallowRef<ItemResult[]>([])
  const rate = new RateTracker()
  let stopListening: Promise<() => void> | null = null

  const runId = computed(() => summary.value?.id ?? null)
  const working = computed(
    () => stage.value === 'preparing' || stage.value === 'running' || stage.value === 'watching',
  )
  const groups = computed<NotCopiedGroup[]>(() => {
    void notCopiedRevision.value

    return groupNotCopied(notCopied.values())
  })
  const notCopiedCount = computed(() => {
    void notCopiedRevision.value

    return notCopied.size
  })
  const etaSeconds = computed(() =>
    progress.value
      ? secondsLeft(progress.value.bytesDone, progress.value.bytesTotal, bytesPerSecond.value)
      : null,
  )

  function ensureListening(): void {
    stopListening ??= listenTransferEvents({
      progress: onProgress,
      items: onItems,
      finished: (event) => {
        void onFinished(event)
      },
    }).catch(() => () => undefined)
  }

  function isOurs(eventRunId: string): boolean {
    return eventRunId === runId.value || eventRunId === requestId.value
  }

  function onProgress(event: TransferProgress): void {
    if (!isOurs(event.runId)) {
      return
    }

    progress.value = event
    rate.add(event.bytesDone)
    bytesPerSecond.value = rate.bytesPerSecond()

    if (runId.value && (stage.value === 'running' || stage.value === 'watching')) {
      jobs.updateProgress(runId.value, {
        current: event.filesDone,
        total: event.filesTotal || null,
        message: event.current ?? '',
      })
    }
  }

  function applyItems(items: ItemResult[]): void {
    const arrived: ItemResult[] = []

    for (const item of items) {
      if (item.status === 'notCopied') {
        notCopied.set(item.relativePath, item)
        continue
      }

      notCopied.delete(item.relativePath)

      if (item.status === 'copied' || item.status === 'arrived') {
        arrived.push(item)
      }
    }

    if (arrived.length > 0) {
      recent.value = [...recent.value, ...arrived].slice(-recentLimit)
    }

    notCopiedRevision.value++
  }

  function onItems(event: ItemsEvent): void {
    if (isOurs(event.runId)) {
      applyItems(event.items)
    }
  }

  async function onFinished(event: FinishedEvent): Promise<void> {
    if (event.runId !== runId.value) {
      return
    }

    errorMessage.value = event.error ?? ''
    const finished = event.summary ?? summary.value
    let status: JobStatus = 'completed'

    if (event.error !== null) {
      status = 'failed'
    } else if (finished?.state === 'cancelled') {
      status = 'cancelled'
    }

    finishJob(status)

    if (finished) {
      await loadRun(finished.id)
    }

    await refreshRuns()
  }

  function settingsFromForm(): TransferSettings {
    return {
      source: source.value.trim(),
      destination: destination.value.trim(),
      mode: preferences.value.mode,
      include: parsePatterns(preferences.value.include),
      exclude: parsePatterns(preferences.value.exclude),
      verify: preferences.value.verify,
      conflict: preferences.value.conflict,
      includeHidden: preferences.value.includeHidden,
    }
  }

  function clearRun(): void {
    summary.value = null
    preflight.value = null
    progress.value = null
    runFolder.value = ''
    lastRecovery.value = null
    lastReport.value = null
    bytesPerSecond.value = null
    rate.reset()
    notCopied.clear()
    notCopiedRevision.value++
    recent.value = []
  }

  /** Back to the setup form, keeping the folders. */
  function newTransfer(): void {
    if (working.value) {
      return
    }

    clearRun()
    errorMessage.value = ''
    stage.value = 'setup'
  }

  /** Lists the source and runs the pre-flight check. */
  async function prepare(): Promise<void> {
    if (working.value) {
      return
    }

    const settings = settingsFromForm()

    if (!settings.source || !settings.destination) {
      return
    }

    ensureListening()
    saveTransferPreferences(preferences.value)
    clearRun()
    errorMessage.value = ''
    stage.value = 'preparing'
    requestId.value = crypto.randomUUID()

    try {
      const response = await prepareTransfer(requestId.value, settings)

      summary.value = response.summary
      preflight.value = response.preflight
      await loadRun(response.summary.id)
    } catch (error) {
      errorMessage.value = errorText(error)
      stage.value = 'setup'
    } finally {
      requestId.value = null
    }
  }

  /** Copy mode: starts copying. Watch mode: starts watching the destination. */
  async function start(): Promise<void> {
    const id = runId.value

    if (!id || stage.value !== 'prepared') {
      return
    }

    ensureListening()
    errorMessage.value = ''
    rate.reset()
    stage.value = preferences.value.mode === 'watch' ? 'watching' : 'running'
    beginJob(id)

    try {
      if (preferences.value.mode === 'watch') {
        await startWatch(id, {
          stableSeconds: preferences.value.stableSeconds,
          quietSeconds: preferences.value.quietSeconds,
        })
      } else {
        await startTransfer(id)
      }
    } catch (error) {
      errorMessage.value = errorText(error)
      stage.value = 'prepared'
      finishJob('failed')
    }
  }

  async function cancel(): Promise<void> {
    const id = requestId.value ?? runId.value

    if (id) {
      await cancelTransfer(id)
    }
  }

  /** Watch mode: stop watching and check what has not arrived. */
  async function finish(): Promise<void> {
    if (runId.value && stage.value === 'watching') {
      await finishWatch(runId.value)
    }
  }

  async function retry(selection: Selection): Promise<void> {
    const id = runId.value

    if (!id || working.value) {
      return
    }

    ensureListening()
    errorMessage.value = ''
    rate.reset()
    stage.value = 'running'
    beginJob(id)

    try {
      await retryTransfer(id, selection)
    } catch (error) {
      errorMessage.value = errorText(error)
      await loadRun(id)
      finishJob('failed')
    }
  }

  async function recover(selection: Selection): Promise<void> {
    const id = runId.value

    if (!id || working.value || busyAction.value) {
      return
    }

    busyAction.value = 'recovery'
    errorMessage.value = ''

    try {
      lastRecovery.value = await copyToRecovery(
        id,
        selection,
        preferences.value.recoveryFolder.trim() || null,
      )
    } catch (error) {
      errorMessage.value = errorText(error)
    } finally {
      busyAction.value = null
    }
  }

  async function exportReport(format: TransferReportFormat): Promise<void> {
    const id = runId.value

    if (!id || busyAction.value) {
      return
    }

    busyAction.value = 'report'
    errorMessage.value = ''

    try {
      const exported = await exportTransferReport(id, format, true)

      if (exported) {
        lastReport.value = exported
      }
    } catch (error) {
      errorMessage.value = errorText(error)
    } finally {
      busyAction.value = null
    }
  }

  function stageFor(details: RunDetails): TransferStage {
    if (details.running) {
      return details.summary.state === 'watching' ? 'watching' : 'running'
    }

    return details.summary.state === 'prepared' ? 'prepared' : 'done'
  }

  /** Opens a stored run (also used to refresh the one on screen). */
  async function loadRun(id: string): Promise<void> {
    try {
      const details = await loadTransferRun(id)

      summary.value = details.summary
      preflight.value = details.preflight
      runFolder.value = details.folder
      source.value = details.summary.settings.source
      destination.value = details.summary.settings.destination
      preferences.value = { ...preferences.value, mode: details.summary.settings.mode }
      notCopied.clear()
      details.notCopied.forEach((item) => notCopied.set(item.relativePath, item))
      notCopiedRevision.value++
      recent.value = details.recent
      stage.value = stageFor(details)
      ensureListening()
    } catch (error) {
      errorMessage.value = errorText(error)
    }
  }

  async function refreshRuns(): Promise<void> {
    try {
      runs.value = await listTransferRuns()
    } catch {
      runs.value = []
    }
  }

  function beginJob(id: string): void {
    jobs.addJob({
      id,
      title: `${baseName(source.value)} → ${baseName(destination.value)}`,
      status: 'running',
      progress: { current: 0, total: null, message: '' },
      cancellable: true,
    })
  }

  function finishJob(status: JobStatus): void {
    const id = runId.value
    const job = jobs.jobs.find((item) => item.id === id)

    if (job) {
      jobs.addJob({ ...job, status })
    }
  }

  return {
    preferences,
    source,
    destination,
    stage,
    summary,
    preflight,
    progress,
    runFolder,
    runs,
    errorMessage,
    busyAction,
    lastRecovery,
    lastReport,
    bytesPerSecond,
    etaSeconds,
    recent,
    groups,
    notCopiedCount,
    runId,
    working,
    ensureListening,
    newTransfer,
    prepare,
    start,
    cancel,
    finish,
    retry,
    recover,
    exportReport,
    loadRun,
    refreshRuns,
    // Exposed for tests.
    applyItems,
    onProgress,
    onFinished,
  }
})
