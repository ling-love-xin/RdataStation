<template>
  <div class="filter-bar">
    <div class="filter-group">
      <span class="filter-label">作用域:</span>
      <div class="filter-buttons">
        <button
          v-for="scope in scopes"
          :key="scope.value ?? 'all'"
          :class="['filter-btn', { active: selectedScope === scope.value }]"
          @click="selectScope(scope.value)"
        >
          {{ scope.label }}
        </button>
      </div>
    </div>

    <div class="filter-group">
      <span class="filter-label">类型:</span>
      <div class="filter-buttons">
        <button
          v-for="type in types"
          :key="type.value ?? 'all'"
          :class="['filter-btn', { active: selectedType === type.value }]"
          @click="selectType(type.value)"
        >
          {{ type.label }}
        </button>
      </div>
    </div>

    <div class="filter-group">
      <span class="filter-label">排序:</span>
      <div class="filter-buttons">
        <button
          v-for="sort in sortOptions"
          :key="sort.value"
          :class="['filter-btn', { active: selectedSort === sort.value }]"
          @click="selectSort(sort.value)"
        >
          {{ sort.label }}
          <span v-if="selectedSort === sort.value" class="sort-indicator">
            {{ sortOrder === 'asc' ? '↑' : '↓' }}
          </span>
        </button>
      </div>
    </div>

    <div v-if="selectedCount > 0" class="selection-info">
      <span>已选择 {{ selectedCount }} 项</span>
      <button class="batch-action-btn danger" @click="emit('batchDelete')">
        🗑️ 批量删除
      </button>
      <button class="clear-selection-btn" @click="clearSelection">
        清空选择
      </button>
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

const sortOptions = [
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

function clearSelection() {
  emit('clearSelection')
}
</script>

<style scoped>
.filter-bar {
  display: flex;
  flex-wrap: wrap;
  gap: 16px;
  padding: 12px 16px;
  border-bottom: 1px solid var(--color-border, #d9d9d9);
  background: var(--color-background-elevated, #f5f5f5);
}

.filter-group {
  display: flex;
  align-items: center;
  gap: 8px;
}

.filter-label {
  font-size: 13px;
  color: var(--text-secondary, #666);
  white-space: nowrap;
}

.filter-buttons {
  display: flex;
  gap: 4px;
}

.filter-btn {
  padding: 6px 12px;
  border: 1px solid var(--color-border, #d9d9d9);
  border-radius: 6px;
  background: var(--color-background, #fff);
  font-size: 13px;
  cursor: pointer;
  transition: all 0.2s;
  white-space: nowrap;
}

.filter-btn:hover {
  border-color: var(--color-primary, #165dff);
}

.filter-btn.active {
  background: var(--color-primary, #165dff);
  border-color: var(--color-primary, #165dff);
  color: white;
}

.selection-info {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-left: auto;
  font-size: 13px;
  color: var(--text-secondary, #666);
}

.clear-selection-btn {
  padding: 4px 8px;
  border: 1px solid var(--color-border, #d9d9d9);
  border-radius: 4px;
  background: transparent;
  font-size: 12px;
  cursor: pointer;
  transition: all 0.15s;
}

.clear-selection-btn:hover {
  border-color: var(--color-error, #f53f3f);
  color: var(--color-error, #f53f3f);
}
</style>
