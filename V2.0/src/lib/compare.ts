import { Channel, invoke, isTauri } from '@tauri-apps/api/core'
import { open } from '@tauri-apps/plugin-dialog'

export type EntryKind = 'file' | 'dir' | 'link'
export type DiffStatus = 'same' | 'different' | 'onlyLeft' | 'onlyRight' | 'kindMismatch'

export interface DiffSide {
  size: number
  files: number
  dirs: number
}

/** One row of a compare. Left is the source, right the destination. */
export interface DiffRow {
  id: number
  name: string
  kind: EntryKind
  left: DiffSide | null
  right: DiffSide | null
  status: DiffStatus
  /** Source files at or below this row that the destination lacks. */
  missing: number
  missingBytes: number
  /** Destination files at or below this row that the source lacks. */
  extra: number
  /** Files at or below this row whose size differs. */
  different: number
  cloud: boolean
  error: string | null
  hasChildren: boolean
}

export interface DiffSummary {
  left: DiffSide
  right: DiffSide
  missing: number
  missingBytes: number
  extra: number
  different: number
  leftErrors: number
  rightErrors: number
  leftCloud: number
  rightCloud: number
}

export interface ScanProgress {
  files: number
  dirs: number
  bytes: number
  current: string
}

export interface CompareProgress {
  left: ScanProgress
  right: ScanProgress
}

export interface CompareResult {
  summary: DiffSummary
  /** Rows directly below the two compared folders. */
  rows: DiffRow[]
  elapsedMs: number
}

type CompareEvent = { kind: 'progress' } & CompareProgress

/** The error text the backend returns for a cancelled compare. */
export const CANCELLED = 'cancelled'

export function isCancelled(error: unknown): boolean {
  return error === CANCELLED
}

export function errorMessage(error: unknown): string {
  if (typeof error === 'string') {
    return error
  }
  return error instanceof Error ? error.message : 'Something went wrong.'
}

/** Scans both folders and compares them. Progress arrives about every 100 ms. */
export async function startCompare(
  left: string,
  right: string,
  onProgress: (progress: CompareProgress) => void,
): Promise<CompareResult> {
  const onEvent = new Channel<CompareEvent>()
  onEvent.onmessage = (event) => {
    onProgress({ left: event.left, right: event.right })
  }
  return invoke<CompareResult>('compare_start', { left, right, onEvent })
}

/** Rows below one folder of the last compare. */
export function loadChildren(id: number): Promise<DiffRow[]> {
  return invoke<DiffRow[]>('compare_children', { id })
}

export function cancelCompare(): Promise<void> {
  return invoke('compare_cancel')
}

/** Native folder picker. `null` when cancelled or outside the desktop app. */
export async function pickFolder(title: string, defaultPath?: string): Promise<string | null> {
  if (!isTauri()) {
    return null
  }
  const picked = await open({
    directory: true,
    multiple: false,
    title,
    defaultPath: defaultPath || undefined,
  })
  return typeof picked === 'string' && picked !== '' ? picked : null
}

export interface ComparePaths {
  left: string
  right: string
}

const pathsKey = 'deepserver2-compare-paths'

export function loadPaths(): ComparePaths {
  try {
    const saved: unknown = JSON.parse(localStorage.getItem(pathsKey) ?? 'null')
    if (saved && typeof saved === 'object') {
      const { left, right } = saved as Record<string, unknown>
      return {
        left: typeof left === 'string' ? left : '',
        right: typeof right === 'string' ? right : '',
      }
    }
  } catch {
    // Unreadable storage: start empty.
  }
  return { left: '', right: '' }
}

export function savePaths(paths: ComparePaths): void {
  try {
    localStorage.setItem(pathsKey, JSON.stringify({ left: paths.left, right: paths.right }))
  } catch {
    // Storage full or blocked: the paths just aren't remembered.
  }
}
