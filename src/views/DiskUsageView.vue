<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import {
  compareSnapshots,
  listDrives,
  listSnapshots,
  openDiskUsage,
  takeSnapshot,
  type ChangeRow,
  type DriveInfo,
  type SnapshotInfo,
} from '@/api/diskusage'
import {
  addRecentLocation,
  folderPathInLocation,
  formatByteChange,
  formatBytes,
  isNetworkPath,
  loadRecentLocations,
  saveRecentLocations,
  snapshotLocalLabel,
} from '@/app/diskUsage'
import { pickNativePath } from '@/app/filePicker'
import DenseDataTable from '@/components/workbench/DenseDataTable.vue'
import WorkbenchShell from '@/components/workbench/WorkbenchShell.vue'
import { useI18n } from '@/i18n'
import { useSessionLaunchStore } from '@/stores/sessionLaunch'
import { useSettingsStore } from '@/stores/settings'
import { useTabsStore } from '@/stores/tabs'

type SortKey = 'folder' | 'change' | 'sizeChange' | 'filesChange'

const route = useRoute()
const router = useRouter()
const { t } = useI18n()
const sessionLaunch = useSessionLaunchStore()
const settings = useSettingsStore()
const tabs = useTabsStore()

const drives = ref<DriveInfo[]>([])
const recentLocations = ref<string[]>(loadRecentLocations())
const locationDraft = ref('')
const location = ref('')
const snapshots = ref<SnapshotInfo[]>([])
const baselinePath = ref('')
const currentPath = ref('')
const showAll = ref(false)
const rows = ref<ChangeRow[]>([])
const warning = ref<string | null>(null)
const errorMessage = ref('')
const snapshotBusy = ref(false)
const compareBusy = ref(false)
const sortKey = ref<SortKey>('sizeChange')
const sortDescending = ref(true)

const locationChoices = computed(() => {
  const driveRoots = drives.value.map((drive) => drive.root)

  return [
    ...driveRoots,
    ...recentLocations.value.filter(
      (path) => !driveRoots.some((root) => root.toLowerCase() === path.toLowerCase()),
    ),
  ]
})
const selectedDrive = computed(() =>
  drives.value.find((drive) => drive.root.toLowerCase() === location.value.toLowerCase()),
)
const sortedRows = computed(() => {
  const sorted = [...rows.value].sort((left, right) => {
    const key = sortKey.value
    const order =
      key === 'folder' || key === 'change'
        ? left[key].localeCompare(right[key])
        : Math.abs(left[key]) - Math.abs(right[key])

    return sortDescending.value ? -order : order
  })

  return sorted
})
const canCompare = computed(
  () =>
    Boolean(baselinePath.value && currentPath.value) && baselinePath.value !== currentPath.value,
)

function errorText(error: unknown): string {
  if (typeof error === 'object' && error !== null && 'message' in error) {
    return String(error.message)
  }

  return String(error)
}

function driveLabel(drive: DriveInfo): string {
  const name = drive.label ? `${drive.root} ${drive.label}` : drive.root

  if (drive.freeBytes === null || drive.totalBytes === null) {
    return name
  }

  return t('ui.diskUsageDriveFree', {
    drive: name,
    free: formatBytes(drive.freeBytes),
    total: formatBytes(drive.totalBytes),
  })
}

function choiceLabel(path: string): string {
  const drive = drives.value.find((item) => item.root === path)

  if (drive) {
    return driveLabel(drive)
  }

  return isNetworkPath(path) ? t('ui.diskUsageNetworkShare', { path }) : path
}

async function loadDrives(): Promise<void> {
  try {
    drives.value = await listDrives()
  } catch {
    drives.value = []
  }
}

async function setLocation(path: string): Promise<void> {
  const trimmed = path.trim()

  locationDraft.value = trimmed

  if (!trimmed || trimmed === location.value) {
    return
  }

  location.value = trimmed
  recentLocations.value = addRecentLocation(recentLocations.value, trimmed)
  saveRecentLocations(recentLocations.value)
  await loadSnapshots()
}

async function browseLocation(): Promise<void> {
  const picked = await pickNativePath({ directory: true })

  if (picked) {
    await setLocation(picked)
  }
}

async function openTreemap(folder = location.value): Promise<void> {
  if (!folder) {
    return
  }

  try {
    errorMessage.value = ''
    await openDiskUsage(folder)
  } catch (error) {
    errorMessage.value = errorText(error)
  }
}

async function loadSnapshots(): Promise<void> {
  rows.value = []
  warning.value = null
  errorMessage.value = ''

  try {
    snapshots.value = await listSnapshots(location.value)
  } catch (error) {
    snapshots.value = []
    errorMessage.value = errorText(error)
  }

  // Newest first: compare the latest snapshot with the one before it.
  currentPath.value = snapshots.value[0]?.path ?? ''
  baselinePath.value = snapshots.value[1]?.path ?? ''
  // The watch on the two paths runs the comparison.
}

async function takeSnapshotNow(): Promise<void> {
  if (!location.value || snapshotBusy.value) {
    return
  }

  snapshotBusy.value = true
  errorMessage.value = ''

  try {
    await takeSnapshot(location.value)
    await loadSnapshots()
  } catch (error) {
    errorMessage.value = errorText(error)
  } finally {
    snapshotBusy.value = false
  }
}

async function runCompare(): Promise<void> {
  if (!canCompare.value) {
    rows.value = []
    warning.value = null

    return
  }

  compareBusy.value = true

  try {
    const comparison = await compareSnapshots(baselinePath.value, currentPath.value, showAll.value)

    rows.value = comparison.rows
    warning.value = comparison.warning
  } catch (error) {
    rows.value = []
    warning.value = null
    errorMessage.value = errorText(error)
  } finally {
    compareBusy.value = false
  }
}

function sortBy(key: SortKey): void {
  if (sortKey.value === key) {
    sortDescending.value = !sortDescending.value
  } else {
    sortKey.value = key
    sortDescending.value = key === 'sizeChange' || key === 'filesChange'
  }
}

function sortIndicator(key: SortKey): string {
  if (sortKey.value !== key) {
    return ''
  }

  return sortDescending.value ? '▼' : '▲'
}

/** Opens Folder Compare with this folder on the left, so it can be compared with a backup or copy. */
function compareFolder(row: ChangeRow): void {
  const folder = folderPathInLocation(location.value, row.folder)
  const title = t('ui.folderCompare')

  sessionLaunch.setPendingLaunch({
    id: crypto.randomUUID(),
    source: 'command',
    sessionType: 'folder-compare',
    title,
    route: '/compare/folder',
    locations: { left: { uri: folder, kind: 'directory', readOnly: false } },
    autoRun: false,
  })
  const opened = tabs.openTab({
    title,
    titleKey: 'ui.folderCompare',
    route: '/compare/folder',
    dirty: false,
    forceNew: settings.openSessionsInNewTab,
  })

  void router.push(opened.route)
}

watch([baselinePath, currentPath, showAll], () => {
  void runCompare()
})

onMounted(async () => {
  await loadDrives()

  const launch = sessionLaunch.consumeLaunch('/disk/usage')
  const requested = [
    typeof route.query.path === 'string' ? route.query.path : undefined,
    launch?.locations.left?.uri,
    recentLocations.value[0],
  ].find((path) => path?.trim())

  if (requested) {
    await setLocation(requested)
  }
})
</script>

<template>
  <WorkbenchShell
    :title="$t('ui.diskUsage')"
    :eyebrow="$t('ui.disk')"
    :subtitle="location || $t('ui.diskUsageChooseLocation')"
  >
    <section class="disk-usage-view">
      <header class="disk-usage-location">
        <label for="disk-usage-location">{{ $t('ui.diskUsageLocation') }}</label>
        <input
          id="disk-usage-location"
          v-model="locationDraft"
          data-testid="disk-usage-location"
          type="text"
          list="disk-usage-location-choices"
          :placeholder="$t('ui.diskUsageLocationPlaceholder')"
          @change="setLocation(locationDraft)"
          @keydown.enter="setLocation(locationDraft)"
        />
        <datalist id="disk-usage-location-choices">
          <option
            v-for="choice in locationChoices"
            :key="choice"
            :value="choice"
          >
            {{ choiceLabel(choice) }}
          </option>
        </datalist>
        <button
          type="button"
          data-testid="disk-usage-browse"
          @click="browseLocation"
        >
          {{ $t('ui.browse') }}
        </button>
        <button
          type="button"
          data-testid="disk-usage-open-treemap"
          :disabled="!location"
          @click="openTreemap()"
        >
          {{ $t('ui.diskUsageOpenTreemap') }}
        </button>
        <span
          v-if="selectedDrive"
          class="disk-usage-drive"
          data-testid="disk-usage-drive"
          >{{ driveLabel(selectedDrive) }}</span
        >
      </header>

      <p
        v-if="errorMessage"
        class="disk-usage-error"
        role="alert"
        data-testid="disk-usage-error"
      >
        {{ errorMessage }}
      </p>

      <section
        v-if="location"
        class="disk-usage-changes"
        data-testid="disk-usage-changes"
      >
        <header>
          <h2>{{ $t('ui.diskUsageWhatChanged') }}</h2>
          <button
            type="button"
            data-testid="disk-usage-take-snapshot"
            :disabled="snapshotBusy"
            @click="takeSnapshotNow"
          >
            {{ snapshotBusy ? $t('ui.diskUsageScanning') : $t('ui.diskUsageTakeSnapshot') }}
          </button>
        </header>

        <p
          v-if="snapshots.length < 2"
          class="disk-usage-hint"
          data-testid="disk-usage-need-snapshots"
        >
          {{ $t('ui.diskUsageNeedTwoSnapshots') }}
        </p>

        <div
          v-else
          class="disk-usage-pickers"
        >
          <label>
            {{ $t('ui.diskUsageBaseline') }}
            <select
              v-model="baselinePath"
              data-testid="disk-usage-baseline"
            >
              <option
                v-for="snapshot in snapshots"
                :key="snapshot.path"
                :value="snapshot.path"
              >
                {{ snapshotLocalLabel(snapshot.takenAtUtc) }}
              </option>
            </select>
          </label>
          <label>
            {{ $t('ui.diskUsageCurrent') }}
            <select
              v-model="currentPath"
              data-testid="disk-usage-current"
            >
              <option
                v-for="snapshot in snapshots"
                :key="snapshot.path"
                :value="snapshot.path"
              >
                {{ snapshotLocalLabel(snapshot.takenAtUtc) }}
              </option>
            </select>
          </label>
          <label class="disk-usage-show-all">
            <input
              v-model="showAll"
              data-testid="disk-usage-show-all"
              type="checkbox"
            />
            {{ $t('ui.diskUsageShowAll') }}
          </label>
        </div>

        <p
          v-if="warning"
          class="disk-usage-warning"
          role="status"
          data-testid="disk-usage-warning"
        >
          {{ warning }}
        </p>

        <DenseDataTable v-if="canCompare">
          <table data-testid="disk-usage-changes-table">
            <thead>
              <tr>
                <th @click="sortBy('folder')">
                  {{ $t('ui.diskUsageFolder') }} {{ sortIndicator('folder') }}
                </th>
                <th @click="sortBy('change')">
                  {{ $t('ui.diskUsageChange') }} {{ sortIndicator('change') }}
                </th>
                <th>{{ $t('ui.diskUsageSizeBefore') }}</th>
                <th>{{ $t('ui.diskUsageSizeNow') }}</th>
                <th
                  data-testid="disk-usage-sort-size-change"
                  @click="sortBy('sizeChange')"
                >
                  {{ $t('ui.diskUsageSizeChange') }} {{ sortIndicator('sizeChange') }}
                </th>
                <th @click="sortBy('filesChange')">
                  {{ $t('ui.diskUsageFilesChange') }} {{ sortIndicator('filesChange') }}
                </th>
                <th></th>
              </tr>
            </thead>
            <tbody>
              <tr v-if="!compareBusy && sortedRows.length === 0">
                <td
                  colspan="7"
                  data-testid="disk-usage-no-changes"
                >
                  {{ $t('ui.diskUsageNoChanges') }}
                </td>
              </tr>
              <tr
                v-for="row in sortedRows"
                :key="row.folder"
                data-testid="disk-usage-change-row"
              >
                <td>{{ row.folder === '.' ? location : row.folder }}</td>
                <td>{{ $t(`ui.diskUsageChangeKind.${row.change}`) }}</td>
                <td>{{ formatBytes(row.baselineSize) }}</td>
                <td>{{ formatBytes(row.currentSize) }}</td>
                <td>{{ formatByteChange(row.sizeChange) }}</td>
                <td>{{ row.filesChange > 0 ? `+${row.filesChange}` : row.filesChange }}</td>
                <td>
                  <button
                    v-if="row.change !== 'removed'"
                    type="button"
                    data-testid="disk-usage-compare-folder"
                    @click="compareFolder(row)"
                  >
                    {{ $t('ui.diskUsageCompareFolder') }}
                  </button>
                </td>
              </tr>
            </tbody>
          </table>
        </DenseDataTable>
      </section>
    </section>
  </WorkbenchShell>
</template>

<style scoped>
.disk-usage-view {
  display: flex;
  flex-direction: column;
  gap: 12px;
  min-width: 0;
  padding: 12px;
}

.disk-usage-location,
.disk-usage-changes > header,
.disk-usage-pickers {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 8px;
}

.disk-usage-location input {
  flex: 1 1 280px;
  min-width: 0;
}

.disk-usage-changes > header h2 {
  margin: 0;
  font-size: 1rem;
}

.disk-usage-drive,
.disk-usage-hint {
  color: var(--app-text-muted);
}

.disk-usage-error {
  color: var(--app-danger, #c42b1c);
}

.disk-usage-warning {
  padding: 6px 8px;
  border: 1px solid var(--app-warning, #9d5d00);
  border-radius: 4px;
}

.disk-usage-changes table {
  width: 100%;
  border-collapse: collapse;
}

.disk-usage-changes th {
  text-align: left;
  white-space: nowrap;
  cursor: pointer;
}

.disk-usage-changes td:nth-child(n + 3):nth-child(-n + 6) {
  text-align: right;
  font-variant-numeric: tabular-nums;
}
</style>
