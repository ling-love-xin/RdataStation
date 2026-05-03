<template>
  <div class="general-tab">
    <!-- 动态表单渲染 -->
    <DynamicFormRenderer
      v-if="hasSchema"
      :sections="formSections"
      :form-data="formData"
      :errors="errors"
      @update:form-data="updateFormData"
      @select-file="handleSelectFile"
      @create-file="handleCreateFile"
    />

    <!-- 无 schema 时的默认提示 -->
    <div v-else class="empty-state">
      <Database :size="48" class="empty-icon" />
      <h3 class="empty-title">未找到表单配置</h3>
      <p class="empty-desc">请在 schemas 目录下添加对应的 JSON 配置文件</p>
    </div>
  </div>
</template>

<script setup lang="ts">
import { Database } from 'lucide-vue-next'
import { computed } from 'vue'

import DynamicFormRenderer from '../DynamicFormRenderer.vue'

import type { DriverDescriptor, ConnectionConfig } from '../../types/connection'
import type { FormSectionConfig } from '../../types/form-schema'


interface Props {
  formData: Partial<ConnectionConfig>
  selectedDriver: DriverDescriptor | null
  hasProject: boolean
  errors: Record<string, string>
  formSections?: FormSectionConfig[]
}

const props = withDefaults(defineProps<Props>(), {
  formSections: () => []
})

const emit = defineEmits<{
  'update:formData': [data: Partial<ConnectionConfig>]
  selectFile: []
  createFile: []
}>()

const hasSchema = computed(() => props.formSections.length > 0)

function updateFormData(data: Record<string, unknown>) {
  emit('update:formData', {
    ...props.formData,
    ...data
  })
}

function handleSelectFile(fieldKey: string) {
  if (fieldKey === 'database') {
    emit('selectFile')
  }
}

function handleCreateFile(fieldKey: string) {
  if (fieldKey === 'database') {
    emit('createFile')
  }
}
</script>

<style scoped>
.general-tab {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 60px 20px;
  text-align: center;
}

.empty-icon {
  color: var(--text-tertiary);
  margin-bottom: 16px;
}

.empty-title {
  font-size: 16px;
  font-weight: 600;
  color: var(--text-primary);
  margin: 0 0 8px;
}

.empty-desc {
  font-size: 13px;
  color: var(--text-secondary);
  margin: 0;
}
</style>
