<template>
  <div class="modal-overlay" @click.self="$emit('close')">
    <div class="modal">
      <div class="modal-header">
        <h3>添加资源</h3>
        <button class="close-btn" @click="$emit('close')">✕</button>
      </div>

      <div class="modal-body">
        <div class="form-group">
          <label>资源类型 *</label>
          <select v-model="form.resource_type" class="form-input">
            <option value="connection">🔌 连接</option>
            <option value="table">📊 表</option>
            <option value="file">📄 文件</option>
          </select>
        </div>

        <div class="form-group">
          <label>资源名称 *</label>
          <input
            v-model="form.name"
            type="text"
            class="form-input"
            placeholder="输入资源名称"
          />
        </div>

        <div class="form-group">
          <label>别名</label>
          <input
            v-model="form.alias"
            type="text"
            class="form-input"
            placeholder="输入别名（可选）"
          />
        </div>

        <div class="form-group">
          <label>作用域 *</label>
          <select v-model="form.scope" class="form-input">
            <option value="global">🌍 全局</option>
            <option value="project">📂 项目</option>
            <option value="session">📌 会话</option>
          </select>
        </div>

        <div v-if="form.resource_type === 'table'" class="form-group">
          <label>行数</label>
          <input
            v-model.number="form.row_count"
            type="number"
            class="form-input"
            placeholder="输入行数"
          />
        </div>

        <div v-if="form.resource_type === 'table'" class="form-group">
          <label>列数</label>
          <input
            v-model.number="form.column_count"
            type="number"
            class="form-input"
            placeholder="输入列数"
          />
        </div>

        <div v-if="form.resource_type === 'file'" class="form-group">
          <label>文件大小（字节）</label>
          <input
            v-model.number="form.file_size"
            type="number"
            class="form-input"
            placeholder="输入文件大小"
          />
        </div>

        <div class="form-group">
          <label>配置 JSON</label>
          <textarea
            v-model="configJson"
            class="form-input form-textarea"
            placeholder='{"connectionId": "xxx", "tableName": "yyy"}'
            rows="4"
          ></textarea>
        </div>
      </div>

      <div class="modal-footer">
        <button class="btn btn-secondary" @click="$emit('close')">
          取消
        </button>
        <button class="btn btn-primary" @click="handleCreate" :disabled="!isValid">
          创建
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue';
import type { CreateResourceRequest, ResourceType, ResourceScope } from '../../types';

const emit = defineEmits<{
  close: [];
  create: [input: CreateResourceRequest];
}>();

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
});

const configJson = ref('{}');

const isValid = computed(() => {
  return form.value.name.trim() !== '';
});

function handleCreate() {
  try {
    const config = JSON.parse(configJson.value);
    
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
      source_query: form.value.source_query,
    };
    
    emit('create', input);
  } catch (e) {
    alert('配置 JSON 格式错误');
  }
}
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
  background: var(--color-background);
  border-radius: 12px;
  width: 90%;
  max-width: 500px;
  max-height: 90vh;
  overflow: hidden;
  display: flex;
  flex-direction: column;
}

.modal-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 16px 20px;
  border-bottom: 1px solid var(--color-border);
}

.modal-header h3 {
  margin: 0;
  font-size: 18px;
  font-weight: 600;
}

.close-btn {
  background: none;
  border: none;
  font-size: 20px;
  cursor: pointer;
  color: var(--color-text-secondary);
}

.close-btn:hover {
  color: var(--color-text);
}

.modal-body {
  padding: 20px;
  overflow-y: auto;
}

.form-group {
  margin-bottom: 16px;
}

.form-group label {
  display: block;
  margin-bottom: 6px;
  font-size: 13px;
  font-weight: 500;
  color: var(--color-text-secondary);
}

.form-input {
  width: 100%;
  padding: 10px 12px;
  border: 1px solid var(--color-border);
  border-radius: 8px;
  background: var(--color-background-elevated);
  font-size: 14px;
  box-sizing: border-box;
}

.form-input:focus {
  outline: none;
  border-color: var(--color-primary);
}

.form-textarea {
  resize: vertical;
  font-family: monospace;
}

.modal-footer {
  display: flex;
  justify-content: flex-end;
  gap: 12px;
  padding: 16px 20px;
  border-top: 1px solid var(--color-border);
}

.btn {
  padding: 10px 20px;
  border: none;
  border-radius: 8px;
  font-size: 14px;
  cursor: pointer;
  transition: all 0.2s;
}

.btn.btn-secondary {
  background: var(--color-background-elevated);
  border: 1px solid var(--color-border);
}

.btn.btn-secondary:hover {
  border-color: var(--color-text-secondary);
}

.btn.btn-primary {
  background: var(--color-primary);
  color: white;
}

.btn.btn-primary:hover {
  background: var(--color-primary-darker);
}

.btn.btn-primary:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
</style>
