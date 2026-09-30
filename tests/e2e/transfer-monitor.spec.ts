import { expect, test } from '@playwright/test'
import {
  emitTauriEvent,
  installTauriInvokeMock,
  lastInvoke,
  setTransferRun,
} from './helpers/tauriMock'

function row(
  relativePath: string,
  status: string,
  reason?: string,
  recoverable = false,
): Record<string, unknown> {
  return {
    relativePath,
    kind: 'file',
    size: 100,
    status,
    reason,
    side: reason === 'accessDenied' ? 'destination' : 'source',
    message: reason ? `os error for ${relativePath}` : undefined,
    attempts: 1,
    inferred: false,
    recoverable,
    atMs: 1,
  }
}

const locked = row('in-use.pst', 'notCopied', 'fileLocked')
const denied = row('private/report.docx', 'notCopied', 'accessDenied', true)

test.beforeEach(async ({ page }) => {
  await installTauriInvokeMock(page)
})

test('copy a folder, group what did not make it, retry, recover and export the report', async ({
  page,
}) => {
  await page.goto('/transfer')
  await page.getByTestId('transfer-source').fill('D:\\Client')
  await page.getByTestId('transfer-destination').fill('E:\\Backup')
  await page.getByTestId('transfer-check').click()

  await expect(page.getByTestId('transfer-preflight')).toContainText('3')
  expect(await lastInvoke(page, 'transfer_prepare')).toMatchObject({
    args: { settings: { source: 'D:\\Client', destination: 'E:\\Backup', mode: 'copy' } },
  })

  await page.getByTestId('transfer-start').click()
  await expect
    .poll(async () => (await lastInvoke(page, 'transfer_start'))?.args)
    .toEqual({
      runId: 'run-e2e',
    })

  await emitTauriEvent(page, 'transfer://items', {
    runId: 'run-e2e',
    items: [row('ok.txt', 'copied'), locked, denied],
  })
  await expect(page.getByTestId('transfer-not-copied-group')).toHaveCount(2)
  await expect(page.getByTestId('transfer-feed-row')).toHaveCount(1)
  await expect(page.getByTestId('transfer-count-not-copied')).toHaveText('2')

  await setTransferRun(page, { state: 'completed', notCopied: [locked, denied], recent: [] })
  await emitTauriEvent(page, 'transfer://finished', {
    runId: 'run-e2e',
    summary: null,
    error: null,
  })
  await expect(page.getByTestId('transfer-export-html')).toBeVisible()

  // Retry the in-use group once the file is closed.
  await page.locator('[data-reason="fileLocked"]').getByTestId('transfer-retry-group').click()
  await expect
    .poll(async () => (await lastInvoke(page, 'transfer_retry'))?.args)
    .toEqual({
      runId: 'run-e2e',
      selection: { reason: 'fileLocked' },
    })
  await setTransferRun(page, { state: 'completed', notCopied: [denied], recent: [] })
  await emitTauriEvent(page, 'transfer://finished', {
    runId: 'run-e2e',
    summary: null,
    error: null,
  })
  await expect(page.getByTestId('transfer-not-copied-group')).toHaveCount(1)

  // The access-denied file goes to the recovery folder beside the destination.
  await page.locator('[data-reason="accessDenied"]').getByTestId('transfer-recover-group').click()
  await expect(page.getByTestId('transfer-recovery-result')).toContainText(
    'E:\\Backup_NotCopied\\run-e2e',
  )

  await page.getByTestId('transfer-export-html').click()
  await expect(page.getByTestId('transfer-report-saved')).toContainText(
    'DeepServer transfer run-e2e.html',
  )
  expect(await lastInvoke(page, 'transfer_export_report')).toMatchObject({
    args: { runId: 'run-e2e', format: 'html', choosePath: true },
  })
})

test('the Home tile opens the Transfer Monitor', async ({ page }) => {
  await page.goto('/')
  await page.locator('[data-session-type="transfer-monitor"]').click()
  await expect(page.getByTestId('transfer-setup')).toBeVisible()
  await expect(page.getByTestId('transfer-check')).toBeDisabled()
})
