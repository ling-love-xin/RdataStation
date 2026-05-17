<template>
  <div class="datasource-header">
    <div class="header-row">
      <span class="header-label">{{ t('navigator.name') }}</span>
      <input
        :value="name"
        type="text"
        class="name-input"
        :placeholder="String(t('navigator.dataSourceNamePlaceholder'))"
        :class="{ error: nameError }"
        @input="$emit('update:name', ($event.target as HTMLInputElement).value)"
      />
      <div class="scope-toggles">
        <label class="scope-checkbox">
          <input
            type="checkbox"
            :checked="saveToGlobal"
            @change="$emit('update:save-to-global', ($event.target as HTMLInputElement).checked)"
          />
          <span class="cb" />
          <span>{{ t('navigator.globalConnection') }}</span>
        </label>
        <label class="scope-checkbox" :class="{ disabled: !hasProject }">
          <input
            type="checkbox"
            :checked="saveToProject"
            :disabled="!hasProject"
            @change="$emit('update:save-to-project', ($event.target as HTMLInputElement).checked)"
          />
          <span class="cb" />
          <span>{{ t('navigator.projectConnection') }}</span>
        </label>
      </div>
    </div>

    <div class="header-row">
      <span class="header-label">{{ t('navigator.description') }}</span>
      <textarea
        :value="description"
        class="desc-input"
        :placeholder="String(t('navigator.dataSourceDescPlaceholder'))"
        rows="1"
        @input="$emit('update:description', ($event.target as HTMLTextAreaElement).value)"
      />
    </div>

    <div class="header-row">
      <span class="header-label">{{ t('navigator.driver') }}</span>
      <select
        class="driver-select"
        :value="selectedDriverId"
        @change="$emit('select-driver', ($event.target as HTMLSelectElement).value)"
      >
        <option
          v-for="d in availableDrivers"
          :key="d.id"
          :value="d.id"
        >
          {{ d.name }}
        </option>
      </select>
      <span class="uri-label">URI</span>
      <div class="uri-display" :title="uri">
        <code>{{ uri }}</code>
      </div>
      <button class="uri-edit-btn" :title="String(t('navigator.editUri'))" @click="$emit('edit-uri')">
        <Pencil :size="14" />
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { Pencil } from 'lucide-vue-next'
import { useI18n } from 'vue-i18n'

import type { DriverDescriptor } from '../../types/connection'

const { t } = useI18n()

interface Props {
  name: string
  description?: string
  uri: string
  saveToGlobal: boolean
  saveToProject: boolean
  hasProject: boolean
  nameError?: string
  availableDrivers: DriverDescriptor[]
  selectedDriverId: string
}

defineProps<Props>()

defineEmits<{
  (e: 'update:name', value: string): void
  (e: 'update:description', value: string): void
  (e: 'update:save-to-global', value: boolean): void
  (e: 'update:save-to-project', value: boolean): void
  (e: 'select-driver', driverId: string): void
  (e: 'edit-uri'): void
}>()
</script>

<style scoped>
.datasource-header {
  padding: 14px 24px 12px;
  border-bottom: 1px solid var(--color-border, rgba(255,255,255,0.07));
  display: flex;
  flex-direction: column;
  gap: 10px;
  flex-shrink: 0;
}

.header-row {
  display: flex;
  align-items: center;
  gap: 10px;
}
.header-label {
  font-size: 12px;
  font-weight: 600;
  color: var(--color-text-muted, #6c7086);
  width: 56px;
  flex-shrink: 0;
  text-align: right;
}

.name-input {
  flex: 1;
  max-width: 260px;
  height: 32px;
  padding: 0 10px;
  background: var(--color-bg-raised, #11111b);
  border: 1px solid rgba(255,255,255,0.06);
  border-radius: 6px;
  color: var(--color-text-primary, #cdd6f4);
  font-size: 13px;
  font-weight: 500;
  outline: none;
  transition: border-color 0.2s;
}
.name-input:focus {
  border-color: var(--color-accent, #89b4fa);
}
.name-input.error {
  border-color: var(--color-danger, #f38ba8);
}

.scope-toggles {
  display: flex;
  align-items: center;
  gap: 14px;
  flex-shrink: 0;
}
.scope-checkbox {
  display: flex;
  align-items: center;
  gap: 5px;
  cursor: pointer;
  font-size: 12px;
  color: var(--color-text-secondary, #a6adc8);
  user-select: none;
}
.scope-checkbox.disabled {
  opacity: 0.35;
  cursor: not-allowed;
}
.scope-checkbox input {
  display: none;
}
.scope-checkbox .cb {
  width: 14px;
  height: 14px;
  border: 1.5px solid rgba(255,255,255,0.15);
  border-radius: 3px;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.15s;
  flex-shrink: 0;
}
.scope-checkbox input:checked + .cb {
  background: var(--color-accent, #89b4fa);
  border-color: var(--color-accent, #89b4fa);
}
.scope-checkbox input:checked + .cb::after {
  content: '✓';
  color: var(--color-bg-raised, #11111b);
  font-size: 10px;
  font-weight: 700;
}

.desc-input {
  flex: 1;
  height: 36px;
  padding: 6px 10px;
  background: var(--color-bg-raised, #11111b);
  border: 1px solid rgba(255,255,255,0.06);
  border-radius: 6px;
  color: var(--color-text-primary, #cdd6f4);
  font-size: 12px;
  outline: none;
  resize: none;
  font-family: inherit;
  transition: border-color 0.2s;
}
.desc-input:focus {
  border-color: var(--color-accent, #89b4fa);
}

.driver-select {
  height: 32px;
  padding: 0 10px;
  min-width: 200px;
  background: var(--color-bg-raised, #11111b);
  border: 1px solid rgba(255,255,255,0.06);
  border-radius: 6px;
  color: var(--color-text-primary, #cdd6f4);
  font-size: 12px;
  outline: none;
  cursor: pointer;
  appearance: none;
  background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='12' height='12' viewBox='0 0 24 24' fill='none' stroke='%236c7086' stroke-width='2'%3E%3Cpolyline points='6 9 12 15 18 9'%3E%3C/polyline%3E%3C/svg%3E");
  background-repeat: no-repeat;
  background-position: right 8px center;
  padding-right: 30px;
  transition: border-color 0.2s;
}
.driver-select:focus {
  border-color: var(--color-accent, #89b4fa);
}

.uri-label {
  font-size: 12px;
  font-weight: 600;
  color: var(--color-text-muted, #6c7086);
  flex-shrink: 0;
}
.uri-display {
  flex: 1;
  height: 32px;
  padding: 0 10px;
  background: var(--color-bg-raised, #11111b);
  border: 1px solid rgba(255,255,255,0.06);
  border-radius: 6px;
  display: flex;
  align-items: center;
  overflow: hidden;
  min-width: 0;
}
.uri-display code {
  font-size: 11px;
  font-family: 'JetBrains Mono', 'Cascadia Code', monospace;
  color: var(--color-success, #a6e3a1);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.uri-edit-btn {
  width: 30px;
  height: 30px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: transparent;
  border: none;
  border-radius: 4px;
  color: var(--color-text-muted, #6c7086);
  cursor: pointer;
  flex-shrink: 0;
  transition: all 0.12s;
}
.uri-edit-btn:hover {
  background: var(--color-bg-hover, rgba(255,255,255,0.05));
  color: var(--color-text-primary, #cdd6f4);
}

@media (max-width: 900px) {
  .header-row {
    flex-wrap: wrap;
  }
  .scope-toggles {
    flex-direction: column;
    align-items: flex-start;
    gap: 4px;
  }
}
</style>