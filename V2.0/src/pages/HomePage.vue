<script setup lang="ts">
import { ChevronRight } from '@lucide/vue'
import { computed } from 'vue'
import PageHeader from '@/components/PageHeader.vue'
import { appInfo } from '@/lib/appInfo'
import { sections } from '@/router'

const tools = sections.slice(1)
const summary = computed(() =>
  appInfo.value.machine
    ? `Working on ${appInfo.value.machine}. Pick a tool to begin.`
    : 'Pick a tool to begin.',
)
</script>

<template>
  <div class="mx-auto max-w-5xl px-10 py-8">
    <PageHeader
      title="Welcome to DeepServer"
      :summary="summary"
    />
    <div class="grid grid-cols-3 gap-4">
      <RouterLink
        v-for="tool in tools"
        :key="tool.path"
        :to="tool.path"
        class="group flex flex-col rounded-lg border border-stroke bg-card p-5 shadow-xs transition-colors hover:bg-card-hover"
      >
        <span
          class="grid size-11 place-items-center rounded-lg bg-accent/12 text-accent"
          aria-hidden="true"
        >
          <component
            :is="tool.icon"
            class="size-5"
            :stroke-width="1.75"
          />
        </span>
        <span class="mt-4 flex items-center gap-1 font-semibold">
          {{ tool.title }}
          <ChevronRight
            class="size-4 text-faint transition-transform group-hover:translate-x-0.5"
          />
        </span>
        <span class="mt-1 text-[13px] leading-5 text-muted">{{ tool.summary }}</span>
      </RouterLink>
    </div>
  </div>
</template>
