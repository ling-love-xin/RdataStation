<template>
  <div v-if="show" class="filter-panel">
    <div class="filter-header">
      <span class="filter-title">过滤器</span>
      <button class="filter-close-btn" @click="$emit('close')">
        <X :size="14" />
      </button>
    </div>
    <div class="filter-options">
      <label class="filter-option">
        <input type="checkbox" :checked="config.showTables" @change="$emit('update:showTables', ($event.target as HTMLInputElement).checked)" />
        <span>显示表</span>
      </label>
      <label class="filter-option">
        <input type="checkbox" :checked="config.showViews" @change="$emit('update:showViews', ($event.target as HTMLInputElement).checked)" />
        <span>显示视图</span>
      </label>
      <label class="filter-option">
        <input type="checkbox" :checked="config.showColumns" @change="$emit('update:showColumns', ($event.target as HTMLInputElement).checked)" />
        <span>显示列</span>
      </label>
      <label class="filter-option">
        <input type="checkbox" :checked="config.showSystemSchemas" @change="$emit('update:showSystemSchemas', ($event.target as HTMLInputElement).checked)" />
        <span>显示系统 Schema</span>
      </label>
    </div>
  </div>
</template>

<script setup lang="ts">
import { X } from 'lucide-vue-next'

interface FilterConfig {
  showTables: boolean
  showViews: boolean
  showColumns: boolean
  showSystemSchemas: boolean
}

defineProps<{
  show: boolean
  config: FilterConfig
}>()

defineEmits<{
  close: []
  'update:showTables': [value: boolean]
  'update:showViews': [value: boolean]
  'update:showColumns': [value: boolean]
  'update:showSystemSchemas': [value: boolean]
}>()
</script>

<style scoped>
.filter-panel {
  padding: 8px;
  background: var(--bg-tertiary);
  border-bottom: 1px solid var(--border-color);
}

.filter-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 8px;
}

.filter-title {
  font-size: 13px;
  font-weight: 500;
  color: var(--text-primary);
}

.filter-close-btn {
  width: 18px;
  height: 18px;
  border: none;
  border-radius: 2px;
  background: transparent;
  color: var(--text-secondary);
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
}

.filter-close-btn:hover {
  background: var(--bg-hover);
  color: var(--text-primary);
}

.filter-options {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.filter-option {
  display: flex;
  align-items: center;
  gap: 6px;
  cursor: pointer;
  font-size: 12px;
  color: var(--text-secondary);
}

.filter-option:hover {
  color: var(--text-primary);
}

.filter-option input[type="checkbox"] {
  margin: 0;
}
</style>
