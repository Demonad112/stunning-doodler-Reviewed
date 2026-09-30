<script setup lang="ts">
import { computed } from 'vue'
import { formatBytes } from '@/app/diskUsage'
import type { ItemResult } from '@/types/transfer'

const props = defineProps<{
  /** Oldest first; the store keeps the last few hundred. */
  items: ItemResult[]
}>()

const newestFirst = computed(() => [...props.items].reverse())

function timeOf(item: ItemResult): string {
  return new Date(item.atMs).toLocaleTimeString()
}
</script>

<template>
  <section
    class="live-feed"
    data-testid="transfer-live-feed"
  >
    <p
      v-if="items.length === 0"
      class="live-feed-empty"
    >
      {{ $t('ui.transferFeedEmpty') }}
    </p>
    <ol v-else>
      <li
        v-for="item in newestFirst"
        :key="`${item.relativePath}-${String(item.atMs)}`"
        data-testid="transfer-feed-row"
      >
        <span class="live-feed-time">{{ timeOf(item) }}</span>
        <span
          class="live-feed-path"
          :title="item.sourceHash ? `BLAKE3 ${item.sourceHash}` : undefined"
          >{{ item.relativePath }}</span
        >
        <span class="live-feed-size">{{ formatBytes(item.size) }}</span>
        <span
          v-if="item.sourceHash"
          class="live-feed-hash"
          >{{ $t('ui.transferHashChecked') }}</span
        >
      </li>
    </ol>
  </section>
</template>

<style scoped>
.live-feed {
  min-width: 0;
  max-height: 420px;
  overflow: auto;
}

.live-feed-empty {
  margin: 0;
  color: var(--app-text-muted);
}

.live-feed ol {
  margin: 0;
  padding: 0;
  list-style: none;
}

.live-feed li {
  display: flex;
  gap: 8px;
  padding: 1px 0;
  border-bottom: 1px solid var(--app-border);
}

.live-feed-time,
.live-feed-size,
.live-feed-hash {
  flex: none;
  color: var(--app-text-muted);
  font-variant-numeric: tabular-nums;
}

.live-feed-path {
  flex: 1 1 auto;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
</style>
