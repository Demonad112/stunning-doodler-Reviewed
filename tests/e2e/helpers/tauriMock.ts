import type { Page } from '@playwright/test'
import { ensureAnimationFrames } from './ensureAnimationFrames'

export interface InvokeCall {
  command: string
  args: unknown
}

type EventCallback = (event: { event: string; id: number; payload: unknown }) => void

/** What `transfer_load_run` answers; specs change it with `setTransferRun`. */
export interface MockTransferRun {
  state: string
  notCopied: unknown[]
  recent: unknown[]
}

type WindowWithInvokeLog = Window & {
  __OPEN_DIFF_INVOKE_LOG__?: InvokeCall[]
  __TAURI_INTERNALS__?: {
    invoke: (command: string, args?: unknown) => Promise<unknown>
    transformCallback: (callback: EventCallback, once?: boolean) => number
  }
  __TAURI_EVENT_PLUGIN_INTERNALS__?: {
    unregisterListener: (event: string, id: number) => void
  }
  /** Delivers a Tauri event to the page's `listen()` handlers. */
  __E2E_EMIT__?: (event: string, payload: unknown) => void
  __E2E_TRANSFER_RUN__?: MockTransferRun
}

export async function installTauriInvokeMock(page: Page): Promise<void> {
  await ensureAnimationFrames(page)
  await page.addInitScript(() => {
    const tauriWindow = window as WindowWithInvokeLog
    const log: InvokeCall[] = []
    const callbacks = new Map<number, EventCallback>()
    const listeners = new Map<string, number[]>()
    let nextCallbackId = 1
    const transferSettings = (args: unknown): Record<string, unknown> =>
      args && typeof args === 'object' && 'settings' in args
        ? (args as { settings: Record<string, unknown> }).settings
        : {}
    let lastSettings: Record<string, unknown> = {}
    const transferSummary = (state: string, notCopied: number): Record<string, unknown> => ({
      id: 'run-e2e',
      settings: lastSettings,
      state,
      createdAtMs: 1,
      startedAtMs: null,
      finishedAtMs: null,
      machine: 'E2E',
      user: 'e2e',
      totals: {
        plannedFiles: 3,
        plannedBytes: 300,
        folders: 0,
        excluded: 0,
        copied: 3 - notCopied,
        copiedBytes: 100,
        skippedIdentical: 0,
        notCopied,
        notCopiedBytes: 200,
      },
      error: null,
    })

    tauriWindow.__OPEN_DIFF_INVOKE_LOG__ = log
    tauriWindow.__E2E_TRANSFER_RUN__ = { state: 'prepared', notCopied: [], recent: [] }
    tauriWindow.__TAURI_EVENT_PLUGIN_INTERNALS__ = {
      unregisterListener: (event: string, id: number) => {
        listeners.set(
          event,
          (listeners.get(event) ?? []).filter((listenerId) => listenerId !== id),
        )
        callbacks.delete(id)
      },
    }
    tauriWindow.__E2E_EMIT__ = (event: string, payload: unknown) => {
      for (const id of listeners.get(event) ?? []) {
        callbacks.get(id)?.({ event, id, payload })
      }
    }
    tauriWindow.__TAURI_INTERNALS__ = {
      transformCallback: (callback: EventCallback) => {
        const id = nextCallbackId++

        callbacks.set(id, callback)

        return id
      },
      invoke: (command: string, args?: unknown) => {
        log.push({ command, args })

        if (command === 'plugin:event|listen') {
          const { event, handler } = args as { event: string; handler: number }

          listeners.set(event, [...(listeners.get(event) ?? []), handler])

          return Promise.resolve(handler)
        }

        if (command === 'plugin:event|unlisten') {
          return Promise.resolve(null)
        }

        if (command === 'transfer_prepare') {
          lastSettings = transferSettings(args)

          return Promise.resolve({
            summary: transferSummary('prepared', 0),
            preflight: {
              runId: 'run-e2e',
              files: 3,
              folders: 0,
              bytes: 300,
              excluded: 0,
              alreadyThere: 0,
              bytesNeeded: 300,
              freeBytes: 1_000_000,
              volume: 'E:\\',
              enoughSpace: true,
              writable: true,
              writeError: null,
              scanErrors: 0,
              blocked: 0,
              issueCount: 0,
              issues: [],
            },
          })
        }

        if (command === 'transfer_load_run') {
          const run = tauriWindow.__E2E_TRANSFER_RUN__ ?? {
            state: 'prepared',
            notCopied: [],
            recent: [],
          }

          return Promise.resolve({
            summary: transferSummary(run.state, run.notCopied.length),
            preflight: null,
            notCopied: run.notCopied,
            recent: run.recent,
            running: false,
            folder: 'C:\\Users\\e2e\\AppData\\Local\\DeepServer\\Transfers\\run-e2e',
          })
        }

        if (command === 'transfer_copy_to_recovery') {
          return Promise.resolve({
            folder: 'E:\\Backup_NotCopied\\run-e2e',
            listFile: 'E:\\Backup_NotCopied\\run-e2e\\NOT-COPIED.csv',
            copied: 1,
            skipped: [],
          })
        }

        if (command === 'transfer_export_report') {
          return Promise.resolve({
            format: 'html',
            outputPath: 'E:\\DeepServer transfer run-e2e.html',
            bytesWritten: 2048,
          })
        }

        if (
          command === 'transfer_start' ||
          command === 'transfer_retry' ||
          command === 'transfer_watch'
        ) {
          return Promise.resolve(null)
        }

        if (command === 'transfer_cancel' || command === 'transfer_watch_finish') {
          return Promise.resolve(true)
        }

        if (command === 'diff_text') {
          return Promise.resolve({
            lines: [],
            stats: { added: 0, deleted: 0, modified: 0, equal: 0 },
          })
        }

        if (command === 'read_text_file') {
          const path =
            args && typeof args === 'object' && 'path' in args
              ? (args as { path: string }).path
              : 'file.txt'

          return Promise.resolve({
            path,
            text: `loaded:${path}`,
            encoding: 'UTF-8',
            lineEnding: 'LF',
            fileStamp: { size: 8, modifiedAtMs: 1 },
          })
        }

        if (command === 'compare_folder_paths') {
          const request =
            args && typeof args === 'object'
              ? (args as { leftRoot?: string; rightRoot?: string })
              : {}

          return Promise.resolve({
            leftRoot: request.leftRoot ?? '',
            rightRoot: request.rightRoot ?? '',
            rows: [],
            summary: { total: 0, same: 0, different: 0, leftOnly: 0, rightOnly: 0 },
          })
        }

        if (command === 'preview_folder_sync') {
          const request =
            args && typeof args === 'object'
              ? (args as { leftRoot?: string; rightRoot?: string; strategy?: string })
              : {}

          return Promise.resolve({
            name: 'preview',
            leftRoot: request.leftRoot ?? '',
            rightRoot: request.rightRoot ?? '',
            strategy: request.strategy ?? 'updateBoth',
            rows: [
              {
                id: 'copy-readme',
                relativePath: 'readme.txt',
                action: 'Copy',
                sourcePath: 'left/readme.txt',
                targetPath: 'right/readme.txt',
                detail: 'Left only',
              },
            ],
            summary: { total: 1, copy: 1, delete: 0, leave: 0, conflict: 0 },
          })
        }

        if (command === 'execute_folder_sync') {
          return Promise.resolve({
            name: 'run',
            total: 0,
            succeeded: 0,
            failed: 0,
            cancelled: 0,
            logs: [],
          })
        }

        if (command === 'build_folder_merge_plan') {
          return Promise.resolve({
            rows: [],
            summary: { actions: 0, automatic: 0, conflicts: 0 },
          })
        }

        if (command === 'execute_folder_merge_plan') {
          return Promise.resolve({
            logs: [],
            summary: { copied: 0, deleted: 0, skipped: 0, conflicts: 0 },
          })
        }

        if (command === 'merge_text_files') {
          return Promise.resolve({
            leftText: '',
            rightText: '',
            centerText: '',
            outputText: '',
            conflicts: [],
          })
        }

        if (command === 'compare_table') {
          return Promise.resolve({
            leftColumns: [],
            rightColumns: [],
            columnMappings: [],
            rows: [],
            changedCells: [],
            leftSheet: 'Sheet1',
            rightSheet: 'Sheet1',
            summary: { rowCount: 0, changedRowCount: 0, changedCellCount: 0 },
          })
        }

        if (command === 'compare_hex_files') {
          return Promise.resolve({
            left: { path: '', totalLen: 0, cells: [] },
            right: { path: '', totalLen: 0, cells: [] },
            diffRanges: [],
            summary: { leftBytes: 0, rightBytes: 0, differentRanges: 0 },
          })
        }

        if (command === 'compare_picture_files') {
          return Promise.resolve({
            left: { name: 'left.png', format: 'PNG', dimensions: '1 x 1', colorDepth: '32-bit' },
            right: { name: 'right.png', format: 'PNG', dimensions: '1 x 1', colorDepth: '32-bit' },
            statistics: {
              totalPixels: 1,
              differentPixels: 0,
              differenceRatio: 0,
              boundingRect: { x: 0, y: 0, width: 1, height: 1 },
            },
            metadataRows: [],
          })
        }

        if (command === 'compare_registry_exports') {
          return Promise.resolve({
            tree: [],
            summary: { added: 0, deleted: 0, modified: 0, same: 0 },
          })
        }

        if (command === 'compare_media_files') {
          return Promise.resolve({
            left: {
              name: 'left.mp3',
              container: 'MP3',
              duration: '00:00.000',
              stream: {
                codec: 'MP3',
                sampleRate: 'Unknown',
                channels: 'Unknown',
                bitrate: 'Unknown',
              },
            },
            right: {
              name: 'right.mp3',
              container: 'MP3',
              duration: '00:00.000',
              stream: {
                codec: 'MP3',
                sampleRate: 'Unknown',
                channels: 'Unknown',
                bitrate: 'Unknown',
              },
            },
            fields: [],
            summary: { added: 0, removed: 0, modified: 0, unchanged: 0 },
          })
        }

        if (command === 'compare_version_files') {
          return Promise.resolve({
            left: {
              name: 'left.exe',
              fileType: 'Application',
              targetOs: 'Windows 32-bit',
              fileVersion: '1.0.0.0',
              productVersion: '1.0.0.0',
            },
            right: {
              name: 'right.exe',
              fileType: 'Application',
              targetOs: 'Windows 32-bit',
              fileVersion: '1.0.0.1',
              productVersion: '1.0.0.0',
            },
            fields: [],
            summary: { added: 0, removed: 0, modified: 0, unchanged: 0 },
          })
        }

        if (command === 'apply_text_patch' || command === 'apply_text_patch_to_file') {
          return Promise.resolve({ text: 'patched' })
        }

        if (command === 'parse_text_patch') {
          return Promise.resolve({ file: 'file.txt', hunks: [] })
        }

        if (
          command === 'export_text_compare_report' ||
          command === 'export_folder_compare_report'
        ) {
          return Promise.resolve({
            format: 'html',
            content: '<html></html>',
            outputPath: 'report.html',
            bytesWritten: 13,
          })
        }

        if (command === 'run_script') {
          return Promise.resolve({
            executed: 1,
            compared: 0,
            different: 0,
            reportsWritten: 0,
            logs: ['ok'],
            cancelled: false,
          })
        }

        if (command === 'stop_script') {
          return Promise.resolve(true)
        }

        if (command === 'apply_live_registry_value') {
          const payload =
            args && typeof args === 'object'
              ? (args as { targetKey?: string; name?: string; kind?: string })
              : {}

          return Promise.resolve({
            targetKey: payload.targetKey ?? '',
            name: payload.name ?? '',
            action: payload.kind ? 'set' : 'delete',
          })
        }

        if (command === 'list_remote_profiles') {
          return Promise.resolve([])
        }

        if (command === 'test_remote_profile') {
          return Promise.resolve('connected')
        }

        if (command === 'save_remote_profile') {
          const draft =
            args && typeof args === 'object' && 'draft' in args
              ? (
                  args as {
                    draft: {
                      id: string
                      name: string
                      protocol: string
                      host: string
                      port: number | null
                      rootPath: string
                      username?: string
                    }
                  }
                ).draft
              : null

          if (!draft) {
            return Promise.resolve([])
          }

          const normalizedRoot = draft.rootPath.startsWith('/')
            ? draft.rootPath
            : `/${draft.rootPath || ''}`
          const scheme = draft.protocol === 'web-dav' ? 'webdav' : draft.protocol

          return Promise.resolve([
            {
              id: draft.id,
              name: draft.name,
              protocol: draft.protocol,
              host: draft.host,
              port: draft.port,
              rootPath: draft.rootPath || '/',
              implemented: [
                'sftp',
                'ftp',
                'ftps',
                'web-dav',
                's3',
                'dropbox',
                'one-drive',
                'subversion',
              ].includes(draft.protocol),
              uri: `${scheme}://profile/${draft.id}${normalizedRoot || '/'}`,
              username: draft.username ?? null,
            },
          ])
        }

        if (command === 'delete_remote_profile') {
          return Promise.resolve([])
        }

        if (command === 'write_git_integration' || command === 'write_svn_integration') {
          return Promise.resolve('Wrote config')
        }

        if (command === 'load_admin_policy') {
          return Promise.resolve({
            savePasswords: true,
            remoteProfiles: true,
            updateChecks: true,
          })
        }

        if (command === 'app_runtime_info') {
          return Promise.resolve({ os: 'linux', family: 'unix' })
        }

        if (
          command === 'register_windows_shell_extension' ||
          command === 'unregister_windows_shell_extension'
        ) {
          return Promise.resolve({
            windows: false,
            applied: false,
            script: '',
            message: 'Windows only',
          })
        }

        if (
          command === 'register_unix_shell_integration' ||
          command === 'unregister_unix_shell_integration'
        ) {
          return Promise.resolve({
            windows: false,
            applied: true,
            script: '',
            message: 'unix shell ok',
          })
        }

        if (command === 'open_path_external') {
          const payload =
            args && typeof args === 'object'
              ? (args as { path?: string; executable?: string | null })
              : {}

          return Promise.resolve({
            path: payload.path ?? '',
            executable: payload.executable ?? null,
            launched: true,
          })
        }

        if (command === 'take_shell_compare_launch') {
          return Promise.resolve(null)
        }

        // A cancelled native picker; Home tiles then open their session empty.
        if (command === 'pick_path') {
          return Promise.resolve(null)
        }

        if (command === 'diskusage_list_drives') {
          return Promise.resolve([
            { root: 'C:\\', label: 'System', kind: 'fixed', totalBytes: 1e12, freeBytes: 4e11 },
          ])
        }

        if (command === 'diskusage_list_snapshots') {
          return Promise.resolve([])
        }

        if (command === 'transfer_list_runs') {
          return Promise.resolve([])
        }
        if (command === 'diskusage_compare') {
          return Promise.resolve({ rows: [], warning: null })
        }

        if (command === 'detect_executables') {
          return Promise.resolve({ deepServer: null, vscode: null })
        }

        return Promise.resolve({})
      },
    }
  })
}

export function invokeLog(page: Page): Promise<InvokeCall[]> {
  return page.evaluate(() => {
    const tauriWindow = window as WindowWithInvokeLog

    return tauriWindow.__OPEN_DIFF_INVOKE_LOG__ ?? []
  })
}

/** Sends a Tauri event (e.g. `transfer://items`) to the page. */
export async function emitTauriEvent(page: Page, event: string, payload: unknown): Promise<void> {
  await page.evaluate(
    ([name, body]) => {
      ;(window as WindowWithInvokeLog).__E2E_EMIT__?.(name, body)
    },
    [event, payload] as const,
  )
}

/** Sets what `transfer_load_run` returns from now on. */
export async function setTransferRun(page: Page, run: MockTransferRun): Promise<void> {
  await page.evaluate((next) => {
    ;(window as WindowWithInvokeLog).__E2E_TRANSFER_RUN__ = next
  }, run)
}

export async function lastInvoke(page: Page, command: string): Promise<InvokeCall | undefined> {
  const log = await invokeLog(page)

  return [...log].reverse().find((call) => call.command === command)
}
