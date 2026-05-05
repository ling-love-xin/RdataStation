<template>
  <div class="modal-overlay" @click.self="$emit('close')">
    <div class="modal large">
      <div class="modal-header">
        <h3>🗑️ 回收站</h3>
        <button class="close-btn" @click="$emit('close')">✕</button>
      </div>

      <div class="modal-body">
        <div v-if="recycleBin.length === 0" class="empty-state">
          <span class="empty-icon">🗑️</span>
          <p>回收站为空</p>
        </div>

        <div v-else class="recycle-list">
          <div
            v-for="item in recycleBin"
            :key="item.id"
            class="recycle-item"
          >
            <span class="resource-icon">
              {{ getResourceIcon(item.resource_type) }}
            </span>

            <div class="resource-info">
              <div class="resource-name">{{ item.resource_name }}</div>
              <div class="resource-meta">
                删除于: {{ formatDate(item.deleted_at) }}
              </div>
            </div>

            <div class="item-actions">
              <button class="btn btn-sm btn-primary" @click="restoreItem(item)">
                ↩️ 恢复
              </button>
              <button class="btn btn-sm btn-danger" @click="permanentDeleteItem(item)">
                🗑️ 永久删除
              </button>
            </div>
          </div>
        </div>
      </div>

      <div class="modal-footer">
        <button class="btn btn-secondary" @click="$emit('close')">
          关闭
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { useAnalyticsResourceStore } from '../stores/analytics-resource-store';
import type { AnalyticsRecycleItem } from '../../types';

const emit = defineEmits<{
  close: [];
}>();

const store = useAnalyticsResourceStore();
const recycleBin = ref<AnalyticsRecycleItem[]>([]);

async function loadRecycleBin() {
  await store.loadRecycleBin();
  recycleBin.value = store.recycleBin;
}

function getResourceIcon(type: string) {
  switch (type) {
    case 'connection': return '🔌';
    case 'table': return '📊';
    case 'file': return '📄';
    default: return '📦';
  }
}

function formatDate(dateStr: string) {
  return new Date(dateStr).toLocaleString('zh-CN');
}

async function restoreItem(item: AnalyticsRecycleItem) {
  if (!confirm(`确定恢复资源 "${item.resource_name}" 吗？`)) {
    return;
  }
  
  await store.restoreResource(item.id);
  await loadRecycleBin();
}

async function permanentDeleteItem(item: AnalyticsRecycleItem) {
  if (!confirm(`确定永久删除资源 "${item.resource_name}" 吗？此操作无法撤销！`)) {
    return;
  }
  
  await store.permanentDeleteResource(item.id);
  await loadRecycleBin();
}

onMounted(async () => {
  await loadRecycleBin();
});
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

.modal.large {
  max-width: 700px;
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
  flex: 1;
}

.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 48px;
  color: var(--color-text-secondary);
}

.empty-icon {
  font-size: 48px;
  margin-bottom: 16px;
  opacity: 0.5;
}

.recycle-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.recycle-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px;
  border: 1px solid var(--color-border);
  border-radius: 8px;
  background: var(--color-background-elevated);
}

.resource-icon {
  font-size: 24px;
}

.resource-info {
  flex: 1;
  min-width: 0;
}

.resource-name {
  font-size: 14px;
  font-weight: 500;
  margin-bottom: 2px;
}

.resource-meta {
  font-size: 12px;
  color: var(--color-text-secondary);
}

.item-actions {
  display: flex;
  gap: 8px;
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

.btn.btn-sm {
  padding: 6px 12px;
  font-size: 13px;
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

.btn.btn-danger {
  background: var(--color-error);
  color: white;
}

.btn.btn-danger:hover {
  background: #d32f2f;
}
</style>
