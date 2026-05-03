<template>
  <div class="duckdb-analysis-input">
    <div class="filter-row">
      <NInput
        :value="sql"
        size="tiny"
        type="textarea"
        :autosize="{ minRows: 1, maxRows: 3 }"
        placeholder="输入 DuckDB SQL 分析语句&#10;用 {table} 或 result_temp 引用临时表"
        @update:value="emit('update:sql', $event || '')"
        @keydown.enter.ctrl="onExecute"
      />
      <div class="action-btns">
        <NButton size="tiny" type="primary" :loading="loading" @click="onExecute">
          <template #icon><Play :size="12" /></template>
          执行
        </NButton>
        <NButton size="tiny" quaternary @click="emit('clear')">
          <template #icon><X :size="12" /></template>
          清除
        </NButton>
      </div>
    </div>
    <div class="quick-actions">
      <span class="label">快捷分析:</span>
      <NButton size="tiny" text @click="emit('quick', 'count')">
        <template #icon><Hash :size="11" /></template>
        计数
      </NButton>
      <NButton size="tiny" text @click="emit('quick', 'distinct')">
        <template #icon><List :size="11" /></template>
        去重
      </NButton>
      <NButton size="tiny" text @click="emit('quick', 'group')">
        <template #icon><BarChart3 :size="11" /></template>
        分组
      </NButton>
      <span class="divider" />
      <NButton size="tiny" text title="将当前前端过滤结果写入 DuckDB 临时表进行分析" @click="emit('bridgeFilter')">
        <template #icon><Zap :size="11" /></template>
        基于前端过滤结果分析
      </NButton>
      <span class="hint">Ctrl+Enter 执行</span>
    </div>
  </div>
</template>

<script setup lang="ts">
import { Play, X, Hash, List, BarChart3, Zap } from 'lucide-vue-next'
import { NInput, NButton } from 'naive-ui'

defineProps<{
  sql: string
  loading: boolean
}>()

const emit = defineEmits<{
  'update:sql': [string]
  execute: []
  clear: []
  quick: [type: string]
  bridgeFilter: []
}>()

function onExecute() { emit('execute') }
</script>

<style scoped>
.duckdb-analysis-input { padding: 4px 8px; border-bottom: 1px solid var(--border-color, #333); flex-shrink: 0; }
.filter-row { display: flex; gap: 4px; align-items: flex-start; }
.action-btns { display: flex; flex-direction: column; gap: 2px; }
.quick-actions {
  display: flex; align-items: center; gap: 6px; margin-top: 4px;
  font-size: 11px; color: var(--text-tertiary, #888);
}
.label { white-space: nowrap; }
.divider { width: 1px; height: 14px; background: var(--border-color, #444); }
.hint { margin-left: auto; }
</style>
