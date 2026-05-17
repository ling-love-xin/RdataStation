<template>
  <header class="modal-header">
    <div class="header-left">
      <h2 class="header-title">{{ title }}</h2>
      <DbIcon v-if="driverIcon" :type="driverIcon" class="header-db-icon" />
    </div>

    <div v-if="showNameInput" class="header-connection-name">
      <input
        :value="name"
        type="text"
        class="connection-name-input"
        placeholder="输入连接名称"
        :class="{ error: nameError }"
        @input="$emit('update:name', ($event.target as HTMLInputElement).value)"
      />
      <span v-if="nameError" class="error-text">{{ nameError }}</span>
    </div>

    <div v-if="showScopeToggle" class="header-scope-toggle">
      <label class="scope-checkbox">
        <input
          type="checkbox"
          :checked="saveToGlobal"
          @change="$emit('update:saveToGlobal', ($event.target as HTMLInputElement).checked)"
        />
        <span class="checkmark"></span>
        <span class="scope-label">{{ globalLabel }}</span>
      </label>
      <label class="scope-checkbox" :class="{ disabled: !hasProject }">
        <input
          type="checkbox"
          :checked="saveToProject"
          :disabled="!hasProject"
          @change="$emit('update:saveToProject', ($event.target as HTMLInputElement).checked)"
        />
        <span class="checkmark"></span>
        <span class="scope-label">{{ projectLabel }}</span>
      </label>
      <NTooltip v-if="!hasProject" trigger="hover">
        <template #trigger>
          <HelpCircle :size="14" class="help-icon" />
        </template>
        {{ noProjectTip }}
      </NTooltip>
    </div>

    <button class="btn-close" @click="$emit('close')">
      <X :size="18" />
    </button>
  </header>
</template>

<script setup lang="ts">
import { HelpCircle, X } from 'lucide-vue-next'
import { NTooltip } from 'naive-ui'

import DbIcon from '@/shared/components/common/DbIcon.vue'

interface Props {
  title: string
  driverIcon?: string | null
  showNameInput?: boolean
  showScopeToggle?: boolean
  name?: string
  nameError?: string
  saveToGlobal?: boolean
  saveToProject?: boolean
  hasProject?: boolean
  globalLabel?: string
  projectLabel?: string
  noProjectTip?: string
}

withDefaults(defineProps<Props>(), {
  showNameInput: true,
  showScopeToggle: true,
  name: '',
  nameError: '',
  saveToGlobal: true,
  saveToProject: false,
  hasProject: false,
  globalLabel: '全局',
  projectLabel: '项目',
  noProjectTip: '请先打开项目',
})

defineEmits<{
  close: []
  'update:name': [value: string]
  'update:saveToGlobal': [value: boolean]
  'update:saveToProject': [value: boolean]
}>()
</script>

<style scoped>
.modal-header {
  display: flex;
  align-items: center;
  gap: 16px;
  padding: 16px 24px;
  border-bottom: 1px solid var(--border-color);
  background: var(--bg-secondary);
}

.header-left {
  display: flex;
  align-items: center;
  gap: 12px;
  flex-shrink: 0;
}

.header-title {
  font-size: 18px;
  font-weight: 600;
  color: var(--text-primary);
  margin: 0;
  white-space: nowrap;
}

.header-db-icon {
  width: 20px;
  height: 20px;
  flex-shrink: 0;
}

.header-connection-name {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 2px;
  min-width: 0;
}

.connection-name-input {
  height: 36px;
  padding: 0 12px;
  background: var(--bg-tertiary);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-md);
  color: var(--text-primary);
  font-size: 14px;
  font-weight: 500;
  outline: none;
  transition: all 0.2s;
  width: 100%;
}

.connection-name-input:focus {
  border-color: var(--primary-color);
  background: var(--bg-primary);
}

.connection-name-input.error {
  border-color: var(--danger-color);
}

.error-text {
  font-size: 11px;
  color: var(--danger-color);
}

.header-scope-toggle {
  display: flex;
  align-items: center;
  gap: 12px;
  flex-shrink: 0;
}

.scope-checkbox {
  display: flex;
  align-items: center;
  gap: 6px;
  cursor: pointer;
  user-select: none;
}

.scope-checkbox.disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.scope-checkbox input[type='checkbox'] {
  display: none;
}

.scope-checkbox .checkmark {
  width: 16px;
  height: 16px;
  border: 2px solid var(--border-color);
  border-radius: 3px;
  position: relative;
  transition: all 0.2s;
}

.scope-checkbox input[type='checkbox']:checked + .checkmark {
  background: var(--primary-color);
  border-color: var(--primary-color);
}

.scope-checkbox input[type='checkbox']:checked + .checkmark::after {
  content: '';
  position: absolute;
  left: 4px;
  top: 1px;
  width: 4px;
  height: 8px;
  border: solid white;
  border-width: 0 2px 2px 0;
  transform: rotate(45deg);
}

.scope-label {
  font-size: 13px;
  color: var(--text-secondary);
  font-weight: 500;
}

.help-icon {
  color: var(--text-tertiary);
  cursor: help;
}

.btn-close {
  width: 32px;
  height: 32px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: transparent;
  border: none;
  border-radius: var(--radius-md);
  color: var(--text-secondary);
  cursor: pointer;
  transition: all 0.2s;
  flex-shrink: 0;
}

.btn-close:hover {
  background: var(--bg-hover);
  color: var(--text-primary);
}
</style>