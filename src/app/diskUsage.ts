export const diskUsageLocationsStorageKey = 'deepserver-disk-usage-locations'

const maxRecentLocations = 10

function locationKey(path: string): string {
  return path
    .trim()
    .replace(/[\\/]+$/, '')
    .replace(/\//g, '\\')
    .toLowerCase()
}

/** Newest first; Windows paths differing only in case or a trailing slash count as one. */
export function addRecentLocation(recent: string[], path: string): string[] {
  const trimmed = path.trim()

  if (!trimmed) {
    return recent
  }

  const key = locationKey(trimmed)

  return [trimmed, ...recent.filter((item) => locationKey(item) !== key)].slice(
    0,
    maxRecentLocations,
  )
}

export function loadRecentLocations(storage: Pick<Storage, 'getItem'> = localStorage): string[] {
  try {
    const parsed: unknown = JSON.parse(storage.getItem(diskUsageLocationsStorageKey) ?? '[]')

    return Array.isArray(parsed)
      ? parsed.filter((item): item is string => typeof item === 'string')
      : []
  } catch {
    return []
  }
}

export function saveRecentLocations(
  recent: string[],
  storage: Pick<Storage, 'setItem'> = localStorage,
): void {
  storage.setItem(diskUsageLocationsStorageKey, JSON.stringify(recent))
}

const byteUnits = ['B', 'KB', 'MB', 'GB', 'TB', 'PB']

export function formatBytes(bytes: number | null): string {
  if (bytes === null || !Number.isFinite(bytes)) {
    return ''
  }

  let value = Math.abs(bytes)
  let unit = 0

  while (value >= 1024 && unit < byteUnits.length - 1) {
    value /= 1024
    unit += 1
  }

  const text = unit === 0 ? String(value) : value.toFixed(1)

  return `${bytes < 0 ? '-' : ''}${text} ${byteUnits[unit]}`
}

export function formatByteChange(bytes: number): string {
  return bytes > 0 ? `+${formatBytes(bytes)}` : formatBytes(bytes)
}

/** `YYYY-MM-DD HH:MM:SS` (UTC, from the snapshot file name) shown in the user's local time. */
export function snapshotLocalLabel(takenAtUtc: string): string {
  const match = /^(\d{4})-(\d{2})-(\d{2}) (\d{2}):(\d{2}):(\d{2})$/.exec(takenAtUtc)

  if (!match) {
    return takenAtUtc
  }

  const [year, month, day, hour, minute, second] = match.slice(1).map(Number)

  return new Date(Date.UTC(year, month - 1, day, hour, minute, second)).toLocaleString()
}

/** Absolute path of a compare row's folder (`.` is the scanned location itself). */
export function folderPathInLocation(location: string, folder: string): string {
  const root = location.replace(/[\\/]+$/, '')

  return folder === '.' ? location.replace(/([^:])[\\/]+$/, '$1') : `${root}\\${folder}`
}

export function isNetworkPath(path: string): boolean {
  return /^(\\\\|\/\/)[^\\/]/.test(path.trim())
}
