<template>
  <div class="datasource-header">
    <!-- 名称 -->
    <div class="header-row">
      <label class="header-label">{{ $t('connection.formName') }}</label>
      <input
        :value="formData.name as string"
        class="header-input"
        :placeholder="$t('connection.namePlaceholder')"
        @input="emitUpdate('name', ($event.target as HTMLInputElement).value)"
      />
    </div>

    <!-- 描述 -->
    <div class="header-row">
      <label class="header-label">{{ $t('connection.formDescription') }}</label>
      <input
        :value="formData.description as string"
        class="header-input"
        :placeholder="$t('connection.descriptionPlaceholder')"
        @input="emitUpdate('description', ($event.target as HTMLInputElement).value)"
      />
    </div>

    <!-- 驱动选择 -->
    <div class="header-row">
      <label class="header-label">{{ $t('connection.formDriver') }}</label>
      <select
        :value="selectedDriverId"
        class="header-select"
        @change="$emit('select-driver', ($event.target as HTMLSelectElement).value)"
      >
        <option value="" disabled>—</option>
        <option
          v-for="drv in drivers"
          :key="drv.id"
          :value="drv.id"
        >
          {{ drv.name || drv.id }}
        </option>
      </select>
    </div>
  </div>
</template>

<script setup lang="ts">
import type { DriverDescriptor } from '../types/connection'

interface Props {
  formData: Record<string, unknown>
  drivers: DriverDescriptor[]
  selectedDriverId?: string
}

defineProps<Props>()

interface Emits {
  (e: 'update:form-data', data: Record<string, unknown>): void
  (e: 'select-driver', id: string): void
}

const emit = defineEmits<Emits>()

function emitUpdate(key: string, value: string) {
  emit('update:form-data', { [key]: value })
}
</script>

<style scoped>
.datasource-header {
  padding: var(--spacing-sm) var(--spacing-md) 0;
  display: flex;
  flex-direction: column;
  gap: var(--spacing-sm);
  flex-shrink: 0;
}

.header-row {
  display: flex;
  align-items: center;
  gap: var(--spacing-sm);
}

.header-label {
  font-size: var(--font-size-sm);
  font-weight: 600;
  color: var(--color-text-muted);
  min-width: 44px;
  flex-shrink: 0;
  text-align: left;
}

.header-input {
  flex: 1;
  height: var(--height-input);
  padding: 0 var(--spacing-sm);
  border: 1px solid var(--color-border-subtle);
  border-radius: var(--border-radius-md);
  background: var(--color-bg-elevated);
  color: var(--color-text-primary);
  font-size: var(--font-size-sm);
  outline: none;
  transition: border-color 0.2s;
}

.header-input:focus {
  border-color: var(--brand-accent);
}

.header-input::placeholder {
  color: var(--color-text-muted);
}

.header-select {
  flex: 1;
  height: var(--height-input);
  padding: 0 var(--spacing-sm);
  border: 1px solid var(--color-border-subtle);
  border-radius: var(--border-radius-md);
  background: var(--color-bg-elevated);
  color: var(--color-text-primary);
  font-size: var(--font-size-sm);
  outline: none;
  cursor: pointer;
  appearance: none;
  background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='10' height='10' viewBox='0 0 24 24' fill='none' stroke='%236b7280' stroke-width='2'%3E%3Cpolyline points='6 9 12 15 18 9'%3E%3C/polyline%3E%3C/svg%3E");
  background-repeat: no-repeat;
  background-position: right 8px center;
  padding-right: 28px;
  transition: border-color 0.2s;
}

.header-select:focus {
  border-color: var(--brand-accent);
}
</style>