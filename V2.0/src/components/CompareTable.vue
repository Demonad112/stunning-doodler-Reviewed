<script setup lang="ts">
import { ChevronRight, Cloud, File, Folder, Link2, LoaderCircle, TriangleAlert } from '@lucide/vue'
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import StatusChip from '@/components/StatusChip.vue'
import type { DiffSide } from '@/lib/compare'
import { sizeDelta, statusLabel, type VisibleRow } from '@/lib/diffRows'
import { formatBytes, formatDelta } from '@/lib/format'

const props = defineProps<{ rows: VisibleRow[]; loading: ReadonlySet<number> }>()
const emit = defineEmits<{ toggle: [id: number] }>()

/** Fixed row height so only the rows on screen are rendered (a fully expanded tree can be huge). */
const rowHeight = 32
const overscan = 12

const viewport = ref<HTMLElement | null>(null)
const scrollTop = ref(0)
const viewportHeight = ref(480)
let observer: ResizeObserver | undefined

onMounted(() => {
  if (viewport.value && typeof ResizeObserver !== 'undefined') {
    observer = new ResizeObserver(([entry]) => {
      if (entry) {
        viewportHeight.value = entry.contentRect.height
      }
    })
    observer.observe(viewport.value)
  }
})
onBeforeUnmount(() => observer?.disconnect())

// A filter change can shrink the list below the current scroll position.
watch(
  () => props.rows.length,
  (length) => {
    const max = Math.max(0, length * rowHeight - viewportHeight.value)
    if (viewport.value && scrollTop.value > max) {
      viewport.value.scrollTop = max
      scrollTop.value = max
    }
  },
)

const first = computed(() => Math.max(0, Math.floor(scrollTop.value / rowHeight) - overscan))
const last = computed(() =>
  Math.min(
    props.rows.length,
    Math.ceil((scrollTop.value + viewportHeight.value) / rowHeight) + overscan,
  ),
)
const shown = computed(() => props.rows.slice(first.value, last.value))

function onScroll(event: Event): void {
  scrollTop.value = (event.target as HTMLElement).scrollTop
}

function size(side: DiffSide | null): string {
  return side ? formatBytes(side.size) : '—'
}

function delta(visible: VisibleRow): string {
  const bytes = sizeDelta(visible.row)
  return bytes === 0 ? '—' : formatDelta(bytes)
}

function onRowClick(visible: VisibleRow): void {
  if (visible.row.hasChildren) {
    emit('toggle', visible.row.id)
  }
}

const columns = 'grid grid-cols-[minmax(0,1fr)_6.5rem_6.5rem_6.5rem_9rem] items-center gap-3 px-3'
</script>

<template>
  <div
    class="flex min-h-0 flex-col overflow-hidden rounded-lg border border-stroke bg-card"
    role="treegrid"
    aria-label="Compare result"
    :aria-rowcount="rows.length"
  >
    <div
      :class="columns"
      class="h-9 shrink-0 border-b border-stroke text-xs font-semibold text-muted"
      role="row"
    >
      <span role="columnheader">Name</span>
      <span
        role="columnheader"
        class="text-right"
        >Source</span
      >
      <span
        role="columnheader"
        class="text-right"
        >Destination</span
      >
      <span
        role="columnheader"
        class="text-right"
        >Difference</span
      >
      <span role="columnheader">Status</span>
    </div>
    <div
      ref="viewport"
      class="min-h-0 flex-1 overflow-auto"
      @scroll.passive="onScroll"
    >
      <div
        class="relative"
        :style="{ height: `${String(rows.length * rowHeight)}px` }"
      >
        <div
          class="absolute inset-x-0"
          :style="{ top: `${String(first * rowHeight)}px` }"
        >
          <div
            v-for="visible in shown"
            :key="visible.row.id"
            :class="[
              columns,
              visible.row.hasChildren ? 'cursor-default' : '',
              visible.row.status === 'onlyLeft' ? 'bg-danger-bg/60 text-danger' : 'hover:bg-subtle',
            ]"
            class="text-[13px]"
            :style="{ height: `${String(rowHeight)}px` }"
            role="row"
            :aria-level="visible.depth + 1"
            :aria-expanded="visible.row.hasChildren ? visible.expanded : undefined"
            :data-status="visible.row.status"
            @click="onRowClick(visible)"
          >
            <span
              class="flex min-w-0 items-center gap-1.5"
              :style="{ paddingLeft: `${String(visible.depth * 20)}px` }"
              role="gridcell"
            >
              <button
                v-if="visible.row.hasChildren"
                type="button"
                class="grid size-5 shrink-0 place-items-center rounded text-muted hover:bg-subtle-strong"
                :aria-label="visible.expanded ? 'Collapse' : 'Expand'"
                @click.stop="emit('toggle', visible.row.id)"
              >
                <LoaderCircle
                  v-if="loading.has(visible.row.id)"
                  class="size-3.5 animate-spin"
                />
                <ChevronRight
                  v-else
                  class="size-3.5 transition-transform"
                  :class="{ 'rotate-90': visible.expanded }"
                />
              </button>
              <span
                v-else
                class="w-5 shrink-0"
              />
              <component
                :is="
                  visible.row.kind === 'dir' ? Folder : visible.row.kind === 'link' ? Link2 : File
                "
                class="size-4 shrink-0"
                :class="visible.row.status === 'onlyLeft' ? '' : 'text-muted'"
                :stroke-width="1.75"
              />
              <span
                class="truncate"
                :title="visible.row.name"
                >{{ visible.row.name }}</span
              >
              <span
                v-if="visible.row.cloud"
                class="shrink-0 text-faint"
                title="Includes online-only cloud files: size shown, data not on this PC"
              >
                <Cloud
                  class="size-3.5"
                  :stroke-width="1.75"
                />
              </span>
              <span
                v-if="visible.row.error"
                class="shrink-0 text-warning"
                :title="`Not fully readable: ${visible.row.error}`"
              >
                <TriangleAlert
                  class="size-3.5"
                  :stroke-width="1.75"
                />
              </span>
            </span>
            <span
              class="text-right tabular-nums"
              role="gridcell"
              >{{ size(visible.row.left) }}</span
            >
            <span
              class="text-right tabular-nums"
              :class="visible.row.right ? '' : 'text-faint'"
              role="gridcell"
              >{{ size(visible.row.right) }}</span
            >
            <span
              class="text-right tabular-nums"
              :class="visible.row.status === 'onlyLeft' ? '' : 'text-muted'"
              role="gridcell"
              >{{ delta(visible) }}</span
            >
            <span
              class="min-w-0"
              role="gridcell"
            >
              <StatusChip v-bind="statusLabel(visible.row)" />
            </span>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
