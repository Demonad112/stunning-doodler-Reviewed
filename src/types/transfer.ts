/** Mirrors `transfer-core` (Rust, camelCase JSON). */

export type TransferMode = 'copy' | 'watch'
export type VerifyLevel = 'sizeAndTime' | 'hash'
export type ConflictPolicy = 'skip' | 'overwriteIfNewer' | 'overwrite'
export type RunState = 'prepared' | 'running' | 'watching' | 'completed' | 'cancelled' | 'failed'
export type ItemKind = 'file' | 'folder'
export type ItemStatus = 'copied' | 'arrived' | 'skippedIdentical' | 'notCopied'
export type TransferPhase = 'scanning' | 'copying' | 'watching' | 'finishing' | 'done'
export type Side = 'source' | 'destination'

export const failureReasons = [
  'accessDenied',
  'fileLocked',
  'diskFull',
  'pathTooLong',
  'invalidName',
  'nameCollision',
  'networkLost',
  'deviceRemoved',
  'mediaError',
  'sourceVanished',
  'targetExistsDifferent',
  'verifyMismatch',
  'missing',
  'incomplete',
  'cancelled',
  'unknown',
] as const

export type FailureReason = (typeof failureReasons)[number]

export interface TransferSettings {
  source: string
  destination: string
  mode: TransferMode
  include: string[]
  exclude: string[]
  verify: VerifyLevel
  conflict: ConflictPolicy
  includeHidden: boolean
}

export interface RunTotals {
  plannedFiles: number
  plannedBytes: number
  folders: number
  excluded: number
  copied: number
  copiedBytes: number
  skippedIdentical: number
  notCopied: number
  notCopiedBytes: number
}

export interface RunSummary {
  id: string
  settings: TransferSettings
  state: RunState
  createdAtMs: number
  startedAtMs: number | null
  finishedAtMs: number | null
  machine: string
  user: string
  totals: RunTotals
  error: string | null
}

export interface ItemResult {
  /** Relative to the source and destination, with `/` separators. */
  relativePath: string
  kind: ItemKind
  size: number
  status: ItemStatus
  reason?: FailureReason
  side?: Side
  osCode?: number
  message?: string
  sourceHash?: string
  destHash?: string
  attempts: number
  /** Watch mode: the reason is a best guess. */
  inferred: boolean
  /** "Copy to recovery folder" can work (the source is readable). */
  recoverable: boolean
  atMs: number
}

export interface TransferProgress {
  runId: string
  phase: TransferPhase
  filesDone: number
  filesTotal: number
  bytesDone: number
  bytesTotal: number
  copied: number
  skipped: number
  notCopied: number
  current: string | null
}

export type PreflightSeverity = 'blocker' | 'warning'

export interface PreflightIssue {
  relativePath: string
  reason: FailureReason
  severity: PreflightSeverity
  detail?: string
}

export interface PreflightReport {
  runId: string
  files: number
  folders: number
  bytes: number
  excluded: number
  alreadyThere: number
  bytesNeeded: number
  freeBytes: number | null
  volume: string | null
  enoughSpace: boolean
  writable: boolean
  writeError: string | null
  scanErrors: number
  blocked: number
  issueCount: number
  issues: PreflightIssue[]
}

export interface PrepareResponse {
  summary: RunSummary
  preflight: PreflightReport
}

export interface RunDetails {
  summary: RunSummary
  preflight: PreflightReport | null
  notCopied: ItemResult[]
  recent: ItemResult[]
  running: boolean
  folder: string
}

/** No paths and no reason selects every not-copied row. */
export interface Selection {
  paths?: string[]
  reason?: FailureReason
}

export interface RecoveryResult {
  folder: string
  listFile: string
  copied: number
  skipped: { relativePath: string; message: string }[]
}

export type TransferReportFormat = 'html' | 'csv' | 'json'

export interface ReportExport {
  format: TransferReportFormat
  outputPath: string
  bytesWritten: number
}

export interface WatchRequest {
  stableSeconds: number
  /** 0 waits for Finish. */
  quietSeconds: number
}

export interface ItemsEvent {
  runId: string
  items: ItemResult[]
}

/** `transfer://recovery`: files copied to the recovery folder so far. */
export interface RecoveryEvent {
  runId: string
  done: number
  total: number
}

export interface PruneResult {
  removed: number
  freedBytes: number
}

export interface FinishedEvent {
  runId: string
  summary: RunSummary | null
  error: string | null
}
