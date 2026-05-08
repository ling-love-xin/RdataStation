<template>
  <div class="filter-preset-selector">
    <NSelect
      v-model:value="selectedPresetId"
      :options="presetOptions"
      :placeholder="t('resultPanel.filterPreset.placeholder')"
      clearable
      size="small"
      class="preset-select"
      @update:value="onSelectPreset"
    >
      <template #header>
        <div class="preset-header">
          <span>{{ t('resultPanel.filterPreset.header') }}</span>
          <NButton size="tiny" quaternary @click.stop="onSaveCurrent">
            <Save :size="14" />
          </NButton>
        </div>
      </template>
      <template #action>
        <div v-if="selectedPresetId" class="preset-action">
          <NButton size="tiny" type="error" quaternary @click.stop="onDeleteSelected">
            <Trash2 :size="14" />
          </NButton>
        </div>
      </template>
    </NSelect>
  </div>
</template>

<script setup lang="ts">
import { Save, Trash2 } from 'lucide-vue-next'
import { NButton, NSelect } from 'naive-ui'
import { computed, ref } from 'vue'
import { useI18n } from 'vue-i18n'

import { useFilterPresets } from '../../../composables/useFilterPresets'

import type { FilterMode } from '../../../types/result'
import type { SelectOption } from 'naive-ui'

export interface PresetSelectEvent {
  id: string
  name: string
  filterMode: FilterMode
  expression: string
}

const props = defineProps<{
  filterMode: FilterMode
  currentExpression: string
}>()

const emit = defineEmits<{
  select: [event: PresetSelectEvent]
  save: [name: string, expression: string, mode: FilterMode]
  delete: [id: string]
}>()

const { t } = useI18n()
const { presets, getPresetsByMode, removePreset } = useFilterPresets()

const selectedPresetId = ref<string | null>(null)

const filteredPresets = computed(() => {
  if (props.filterMode) {
    return getPresetsByMode(props.filterMode)
  }
  return presets.value
})

const presetOptions = computed<SelectOption[]>(() =>
  filteredPresets.value.map(p => ({
    label: p.name,
    value: p.id,
    disabled: false,
  }))
)

function onSelectPreset(value: string | null) {
  if (!value) return
  selectedPresetId.value = value
  const preset = presets.value.find(p => p.id === value)
  if (preset) {
    emit('select', {
      id: preset.id,
      name: preset.name,
      filterMode: preset.filterMode,
      expression: preset.expression,
    })
  }
}

function onSaveCurrent() {
  const name = prompt(t('resultPanel.filterPreset.savePrompt') ?? 'Preset name:')
  if (name && name.trim()) {
    emit('save', name.trim(), props.currentExpression, props.filterMode)
  }
}

function onDeleteSelected() {
  if (!selectedPresetId.value) return
  const preset = presets.value.find(p => p.id === selectedPresetId.value)
  if (preset && confirm(`删除预设 "${preset.name}"？`)) {
    removePreset(selectedPresetId.value)
    emit('delete', selectedPresetId.value)
    selectedPresetId.value = null
  }
}
</script>

<style scoped>
.filter-preset-selector {
  display: flex;
  align-items: center;
}

.preset-select {
  min-width: 150px;
  max-width: 240px;
}

.preset-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 4px 8px;
  font-size: 12px;
  color: var(--text-color-secondary);
}

.preset-action {
  display: flex;
  justify-content: flex-end;
  padding: 4px 8px;
}
</style>
