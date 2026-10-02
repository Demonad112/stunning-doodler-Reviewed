<script setup lang="ts">
import { sections, settingsSection } from '@/router'
</script>

<template>
  <nav
    class="flex w-56 shrink-0 flex-col gap-1 px-1.5 pb-2"
    aria-label="Sections"
  >
    <RouterLink
      v-for="section in [...sections, settingsSection]"
      :key="section.path"
      :to="section.path"
      class="relative flex h-9 items-center gap-3 rounded-md px-3 text-fg hover:bg-subtle"
      :class="{ 'mt-auto': section === settingsSection }"
      active-class=""
      exact-active-class="bg-subtle-strong is-active"
    >
      <component
        :is="section.icon"
        class="size-4"
        :stroke-width="1.75"
      />
      <span>{{ section.title }}</span>
    </RouterLink>
  </nav>
</template>

<style scoped>
/* Fluent selection pill on the active item. */
.is-active::before {
  position: absolute;
  top: 50%;
  left: 0;
  width: 3px;
  height: 16px;
  border-radius: 2px;
  background: var(--app-accent);
  content: '';
  transform: translateY(-50%);
}
</style>
