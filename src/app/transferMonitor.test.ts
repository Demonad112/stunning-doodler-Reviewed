import { beforeEach, describe, expect, it } from 'vitest'
import {
  defaultTransferPreferences,
  formatDuration,
  groupNotCopied,
  joinTransferPath,
  loadTransferPreferences,
  parsePatterns,
  RateTracker,
  saveTransferPreferences,
  secondsLeft,
} from './transferMonitor'
import type { FailureReason, ItemResult } from '@/types/transfer'

function notCopied(path: string, reason: FailureReason, size = 10, recoverable = true): ItemResult {
  return {
    relativePath: path,
    kind: 'file',
    size,
    status: 'notCopied',
    reason,
    attempts: 1,
    inferred: false,
    recoverable,
    atMs: 0,
  }
}

describe('transfer monitor helpers', () => {
  beforeEach(() => {
    localStorage.clear()
  })

  it('groups not-copied files by reason, largest group first', () => {
    const groups = groupNotCopied([
      notCopied('b.txt', 'fileLocked', 5, false),
      notCopied('z/a.txt', 'accessDenied'),
      notCopied('a.txt', 'fileLocked', 7),
    ])

    expect(groups.map((group) => group.reason)).toEqual(['fileLocked', 'accessDenied'])
    expect(groups.at(0)?.items.map((item) => item.relativePath)).toEqual(['a.txt', 'b.txt'])
    expect(groups.at(0)?.bytes).toBe(12)
    expect(groups.at(0)?.recoverable).toBe(1)
  })

  it('splits filter patterns on semicolons, commas and new lines', () => {
    expect(parsePatterns(' *.tmp; node_modules ,\n*.bak;; ')).toEqual([
      '*.tmp',
      'node_modules',
      '*.bak',
    ])
    expect(parsePatterns('')).toEqual([])
  })

  it('joins a relative path onto a Windows root', () => {
    expect(joinTransferPath('E:\\Backup\\', 'docs/a.txt')).toBe('E:\\Backup\\docs\\a.txt')
    expect(joinTransferPath('E:\\Backup', '')).toBe('E:\\Backup')
  })

  it('works out the rate over the last few seconds and the time left', () => {
    const rate = new RateTracker(8000)

    rate.add(0, 0)
    expect(rate.bytesPerSecond()).toBeNull()
    rate.add(1000, 1000)
    rate.add(3000, 2000)
    expect(rate.bytesPerSecond()).toBe(1500)
    expect(secondsLeft(3000, 6000, 1500)).toBe(2)
    expect(secondsLeft(10, 5, 1500)).toBeNull()
    expect(secondsLeft(0, 5, null)).toBeNull()

    // A retry counts from zero again.
    rate.add(10, 3000)
    expect(rate.bytesPerSecond()).toBeNull()
  })

  it('formats durations for the summary strip', () => {
    expect(formatDuration(12)).toBe('12 s')
    expect(formatDuration(200)).toBe('3 min 20 s')
    expect(formatDuration(3900)).toBe('1 h 05 min')
  })

  it('remembers preferences and falls back to defaults', () => {
    expect(loadTransferPreferences()).toEqual(defaultTransferPreferences)

    saveTransferPreferences({ ...defaultTransferPreferences, verify: 'hash', exclude: '*.tmp' })

    expect(loadTransferPreferences()).toMatchObject({ verify: 'hash', exclude: '*.tmp' })
    localStorage.setItem('deepserver-transfer-preferences', '{broken')
    expect(loadTransferPreferences()).toEqual(defaultTransferPreferences)
  })
})
