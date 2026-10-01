<script setup lang="ts">
import { computed, onMounted, watch } from 'vue'
import { storeToRefs } from 'pinia'
import { useRoute } from 'vue-router'
import { openPathExternal, revealPathInOs } from '@/api/integration'
import { formatBytes } from '@/app/diskUsage'
import { pickNativePath } from '@/app/filePicker'
import { formatDuration, saveTransferPreferences } from '@/app/transferMonitor'
import LiveFeed from '@/components/transfer/LiveFeed.vue'
import NotCopiedPanel from '@/components/transfer/NotCopiedPanel.vue'
import WorkbenchShell from '@/components/workbench/WorkbenchShell.vue'
import { useI18n } from '@/i18n'
import { useSessionLaunchStore } from '@/stores/sessionLaunch'
import { useTransferStore } from '@/stores/transfer'
import type { RunSummary } from '@/types/transfer'

const route = useRoute()
const { t } = useI18n()
const sessionLaunch = useSessionLaunchStore()
const transfer = useTransferStore()
const {
  preferences,
  source,
  destination,
  stage,
  summary,
  preflight,
  progress,
  runs,
  errorMessage,
  busyAction,
  recoveryProgress,
  lastPrune,
  lastRecovery,
  lastReport,
  bytesPerSecond,
  etaSeconds,
  recent,
  groups,
  notCopiedCount,
  working,
} = storeToRefs(transfer)

const editable = computed(() => stage.value === 'setup')
const watchMode = computed(() => preferences.value.mode === 'watch')
const canCheck = computed(
  () => editable.value && Boolean(source.value.trim() && destination.value.trim()),
)
const longPathWarnings = computed(
  () => preflight.value?.issues.filter((issue) => issue.reason === 'pathTooLong').length ?? 0,
)
const totals = computed(() => summary.value?.totals)
const percent = computed(() => {
  const current = progress.value

  if (!current || current.bytesTotal <= 0) {
    return 0
  }

  return Math.min(100, Math.round((current.bytesDone / current.bytesTotal) * 100))
})
const phaseLabel = computed(() => {
  if (stage.value === 'preparing') {
    return t('ui.transferPhase.scanning')
  }

  if (stage.value === 'running') {
    return t('ui.transferPhase.copying')
  }

  if (stage.value === 'watching') {
    return progress.value?.phase === 'finishing'
      ? t('ui.transferPhase.finishing')
      : t('ui.transferPhase.watching')
  }

  return summary.value ? t(`ui.transferState.${summary.value.state}`) : ''
})
const recoveryFolderLabel = computed(() =>
  preferences.value.recoveryFolder.trim()
    ? preferences.value.recoveryFolder
    : t('ui.transferRecoveryDefault', {
        folder: `${destination.value.replace(/[\\/]+$/, '')}_NotCopied`,
      }),
)

function baseName(path: string): string {
  return (
    path
      .replace(/[\\/]+$/, '')
      .split(/[\\/]/)
      .pop() ?? path
  )
}

async function browse(side: 'source' | 'destination' | 'recovery'): Promise<void> {
  const picked = await pickNativePath({ directory: true })

  if (!picked) {
    return
  }

  if (side === 'source') {
    source.value = picked
  } else if (side === 'destination') {
    destination.value = picked
  } else {
    preferences.value.recoveryFolder = picked
    saveTransferPreferences(preferences.value)
  }
}

function resetRecoveryFolder(): void {
  preferences.value.recoveryFolder = ''
  saveTransferPreferences(preferences.value)
}

async function reveal(path: string): Promise<void> {
  try {
    await revealPathInOs(path)
  } catch {
    // Nothing to show when the path is gone; the row already says why.
  }
}

async function openPath(path: string): Promise<void> {
  try {
    await openPathExternal(path)
  } catch {
    await reveal(path)
  }
}

function runLabel(run: RunSummary): string {
  return t('ui.transferRunLine', {
    source: baseName(run.settings.source),
    destination: run.settings.destination,
  })
}

function runTime(run: RunSummary): string {
  return new Date(run.startedAtMs ?? run.createdAtMs).toLocaleString()
}

watch(
  preferences,
  (value) => {
    saveTransferPreferences(value)
  },
  { deep: true },
)

onMounted(async () => {
  transfer.ensureListening()
  void transfer.refreshRuns()

  const launch = sessionLaunch.consumeLaunch('/transfer')
  const requestedRun = typeof route.query.run === 'string' ? route.query.run : undefined

  if (requestedRun && !working.value) {
    await transfer.loadRun(requestedRun)

    return
  }

  if (!launch || working.value) {
    return
  }

  transfer.newTransfer()
  source.value = launch.locations.left?.uri ?? source.value
  destination.value = launch.locations.right?.uri ?? ''

  if (launch.autoRun && source.value && destination.value) {
    // Only the check runs by itself; copying always waits for Start.
    await transfer.prepare()
  }
})
</script>

<template>
  <WorkbenchShell
    :title="$t('ui.transferMonitor')"
    :eyebrow="$t('ui.homeGroupCopySync')"
    :subtitle="
      source && destination
        ? $t('ui.transferRunLine', { source: baseName(source), destination })
        : $t('ui.transferChooseFolders')
    "
  >
    <section class="transfer-view">
      <form
        class="transfer-setup"
        data-testid="transfer-setup"
        @submit.prevent="transfer.prepare()"
      >
        <fieldset
          class="transfer-mode"
          :disabled="!editable"
        >
          <label>
            <input
              v-model="preferences.mode"
              type="radio"
              value="copy"
              data-testid="transfer-mode-copy"
            />
            {{ $t('ui.transferModeCopy') }}
          </label>
          <label>
            <input
              v-model="preferences.mode"
              type="radio"
              value="watch"
              data-testid="transfer-mode-watch"
            />
            {{ $t('ui.transferModeWatch') }}
          </label>
        </fieldset>

        <div class="transfer-paths">
          <label for="transfer-source">{{ $t('ui.transferSource') }}</label>
          <input
            id="transfer-source"
            v-model="source"
            type="text"
            data-testid="transfer-source"
            :disabled="!editable"
            :placeholder="$t('ui.transferSourcePlaceholder')"
          />
          <button
            type="button"
            data-testid="transfer-browse-source"
            :disabled="!editable"
            @click="browse('source')"
          >
            {{ $t('ui.browse') }}
          </button>

          <label for="transfer-destination">{{ $t('ui.transferDestination') }}</label>
          <input
            id="transfer-destination"
            v-model="destination"
            type="text"
            data-testid="transfer-destination"
            :disabled="!editable"
            :placeholder="$t('ui.transferDestinationPlaceholder')"
          />
          <button
            type="button"
            data-testid="transfer-browse-destination"
            :disabled="!editable"
            @click="browse('destination')"
          >
            {{ $t('ui.browse') }}
          </button>
        </div>

        <fieldset
          class="transfer-options"
          :disabled="!editable"
        >
          <label>
            {{ $t('ui.transferInclude') }}
            <input
              v-model="preferences.include"
              type="text"
              data-testid="transfer-include"
              :placeholder="$t('ui.transferIncludePlaceholder')"
            />
          </label>
          <label>
            {{ $t('ui.transferExclude') }}
            <input
              v-model="preferences.exclude"
              type="text"
              data-testid="transfer-exclude"
              :placeholder="$t('ui.transferExcludePlaceholder')"
            />
          </label>
          <label>
            {{ $t('ui.transferVerify') }}
            <select
              v-model="preferences.verify"
              data-testid="transfer-verify"
            >
              <option value="sizeAndTime">{{ $t('ui.transferVerifySizeAndTime') }}</option>
              <option value="hash">{{ $t('ui.transferVerifyHash') }}</option>
            </select>
          </label>
          <label v-if="!watchMode">
            {{ $t('ui.transferConflict') }}
            <select
              v-model="preferences.conflict"
              data-testid="transfer-conflict"
            >
              <option value="skip">{{ $t('ui.transferConflictSkip') }}</option>
              <option value="overwriteIfNewer">{{ $t('ui.transferConflictNewer') }}</option>
            </select>
          </label>
          <template v-else>
            <label>
              {{ $t('ui.transferStableSeconds') }}
              <input
                v-model.number="preferences.stableSeconds"
                type="number"
                min="1"
                max="600"
                data-testid="transfer-stable-seconds"
              />
            </label>
            <label>
              {{ $t('ui.transferQuietSeconds') }}
              <input
                v-model.number="preferences.quietSeconds"
                type="number"
                min="0"
                max="86400"
                data-testid="transfer-quiet-seconds"
              />
            </label>
          </template>
          <label class="transfer-checkbox">
            <input
              v-model="preferences.includeHidden"
              type="checkbox"
              data-testid="transfer-include-hidden"
            />
            {{ $t('ui.transferIncludeHidden') }}
          </label>
        </fieldset>

        <div class="transfer-buttons">
          <button
            v-if="stage === 'setup' || stage === 'preparing'"
            type="submit"
            class="primary"
            data-testid="transfer-check"
            :disabled="!canCheck"
          >
            {{ stage === 'preparing' ? $t('ui.transferChecking') : $t('ui.transferCheck') }}
          </button>
          <button
            v-if="stage === 'prepared'"
            type="button"
            class="primary"
            data-testid="transfer-start"
            @click="transfer.start()"
          >
            {{ watchMode ? $t('ui.transferStartWatch') : $t('ui.transferStartCopy') }}
          </button>
          <button
            v-if="stage === 'watching'"
            type="button"
            class="primary"
            data-testid="transfer-finish"
            :disabled="progress?.phase === 'finishing'"
            @click="transfer.finish()"
          >
            {{ $t('ui.transferFinish') }}
          </button>
          <button
            v-if="working"
            type="button"
            data-testid="transfer-cancel"
            @click="transfer.cancel()"
          >
            {{ $t('ui.cancel') }}
          </button>
          <button
            v-if="stage === 'prepared' || stage === 'done'"
            type="button"
            data-testid="transfer-new"
            @click="transfer.newTransfer()"
          >
            {{ stage === 'prepared' ? $t('ui.transferChangeSettings') : $t('ui.transferNew') }}
          </button>
          <template v-if="stage === 'done' || (stage === 'prepared' && notCopiedCount > 0)">
            <button
              type="button"
              data-testid="transfer-export-html"
              :disabled="busyAction !== null"
              @click="transfer.exportReport('html')"
            >
              {{ $t('ui.transferExportHtml') }}
            </button>
            <button
              type="button"
              data-testid="transfer-export-csv"
              :disabled="busyAction !== null"
              @click="transfer.exportReport('csv')"
            >
              {{ $t('ui.transferExportCsv') }}
            </button>
          </template>
        </div>
      </form>

      <p
        v-if="errorMessage"
        class="transfer-error"
        role="alert"
        data-testid="transfer-error"
      >
        {{ errorMessage }}
      </p>

      <p
        v-if="lastReport"
        class="transfer-note"
        role="status"
        data-testid="transfer-report-saved"
      >
        {{ $t('ui.transferReportSaved', { path: lastReport.outputPath }) }}
        <button
          type="button"
          @click="openPath(lastReport.outputPath)"
        >
          {{ $t('ui.transferOpen') }}
        </button>
      </p>

      <section
        v-if="preflight && (stage === 'prepared' || stage === 'done')"
        class="transfer-preflight"
        data-testid="transfer-preflight"
      >
        <h2>{{ $t('ui.transferPreflight') }}</h2>
        <ul>
          <li>
            {{
              $t('ui.transferPreflightFiles', {
                files: preflight.files,
                size: formatBytes(preflight.bytes),
                folders: preflight.folders,
              })
            }}
          </li>
          <li v-if="preflight.alreadyThere > 0">
            {{ $t('ui.transferPreflightAlreadyThere', { count: preflight.alreadyThere }) }}
          </li>
          <li v-if="preflight.excluded > 0">
            {{ $t('ui.transferPreflightExcluded', { count: preflight.excluded }) }}
          </li>
          <li
            v-if="preflight.freeBytes !== null && !watchMode"
            :class="{ 'transfer-bad': !preflight.enoughSpace }"
            data-testid="transfer-preflight-space"
          >
            {{
              $t(
                preflight.enoughSpace ? 'ui.transferPreflightSpace' : 'ui.transferPreflightNoSpace',
                {
                  needed: formatBytes(preflight.bytesNeeded),
                  free: formatBytes(preflight.freeBytes),
                  volume: preflight.volume ?? destination,
                },
              )
            }}
          </li>
          <li
            v-if="!preflight.writable"
            class="transfer-bad"
            data-testid="transfer-preflight-not-writable"
          >
            {{ $t('ui.transferPreflightNotWritable', { error: preflight.writeError ?? '' }) }}
          </li>
          <li
            v-if="preflight.scanErrors > 0"
            class="transfer-bad"
          >
            {{ $t('ui.transferPreflightScanErrors', { count: preflight.scanErrors }) }}
          </li>
          <li
            v-if="preflight.blocked > 0"
            class="transfer-bad"
          >
            {{ $t('ui.transferPreflightBlocked', { count: preflight.blocked }) }}
          </li>
          <li
            v-if="longPathWarnings > 0"
            data-testid="transfer-preflight-long-paths"
          >
            {{ $t('ui.transferPreflightLongPaths', { count: longPathWarnings }) }}
          </li>
        </ul>
      </section>

      <section
        v-if="progress || totals"
        class="transfer-summary"
        data-testid="transfer-summary"
      >
        <div class="transfer-summary-line">
          <strong data-testid="transfer-phase">{{ phaseLabel }}</strong>
          <span v-if="bytesPerSecond && working">
            {{ $t('ui.transferRate', { rate: formatBytes(Math.round(bytesPerSecond)) }) }}
          </span>
          <span v-if="etaSeconds !== null && stage === 'running'">
            {{ $t('ui.transferEta', { time: formatDuration(etaSeconds) }) }}
          </span>
        </div>
        <div
          v-if="working && progress"
          class="transfer-bar"
          role="progressbar"
          :aria-valuenow="percent"
          aria-valuemin="0"
          aria-valuemax="100"
        >
          <span :style="{ width: `${String(percent)}%` }" />
        </div>
        <dl class="transfer-counts">
          <div>
            <dt>{{ $t('ui.transferPlanned') }}</dt>
            <dd>
              {{ progress?.filesTotal || totals?.plannedFiles || 0 }} ·
              {{ formatBytes(progress?.bytesTotal || totals?.plannedBytes || 0) }}
            </dd>
          </div>
          <div>
            <dt>{{ watchMode ? $t('ui.transferArrived') : $t('ui.transferCopied') }}</dt>
            <dd data-testid="transfer-count-copied">
              {{ working ? (progress?.copied ?? 0) : (totals?.copied ?? 0) }}
            </dd>
          </div>
          <div v-if="!watchMode">
            <dt>{{ $t('ui.transferSkipped') }}</dt>
            <dd>{{ working ? (progress?.skipped ?? 0) : (totals?.skippedIdentical ?? 0) }}</dd>
          </div>
          <div :class="{ 'transfer-bad': notCopiedCount > 0 }">
            <dt>{{ $t('ui.transferNotCopied') }}</dt>
            <dd data-testid="transfer-count-not-copied">{{ notCopiedCount }}</dd>
          </div>
        </dl>
        <p
          v-if="working && progress?.current"
          class="transfer-current"
        >
          {{ progress.current }}
        </p>
      </section>

      <div
        v-if="stage !== 'setup'"
        class="transfer-panels"
      >
        <section class="transfer-panel">
          <header class="transfer-panel-header">
            <h2>{{ $t('ui.transferNotCopied') }}</h2>
            <button
              v-if="notCopiedCount > 0"
              type="button"
              data-testid="transfer-retry-all"
              :disabled="working || busyAction === 'recovery'"
              @click="transfer.retry({})"
            >
              {{ $t('ui.transferRetryAll') }}
            </button>
            <button
              v-if="notCopiedCount > 0"
              type="button"
              data-testid="transfer-recover-all"
              :disabled="working || busyAction !== null"
              @click="transfer.recover({})"
            >
              {{ $t('ui.transferRecoverAll') }}
            </button>
          </header>
          <p
            v-if="notCopiedCount > 0"
            class="transfer-recovery-folder"
          >
            {{ $t('ui.transferRecoveryFolder') }}
            <span data-testid="transfer-recovery-folder">{{ recoveryFolderLabel }}</span>
            <button
              type="button"
              data-testid="transfer-choose-recovery"
              @click="browse('recovery')"
            >
              {{ $t('ui.browse') }}
            </button>
            <button
              v-if="preferences.recoveryFolder"
              type="button"
              @click="resetRecoveryFolder"
            >
              {{ $t('ui.transferRecoveryReset') }}
            </button>
          </p>
          <p
            v-if="recoveryProgress"
            class="transfer-note"
            role="status"
            data-testid="transfer-recovery-progress"
          >
            {{
              recoveryProgress.total > 0
                ? $t('ui.transferRecovering', {
                    done: recoveryProgress.done,
                    total: recoveryProgress.total,
                  })
                : $t('ui.transferRecoveringStart')
            }}
            <button
              type="button"
              data-testid="transfer-cancel-recovery"
              @click="transfer.cancelRecovery()"
            >
              {{ $t('ui.cancel') }}
            </button>
          </p>
          <p
            v-if="lastRecovery"
            class="transfer-note"
            role="status"
            data-testid="transfer-recovery-result"
          >
            {{
              $t('ui.transferRecovered', {
                count: lastRecovery.copied,
                folder: lastRecovery.folder,
              })
            }}
            <span v-if="lastRecovery.skipped.length > 0">
              {{ $t('ui.transferRecoverySkipped', { count: lastRecovery.skipped.length }) }}
            </span>
            <button
              type="button"
              @click="openPath(lastRecovery.folder)"
            >
              {{ $t('ui.transferOpen') }}
            </button>
          </p>
          <NotCopiedPanel
            :groups="groups"
            :source="source"
            :destination="destination"
            :busy="working || busyAction !== null"
            @retry="transfer.retry"
            @recover="transfer.recover"
            @reveal="reveal"
          />
        </section>

        <section class="transfer-panel">
          <header class="transfer-panel-header">
            <h2>{{ watchMode ? $t('ui.transferFeedArrived') : $t('ui.transferFeedCopied') }}</h2>
          </header>
          <LiveFeed :items="recent" />
        </section>
      </div>

      <details
        class="transfer-runs"
        data-testid="transfer-runs"
      >
        <summary>{{ $t('ui.transferPreviousRuns') }} ({{ runs.length }})</summary>
        <p class="transfer-note">
          {{ $t('ui.transferRunsKept') }}
          <button
            type="button"
            data-testid="transfer-prune-runs"
            :disabled="working"
            @click="transfer.pruneRuns()"
          >
            {{ $t('ui.transferPruneRuns') }}
          </button>
          <span
            v-if="lastPrune"
            role="status"
            data-testid="transfer-prune-result"
          >
            {{
              $t('ui.transferPruned', {
                count: lastPrune.removed,
                size: formatBytes(lastPrune.freedBytes),
              })
            }}
          </span>
        </p>
        <p
          v-if="runs.length === 0"
          class="transfer-note"
        >
          {{ $t('ui.transferNoRuns') }}
        </p>
        <ul v-else>
          <li
            v-for="run in runs"
            :key="run.id"
            data-testid="transfer-run"
          >
            <span class="transfer-run-time">{{ runTime(run) }}</span>
            <span class="transfer-run-line">{{ runLabel(run) }}</span>
            <span>{{ $t(`ui.transferState.${run.state}`) }}</span>
            <span>
              {{
                $t('ui.transferRunCounts', {
                  copied: run.totals.copied,
                  notCopied: run.totals.notCopied,
                })
              }}
            </span>
            <button
              type="button"
              data-testid="transfer-open-run"
              :disabled="working"
              @click="transfer.loadRun(run.id)"
            >
              {{ $t('ui.transferOpen') }}
            </button>
          </li>
        </ul>
      </details>
    </section>
  </WorkbenchShell>
</template>

<style scoped>
.transfer-view {
  display: flex;
  flex-direction: column;
  gap: 12px;
  min-width: 0;
  padding: 12px;
}

.transfer-setup {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.transfer-mode,
.transfer-options {
  display: flex;
  flex-wrap: wrap;
  gap: 8px 16px;
  margin: 0;
  padding: 0;
  border: 0;
}

.transfer-options label {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.transfer-options label.transfer-checkbox {
  flex-direction: row;
  align-items: center;
  align-self: flex-end;
}

.transfer-paths {
  display: grid;
  grid-template-columns: max-content minmax(0, 1fr) max-content;
  align-items: center;
  gap: 6px 8px;
}

.transfer-buttons,
.transfer-summary-line,
.transfer-panel-header,
.transfer-recovery-folder {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 8px;
}

.transfer-panel-header h2,
.transfer-preflight h2 {
  margin: 0;
  font-size: 1rem;
}

.transfer-preflight ul {
  margin: 4px 0 0;
  padding-left: 20px;
}

.transfer-error,
.transfer-bad {
  color: var(--app-danger, #c42b1c);
}

.transfer-note,
.transfer-current,
.transfer-run-time {
  color: var(--app-text-muted);
}

.transfer-current {
  margin: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.transfer-bar {
  height: 6px;
  overflow: hidden;
  border-radius: 3px;
  background: var(--app-border);
}

.transfer-bar span {
  display: block;
  height: 100%;
  background: var(--app-accent, #0f6cbd);
}

.transfer-counts {
  display: flex;
  flex-wrap: wrap;
  gap: 16px;
  margin: 6px 0 0;
}

.transfer-counts dt {
  color: var(--app-text-muted);
  font-size: 0.85rem;
}

.transfer-counts dd {
  margin: 0;
  font-size: 1.1rem;
  font-variant-numeric: tabular-nums;
}

.transfer-panels {
  display: grid;
  grid-template-columns: minmax(0, 3fr) minmax(0, 2fr);
  gap: 12px;
}

.transfer-panel {
  display: flex;
  flex-direction: column;
  gap: 8px;
  min-width: 0;
}

.transfer-runs ul {
  margin: 6px 0 0;
  padding: 0;
  list-style: none;
}

.transfer-runs li {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 10px;
  padding: 2px 0;
}

.transfer-run-line {
  flex: 1 1 240px;
  min-width: 0;
  overflow-wrap: anywhere;
}

@media (width <= 900px) {
  .transfer-panels {
    grid-template-columns: minmax(0, 1fr);
  }
}
</style>
