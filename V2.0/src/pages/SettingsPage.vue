<script setup lang="ts">
import { Monitor, Moon, Palette, Sun } from '@lucide/vue'
import PageHeader from '@/components/PageHeader.vue'
import { appInfo } from '@/lib/appInfo'
import { setThemeChoice, themeChoice, type ThemeChoice } from '@/lib/theme'

const themes: { value: ThemeChoice; label: string; icon: typeof Sun }[] = [
  { value: 'light', label: 'Light', icon: Sun },
  { value: 'dark', label: 'Dark', icon: Moon },
  { value: 'system', label: 'Use Windows setting', icon: Monitor },
]
</script>

<template>
  <div class="mx-auto max-w-3xl px-10 py-8">
    <PageHeader title="Settings" />
    <section class="flex items-center gap-4 rounded-lg border border-stroke bg-card px-5 py-4">
      <Palette
        class="size-5 shrink-0"
        :stroke-width="1.5"
      />
      <div class="flex-1">
        <p>App theme</p>
        <p class="text-xs text-muted">Choose how DeepServer looks.</p>
      </div>
      <div
        class="flex rounded-md border border-stroke bg-subtle p-0.5"
        role="radiogroup"
        aria-label="App theme"
      >
        <button
          v-for="theme in themes"
          :key="theme.value"
          role="radio"
          :aria-checked="themeChoice === theme.value"
          class="flex h-8 items-center gap-2 rounded px-3 text-[13px]"
          :class="
            themeChoice === theme.value
              ? 'bg-accent text-on-accent'
              : 'text-fg hover:bg-subtle-strong'
          "
          @click="setThemeChoice(theme.value)"
        >
          <component
            :is="theme.icon"
            class="size-3.5"
          />
          {{ theme.label }}
        </button>
      </div>
    </section>
    <p class="mt-6 text-xs text-faint">DeepServer {{ appInfo.version }}</p>
  </div>
</template>
