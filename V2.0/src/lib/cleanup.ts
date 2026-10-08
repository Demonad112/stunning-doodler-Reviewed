import { Channel, invoke, isTauri } from '@tauri-apps/api/core'
import { reactive, ref } from 'vue'
import { errorMessage, type EntryKind, type ScanProgress } from './compare'
import { currentJob } from './job'

export interface Drive {
  /** "C:\". */
  path: string
  label: string
  total: number
  free: number
  removable: boolean
}

export interface CleanupRow {
  id: number
  name: string
  kind: EntryKind
  size: number
  files: number
  dirs: number
  cloudFiles: number
  /** Online-only bytes below this row, left out of `size`. */
  cloudBytes: number
  errors: number
  error: string | null
  modifiedMs: number | null
  hasChildren: boolean
}

export interface FileRow {
  id: number
  name: string
  folder: string
  size: number
  modifiedMs: number | null
}

export interface TypeRow {
  /** Lowercase, no dot; empty when the files have no extension. */
  extension: string
  size: number
  files: number
}

export interface Overview {
  root: CleanupRow
  largestFiles: FileRow[]
  /** Every extension, largest first. */
  fileTypes: TypeRow[]
}

interface ScanResult {
  path: string
  overview: Overview
  rows: CleanupRow[]
  elapsedMs: number
  reportId: string | null
}

/**
 * The scan and where the user is in it. Kept outside the page so a long drive scan survives
 * moving to another page and back.
 */
export const cleanupJob = reactive({
  running: false,
  /** The folder being scanned, or the one the result is for. */
  path: '',
  progress: null as ScanProgress | null,
  startedAt: 0,
  elapsedMs: 0,
  finishedAt: null as Date | null,
  overview: null as Overview | null,
  /** Folders from the scanned root down to the one shown; the root has id 0. */
  trail: [] as CleanupRow[],
  /** What is directly inside the last folder of the trail. */
  rows: [] as CleanupRow[],
  error: '',
  notice: '',
  /** The scan's saved report on the Reports page. */
  reportId: null as string | null,
})

/** This PC's drives, for the drive tiles on Home and Disk Cleanup. */
export const drives = ref<Drive[]>([])

export async function refreshDrives(): Promise<void> {
  if (!isTauri()) {
    return
  }
  try {
    drives.value = await invoke<Drive[]>('cleanup_drives')
  } catch {
    // No tiles; a path can still be typed.
  }
}

/** Scans a drive or folder; progress arrives about every 100 ms. */
export async function runScan(path: string): Promise<void> {
  if (cleanupJob.running) {
    return
  }
  Object.assign(cleanupJob, {
    running: true,
    path,
    progress: null,
    startedAt: Date.now(),
    overview: null,
    trail: [],
    rows: [],
    error: '',
    notice: '',
    reportId: null,
  })
  const onProgress = new Channel<ScanProgress>()
  onProgress.onmessage = (progress) => {
    cleanupJob.progress = progress
  }
  try {
    const result = await invoke<ScanResult>('cleanup_scan', {
      path,
      job: currentJob(),
      onProgress,
    })
    Object.assign(cleanupJob, {
      path: result.path,
      overview: result.overview,
      trail: [result.overview.root],
      rows: result.rows,
      elapsedMs: result.elapsedMs,
      reportId: result.reportId,
      finishedAt: new Date(),
    })
  } catch (err) {
    if (err === 'cancelled') {
      cleanupJob.notice = 'Scan cancelled.'
    } else {
      cleanupJob.error = errorMessage(err)
    }
  } finally {
    cleanupJob.running = false
  }
}

export function cancelScan(): Promise<void> {
  return invoke('cleanup_cancel')
}

/** Shows what is inside `row`, a folder in the shown one. */
export async function openFolder(row: CleanupRow): Promise<void> {
  cleanupJob.rows = await invoke<CleanupRow[]>('cleanup_children', { id: row.id })
  cleanupJob.trail = [...cleanupJob.trail, row]
}

/** Goes back up to the folder at `index` in the trail. */
export async function openTrail(index: number): Promise<void> {
  const folder = cleanupJob.trail[index]
  if (!folder) {
    return
  }
  cleanupJob.rows = await invoke<CleanupRow[]>('cleanup_children', { id: folder.id })
  cleanupJob.trail = cleanupJob.trail.slice(0, index + 1)
}

/**
 * Recycles (or deletes for good) one item, then refreshes the totals, the lists and the shown
 * folder. Only the trail's names and ids are used, so its other sizes may be stale.
 */
export async function removeItem(id: number, permanent: boolean): Promise<void> {
  const overview = await invoke<Overview>('cleanup_delete', { id, permanent })
  cleanupJob.overview = overview
  cleanupJob.trail = [overview.root, ...cleanupJob.trail.slice(1)]
  const shown = cleanupJob.trail.at(-1)
  if (shown) {
    cleanupJob.rows = await invoke<CleanupRow[]>('cleanup_children', { id: shown.id })
  }
}

export function itemPath(id: number): Promise<string> {
  return invoke<string>('cleanup_path', { id })
}

/** Opens Explorer with the item selected. */
export function revealItem(id: number): Promise<void> {
  return invoke('cleanup_reveal', { id })
}

/** "C:", "c:\", "C:/" all match the drive "C:\". */
export function driveFor(path: string, drives: Drive[]): Drive | undefined {
  const key = (value: string): string =>
    value
      .trim()
      .replace(/[\\/]+$/, '')
      .toLowerCase()
  return drives.find((drive) => key(drive.path) === key(path))
}

/**
 * How far a drive scan has got, from the bytes found so far against the drive's used space.
 * Capped at 99 until the scan ends; `null` when the total isn't known.
 */
export function scanPercent(bytes: number, drive: Drive | undefined): number | null {
  const used = drive ? drive.total - drive.free : 0
  if (used <= 0) {
    return null
  }
  return Math.min(99, Math.floor((bytes / used) * 100))
}

export type QuickId =
  'userTemp' | 'windowsTemp' | 'browserCache' | 'updateCache' | 'recycleBin' | 'oldInstallers'

/** A quick cleanup card. */
export interface QuickWin {
  id: QuickId
  title: string
  description: string
  needsAdmin: boolean
  /** What it would free now. */
  size: number
  files: number
}

export interface QuickList {
  /** Running as administrator, so the system cleanups can run. */
  elevated: boolean
  items: QuickWin[]
}

export interface CleanedItem {
  title: string
  freed: number
  removedFiles: number
  skippedFiles: number
  error: string | null
}

export interface QuickResult {
  report: { items: CleanedItem[] }
  reportId: string | null
}

/** Measures every quick cleanup; takes a few seconds on a big profile. */
export function loadQuickWins(): Promise<QuickList> {
  return invoke<QuickList>('cleanup_quick_list')
}

/** Runs the chosen quick cleanups and saves a report of what they freed. */
export function runQuickClean(ids: QuickId[]): Promise<QuickResult> {
  return invoke<QuickResult>('cleanup_quick_clean', { ids, job: currentJob() })
}

/** Restarts DeepServer as administrator; Windows asks first. */
export function restartAsAdmin(): Promise<void> {
  return invoke('app_restart_admin')
}

/** Which cards can be ticked: something to free, and admin rights when needed. */
export function canClean(item: QuickWin, elevated: boolean): boolean {
  return item.size > 0 && (!item.needsAdmin || elevated)
}
