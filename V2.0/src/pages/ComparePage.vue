<script setup lang="ts">
import { isTauri } from '@tauri-apps/api/core'
import { getCurrentWebview } from '@tauri-apps/api/webview'
import { ArrowLeftRight, CircleCheck, Info, LoaderCircle, TriangleAlert } from '@lucide/vue'
import { computed, onBeforeUnmount, onMounted, reactive, ref, shallowRef, watch } from 'vue'
import CompareTable from '@/components/CompareTable.vue'
import PageHeader from '@/components/PageHeader.vue'
import PathField from '@/components/PathField.vue'
import SummaryStrip from '@/components/SummaryStrip.vue'
import {
  cancelCompare,
  errorMessage,
  isCancelled,
  loadChildren,
  loadPaths,
  pickFolder,
  savePaths,
  startCompare,
  type CompareProgress,
  type CompareResult,
  type DiffRow,
} from '@/lib/compare'
import { visibleRows, type CompareFilter } from '@/lib/diffRows'
import { formatBytes, formatDuration, plural } from '@/lib/format'
import { sections } from '@/router'

type Side = 'left' | 'right'

const section = sections.find((candidate) => candidate.path === '/compare')
const paths = reactive(loadPaths())
watch(paths, () => savePaths(paths))

const running = ref(false)
const progress = ref<CompareProgress | null>(null)
const result = shallowRef<CompareResult | null>(null)
const compared = ref({ left: '', right: '' })
const error = ref('')

const children = shallowRef(new Map<number, DiffRow[]>())
const expanded = shallowRef(new Set<number>())
const loading = shallowRef(new Set<number>())
const filter = ref<CompareFilter>('all')

const canCompare = computed(
  () => !running.value && paths.left.trim() !== '' && paths.right.trim() !== '',
)
const rows = computed(() =>
  result.value ? visibleRows(result.value.rows, children.value, expanded.value, filter.value) : [],
)

const unreadable = computed(() =>
  result.value ? result.value.summary.leftErrors + result.value.summary.rightErrors : 0,
)
const cloudFiles = computed(() =>
  result.value ? result.value.summary.leftCloud + result.value.summary.rightCloud : 0,
)

const filters: { value: CompareFilter; label: string }[] = [
  { value: 'all', label: 'All' },
  { value: 'changes', label: 'Differences' },
  { value: 'missing', label: 'Missing' },
]

async function runCompare(): Promise<void> {
  if (!canCompare.value) {
    return
  }
  const left = paths.left.trim()
  const right = paths.right.trim()
  running.value = true
  error.value = ''
  progress.value = null
  result.value = null
  children.value = new Map()
  expanded.value = new Set()
  loading.value = new Set()
  try {
    result.value = await startCompare(left, right, (update) => {
      progress.value = update
    })
    compared.value = { left, right }
    filter.value = result.value.summary.missing > 0 ? 'missing' : 'all'
  } catch (err) {
    if (!isCancelled(err)) {
      error.value = errorMessage(err)
    }
  } finally {
    running.value = false
  }
}

async function cancel(): Promise<void> {
  await cancelCompare()
}

async function browse(side: Side): Promise<void> {
  const picked = await pickFolder(
    side === 'left' ? 'Choose the source folder' : 'Choose the destination folder',
    paths[side],
  )
  if (picked) {
    paths[side] = picked
  }
}

function swap(): void {
  ;[paths.left, paths.right] = [paths.right, paths.left]
}

async function toggle(id: number): Promise<void> {
  const next = new Set(expanded.value)
  if (next.has(id)) {
    next.delete(id)
    expanded.value = next
    return
  }
  next.add(id)
  expanded.value = next
  if (children.value.has(id) || loading.value.has(id)) {
    return
  }
  loading.value = new Set(loading.value).add(id)
  try {
    const loaded = await loadChildren(id)
    children.value = new Map(children.value).set(id, loaded)
  } catch (err) {
    error.value = errorMessage(err)
    const collapsed = new Set(expanded.value)
    collapsed.delete(id)
    expanded.value = collapsed
  } finally {
    const done = new Set(loading.value)
    done.delete(id)
    loading.value = done
  }
}

// Dropping a folder from Explorer onto either field fills it; dropping elsewhere fills the first
// empty field.
const fieldRefs = reactive<Record<Side, HTMLElement | null>>({ left: null, right: null })
const dropSide = ref<Side | null>(null)
let stopDragDrop: (() => void) | undefined

function sideAt(x: number, y: number): Side | null {
  for (const side of ['left', 'right'] as const) {
    const box = fieldRefs[side]?.getBoundingClientRect()
    if (box && x >= box.left && x <= box.right && y >= box.top && y <= box.bottom) {
      return side
    }
  }
  return null
}

onMounted(async () => {
  if (!isTauri()) {
    return
  }
  stopDragDrop = await getCurrentWebview().onDragDropEvent(({ payload }) => {
    if (payload.type === 'leave') {
      dropSide.value = null
      return
    }
    const scale = window.devicePixelRatio || 1
    const side = sideAt(payload.position.x / scale, payload.position.y / scale)
    if (payload.type === 'drop') {
      dropSide.value = null
      const [path] = payload.paths
      const target = side ?? (paths.left ? 'right' : 'left')
      if (path && !running.value) {
        paths[target] = path
      }
      return
    }
    dropSide.value = side
  })
})
onBeforeUnmount(() => stopDragDrop?.())
</script>

<template>
  <div class="mx-auto flex h-full max-w-6xl min-w-[44rem] flex-col px-10 py-8">
    <div class="flex items-start justify-between gap-6">
      <PageHeader
        :title="section?.title ?? 'Compare'"
        summary="Compare two folders by size and find every file that is missing at the destination."
      />
      <button
        v-if="!running"
        type="button"
        class="mt-1.5 h-8 shrink-0 rounded-md bg-accent px-5 font-semibold text-on-accent hover:bg-accent-hover disabled:opacity-50"
        :disabled="!canCompare"
        @click="runCompare"
      >
        Compare
      </button>
      <button
        v-else
        type="button"
        class="mt-1.5 h-8 shrink-0 rounded-md border border-stroke bg-card px-5 hover:bg-card-hover"
        @click="cancel"
      >
        Cancel
      </button>
    </div>

    <div class="grid grid-cols-[minmax(0,1fr)_auto_minmax(0,1fr)] items-center gap-2">
      <div :ref="(el) => (fieldRefs.left = el as HTMLElement | null)">
        <PathField
          v-model="paths.left"
          label="Source"
          hint="The folder that was copied from"
          :disabled="running"
          :drop-target="dropSide === 'left'"
          @browse="browse('left')"
          @submit="runCompare"
        />
      </div>
      <button
        type="button"
        class="grid size-8 place-items-center rounded-md text-muted hover:bg-subtle-strong disabled:opacity-50"
        title="Swap source and destination"
        aria-label="Swap source and destination"
        :disabled="running"
        @click="swap"
      >
        <ArrowLeftRight class="size-4" />
      </button>
      <div :ref="(el) => (fieldRefs.right = el as HTMLElement | null)">
        <PathField
          v-model="paths.right"
          label="Destination"
          hint="The folder that was copied to"
          :disabled="running"
          :drop-target="dropSide === 'right'"
          @browse="browse('right')"
          @submit="runCompare"
        />
      </div>
    </div>

    <p
      v-if="error"
      class="mt-4 flex items-start gap-2 rounded-lg border border-danger/40 bg-danger-bg px-4 py-3 text-[13px] text-danger"
      role="alert"
    >
      <TriangleAlert class="mt-0.5 size-4 shrink-0" />
      {{ error }}
    </p>

    <section
      v-if="running"
      class="mt-6 rounded-lg border border-stroke bg-card px-5 py-4"
      aria-live="polite"
    >
      <p class="flex items-center gap-2 font-semibold">
        <LoaderCircle class="size-4 animate-spin text-accent" />
        Reading both folders…
      </p>
      <div class="indeterminate mt-3 h-1 overflow-hidden rounded-full bg-subtle-strong" />
      <div class="mt-3 grid grid-cols-2 gap-6">
        <div
          v-for="side in ['left', 'right'] as const"
          :key="side"
          class="min-w-0"
        >
          <p class="text-xs text-muted">{{ side === 'left' ? 'Source' : 'Destination' }}</p>
          <p class="tabular-nums">
            {{ plural(progress?.[side].files ?? 0, 'file') }} ·
            {{ formatBytes(progress?.[side].bytes ?? 0) }}
          </p>
          <p
            class="truncate text-xs text-faint"
            :title="progress?.[side].current"
          >
            {{ progress?.[side].current || ' ' }}
          </p>
        </div>
      </div>
    </section>

    <template v-else-if="result">
      <div class="mt-5">
        <SummaryStrip :summary="result.summary" />
      </div>
      <div
        v-if="unreadable > 0 || cloudFiles > 0"
        class="mt-3 flex flex-wrap gap-x-6 gap-y-1 text-[13px]"
      >
        <p
          v-if="unreadable > 0"
          class="flex items-center gap-2 text-warning"
        >
          <TriangleAlert class="size-4 shrink-0" />
          {{ plural(unreadable, 'folder') }} not fully readable (marked below); sizes there may be
          low.
        </p>
        <p
          v-if="cloudFiles > 0"
          class="flex items-center gap-2 text-muted"
        >
          <Info class="size-4 shrink-0" />
          {{ plural(cloudFiles, 'online-only cloud file') }}: size compared, nothing downloaded.
        </p>
      </div>

      <div class="mt-4 flex items-center justify-between gap-4">
        <div
          class="flex rounded-md border border-stroke bg-subtle p-0.5"
          role="radiogroup"
          aria-label="Show"
        >
          <button
            v-for="option in filters"
            :key="option.value"
            type="button"
            role="radio"
            :aria-checked="filter === option.value"
            class="flex h-7 items-center rounded px-3 text-[13px]"
            :class="
              filter === option.value
                ? 'bg-accent text-on-accent'
                : 'text-fg hover:bg-subtle-strong'
            "
            @click="filter = option.value"
          >
            {{ option.label }}
          </button>
        </div>
        <p
          class="truncate text-xs text-muted tabular-nums"
          :title="`${compared.left} → ${compared.right}`"
        >
          {{ plural(rows.length, 'row') }} · compared in {{ formatDuration(result.elapsedMs) }}
        </p>
      </div>

      <CompareTable
        v-if="rows.length > 0"
        class="mt-3 min-h-64 flex-1"
        :rows="rows"
        :loading="loading"
        @toggle="toggle"
      />
      <div
        v-else
        class="mt-3 grid min-h-40 flex-1 place-items-center rounded-lg border border-stroke bg-card text-center"
      >
        <div>
          <CircleCheck
            class="mx-auto size-8 text-success"
            :stroke-width="1.5"
          />
          <p class="mt-2 font-semibold">
            {{
              filter === 'missing'
                ? 'Nothing is missing'
                : filter === 'changes'
                  ? 'No differences'
                  : 'Both folders are empty'
            }}
          </p>
          <p class="mt-1 text-[13px] text-muted">
            {{
              filter === 'missing'
                ? 'Every source file is at the destination.'
                : filter === 'changes'
                  ? 'Both folders hold the same files with the same sizes.'
                  : 'There is nothing to compare.'
            }}
          </p>
        </div>
      </div>
    </template>

    <div
      v-else-if="!error"
      class="mt-6 grid flex-1 place-items-center rounded-lg border border-dashed border-stroke-strong px-6 py-12 text-center"
    >
      <div>
        <component
          :is="section?.icon"
          class="mx-auto size-10 text-faint"
          :stroke-width="1.25"
        />
        <p class="mt-3 font-semibold">Pick two folders</p>
        <p class="mt-1 max-w-md text-[13px] text-muted">
          Folder sizes include everything inside them. Files that are in the source but not at the
          destination are shown in red.
        </p>
      </div>
    </div>
  </div>
</template>

<style scoped>
/* Fluent indeterminate progress: a short accent bar sliding across. */
.indeterminate {
  position: relative;
}

.indeterminate::after {
  position: absolute;
  inset: 0 auto 0 0;
  width: 30%;
  border-radius: 999px;
  background: var(--app-accent);
  animation: slide 1.4s ease-in-out infinite;
  content: '';
}

@keyframes slide {
  from {
    transform: translateX(-100%);
  }

  to {
    transform: translateX(340%);
  }
}
</style>
