<template>
  <div>
    <!-- 文本输入 -->
    <template v-if="field.type === 'text'">
      <label class="field-label">
        {{ field.label }}
        <span v-if="field.required" class="required">*</span>
        <NTooltip v-if="field.tooltip" trigger="hover">
          <template #trigger>
            <HelpCircle :size="12" class="tooltip-icon" />
          </template>
          {{ field.tooltip }}
        </NTooltip>
      </label>
      <input
        :value="String(formData[field.key] ?? '')"
        type="text"
        class="field-input"
        :placeholder="field.placeholder"
        :class="{ error: errors[field.key] }"
        @input="handleInput($event, 'text')"
      />
      <span v-if="errors[field.key]" class="error-text">{{ errors[field.key] }}</span>
    </template>

    <!-- 密码输入 -->
    <template v-else-if="field.type === 'password'">
      <label class="field-label">
        {{ field.label }}
        <span v-if="field.required" class="required">*</span>
      </label>
      <div class="password-wrapper">
        <input
          :value="formData[field.key]"
          :type="passwordVisible[field.key] ? 'text' : 'password'"
          class="field-input"
          :placeholder="field.placeholder"
          :class="{ error: errors[field.key] }"
          @input="handleInput($event, 'text')"
        />
        <button type="button" class="btn-toggle-password" @click="togglePassword">
          <Eye v-if="!passwordVisible[field.key]" :size="16" />
          <EyeOff v-else :size="16" />
        </button>
      </div>
      <span v-if="errors[field.key]" class="error-text">{{ errors[field.key] }}</span>
    </template>

    <!-- 数字输入 -->
    <template v-else-if="field.type === 'number'">
      <label class="field-label">
        {{ field.label }}
        <span v-if="field.required" class="required">*</span>
      </label>
      <input
        :value="formData[field.key]"
        type="number"
        class="field-input"
        :placeholder="field.placeholder"
        :min="field.validation?.min"
        :max="field.validation?.max"
        :class="{ error: errors[field.key] }"
        @input="handleInput($event, 'number')"
      />
      <span v-if="errors[field.key]" class="error-text">{{ errors[field.key] }}</span>
    </template>

    <!-- 下拉选择 -->
    <template v-else-if="field.type === 'select'">
      <label class="field-label">
        {{ field.label }}
        <span v-if="field.required" class="required">*</span>
      </label>
      <select
        :value="formData[field.key]"
        class="field-select"
        :class="{ error: errors[field.key] }"
        @change="handleSelect"
      >
        <option v-for="opt in field.options" :key="opt.value" :value="opt.value">
          {{ opt.label }}
        </option>
      </select>
      <span v-if="errors[field.key]" class="error-text">{{ errors[field.key] }}</span>
    </template>

    <!-- 复选框 -->
    <template v-else-if="field.type === 'checkbox'">
      <label class="checkbox-wrapper">
        <input type="checkbox" :checked="!!formData[field.key]" @change="handleCheckbox" />
        <span class="checkmark"></span>
        <span class="checkbox-label">
          {{ field.label }}
          <NTooltip v-if="field.tooltip" trigger="hover">
            <template #trigger>
              <HelpCircle :size="12" class="tooltip-icon" />
            </template>
            {{ field.tooltip }}
          </NTooltip>
        </span>
      </label>
    </template>

    <!-- 文件选择 -->
    <template v-else-if="field.type === 'file'">
      <label class="field-label">
        {{ field.label }}
        <span v-if="field.required" class="required">*</span>
      </label>
      <div class="file-input-wrapper">
        <input
          :value="formData[field.key]"
          type="text"
          class="field-input"
          :placeholder="field.placeholder"
          :class="{ error: errors[field.key] }"
          @input="handleInput($event, 'text')"
        />
        <div class="file-buttons">
          <button type="button" class="btn-file" @click="handleSelectFile">
            <FolderOpen :size="14" />
            <span>选择</span>
          </button>
          <button type="button" class="btn-create" @click="handleCreateFile">
            <Plus :size="14" />
            <span>新建</span>
          </button>
        </div>
      </div>
      <span v-if="errors[field.key]" class="error-text">{{ errors[field.key] }}</span>
    </template>

    <!-- 文本域 -->
    <template v-else-if="field.type === 'textarea'">
      <label class="field-label">
        {{ field.label }}
        <span v-if="field.required" class="required">*</span>
      </label>
      <textarea
        :value="String(formData[field.key] ?? '')"
        class="field-textarea"
        :placeholder="field.placeholder"
        :rows="3"
        :class="{ error: errors[field.key] }"
        @input="handleInput($event, 'textarea')"
      ></textarea>
      <span v-if="errors[field.key]" class="error-text">{{ errors[field.key] }}</span>
    </template>
  </div>
</template>

<script setup lang="ts">
import { HelpCircle, Eye, EyeOff, FolderOpen, Plus } from 'lucide-vue-next'
import { NTooltip } from 'naive-ui'

import type { FormFieldConfig } from '../types/form-schema'

interface Props {
  field: FormFieldConfig
  formData: Record<string, unknown>
  errors: Record<string, string>
  passwordVisible: Record<string, boolean>
}

const props = defineProps<Props>()

const emit = defineEmits<{
  'update:formData': [data: Record<string, unknown>]
  selectFile: [fieldKey: string]
  createFile: [fieldKey: string]
  togglePassword: [fieldKey: string]
}>()

function handleInput(event: Event, type: string) {
  const target = event.target as HTMLInputElement
  const value = type === 'number' ? Number(target.value) : target.value
  emit('update:formData', {
    [props.field.key]: value,
  })
}

function handleSelect(event: Event) {
  const target = event.target as HTMLSelectElement
  emit('update:formData', {
    [props.field.key]: target.value,
  })
}

function handleCheckbox(event: Event) {
  const target = event.target as HTMLInputElement
  emit('update:formData', {
    [props.field.key]: target.checked,
  })
}

function togglePassword() {
  emit('togglePassword', props.field.key)
}

function handleSelectFile() {
  emit('selectFile', props.field.key)
}

function handleCreateFile() {
  emit('createFile', props.field.key)
}
</script>

<style scoped>
.field-label {
  font-size: 13px;
  font-weight: 500;
  color: var(--text-primary);
  display: flex;
  align-items: center;
  gap: 4px;
}

.required {
  color: var(--danger-color);
}

.tooltip-icon {
  color: var(--text-tertiary);
  cursor: help;
}

.field-input,
.field-select,
.field-textarea {
  height: 36px;
  padding: 0 12px;
  background: var(--bg-tertiary);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-md);
  color: var(--text-primary);
  font-size: 13px;
  outline: none;
  transition: all 0.2s;
}

.field-textarea {
  height: auto;
  resize: vertical;
}

.field-input:focus,
.field-select:focus,
.field-textarea:focus {
  border-color: var(--primary-color);
  background: var(--bg-primary);
}

.field-input.error,
.field-select.error,
.field-textarea.error {
  border-color: var(--danger-color);
}

.error-text {
  font-size: 12px;
  color: var(--danger-color);
}

.password-wrapper {
  position: relative;
}

.password-wrapper .field-input {
  width: 100%;
  padding-right: 40px;
}

.btn-toggle-password {
  position: absolute;
  right: 8px;
  top: 50%;
  transform: translateY(-50%);
  width: 24px;
  height: 24px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: transparent;
  border: none;
  color: var(--text-tertiary);
  cursor: pointer;
  transition: all 0.2s;
}

.btn-toggle-password:hover {
  color: var(--text-primary);
}

.file-input-wrapper {
  display: flex;
  gap: 8px;
}

.file-input-wrapper .field-input {
  flex: 1;
}

.file-buttons {
  display: flex;
  gap: 6px;
}

.btn-file,
.btn-create {
  height: 36px;
  padding: 0 12px;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 4px;
  background: var(--bg-tertiary);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-md);
  color: var(--text-secondary);
  cursor: pointer;
  transition: all 0.2s;
  font-size: 12px;
  white-space: nowrap;
}

.btn-file:hover,
.btn-create:hover {
  background: var(--bg-hover);
  color: var(--text-primary);
}

.btn-create {
  background: var(--primary-color);
  border-color: var(--primary-color);
  color: white;
}

.btn-create:hover {
  background: var(--primary-color-dark, var(--primary-color));
}

.checkbox-wrapper {
  display: flex;
  align-items: center;
  gap: 8px;
  cursor: pointer;
}

.checkbox-wrapper input[type='checkbox'] {
  display: none;
}

.checkbox-wrapper .checkmark {
  width: 16px;
  height: 16px;
  border: 2px solid var(--border-color);
  border-radius: 3px;
  position: relative;
  transition: all 0.2s;
}

.checkbox-wrapper input[type='checkbox']:checked + .checkmark {
  background: var(--primary-color);
  border-color: var(--primary-color);
}

.checkbox-wrapper input[type='checkbox']:checked + .checkmark::after {
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

.checkbox-label {
  font-size: 13px;
  color: var(--text-secondary);
  display: flex;
  align-items: center;
  gap: 4px;
}
</style>
