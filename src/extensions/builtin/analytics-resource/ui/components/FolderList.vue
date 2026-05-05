<template>
  <div class="folders-section">
    <div class="folders-header">
      <span class="folders-title">文件夹</span>
      <button class="btn btn-sm" @click="emit('createFolder')">
        <span class="icon">+</span>
        新建
      </button>
    </div>
    <div class="folders-list">
      <div
        :class="['folder-item', { active: selectedFolderId === null }]"
        @click="selectFolder(null)"
      >
        <span class="folder-icon">📁</span>
        <span class="folder-name">全部资源</span>
      </div>
      <div
        v-for="folder in folders"
        :key="folder.id"
        :class="['folder-item', { active: selectedFolderId === folder.id }]"
        @click="selectFolder(folder.id)"
      >
        <span class="folder-icon">{{ folder.icon || '📁' }}</span>
        <span class="folder-name">{{ folder.name }}</span>
        <span
          v-if="folder.color"
          class="folder-color"
          :style="{ backgroundColor: folder.color }"
        />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import type { AnalyticsFolder } from '../../types'

defineProps<{
  folders: AnalyticsFolder[]
  selectedFolderId: string | null
}>()

const emit = defineEmits<{
  selectFolder: [id: string | null]
  createFolder: []
}>()

function selectFolder(id: string | null) {
  emit('selectFolder', id)
}
</script>

<style scoped>
.folders-section {
  padding: 12px 16px;
  border-bottom: 1px solid var(--color-border, #d9d9d9);
}

.folders-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 8px;
}

.folders-title {
  font-size: 13px;
  font-weight: 600;
  color: var(--text-secondary, #666);
  text-transform: uppercase;
}

.btn {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 4px 8px;
  border: 1px solid var(--color-border, #d9d9d9);
  border-radius: 4px;
  background: transparent;
  font-size: 12px;
  cursor: pointer;
  transition: all 0.15s;
}

.btn:hover {
  border-color: var(--color-primary, #165dff);
  color: var(--color-primary, #165dff);
}

.btn-sm {
  padding: 4px 8px;
  font-size: 12px;
}

.icon {
  font-size: 14px;
}

.folders-list {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}

.folder-item {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 8px 12px;
  border: 1px solid var(--color-border, #d9d9d9);
  border-radius: 8px;
  background: var(--color-background, #fff);
  cursor: pointer;
  transition: all 0.2s;
}

.folder-item:hover {
  border-color: var(--color-primary, #165dff);
}

.folder-item.active {
  border-color: var(--color-primary, #165dff);
  background: var(--color-primary-lighter, #e8f0ff);
}

.folder-icon {
  font-size: 16px;
}

.folder-name {
  font-size: 14px;
}

.folder-color {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  margin-left: 4px;
}
</style>
