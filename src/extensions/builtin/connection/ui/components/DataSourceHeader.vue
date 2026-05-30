<template>
  <div class="right-header">
    <!-- Row 1: Name + Scope -->
    <div class="rh-row">
      <span class="rh-label">{{ nameLabel }}</span>
      <NInput
        :value="name"
        :placeholder="namePlaceholder"
        size="small"
        class="rh-name-input"
        @update:value="(v: string) => emit('update:name', v)"
      />
      <NCheckbox
        :checked="scopeGlobal"
        size="small"
        @update:checked="(v: boolean) => emit('update:scopeGlobal', v)"
      >
        {{ globalLabel }}
      </NCheckbox>
      <NCheckbox
        :checked="scopeProject"
        size="small"
        @update:checked="(v: boolean) => emit('update:scopeProject', v)"
      >
        {{ projectLabel }}
      </NCheckbox>
    </div>
    <!-- Row 2: Description standalone -->
    <div class="rh-row">
      <span class="rh-label">{{ descLabel }}</span>
      <NInput
        :value="description"
        type="textarea"
        :placeholder="descPlaceholder"
        size="small"
        :rows="2"
        class="rh-desc-input"
        @update:value="(v: string) => emit('update:description', v)"
      />
    </div>
    <!-- Row 3: Driver + URI -->
    <div class="rh-row uri-row">
      <span class="rh-label">{{ driverLabel }}</span>
      <NSelect
        :value="selectedDriverId"
        :options="driverOptions"
        :placeholder="driverPlaceholder"
        size="small"
        class="rh-driver-select"
        @update:value="onDriverSelect"
      />
      <span class="uri-label">{{ uriLabel }}</span>
      <NInput
        v-if="uriEditing"
        :value="manualUri"
        size="small"
        class="uri-edit-input"
        :placeholder="uriPlaceholder"
        @update:value="(v: string) => emit('update:manualUri', v)"
      />
      <div v-else class="uri-display">{{ uriPreview || '—' }}</div>
      <NButton
        size="tiny"
        quaternary
        :type="uriEditing ? 'primary' : 'default'"
        @click="emit('update:uriEditing', !uriEditing)"
      >
        <template #icon><Edit :size="13" /></template>
      </NButton>
    </div>
  </div>
</template>

<script setup lang="ts">
import { Edit } from 'lucide-vue-next'
import { NButton, NCheckbox, NInput, NSelect } from 'naive-ui'

import type { SelectOption } from 'naive-ui'

interface Props {
  name: string
  description: string
  scopeGlobal: boolean
  scopeProject: boolean
  selectedDriverId: string | null
  driverOptions: SelectOption[]
  uriPreview: string
  uriEditing: boolean
  manualUri: string
  nameLabel: string
  namePlaceholder: string
  descLabel: string
  descPlaceholder: string
  globalLabel: string
  projectLabel: string
  driverLabel: string
  driverPlaceholder: string
  uriLabel: string
  uriPlaceholder: string
}

withDefaults(defineProps<Props>(), {
  name: '',
  description: '',
  scopeGlobal: true,
  scopeProject: false,
  selectedDriverId: null,
  driverOptions: () => [],
  uriPreview: '',
  uriEditing: false,
  manualUri: '',
  nameLabel: '',
  namePlaceholder: '',
  descLabel: '',
  descPlaceholder: '',
  globalLabel: '',
  projectLabel: '',
  driverLabel: '',
  driverPlaceholder: '',
  uriLabel: 'URI',
  uriPlaceholder: 'jdbc:mysql://...',
})

const emit = defineEmits<{
  (e: 'update:name', v: string): void
  (e: 'update:description', v: string): void
  (e: 'update:scopeGlobal', v: boolean): void
  (e: 'update:scopeProject', v: boolean): void
  (e: 'update:selectedDriverId', v: string | null): void
  (e: 'update:uriEditing', v: boolean): void
  (e: 'update:manualUri', v: string): void
  (e: 'driver-change', driverId: string): void
}>()

function onDriverSelect(value: string | number | null) {
  const driverId = value as string
  emit('update:selectedDriverId', driverId)
  emit('driver-change', driverId)
}
</script>

<style scoped>
.right-header {
  padding: var(--spacing-md) var(--spacing-md) var(--spacing-sm);
  border-bottom: 1px solid var(--color-border-subtle);
  display: flex;
  flex-direction: column;
  gap: var(--spacing-sm);
  flex-shrink: 0;
}

.rh-row {
  display: flex;
  align-items: center;
  gap: var(--spacing-sm);
}

.rh-label {
  font-size: var(--font-size-sm);
  font-weight: 600;
  color: var(--color-text-muted);
  width: 48px;
  flex-shrink: 0;
  text-align: right;
}

.rh-name-input {
  flex: 1;
  max-width: 280px;
}
.rh-desc-input {
  flex: 1;
}

.rh-driver-select {
  flex: 0 0 200px;
}

/* URI row */
.uri-row {
  gap: var(--spacing-xs);
}
.uri-label {
  font-size: 11px;
  color: var(--color-text-muted);
  flex-shrink: 0;
  padding: 0 2px;
}

.uri-display {
  flex: 1;
  height: 28px;
  padding: 0 10px;
  font-size: 11px;
  font-family: 'JetBrains Mono', monospace;
  color: var(--brand-success);
  background: var(--color-bg-elevated);
  border: 1px solid var(--color-border-subtle);
  border-radius: var(--border-radius-sm);
  display: flex;
  align-items: center;
  overflow: hidden;
  white-space: nowrap;
  text-overflow: ellipsis;
  min-width: 0;
}

.uri-edit-input {
  flex: 1;
  min-width: 0;
}
.uri-edit-input :deep(.n-input__input) {
  font-family: 'JetBrains Mono', monospace;
  font-size: 11px;
}
</style>
