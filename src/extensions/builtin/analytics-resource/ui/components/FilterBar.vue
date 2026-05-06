<template>
  <div class="filter-bar">
    <div class="filter-row">
      <div class="filter-group compact">
        <span class="filter-label">作用域:</span>
        <select
          class="filter-select"
          :value="selectedScope ?? 'all'"
          @change="selectScope(($event.target as HTMLSelectElement).value === 'all' ? null : ($event.target as HTMLSelectElement).value)"
        >
          <option value="all">全部</option>
          <option value="global">🌍 全局</option>
          <option value="project">📂 项目</option>
          <option value="session">📌 会话</option>
        </select>
      </div>

      <div class="filter-group compact">
        <span class="filter-label">类型:</span>
        <select
          class="filter-select"
          :value="selectedType ?? 'all'"
          @change="selectType(($event.target as HTMLSelectElement).value === 'all' ? null : ($event.target as HTMLSelectElement).value)"
        >
          <option value="all">全部</option>
          <option value="connection">🔌 连接</option>
          <option value="table">📊 表</option>
          <option value="file">📄 文件</option>
        </select>
      </div>

      <div class="filter-group compact">
        <span class="filter-label">排序:</span>
        <select
          class="filter-select"
          :value="selectedSort ?? 'name'"
          @change="selectSort(($event.target as HTMLSelectElement).value as SortField)"
        >
          <option value="name">名称</option>
          <option value="created_at">创建时间</option>
          <option value="updated_at">更新时间</option>
          <option value="row_count">行数</option>
          <option value="file_size">大小</option>
        </select>
        <button
          class="sort-order-btn"
          :title="sortOrder === 'asc' ? '升序' : '降序'"
          @click="toggleSortOrder"
        >
          {{ sortOrder === 'asc' ? '↑' : '↓' }}
        </button>
      </div>

      <div v-if="selectedCount > 0" class="selection-info">
        <span>已选 {{ selectedCount }} 项</span>
        <button class="batch-action-btn danger" @click="emit('batchDelete')">🗑️</button>
        <button class="clear-selection-btn" @click="clearSelection">清空</button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import type { SortField, SortOrder } from '../../types'

const props = defineProps<{
  selectedScope: string | null
  selectedType: string | null
  selectedCount: number
  selectedSort: SortField | null
  sortOrder: SortOrder
}>()

const emit = defineEmits<{
  'update:selectedScope': [value: string | null]
  'update:selectedType': [value: string | null]
  'update:selectedSort': [value: SortField | null]
  'update:sortOrder': [value: SortOrder]
  'clearSelection': []
  'batchDelete': []
}>()

const scopes = [
  { value: null, label: '全部' },
  { value: 'global', label: '🌍 全局' },
  { value: 'project', label: '📂 项目' },
  { value: 'session', label: '📌 会话' },
]

const types = [
  { value: null, label: '全部' },
  { value: 'connection', label: '🔌 连接' },
  { value: 'table', label: '📊 表' },
  { value: 'file', label: '📄 文件' },
]

const sortOptions: { value: SortField; label: string }[] = [
  { value: 'name', label: '名称' },
  { value: 'created_at', label: '创建时间' },
  { value: 'updated_at', label: '更新时间' },
  { value: 'row_count', label: '行数' },
  { value: 'file_size', label: '大小' },
]

function selectScope(value: string | null) {
  emit('update:selectedScope', value)
}

function selectType(value: string | null) {
  emit('update:selectedType', value)
}

function selectSort(value: SortField | null) {
  if (props.selectedSort === value) {
    emit('update:sortOrder', props.sortOrder === 'asc' ? 'desc' : 'asc')
  } else {
    emit('update:selectedSort', value)
    emit('update:sortOrder', 'asc')
  }
}

function toggleSortOrder() {
  emit('update:sortOrder', props.sortOrder === 'asc' ? 'desc' : 'asc')
}

function clearSelection() {
  emit('clearSelection')
}
</script>

<style scoped>
.filter-bar {
  padding: var(--size-sm) var(--size-md);
  border-bottom: 1px solid var(--border-color);
  background: var(--bg-secondary);
}

.filter-row {
  display: flex;
  align-items: center;
  gap: var(--size-md);
  flex-wrap: nowrap;
}

.filter-group {
  display: flex;
  align-items: center;
  gap: var(--size-xs);
}

.filter-group.compact {
  gap: 4px;
}

.filter-label {
  font-size: 12px;
  color: var(--text-secondary);
  white-space: nowrap;
}

.filter-select {
  padding: 2px 6px;
  border: 1px solid var(--border-color);
  border-radius: var(--radius-sm);
  background: var(--bg-primary);
  color: var(--text-primary);
  font-size: 12px;
  cursor: pointer;
  height: 26px;
  min-width: 80px;
  outline: none;
  transition: border-color 0.2s;
}

.filter-select:hover {
  border-color: var(--primary-color);
}

.filter-select:focus {
  border-color: var(--primary-color);
  box-shadow: 0 0 0 1px var(--primary-color);
}

.sort-order-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 24px;
  height: 26px;
  border: 1px solid var(--border-color);
  border-radius: var(--radius-sm);
  background: var(--bg-primary);
  color: var(--text-secondary);
  font-size: 12px;
  cursor: pointer;
  transition: all 0.2s;
}

.sort-order-btn:hover {
  border-color: var(--primary-color);
  color: var(--primary-color);
}

.selection-info {
  display: flex;
  align-items: center;
  gap: var(--size-sm);
  margin-left: auto;
  font-size: 12px;
  color: var(--text-secondary);
}

.clear-selection-btn {
  padding: 2px 6px;
  border: 1px solid var(--border-color);
  border-radius: var(--radius-sm);
  background: transparent;
  color: var(--text-secondary);
  font-size: 11px;
  cursor: pointer;
  transition: all 0.15s;
  height: 24px;
}

.clear-selection-btn:hover {
  border-color: var(--danger-color);
  color: var(--danger-color);
}

.batch-action-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 24px;
  height: 24px;
  border: 1px solid var(--border-color);
  border-radius: var(--radius-sm);
  background: transparent;
  font-size: 12px;
  cursor: pointer;
  transition: all 0.15s;
}

.batch-action-btn.danger {
  color: var(--danger-color);
}

.batch-action-btn.danger:hover {
  border-color: var(--danger-color);
  background: var(--danger-light);
}
</style>
