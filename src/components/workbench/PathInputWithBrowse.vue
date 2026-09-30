<script setup lang="ts">
import { computed } from 'vue'
import { pickNativePath } from '@/app/filePicker'
import { useI18n } from '@/i18n'

/** A path text box with a Browse button and optional suggestions (E22: no typed-only paths). */
const props = withDefaults(
  defineProps<{
    testId: string
    placeholder?: string
    /** Pick a folder instead of a file. */
    directory?: boolean
    /** Offered in the box's dropdown, e.g. programs found on this PC. */
    suggestions?: (string | null)[]
  }>(),
  { placeholder: undefined, directory: false, suggestions: () => [] },
)
const model = defineModel<string>({ required: true })
const emit = defineEmits<{
  change: []
}>()
const { t } = useI18n()

const listId = computed(() => `${props.testId}-suggestions`)
const choices = computed(() =>
  props.suggestions.filter((item): item is string => Boolean(item?.trim())),
)

async function browse(): Promise<void> {
  const picked = await pickNativePath({ directory: props.directory })

  if (picked) {
    model.value = picked
    emit('change')
  }
}
</script>

<template>
  <span class="path-input-with-browse">
    <input
      v-model="model"
      type="text"
      :data-testid="testId"
      :placeholder="placeholder"
      :list="choices.length > 0 ? listId : undefined"
      @change="emit('change')"
    />
    <datalist
      v-if="choices.length > 0"
      :id="listId"
    >
      <option
        v-for="choice in choices"
        :key="choice"
        :value="choice"
      />
    </datalist>
    <button
      type="button"
      :data-testid="`${testId}-browse`"
      @click="browse"
    >
      {{ t('ui.browse') }}
    </button>
  </span>
</template>

<style scoped>
.path-input-with-browse {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  min-width: 0;
}

.path-input-with-browse input {
  flex: 1 1 auto;
  min-width: 0;
}
</style>
