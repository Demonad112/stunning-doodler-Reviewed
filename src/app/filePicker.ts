import { invoke } from '@tauri-apps/api/core'
import { isTauriRuntime } from './desktopDrop'
import { addRecentPath, type RecentPath, type RecentPathKind } from './pathHistory'

export interface FilePickerOpenOptions {
  directory: boolean
}

export type FilePickerOpen = (options: FilePickerOpenOptions) => Promise<string | string[] | null>

export interface PickRecentPathOptions {
  kind: RecentPathKind
  history: RecentPath[]
  open: FilePickerOpen
}

export interface PickRecentPathResult {
  selected: RecentPath | null
  history: RecentPath[]
}

export async function pickRecentPath(
  options: PickRecentPathOptions,
): Promise<PickRecentPathResult> {
  const selected = await options.open({ directory: options.kind === 'folder' })

  if (!selected) {
    return {
      selected: null,
      history: options.history,
    }
  }

  const path = Array.isArray(selected) ? selected[0] : selected

  if (!path) {
    return {
      selected: null,
      history: options.history,
    }
  }

  const recentPath = { path, kind: options.kind }

  return {
    selected: recentPath,
    history: addRecentPath(options.history, recentPath),
  }
}

/** Opens a native file/folder picker via Tauri `pick_path` (rfd). Returns null when cancelled / non-Tauri. */
export async function pickNativePath(options: FilePickerOpenOptions): Promise<string | null> {
  if (!isTauriRuntime()) {
    return null
  }

  try {
    const selected = await invoke<string | null>('pick_path', { directory: options.directory })

    return selected
  } catch {
    return null
  }
}

export const tauriFilePickerOpen: FilePickerOpen = (options) => pickNativePath(options)

export interface DetectedExecutables {
  /** This running DeepServer program (what Git, SVN and the Explorer menu should call). */
  deepServer: string | null
  vscode: string | null
}

/** Programs found on this PC, for path suggestions in Settings. Empty outside the desktop app. */
export async function detectExecutables(): Promise<DetectedExecutables> {
  const none: DetectedExecutables = { deepServer: null, vscode: null }

  if (!isTauriRuntime()) {
    return none
  }

  try {
    return await invoke<DetectedExecutables>('detect_executables')
  } catch {
    return none
  }
}
