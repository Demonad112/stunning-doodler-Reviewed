<script setup lang="ts">
import { ref } from 'vue'
import { formatBytes } from '@/app/diskUsage'
import { joinTransferPath, type NotCopiedGroup } from '@/app/transferMonitor'
import type { FailureReason, ItemResult, Selection } from '@/types/transfer'

const props = defineProps<{
  groups: NotCopiedGroup[]
  source: string
  destination: string
  /** A copy, retry or watch is running. */
  busy: boolean
}>()

const emit = defineEmits<{
  retry: [selection: Selection]
  recover: [selection: Selection]
  reveal: [path: string]
}>()

/** Rows shown per group before "Show more". */
const pageSize = 100
const shown = ref<Partial<Record<FailureReason, number>>>({})
const copiedGroup = ref<FailureReason | null>(null)

/** Retrying these cannot help until the name is changed in the source. */
const notRetryable = new Set<FailureReason>(['invalidName', 'nameCollision'])

function visibleItems(group: NotCopiedGroup): ItemResult[] {
  return group.items.slice(0, shown.value[group.reason] ?? pageSize)
}

function showMore(group: NotCopiedGroup): void {
  shown.value = {
    ...shown.value,
    [group.reason]: (shown.value[group.reason] ?? pageSize) + pageSize,
  }
}

async function copyList(group: NotCopiedGroup): Promise<void> {
  const lines = group.items.map((item) => joinTransferPath(props.source, item.relativePath))

  try {
    await navigator.clipboard.writeText(lines.join('\r\n'))
    copiedGroup.value = group.reason
  } catch {
    copiedGroup.value = null
  }
}
</script>

<template>
  <section
    class="not-copied-panel"
    data-testid="transfer-not-copied"
  >
    <p
      v-if="groups.length === 0"
      class="not-copied-empty"
      data-testid="transfer-not-copied-empty"
    >
      {{ $t('ui.transferNothingNotCopied') }}
    </p>

    <article
      v-for="group in groups"
      :key="group.reason"
      class="not-copied-group"
      :data-reason="group.reason"
      data-testid="transfer-not-copied-group"
    >
      <header>
        <div class="not-copied-title">
          <h3>{{ $t(`ui.transferReason.${group.reason}`) }}</h3>
          <span data-testid="transfer-group-count">
            {{
              $t('ui.transferGroupCount', {
                count: group.items.length,
                size: formatBytes(group.bytes),
              })
            }}
          </span>
          <span
            v-if="group.inferred"
            class="not-copied-inferred"
            :title="$t('ui.transferInferredHint')"
            >{{ $t('ui.transferInferred') }}</span
          >
        </div>
        <p class="not-copied-hint">{{ $t(`ui.transferReasonHint.${group.reason}`) }}</p>
        <div class="not-copied-actions">
          <button
            type="button"
            data-testid="transfer-retry-group"
            :disabled="busy || notRetryable.has(group.reason)"
            @click="emit('retry', { reason: group.reason })"
          >
            {{ $t('ui.transferRetryGroup') }}
          </button>
          <button
            type="button"
            data-testid="transfer-recover-group"
            :disabled="busy || group.recoverable === 0"
            :title="group.recoverable === 0 ? $t('ui.transferRecoverUnavailable') : undefined"
            @click="emit('recover', { reason: group.reason })"
          >
            {{ $t('ui.transferRecoverGroup') }}
          </button>
          <button
            type="button"
            data-testid="transfer-copy-list"
            @click="copyList(group)"
          >
            {{
              copiedGroup === group.reason ? $t('ui.transferListCopied') : $t('ui.transferCopyList')
            }}
          </button>
        </div>
      </header>

      <table>
        <tbody>
          <tr
            v-for="item in visibleItems(group)"
            :key="item.relativePath"
            data-testid="transfer-not-copied-row"
          >
            <td
              class="not-copied-path"
              :title="joinTransferPath(source, item.relativePath)"
            >
              {{ item.relativePath }}{{ item.kind === 'folder' ? '\\' : '' }}
            </td>
            <td class="not-copied-size">
              {{ item.kind === 'file' ? formatBytes(item.size) : '' }}
            </td>
            <td class="not-copied-message">{{ item.message }}</td>
            <td class="not-copied-row-actions">
              <button
                type="button"
                data-testid="transfer-retry-row"
                :disabled="busy || notRetryable.has(group.reason)"
                @click="emit('retry', { paths: [item.relativePath] })"
              >
                {{ $t('ui.transferRetry') }}
              </button>
              <button
                type="button"
                data-testid="transfer-reveal-source"
                @click="emit('reveal', joinTransferPath(source, item.relativePath))"
              >
                {{ $t('ui.transferRevealSource') }}
              </button>
              <button
                type="button"
                data-testid="transfer-reveal-destination"
                @click="emit('reveal', joinTransferPath(destination, item.relativePath))"
              >
                {{ $t('ui.transferRevealDestination') }}
              </button>
            </td>
          </tr>
        </tbody>
      </table>
      <button
        v-if="group.items.length > visibleItems(group).length"
        type="button"
        class="not-copied-more"
        data-testid="transfer-show-more"
        @click="showMore(group)"
      >
        {{ $t('ui.transferShowMore', { count: group.items.length - visibleItems(group).length }) }}
      </button>
    </article>
  </section>
</template>

<style scoped>
.not-copied-panel {
  display: flex;
  flex-direction: column;
  gap: 10px;
  min-width: 0;
}

.not-copied-empty,
.not-copied-hint {
  margin: 0;
  color: var(--app-text-muted);
}

.not-copied-group {
  padding: 8px;
  border: 1px solid var(--app-border);
  border-left: 3px solid var(--app-danger, #c42b1c);
  border-radius: 4px;
}

.not-copied-group > header {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.not-copied-title,
.not-copied-actions {
  display: flex;
  flex-wrap: wrap;
  align-items: baseline;
  gap: 8px;
}

.not-copied-title h3 {
  margin: 0;
  font-size: 0.95rem;
}

.not-copied-inferred {
  padding: 0 6px;
  border: 1px solid var(--app-border);
  border-radius: 8px;
  font-size: 0.8rem;
}

.not-copied-group table {
  width: 100%;
  margin-top: 6px;
  border-collapse: collapse;
  table-layout: fixed;
}

.not-copied-group td {
  padding: 2px 4px;
  border-top: 1px solid var(--app-border);
  vertical-align: top;
}

.not-copied-path {
  width: 42%;
  overflow-wrap: anywhere;
}

.not-copied-size {
  width: 10%;
  text-align: right;
  font-variant-numeric: tabular-nums;
  white-space: nowrap;
}

.not-copied-message {
  color: var(--app-text-muted);
  overflow-wrap: anywhere;
}

.not-copied-row-actions {
  width: 250px;
  text-align: right;
  white-space: nowrap;
}

.not-copied-more {
  margin-top: 4px;
}
</style>
