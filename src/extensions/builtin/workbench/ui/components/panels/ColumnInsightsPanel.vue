<template>
  <div class="column-insights-panel">
    <div v-if="loading" class="loading-state">
      <NSpin size="small" />
      <span>正在分析...</span>
    </div>
    <template v-else-if="stats">
      <div class="insight-header">
        <span class="col-name">{{ stats.column_name }}</span>
        <span class="col-type">{{ stats.data_type }}</span>
      </div>
      <div class="insight-section">
        <div class="insight-row">
          <span class="label">总行数</span>
          <span class="value">{{ stats.total_count }}</span>
        </div>
        <div class="insight-row">
          <span class="label">非空值</span>
          <span class="value">{{ stats.total_count - stats.null_count }}</span>
        </div>
        <div class="insight-row">
          <span class="label">NULL 值</span>
          <span class="value">{{ stats.null_count }}</span>
        </div>
        <div class="insight-row">
          <span class="label">唯一值</span>
          <span class="value">{{ stats.unique_count ?? '-' }}</span>
        </div>
      </div>
      <div v-if="stats.numeric_stats" class="insight-section">
        <div class="section-title">数值统计</div>
        <div class="insight-row"><span class="label">最小值</span><span class="value">{{ formatNum(stats.numeric_stats.min) }}</span></div>
        <div class="insight-row"><span class="label">最大值</span><span class="value">{{ formatNum(stats.numeric_stats.max) }}</span></div>
        <div class="insight-row"><span class="label">平均值</span><span class="value">{{ formatNum(stats.numeric_stats.avg) }}</span></div>
        <div class="insight-row"><span class="label">中位数</span><span class="value">{{ formatNum(stats.numeric_stats.median) }}</span></div>
        <div class="insight-row"><span class="label">总和</span><span class="value">{{ formatNum(stats.numeric_stats.sum) }}</span></div>
        <div v-if="stats.numeric_stats.stddev" class="insight-row">
          <span class="label">标准差</span><span class="value">{{ formatNum(stats.numeric_stats.stddev) }}</span>
        </div>
      </div>
      <div v-if="stats.text_stats" class="insight-section">
        <div class="section-title">文本统计</div>
        <div class="insight-row">
          <span class="label">最短长度</span><span class="value">{{ stats.text_stats.min_length }}</span>
        </div>
        <div class="insight-row">
          <span class="label">最长长度</span><span class="value">{{ stats.text_stats.max_length }}</span>
        </div>
        <div class="section-subtitle">TOP 10 频率分布</div>
        <div v-for="(item, i) in stats.text_stats.top_values" :key="i" class="freq-row">
          <span class="freq-value">{{ item[0] }}</span>
          <span class="freq-count">{{ item[1] }}</span>
        </div>
      </div>
    </template>
    <div v-else class="empty-state">
      <BarChart3 :size="24" />
      <span>右键表格列 → 列洞察</span>
    </div>
  </div>
</template>

<script setup lang="ts">
import { BarChart3 } from 'lucide-vue-next'
import { NSpin } from 'naive-ui'
import { ref, onMounted, onUnmounted } from 'vue'

import { getColumnInsights } from '../../services/result-analysis'

import type { ColumnStats } from '../../services/result-analysis'

const stats = ref<ColumnStats | null>(null)
const loading = ref(false)
const currentTempTable = ref('')

const handleColumnInsight = async (event: CustomEvent) => {
  const columnName = event.detail?.column
  const tempTable = event.detail?.tempTable || currentTempTable.value
  if (!columnName) return
  if (!tempTable) {
    stats.value = null
    return
  }
  currentTempTable.value = tempTable
  loading.value = true
  try {
    const s = await getColumnInsights(tempTable, columnName)
    stats.value = s
  } catch {
    stats.value = null
  } finally {
    loading.value = false
  }
}

onMounted(() => {
  window.addEventListener('open-column-insight', handleColumnInsight as (e: Event) => void)
})
onUnmounted(() => {
  window.removeEventListener('open-column-insight', handleColumnInsight as (e: Event) => void)
})

function formatNum(n: number): string {
  if (Number.isInteger(n)) return n.toLocaleString()
  return n.toLocaleString(undefined, { maximumFractionDigits: 4 })
}
</script>

<style scoped>
.column-insights-panel {
  display: flex;
  flex-direction: column;
  height: 100%;
  padding: 12px;
  overflow-y: auto;
  background: var(--bg-primary);
  font-size: 12px;
  color: var(--text-primary);
}
.loading-state, .empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100%;
  gap: 8px;
  color: var(--text-tertiary, #888);
}
.insight-header { display: flex; align-items: center; gap: 8px; margin-bottom: 12px; }
.col-name { font-size: 14px; font-weight: 600; }
.col-type { font-size: 11px; color: var(--text-tertiary, #888); background: var(--bg-secondary, #333); padding: 1px 6px; border-radius: 3px; }
.insight-section { margin-bottom: 12px; }
.section-title { font-size: 11px; font-weight: 600; color: var(--text-secondary, #aaa); margin-bottom: 6px; text-transform: uppercase; }
.section-subtitle { font-size: 11px; font-weight: 600; color: var(--text-secondary, #aaa); margin: 8px 0 4px; }
.insight-row { display: flex; justify-content: space-between; padding: 3px 0; border-bottom: 1px solid var(--border-color, #333); }
.label { color: var(--text-secondary, #888); }
.value { font-family: monospace; color: var(--text-primary); }
.freq-row { display: flex; justify-content: space-between; padding: 2px 4px; font-size: 11px; }
.freq-value { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; flex: 1; }
.freq-count { font-family: monospace; color: var(--primary-color, #0078d4); margin-left: 8px; }
</style>
