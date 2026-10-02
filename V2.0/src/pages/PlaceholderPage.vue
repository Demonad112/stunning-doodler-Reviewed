<script setup lang="ts">
import PageHeader from '@/components/PageHeader.vue'
import type { Section } from '@/router'

defineProps<{ section: Section }>()
</script>

<template>
  <div class="mx-auto max-w-5xl px-10 py-8">
    <PageHeader
      :title="section.title"
      :summary="section.summary"
    />
    <div
      class="grid place-items-center rounded-lg border border-dashed border-stroke-strong bg-card px-6 py-16 text-center"
    >
      <component
        :is="section.icon"
        class="size-10 text-faint"
        :stroke-width="1.25"
      />
      <p class="mt-3 font-semibold">Being built</p>
      <p class="mt-1 text-[13px] text-muted">This tool arrives in an upcoming 2.0 build.</p>
    </div>
  </div>
</template>
