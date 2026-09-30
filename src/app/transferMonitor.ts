import type {
  ConflictPolicy,
  FailureReason,
  ItemResult,
  TransferMode,
  VerifyLevel,
} from '@/types/transfer'

export interface NotCopiedGroup {
  reason: FailureReason
  items: ItemResult[]
  bytes: number
  /** Rows that "Copy to recovery folder" can handle. */
  recoverable: number
  inferred: boolean
}

/** Not-copied rows grouped by reason, largest group first, paths sorted within a group. */
export function groupNotCopied(items: Iterable<ItemResult>): NotCopiedGroup[] {
  const groups = new Map<FailureReason, NotCopiedGroup>()

  for (const item of items) {
    const reason = item.reason ?? 'unknown'
    let group = groups.get(reason)

    if (!group) {
      group = { reason, items: [], bytes: 0, recoverable: 0, inferred: false }
      groups.set(reason, group)
    }

    group.items.push(item)
    group.bytes += item.size
    group.recoverable += item.recoverable ? 1 : 0
    group.inferred ||= item.inferred
  }

  return [...groups.values()]
    .map((group) => ({
      ...group,
      items: group.items.sort((left, right) => left.relativePath.localeCompare(right.relativePath)),
    }))
    .sort(
      (left, right) =>
        right.items.length - left.items.length || left.reason.localeCompare(right.reason),
    )
}

/** Splits `*.tmp; node_modules` style input into patterns. */
export function parsePatterns(text: string): string[] {
  return text
    .split(/[;,\n]+/)
    .map((pattern) => pattern.trim())
    .filter(Boolean)
}

/** `root` + a `/`-separated relative path, with Windows separators. */
export function joinTransferPath(root: string, relativePath: string): string {
  const base = root.replace(/[\\/]+$/, '')

  return relativePath ? `${base}\\${relativePath.replaceAll('/', '\\')}` : base
}

interface RateSample {
  at: number
  bytes: number
}

/** Transfer rate over the last few seconds, for the summary strip. */
export class RateTracker {
  private samples: RateSample[] = []

  constructor(private readonly windowMs = 8000) {}

  reset(): void {
    this.samples = []
  }

  add(bytes: number, at = Date.now()): void {
    const last = this.samples.at(-1)

    if (last && bytes < last.bytes) {
      // A retry starts counting again.
      this.samples = []
    }

    this.samples.push({ at, bytes })

    while (this.samples.length > 2 && at - (this.samples.at(0)?.at ?? at) > this.windowMs) {
      this.samples.shift()
    }
  }

  /** Bytes per second, or null until there are two samples a moment apart. */
  bytesPerSecond(): number | null {
    const first = this.samples.at(0)
    const last = this.samples.at(-1)

    if (!first || !last || last.at - first.at < 500) {
      return null
    }

    return ((last.bytes - first.bytes) * 1000) / (last.at - first.at)
  }
}

/** Seconds left at `rate`, or null when unknown. */
export function secondsLeft(done: number, total: number, rate: number | null): number | null {
  if (!rate || rate <= 0 || total <= done) {
    return null
  }

  return Math.ceil((total - done) / rate)
}

/** `1 h 05 min`, `3 min 20 s`, `12 s`. */
export function formatDuration(seconds: number): string {
  const whole = Math.max(0, Math.round(seconds))
  const hours = Math.floor(whole / 3600)
  const minutes = Math.floor((whole % 3600) / 60)
  const rest = whole % 60

  if (hours > 0) {
    return `${String(hours)} h ${String(minutes).padStart(2, '0')} min`
  }

  if (minutes > 0) {
    return `${String(minutes)} min ${String(rest).padStart(2, '0')} s`
  }

  return `${String(rest)} s`
}

export interface TransferPreferences {
  mode: TransferMode
  verify: VerifyLevel
  conflict: ConflictPolicy
  include: string
  exclude: string
  includeHidden: boolean
  stableSeconds: number
  quietSeconds: number
  /** Empty: beside the destination. */
  recoveryFolder: string
}

export const defaultTransferPreferences: TransferPreferences = {
  mode: 'copy',
  verify: 'sizeAndTime',
  conflict: 'skip',
  include: '',
  exclude: '',
  includeHidden: true,
  stableSeconds: 5,
  quietSeconds: 0,
  recoveryFolder: '',
}

const preferencesKey = 'deepserver-transfer-preferences'

export function loadTransferPreferences(): TransferPreferences {
  try {
    const stored = localStorage.getItem(preferencesKey)

    if (!stored) {
      return { ...defaultTransferPreferences }
    }

    const parsed = JSON.parse(stored) as Partial<TransferPreferences>

    return { ...defaultTransferPreferences, ...parsed }
  } catch {
    return { ...defaultTransferPreferences }
  }
}

export function saveTransferPreferences(preferences: TransferPreferences): void {
  try {
    localStorage.setItem(preferencesKey, JSON.stringify(preferences))
  } catch {
    // Storage full or blocked: the defaults come back next time.
  }
}
