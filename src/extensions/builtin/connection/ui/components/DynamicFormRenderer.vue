<template>
  <div class="dynamic-form-renderer">
    <div
      v-for="section in visibleSections"
      :key="section.key"
      class="form-section-wrapper"
    >
      <!-- 分区头部 -->
      <div
        v-if="section.title"
        class="section-header"
        :class="{ collapsible: section.collapsible }"
        @click="section.collapsible && toggleSection(section.key)"
      >
        <div class="header-left">
          <component
            :is="getIconComponent(section.icon)"
            v-if="section.icon"
            :size="16"
            class="section-icon"
          />
          <h4 class="section-title">{{ section.title }}</h4>
        </div>
        <div class="header-right">
          <!-- 启用开关（如果有 enableField 配置） -->
          <label
            v-if="section.enableField"
            class="section-toggle"
            @click.stop
          >
            <input
              :checked="formData[section.enableField] === true"
              type="checkbox"
              @change="toggleSectionEnable(section.enableField, $event)"
            />
            <span class="toggle-slider"></span>
          </label>
          <ChevronDown
            v-if="section.collapsible"
            :size="16"
            class="collapse-icon"
            :class="{ collapsed: collapsedSections[section.key] }"
          />
        </div>
      </div>

      <!-- 分区描述 -->
      <p v-if="section.description" class="section-description">
        {{ section.description }}
      </p>

      <!-- 字段列表 -->
      <div
        v-show="!section.collapsible || !collapsedSections[section.key]"
        v-if="!section.enableField || formData[section.enableField] === true"
        class="section-fields"
      >
        <!-- 处理行内布局：将连续的 inline 字段组合成一行 -->
        <template v-for="(row, rowIndex) in getGroupedFields(section.fields)" :key="rowIndex">
          <!-- 单字段行 -->
          <div v-if="row.fields.length === 1" class="field-wrapper">
            <FieldRenderer
              :field="row.fields[0]"
              :form-data="formData"
              :errors="errors"
              :password-visible="passwordVisibility"
              @update:form-data="updateFormData"
              @select-file="selectFile"
              @create-file="createFile"
              @toggle-password="togglePasswordVisibility"
            />
          </div>

          <!-- 多字段行（inline 布局） -->
          <div v-else class="inline-row">
            <div
              v-for="field in row.fields"
              :key="field.key"
              class="field-wrapper inline-field"
              :style="{ flex: field.flex || 1 }"
            >
              <FieldRenderer
                :field="field"
                :form-data="formData"
                :errors="errors"
                :password-visible="passwordVisibility"
                @update:form-data="updateFormData"
                @select-file="selectFile"
                @create-file="createFile"
                @toggle-password="togglePasswordVisibility"
              />
            </div>
          </div>
        </template>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import {
  ChevronDown
} from 'lucide-vue-next'
import { ref, computed } from 'vue'

import FieldRenderer from './FieldRenderer.vue'

import type { FormSectionConfig, FormFieldConfig } from '../types/form-schema'


interface Props {
  sections: FormSectionConfig[]
  formData: Record<string, unknown>
  errors: Record<string, string>
}

const props = defineProps<Props>()

const emit = defineEmits<{
  'update:formData': [data: Record<string, unknown>]
  selectFile: [fieldKey: string]
  createFile: [fieldKey: string]
}>()

const passwordVisibility = ref<Record<string, boolean>>({})
const collapsedSections = ref<Record<string, boolean>>({})

const visibleSections = computed(() => {
  return props.sections.filter(section => {
    return !section.fields.every(field => isFieldHidden(field))
  })
})

function isFieldHidden(field: FormFieldConfig): boolean {
  if (field.hidden) return true

  if (field.dependsOn) {
    const depValue = props.formData[field.dependsOn.field]
    return depValue !== field.dependsOn.value
  }

  return false
}

function getGroupedFields(fields: FormFieldConfig[]): Array<{ fields: FormFieldConfig[] }> {
  const groups: Array<{ fields: FormFieldConfig[] }> = []
  let currentGroup: FormFieldConfig[] = []

  for (const field of fields) {
    if (field.inline) {
      currentGroup.push(field)
    } else {
      if (currentGroup.length > 0) {
        groups.push({ fields: [...currentGroup] })
        currentGroup = []
      }
      groups.push({ fields: [field] })
    }
  }

  if (currentGroup.length > 0) {
    groups.push({ fields: currentGroup })
  }

  return groups
}

function toggleSection(sectionKey: string) {
  collapsedSections.value[sectionKey] = !collapsedSections.value[sectionKey]
}

function toggleSectionEnable(fieldKey: string, event: Event) {
  const target = event.target as HTMLInputElement
  emit('update:formData', {
    ...props.formData,
    [fieldKey]: target.checked
  })
}

function togglePasswordVisibility(fieldKey: string) {
  passwordVisibility.value[fieldKey] = !passwordVisibility.value[fieldKey]
}

function selectFile(fieldKey: string) {
  emit('selectFile', fieldKey)
}

function createFile(fieldKey: string) {
  emit('createFile', fieldKey)
}

function updateFormData(data: Record<string, unknown>) {
  emit('update:formData', {
    ...props.formData,
    ...data
  })
}

function getIconComponent(iconName: string) {
  const iconMap: Record<string, string> = {
    'shield': 'Shield',
    'lock': 'Lock',
    'database': 'Database',
    'settings': 'Settings',
    'user': 'User',
    'zap': 'Zap'
  }
  return iconMap[iconName] || 'Settings'
}
</script>

<style scoped>
.dynamic-form-renderer {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.form-section-wrapper {
  display: flex;
  flex-direction: column;
  gap: 12px;
  padding: 12px;
  background: var(--bg-secondary);
  border-radius: var(--radius-md);
}

.section-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 4px 0;
  margin-bottom: 4px;
}

.section-header.collapsible {
  cursor: pointer;
  user-select: none;
}

.section-header.collapsible:hover .section-title {
  color: var(--primary-color);
}

.header-left {
  display: flex;
  align-items: center;
  gap: 8px;
}

.header-right {
  display: flex;
  align-items: center;
  gap: 8px;
}

.section-toggle {
  display: flex;
  align-items: center;
  cursor: pointer;
}

.section-toggle input {
  display: none;
}

.section-toggle .toggle-slider {
  width: 32px;
  height: 18px;
  background: var(--bg-tertiary);
  border-radius: 9px;
  position: relative;
  transition: background 0.2s;
}

.section-toggle .toggle-slider::after {
  content: '';
  position: absolute;
  width: 14px;
  height: 14px;
  background: white;
  border-radius: 50%;
  top: 2px;
  left: 2px;
  transition: transform 0.2s;
}

.section-toggle input:checked + .toggle-slider {
  background: var(--primary-color);
}

.section-toggle input:checked + .toggle-slider::after {
  transform: translateX(14px);
}

.section-icon {
  color: var(--primary-color);
}

.section-title {
  font-size: 13px;
  font-weight: 600;
  color: var(--text-primary);
  margin: 0;
  transition: color 0.2s;
}

.collapse-icon {
  color: var(--text-tertiary);
  transition: transform 0.2s;
}

.collapse-icon.collapsed {
  transform: rotate(-90deg);
}

.section-description {
  font-size: 12px;
  color: var(--text-secondary);
  margin: 0;
}

.section-fields {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.field-wrapper {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.inline-row {
  display: flex;
  gap: 12px;
  align-items: flex-end;
}

.inline-field {
  min-width: 0;
}

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

/* 密码输入 */
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

/* 文件输入 */
.file-input-wrapper {
  display: flex;
  gap: 8px;
}

.file-input-wrapper .field-input {
  flex: 1;
}

.btn-file {
  width: 36px;
  height: 36px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--bg-tertiary);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-md);
  color: var(--text-secondary);
  cursor: pointer;
  transition: all 0.2s;
}

.btn-file:hover {
  background: var(--bg-hover);
  color: var(--text-primary);
}

/* 复选框 */
.checkbox-wrapper {
  display: flex;
  align-items: center;
  gap: 8px;
  cursor: pointer;
}

.checkbox-wrapper input[type="checkbox"] {
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

.checkbox-wrapper input[type="checkbox"]:checked + .checkmark {
  background: var(--primary-color);
  border-color: var(--primary-color);
}

.checkbox-wrapper input[type="checkbox"]:checked + .checkmark::after {
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
