<template>
  <div class="result-statusbar">
    <div class="status-left">
      <NButton size="tiny" quaternary @click="$emit('refresh')">
        <template #icon><RotateCw :size="12" /></template>
        刷新
      </NButton>
      <NButton size="tiny" quaternary :disabled="!hasDirty" @click="$emit('save')">
        <template #icon><Save :size="12" /></template>
        保存
      </NButton>
      <NButton size="tiny" quaternary :disabled="!hasDirty" @click="$emit('cancel')">
        <template #icon><X :size="12" /></template>
        取消
      </NButton>
      <NButton size="tiny" quaternary @click="$emit('export')">
        <template #icon><Download :size="12" /></template>
        导出数据...
      </NButton>
    </div>
    <div class="status-center">
      <span class="mode-tag" :class="filterMode">{{ modeLabel }}</span>
      <span class="row-info">{{ rowInfoText }}</span>
    </div>
    <div class="status-right">
      <span v-if="durationText" class="duration">{{ durationText }}</span>
      <span class="timestamp">{{ timestamp }}</span>
    </div>
  </div>
</template>

<script setup lang="ts">
import { RotateCw, Save, X, Download } from 'lucide-vue-next'
import { NButton } from 'naive-ui'
import { computed } from 'vue'

import type { FilterMode } from '../../../stores/result-store'

const props = defineProps<{
  filterMode: FilterMode
  totalRows: number
  visibleRows: number
  originalRows: number
  hasDirty: boolean
  duration: number
  timestamp: string
}>()

defineEmits<{
  refresh: []
  save: []
  cancel: []
  export: []
}>()

const modeLabel = computed(() => {
  const map: Record<FilterMode, string> = { quick: '即时过滤', sql: 'SQL过滤', duckdb: 'DuckDB分析' }
  return map[props.filterMode]
})

const rowInfoText = computed(() => {
  if (props.filterMode === 'duckdb') {
    return `原始 ${props.originalRows} 行 | 分析结果 ${props.visibleRows} 行`
  }
  if (props.visibleRows !== props.totalRows) {
    return `原始 ${props.totalRows} 行 → 过滤后 ${props.visibleRows} 行`
  }
  return `${props.totalRows} 行`
})

const durationText = computed(() => {
  if (!props.duration) return ''
  const sec = (props.duration / 1000).toFixed(3)
  const map: Record<FilterMode, string> = { quick: '', sql: `数据库 ${sec}s`, duckdb: `DuckDB ${sec}s` }
  return map[props.filterMode]
})
</script>

<style scoped>
.result-statusbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  height: 26px;
  padding: 0 4px;
  background: var(--bg-tertiary, #2d2d30);
  border-top: 1px solid var(--border-color, #3e3e42);
  font-size: 11px;
  color: var(--text-secondary, #858585);
  flex-shrink: 0;
}
.status-left, .status-center, .status-right {
  display: flex;
  align-items: center;
  gap: 2px;
}
.mode-tag {
  padding: 0 6px;
  border-radius: 3px;
  font-size: 10px;
  font-weight: 600;
}
.mode-tag.quick { background: #2d6a4f33; color: #52c41a; }
.mode-tag.sql { background: #1a5a8a33; color: #1890ff; }
.mode-tag.duckdb { background: #613a8a33; color: #b37feb; }
.row-info { margin-left: 8px; }
.duration { color: var(--primary-color, #0078d4); }
.timestamp { color: var(--text-tertiary, #666); font-size: 10px; margin-left: 8px; }
</style>
