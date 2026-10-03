import type { DiffRow } from './compare'
import { formatCount } from './format'

export type CompareFilter = 'all' | 'changes' | 'missing'

export interface VisibleRow {
  row: DiffRow
  depth: number
  expanded: boolean
}

/** Whether a row (and so its folder) belongs in the table under `filter`. */
export function matchesFilter(row: DiffRow, filter: CompareFilter): boolean {
  switch (filter) {
    case 'all':
      return true
    case 'changes':
      return row.status !== 'same'
    case 'missing':
      return row.status === 'onlyLeft' || row.missing > 0
  }
}

/**
 * Flattens the loaded part of the tree into table rows, depth first. A folder's rows appear only
 * when it is expanded and its children are loaded.
 */
export function visibleRows(
  roots: readonly DiffRow[],
  children: ReadonlyMap<number, readonly DiffRow[]>,
  expanded: ReadonlySet<number>,
  filter: CompareFilter,
): VisibleRow[] {
  const out: VisibleRow[] = []
  const visit = (rows: readonly DiffRow[], depth: number): void => {
    for (const row of rows) {
      if (!matchesFilter(row, filter)) {
        continue
      }
      const isExpanded = row.hasChildren && expanded.has(row.id)
      out.push({ row, depth, expanded: isExpanded })
      const below = isExpanded ? children.get(row.id) : undefined
      if (below) {
        visit(below, depth + 1)
      }
    }
  }
  visit(roots, 0)
  return out
}

/** Destination minus source; a side that doesn't exist counts as 0. */
export function sizeDelta(row: DiffRow): number {
  return (row.right?.size ?? 0) - (row.left?.size ?? 0)
}

export type Tone = 'success' | 'warning' | 'danger' | 'neutral'

/** Short status text and its colour for the status column. */
export function statusLabel(row: DiffRow): { text: string; tone: Tone } {
  switch (row.status) {
    case 'same':
      return { text: 'Match', tone: 'success' }
    case 'onlyLeft':
      return { text: 'Missing', tone: 'danger' }
    case 'onlyRight':
      return { text: 'Only at destination', tone: 'neutral' }
    case 'kindMismatch':
      return { text: 'Type differs', tone: 'warning' }
    case 'different':
      if (row.kind !== 'dir') {
        return { text: 'Size differs', tone: 'warning' }
      }
      return row.missing > 0
        ? { text: `${formatCount(row.missing)} missing`, tone: 'danger' }
        : { text: 'Differs', tone: 'warning' }
  }
}
