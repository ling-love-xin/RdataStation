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
        <button class="btn btn-primary" :disabled="!isValid" @click="handleCreate">
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
