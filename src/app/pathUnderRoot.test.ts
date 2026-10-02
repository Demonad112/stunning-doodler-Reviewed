import { describe, expect, it } from 'vitest'
import { isPathUnderRoot } from './pathUnderRoot'

describe('isPathUnderRoot', () => {
  it('matches across slash style, case and a trailing separator', () => {
    expect(isPathUnderRoot('C:/Data/Right/t.csv', 'c:\\data\\right\\')).toBe(true)
    expect(isPathUnderRoot('C:/data/right', 'C:\\data\\right')).toBe(true)
  })

  it('does not match a sibling that shares a prefix, or an empty root', () => {
    expect(isPathUnderRoot('C:/data/right2/t.csv', 'C:\\data\\right')).toBe(false)
    expect(isPathUnderRoot('C:/data/right/t.csv', '')).toBe(false)
  })
})
