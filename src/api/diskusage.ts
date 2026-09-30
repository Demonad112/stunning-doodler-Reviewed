import { invoke } from '@tauri-apps/api/core'

export type DriveKind = 'fixed' | 'removable' | 'network' | 'optical' | 'ramDisk' | 'unknown'

export interface DriveInfo {
  /** `C:\` */
  root: string
  label: string
  kind: DriveKind
  /** `null` when the drive isn't ready (empty card reader, disconnected share). */
  totalBytes: number | null
  freeBytes: number | null
}

export interface SnapshotInfo {
  path: string
  fileName: string
  /** `YYYY-MM-DD HH:MM:SS` in UTC. */
  takenAtUtc: string
  sizeBytes: number
}

export type ChangeKind = 'added' | 'removed' | 'grown' | 'shrunk' | 'filesChanged' | 'unchanged'

export interface ChangeRow {
  /** Folder relative to the scanned location; `.` is the location itself. */
  folder: string
  change: ChangeKind
  baselineSize: number | null
  currentSize: number | null
  sizeChange: number
  baselineFiles: number | null
  currentFiles: number | null
  filesChange: number
}

export interface Comparison {
  rows: ChangeRow[]
  /** Set when the result is valid but may mislead, e.g. the exclusion filters changed between scans. */
  warning: string | null
}

export function listDrives(): Promise<DriveInfo[]> {
  return invoke<DriveInfo[]>('diskusage_list_drives')
}

/** Opens the disk-usage treemap window on a folder or drive. */
export function openDiskUsage(path: string): Promise<void> {
  return invoke('diskusage_open', { path })
}

/** Stored snapshots of a location, newest first. */
export function listSnapshots(path: string): Promise<SnapshotInfo[]> {
  return invoke<SnapshotInfo[]>('diskusage_list_snapshots', { path })
}

/** Scans a folder or drive in the background and stores a snapshot (minutes on a large drive). */
export function takeSnapshot(path: string): Promise<SnapshotInfo> {
  return invoke<SnapshotInfo>('diskusage_snapshot', { path })
}

/** Folder changes between two snapshots; `all` includes small changes, not only significant ones. */
export function compareSnapshots(
  baseline: string,
  current: string,
  all = false,
): Promise<Comparison> {
  return invoke<Comparison>('diskusage_compare', { baseline, current, all })
}
