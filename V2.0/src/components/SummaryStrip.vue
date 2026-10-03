<script setup lang="ts">
import { computed } from 'vue'
import type { DiffSummary } from '@/lib/compare'
import { formatBytes, formatDelta, plural } from '@/lib/format'

const props = defineProps<{ summary: DiffSummary }>()

const delta = computed(() => props.summary.right.size - props.summary.left.size)
const deltaText = computed(() => {
  if (delta.value === 0) {
    return 'same size'
  }
  return formatDelta(delta.value)
})
</script>

<template>
  <div class="grid grid-cols-4 gap-3">
    <div class="rounded-lg border border-stroke bg-card px-4 py-3">
      <p class="text-xs text-muted">Source</p>
      <p class="mt-1 font-display text-xl font-semibold tabular-nums">
        {{ formatBytes(summary.left.size) }}
      </p>
      <p class="mt-0.5 text-xs text-muted tabular-nums">
        {{ plural(summary.left.files, 'file') }} · {{ plural(summary.left.dirs, 'folder') }}
      </p>
    </div>
    <div class="rounded-lg border border-stroke bg-card px-4 py-3">
      <p class="text-xs text-muted">Destination</p>
      <p class="mt-1 font-display text-xl font-semibold tabular-nums">
        {{ formatBytes(summary.right.size) }}
      </p>
      <p class="mt-0.5 text-xs text-muted tabular-nums">
        {{ plural(summary.right.files, 'file') }} · {{ deltaText }}
      </p>
    </div>
    <div
      class="rounded-lg border px-4 py-3"
      :class="summary.missing > 0 ? 'border-danger/40 bg-danger-bg' : 'border-stroke bg-card'"
      data-testid="missing-tile"
    >
      <p class="text-xs text-muted">Missing at destination</p>
      <p
        class="mt-1 font-display text-xl font-semibold tabular-nums"
        :class="summary.missing > 0 ? 'text-danger' : 'text-success'"
      >
        {{ summary.missing > 0 ? plural(summary.missing, 'file') : 'None' }}
      </p>
      <p class="mt-0.5 text-xs text-muted tabular-nums">
        {{
          summary.missing > 0
            ? `${formatBytes(summary.missingBytes)} not copied`
            : 'Every source file is there'
        }}
      </p>
    </div>
    <div class="rounded-lg border border-stroke bg-card px-4 py-3">
      <p class="text-xs text-muted">Other differences</p>
      <p
        class="mt-1 font-display text-xl font-semibold tabular-nums"
        :class="summary.different > 0 ? 'text-warning' : ''"
      >
        {{ plural(summary.different, 'size change', 'size changes') }}
      </p>
      <p class="mt-0.5 text-xs text-muted tabular-nums">
        {{ plural(summary.extra, 'file') }} only at destination
      </p>
    </div>
  </div>
</template>
