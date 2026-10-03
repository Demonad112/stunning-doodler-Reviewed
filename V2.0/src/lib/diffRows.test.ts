import { describe, expect, it } from 'vitest'
import type { DiffRow } from './compare'
import { matchesFilter, sizeDelta, statusLabel, visibleRows } from './diffRows'

function row(id: number, overrides: Partial<DiffRow> = {}): DiffRow {
  return {
    id,
    name: `row${String(id)}`,
    kind: 'file',
    left: { size: 10, files: 1, dirs: 0 },
    right: { size: 10, files: 1, dirs: 0 },
    status: 'same',
    missing: 0,
    missingBytes: 0,
    extra: 0,
    different: 0,
    cloud: false,
    error: null,
    hasChildren: false,
    ...overrides,
  }
}

const folder = row(1, { kind: 'dir', status: 'different', missing: 1, hasChildren: true })
const same = row(2)
const lost = row(3, { status: 'onlyLeft', right: null, missing: 1, missingBytes: 10 })
const extra = row(4, { status: 'onlyRight', left: null, extra: 1 })
const changed = row(5, { status: 'different', different: 1, right: { size: 4, files: 1, dirs: 0 } })

describe('visibleRows', () => {
  const children = new Map([[1, [lost, same]]])

  it('shows only top-level rows until a folder is expanded', () => {
    const rows = visibleRows([folder, extra], children, new Set(), 'all')
    expect(rows.map((visible) => visible.row.id)).toEqual([1, 4])
    expect(rows[0]?.expanded).toBe(false)
  })

  it('inserts loaded children after an expanded folder, one level deeper', () => {
    const rows = visibleRows([folder, extra], children, new Set([1]), 'all')
    expect(rows.map((visible) => [visible.row.id, visible.depth])).toEqual([
      [1, 0],
      [3, 1],
      [2, 1],
      [4, 0],
    ])
    expect(rows[0]?.expanded).toBe(true)
  })

  it('keeps an expanded folder whose children are still loading', () => {
    const rows = visibleRows([folder], new Map(), new Set([1]), 'all')
    expect(rows).toHaveLength(1)
  })

  it('filters at every level', () => {
    const missing = visibleRows([folder, extra, changed], children, new Set([1]), 'missing')
    expect(missing.map((visible) => visible.row.id)).toEqual([1, 3])
    const changes = visibleRows([folder, extra, changed], children, new Set([1]), 'changes')
    expect(changes.map((visible) => visible.row.id)).toEqual([1, 3, 4, 5])
  })
})

describe('row helpers', () => {
  it('matches filters', () => {
    expect(matchesFilter(same, 'all')).toBe(true)
    expect(matchesFilter(same, 'changes')).toBe(false)
    expect(matchesFilter(extra, 'missing')).toBe(false)
    expect(matchesFilter(row(9, { kind: 'dir', status: 'onlyLeft', missing: 0 }), 'missing')).toBe(
      true,
    )
  })

  it('computes destination minus source', () => {
    expect(sizeDelta(changed)).toBe(-6)
    expect(sizeDelta(lost)).toBe(-10)
    expect(sizeDelta(extra)).toBe(10)
  })

  it('labels statuses', () => {
    expect(statusLabel(same)).toEqual({ text: 'Match', tone: 'success' })
    expect(statusLabel(lost).tone).toBe('danger')
    expect(statusLabel(changed).text).toBe('Size differs')
    expect(statusLabel(folder)).toEqual({ text: '1 missing', tone: 'danger' })
    expect(statusLabel(row(7, { kind: 'dir', status: 'different' })).text).toBe('Differs')
  })
})
