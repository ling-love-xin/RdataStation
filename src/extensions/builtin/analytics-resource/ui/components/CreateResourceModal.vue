<template>
  <div class="modal-overlay" @click.self="$emit('close')">
    <div class="modal">
      <div class="modal-header">
        <h3>{{
          isEdit ? t('analyticsResource.editResource') : t('analyticsResource.createResource')
        }}</h3>
        <button class="close-btn" @click="$emit('close')">✕</button>
      </div>

      <div class="modal-body">
        <div class="form-group">
          <label>{{ t('analyticsResource.resourceType') }} *</label>
          <select v-model="form.resource_type" class="form-input" :disabled="isEdit">
            <option value="connection">🔌 {{ t('analyticsResource.connection') }}</option>
            <option value="table">📊 {{ t('analyticsResource.table') }}</option>
            <option value="file">📄 {{ t('analyticsResource.file') }}</option>
          </select>
        </div>

        <div class="form-group">
          <label>{{ t('analyticsResource.resourceName') }} *</label>
          <input
            v-model="form.name"
            type="text"
            class="form-input"
            :placeholder="t('analyticsResource.resourceName')"
          />
        </div>

        <div class="form-group">
          <label>{{ t('analyticsResource.alias') }}</label>
          <input
            v-model="form.alias"
            type="text"
            class="form-input"
            :placeholder="t('analyticsResource.aliasPlaceholder')"
          />
        </div>

        <div class="form-group">
          <label>{{ t('analyticsResource.scope') }} *</label>
          <select v-model="form.scope" class="form-input">
            <option value="global">🌍 {{ t('analyticsResource.global') }}</option>
            <option value="project">📂 {{ t('analyticsResource.project') }}</option>
            <option value="session">📌 {{ t('analyticsResource.session') }}</option>
          </select>
        </div>

        <div v-if="form.resource_type === 'table'" class="form-group">
          <label>{{ t('analyticsResource.rowCountLabel') }}</label>
          <input
            v-model.number="form.row_count"
            type="number"
            class="form-input"
            :placeholder="t('analyticsResource.rowCountLabel')"
          />
        </div>

        <div v-if="form.resource_type === 'table'" class="form-group">
          <label>{{ t('analyticsResource.columnCount') }}</label>
          <input
            v-model.number="form.column_count"
            type="number"
            class="form-input"
            :placeholder="t('analyticsResource.columnCount')"
          />
        </div>

        <div v-if="form.resource_type === 'file'" class="form-group">
          <label>{{ t('analyticsResource.fileSizeLabel') }}</label>
          <input
            v-model.number="form.file_size"
            type="number"
            class="form-input"
            :placeholder="t('analyticsResource.fileSizeLabel')"
          />
        </div>

        <div class="form-group">
          <label>{{ t('analyticsResource.sourceQuery') }}</label>
          <textarea
            v-model="form.source_query"
            class="form-input form-textarea"
            placeholder="SELECT * FROM table_name"
            rows="3"
          ></textarea>
        </div>

        <div class="form-group">
          <label>{{ t('analyticsResource.configJson') }}</label>
          <textarea
            v-model="configJson"
            class="form-input form-textarea"
            :placeholder="t('analyticsResource.configJsonPlaceholder')"
            rows="4"
          ></textarea>
        </div>
      </div>

      <div class="modal-footer">
        <button class="btn btn-secondary" @click="$emit('close')">
          {{ t('analyticsResource.cancel') }}
        </button>
        <button class="btn btn-primary" :disabled="!isValid" @click="handleSubmit">
          {{ isEdit ? t('analyticsResource.save') : t('analyticsResource.create') }}
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'

import type {
  CreateResourceRequest,
  ResourceType,
  ResourceScope,
  AnalyticsResource,
} from '../../types'

const { t } = useI18n()

const props = defineProps<{
  editResource?: AnalyticsResource
}>()

const emit = defineEmits<{
  close: []
  create: [input: CreateResourceRequest]
  update: [id: string, input: CreateResourceRequest]
}>()

const isEdit = computed(() => !!props.editResource)

const form = ref({
  resource_type: 'connection' as ResourceType,
  name: '',
  alias: '',
  scope: 'project' as ResourceScope,
  row_count: undefined as number | undefined,
  column_count: undefined as number | undefined,
  file_size: undefined as number | undefined,
  parent_resource_id: undefined as string | undefined,
  source_query: undefined as string | undefined,
})

const configJson = ref('{}')

const isValid = computed(() => {
  return form.value.name.trim() !== ''
})

function handleSubmit() {
  try {
    const config = JSON.parse(configJson.value)

    const input: CreateResourceRequest = {
      resource_type: form.value.resource_type,
      name: form.value.name.trim(),
      alias: form.value.alias.trim() || undefined,
      scope: form.value.scope,
      config,
      row_count: form.value.row_count,
      column_count: form.value.column_count,
      file_size: form.value.file_size,
      parent_resource_id: form.value.parent_resource_id,
      source_query: form.value.source_query || undefined,
    }

    if (isEdit.value && props.editResource) {
      emit('update', props.editResource.id, input)
    } else {
      emit('create', input)
    }
  } catch (e) {
    alert(t('analyticsResource.jsonFormatError'))
  }
}

onMounted(() => {
  if (props.editResource) {
    const r = props.editResource
    form.value = {
      resource_type: r.resource_type,
      name: r.name,
      alias: r.alias || '',
      scope: r.scope,
      row_count: r.row_count,
      column_count: r.column_count,
      file_size: r.file_size,
      parent_resource_id: r.parent_resource_id,
      source_query: r.source_query,
    }
    configJson.value = JSON.stringify(r.config, null, 2)
  }
})
</script>

<style scoped>
.modal-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
}

.modal {
  background: var(--bg-primary);
  border-radius: var(--radius-xl);
  width: 90%;
  max-width: 500px;
  max-height: 90vh;
  overflow: hidden;
  display: flex;
  flex-direction: column;
  box-shadow: var(--shadow-lg);
}

.modal-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: var(--size-lg) var(--size-xl);
  border-bottom: 1px solid var(--border-color);
}

.modal-header h3 {
  margin: 0;
  font-size: 16px;
  font-weight: 600;
  color: var(--text-primary);
}

.close-btn {
  background: none;
  border: none;
  font-size: 18px;
  cursor: pointer;
  color: var(--text-tertiary);
  transition: color 0.15s;
}

.close-btn:hover {
  color: var(--text-primary);
}

.modal-body {
  padding: var(--size-xl);
  overflow-y: auto;
}

.form-group {
  margin-bottom: var(--size-lg);
}

.form-group label {
  display: block;
  margin-bottom: 6px;
  font-size: 12px;
  font-weight: 500;
  color: var(--text-secondary);
}

.form-input {
  width: 100%;
  padding: 6px 12px;
  border: 1px solid var(--border-color);
  border-radius: var(--radius-md);
  background: var(--bg-secondary);
  color: var(--text-primary);
  font-size: 13px;
  box-sizing: border-box;
  height: var(--height-input);
}

.form-input:focus {
  outline: none;
  border-color: var(--primary-color);
}

.form-textarea {
  resize: vertical;
  font-family: var(--font-mono);
}

.modal-footer {
  display: flex;
  justify-content: flex-end;
  gap: var(--size-md);
  padding: var(--size-lg) var(--size-xl);
  border-top: 1px solid var(--border-color);
}

.btn {
  padding: 6px 16px;
  border: none;
  border-radius: var(--radius-md);
  font-size: 13px;
  cursor: pointer;
  transition: all 0.2s;
  height: var(--height-btn);
}

.btn.btn-secondary {
  background: var(--bg-secondary);
  border: 1px solid var(--border-color);
  color: var(--text-secondary);
}

.btn.btn-secondary:hover {
  border-color: var(--text-secondary);
}

.btn.btn-primary {
  background: var(--primary-color);
  color: white;
}

.btn.btn-primary:hover {
  background: var(--primary-dark);
}

.btn.btn-primary:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
</style>
