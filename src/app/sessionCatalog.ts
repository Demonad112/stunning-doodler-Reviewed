import type { SessionType } from '@/types/session'

export type SessionPriority = 'P0' | 'P1' | 'P2' | 'P3'

/** Launchable UI exists when implemented; maturity is honesty about depth. */
export type SessionMaturity = 'ready' | 'partial' | 'limited'

export interface SessionCatalogEntry {
  type: SessionType
  title: string
  titleKey: string
  summary: string
  summaryKey: string
  priority: SessionPriority
  /** Has a routable session UI users can open. */
  implemented: boolean
  /** ready = solid core; partial = real ops with visible gaps; limited = thin/shell. */
  maturity: SessionMaturity
  route?: string
}

export const sessionCatalog: SessionCatalogEntry[] = [
  {
    type: 'text-compare',
    title: 'Text Compare',
    titleKey: 'ui.textCompare',
    summary: 'Compare two text files and see what changed',
    summaryKey: 'session.summary.textCompare',
    priority: 'P0',
    implemented: true,
    maturity: 'ready',
    route: '/compare/text',
  },
  {
    type: 'folder-compare',
    title: 'Folder Compare',
    titleKey: 'ui.folderCompare',
    summary: 'Compare two folders and find added, missing, or changed files',
    summaryKey: 'session.summary.folderCompare',
    priority: 'P0',
    implemented: true,
    maturity: 'ready',
    route: '/compare/folder',
  },
  {
    type: 'folder-sync',
    title: 'Folder Sync',
    titleKey: 'ui.folderSync',
    summary: 'Preview, then copy or delete files to make folders match',
    summaryKey: 'session.summary.folderSync',
    priority: 'P1',
    implemented: true,
    maturity: 'ready',
    route: '/sync/folder',
  },
  {
    type: 'text-merge',
    title: 'Text Merge',
    titleKey: 'ui.textMerge',
    summary: 'Combine left, right, and a base file into one result',
    summaryKey: 'session.summary.textMerge',
    priority: 'P1',
    implemented: true,
    maturity: 'ready',
    route: '/merge/text',
  },
  {
    type: 'table-compare',
    title: 'Table Compare',
    titleKey: 'ui.tableCompare',
    summary: 'Compare tables (CSV, Excel, or HTML)',
    summaryKey: 'session.summary.tableCompare',
    priority: 'P1',
    implemented: true,
    maturity: 'ready',
    route: '/compare/table',
  },
  {
    type: 'hex-compare',
    title: 'Hex Compare',
    titleKey: 'ui.hexCompare',
    summary: 'Compare two files byte by byte',
    summaryKey: 'session.summary.hexCompare',
    priority: 'P1',
    implemented: true,
    maturity: 'ready',
    route: '/compare/hex',
  },
  {
    type: 'picture-compare',
    title: 'Picture Compare',
    titleKey: 'ui.pictureCompare',
    summary: 'Compare two images and see which pixels differ',
    summaryKey: 'session.summary.pictureCompare',
    priority: 'P1',
    implemented: true,
    maturity: 'ready',
    route: '/compare/picture',
  },
  {
    type: 'folder-merge',
    title: 'Folder Merge',
    titleKey: 'ui.folderMerge',
    summary: 'Merge three folders into one output folder',
    summaryKey: 'session.summary.folderMerge',
    priority: 'P2',
    implemented: true,
    maturity: 'ready',
    route: '/merge/folder',
  },
  {
    type: 'text-edit',
    title: 'Text Edit',
    titleKey: 'ui.textEdit',
    summary: 'Open and edit one text file',
    summaryKey: 'session.summary.textEdit',
    priority: 'P2',
    implemented: true,
    maturity: 'ready',
    route: '/edit/text',
  },
  {
    type: 'text-patch',
    title: 'Text Patch',
    titleKey: 'ui.textPatch',
    summary: 'Review and apply a patch file',
    summaryKey: 'session.summary.textPatch',
    priority: 'P2',
    implemented: true,
    maturity: 'ready',
    route: '/patch/text',
  },
  {
    type: 'clipboard-compare',
    title: 'Clipboard Compare',
    titleKey: 'ui.clipboardCompare',
    summary: 'Compare two pieces of copied text',
    summaryKey: 'session.summary.clipboardCompare',
    priority: 'P2',
    implemented: true,
    maturity: 'ready',
    route: '/compare/clipboard',
  },
  {
    type: 'registry-compare',
    title: 'Registry Compare',
    titleKey: 'ui.registryCompare',
    summary: 'Compare .reg exports, offline hive files, and live Windows keys with apply/write',
    summaryKey: 'session.summary.registryCompare',
    priority: 'P3',
    implemented: true,
    maturity: 'ready',
    route: '/compare/registry',
  },
  {
    type: 'media-compare',
    title: 'Media Compare',
    titleKey: 'ui.mediaCompare',
    summary: 'Compare media metadata with local audio/video preview',
    summaryKey: 'session.summary.mediaCompare',
    priority: 'P3',
    implemented: true,
    maturity: 'ready',
    route: '/compare/media',
  },
  {
    type: 'version-compare',
    title: 'Version Compare',
    titleKey: 'ui.versionCompare',
    summary: 'Compare version info inside program files',
    summaryKey: 'session.summary.versionCompare',
    priority: 'P3',
    implemented: true,
    maturity: 'ready',
    route: '/compare/version',
  },
  {
    type: 'archive-compare',
    title: 'Archive Compare',
    titleKey: 'ui.archiveCompare',
    summary: 'Opens Folder Compare for ZIP/TAR/7z sides with extract-on-copy into a folder',
    summaryKey: 'session.summary.archiveCompare',
    priority: 'P2',
    implemented: true,
    maturity: 'ready',
    route: '/compare/folder',
  },
  {
    type: 'script',
    title: 'Script',
    titleKey: 'ui.script',
    summary:
      'Run automation scripts with LOAD/COMPARE/REPORT, file ops, IF/ELSE, CALL/INCLUDE, samples, run log, and Stop',
    summaryKey: 'session.summary.script',
    priority: 'P2',
    implemented: true,
    maturity: 'ready',
    route: '/reports/scripts',
  },
  {
    type: 'disk-usage',
    title: 'Disk Usage',
    titleKey: 'ui.diskUsage',
    summary: 'See what takes up space on a drive or share, and what changed between snapshots',
    summaryKey: 'session.summary.diskUsage',
    priority: 'P1',
    implemented: true,
    maturity: 'ready',
    route: '/disk/usage',
  },
  {
    type: 'transfer-monitor',
    title: 'Transfer Monitor',
    titleKey: 'ui.transferMonitor',
    summary:
      'Copy a folder with verification, or watch another program copy it, and see every file that did not make it',
    summaryKey: 'session.summary.transferMonitor',
    priority: 'P1',
    implemented: true,
    maturity: 'ready',
    route: '/transfer',
  },
]

export const sessionPriorities: SessionPriority[] = ['P0', 'P1', 'P2', 'P3']
