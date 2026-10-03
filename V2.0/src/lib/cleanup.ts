import { Channel, invoke, isTauri } from '@tauri-apps/api/core'
import { reactive, ref } from 'vue'
import { errorMessage, type EntryKind, type ScanProgress } from './compare'

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
  })
  const onProgress = new Channel<ScanProgress>()
  onProgress.onmessage = (progress) => {
    cleanupJob.progress = progress
  }
  try {
    const result = await invoke<ScanResult>('cleanup_scan', { path, onProgress })
    Object.assign(cleanupJob, {
      path: result.path,
      overview: result.overview,
      trail: [result.overview.root],
      rows: result.rows,
      elapsedMs: result.elapsedMs,
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
