<template>
  <div class="modal-overlay" @click.self="$emit('close')">
    <div class="modal">
      <div class="modal-header">
        <h3>创建文件夹</h3>
        <button class="close-btn" @click="$emit('close')">✕</button>
      </div>

      <div class="modal-body">
        <div class="form-group">
          <label>文件夹名称 *</label>
          <input
            v-model="form.name"
            type="text"
            class="form-input"
            placeholder="输入文件夹名称"
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

        <div class="form-group">
          <label>颜色</label>
          <div class="color-picker">
            <input
              v-model="form.color"
              type="color"
              class="color-input"
            />
            <span class="color-preview" :style="{ backgroundColor: form.color || '#6366f1' }"></span>
          </div>
        </div>

        <div class="form-group">
          <label>图标</label>
          <div class="icon-selector">
            <button
              v-for="icon in icons"
              :key="icon"
              :class="['icon-btn', { active: form.icon === icon }]"
              @click="form.icon = icon"
              type="button"
            >
              {{ icon }}
            </button>
          </div>
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
import type { CreateFolderRequest, ResourceScope } from '../../types';

const emit = defineEmits<{
  close: [];
  create: [input: CreateFolderRequest];
}>();

const form = ref({
  name: '',
  scope: 'project' as ResourceScope,
  parent_folder_id: undefined as string | undefined,
  color: '#6366f1',
  icon: '📁',
});

const icons = ['📁', '📂', '🗂️', '📦', '💼', '📋', '📊', '📈', '🎯', '⭐'];

const isValid = computed(() => {
  return form.value.name.trim() !== '';
});

function handleCreate() {
  const input: CreateFolderRequest = {
    name: form.value.name.trim(),
    scope: form.value.scope,
    parent_folder_id: form.value.parent_folder_id,
    color: form.value.color,
    icon: form.value.icon,
  };
  
  emit('create', input);
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

.color-picker {
  display: flex;
  align-items: center;
  gap: 12px;
}

.color-input {
  width: 40px;
  height: 40px;
  padding: 0;
  border: 1px solid var(--color-border);
  border-radius: 8px;
  cursor: pointer;
}

.color-preview {
  width: 40px;
  height: 40px;
  border-radius: 8px;
  border: 1px solid var(--color-border);
}

.icon-selector {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}

.icon-btn {
  width: 40px;
  height: 40px;
  display: flex;
  align-items: center;
  justify-content: center;
  border: 1px solid var(--color-border);
  border-radius: 8px;
  background: var(--color-background-elevated);
  font-size: 20px;
  cursor: pointer;
  transition: all 0.2s;
}

.icon-btn:hover {
  border-color: var(--color-primary);
}

.icon-btn.active {
  border-color: var(--color-primary);
  background: var(--color-primary-lighter);
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
