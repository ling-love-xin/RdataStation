<template>
  <div class="column-insights-panel">
    <div v-if="loading" class="loading-state">
      <NSpin size="small" />
      <span>{{ t('workbench.analyzing') }}</span>
    </div>
    <template v-else-if="stats">
      <div class="insight-header">
        <span class="col-name">{{ stats.column_name }}</span>
        <span class="col-type">{{ stats.data_type }}</span>
      </div>
      <div class="insight-section">
        <div class="insight-row">
          <span class="label">{{ t('workbench.totalRows') }}</span>
          <span class="value">{{ stats.total_count }}</span>
        </div>
        <div class="insight-row">
          <span class="label">{{ t('workbench.nonNullValues') }}</span>
          <span class="value">{{ stats.total_count - stats.null_count }}</span>
        </div>
        <div class="insight-row">
          <span class="label">{{ t('workbench.nullValues') }}</span>
          <span class="value">{{ stats.null_count }}</span>
        </div>
        <div class="insight-row">
          <span class="label">{{ t('workbench.uniqueValues') }}</span>
          <span class="value">{{ stats.unique_count ?? '-' }}</span>
        </div>
      </div>
      <div v-if="numericStats" class="insight-section">
        <div class="section-title">{{ t('workbench.numericStats') }}</div>
        <div class="insight-row"
          ><span class="label">{{ t('resultPanel.minLabel') }}</span
          ><span class="value">{{ formatNum(numericStats.min) }}</span></div
        >
        <div class="insight-row"
          ><span class="label">{{ t('resultPanel.maxLabel') }}</span
          ><span class="value">{{ formatNum(numericStats.max) }}</span></div
        >
        <div class="insight-row"
          ><span class="label">{{ t('resultPanel.meanLabel') }}</span
          ><span class="value">{{ formatNum(numericStats.avg) }}</span></div
        >
        <div class="insight-row"
          ><span class="label">{{ t('resultPanel.medianLabel') }}</span
          ><span class="value">{{ formatNum(numericStats.median) }}</span></div
        >
        <div class="insight-row"
          ><span class="label">{{ t('workbench.sum') }}</span
          ><span class="value">{{ formatNum(numericStats.sum) }}</span></div
        >
        <div v-if="numericStats.stddev" class="insight-row">
          <span class="label">{{ t('resultPanel.stddevLabel') }}</span
          ><span class="value">{{ formatNum(numericStats.stddev) }}</span>
        </div>
      </div>
      <div v-if="textStats" class="insight-section">
        <div class="section-title">{{ t('workbench.textStats') }}</div>
        <div class="insight-row">
          <span class="label">{{ t('workbench.minLength') }}</span
          ><span class="value">{{ textStats.min_length }}</span>
        </div>
        <div class="insight-row">
          <span class="label">{{ t('workbench.maxLength') }}</span
          ><span class="value">{{ textStats.max_length }}</span>
        </div>
        <div class="section-subtitle">{{ t('workbench.top10Frequency') }}</div>
        <div v-for="(item, i) in textStats.top_values" :key="i" class="freq-row">
          <span class="freq-value">{{ item.value }}</span>
          <span class="freq-count">{{ item.count }}</span>
        </div>
      </div>
    </template>
    <div v-else class="empty-state">
      <BarChart3 :size="24" />
      <span>{{ t('workbench.rightClickInsight') }}</span>
    </div>
  </div>
</template>

<script setup lang="ts">
import { BarChart3 } from 'lucide-vue-next'
import { NSpin } from 'naive-ui'
import { ref, computed, watch } from 'vue'
import { useI18n } from 'vue-i18n'

import { getColumnInsights } from '../../services/result-analysis'
import { useInsightStore } from '../../stores/insight-store'

import type {
  ColumnStats,
  NumericStatsDetail,
  TextStatsDetail,
} from '../../services/result-analysis'

const { t } = useI18n()
const insightStore = useInsightStore()

const stats = ref<ColumnStats | null>(null)
const loading = ref(false)
const currentTempTable = ref('')

const numericStats = computed((): NumericStatsDetail | null => {
  if (!stats.value || stats.value.stats_detail.kind !== 'Numeric') return null
  return stats.value.stats_detail as NumericStatsDetail
})

const textStats = computed((): TextStatsDetail | null => {
  if (!stats.value || stats.value.stats_detail.kind !== 'Text') return null
  return stats.value.stats_detail as TextStatsDetail
})

watch(
  () => ({ col: insightStore.currentColumn, table: insightStore.currentTempTable }),
  async ({ col, table }) => {
    if (!col || !table) {
      stats.value = null
      return
    }
    currentTempTable.value = table
    loading.value = true
    try {
      stats.value = await getColumnInsights(table, col)
    } catch {
      console.error('[ColumnInsightsPanel] loadInsights failed')
      stats.value = null
    } finally {
      loading.value = false
    }
  },
  { immediate: true }
)

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
.loading-state,
.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100%;
  gap: 8px;
  color: var(--text-tertiary, #888);
}
.insight-header {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 12px;
}
.col-name {
  font-size: 14px;
  font-weight: 600;
}
.col-type {
  font-size: 11px;
  color: var(--text-tertiary, #888);
  background: var(--bg-secondary, #333);
  padding: 1px 6px;
  border-radius: 3px;
}
.insight-section {
  margin-bottom: 12px;
}
.section-title {
  font-size: 11px;
  font-weight: 600;
  color: var(--text-secondary, #aaa);
  margin-bottom: 6px;
  text-transform: uppercase;
}
.section-subtitle {
  font-size: 11px;
  font-weight: 600;
  color: var(--text-secondary, #aaa);
  margin: 8px 0 4px;
}
.insight-row {
  display: flex;
  justify-content: space-between;
  padding: 3px 0;
  border-bottom: 1px solid var(--border-color, #333);
}
.label {
  color: var(--text-secondary, #888);
}
.value {
  font-family: monospace;
  color: var(--text-primary);
}
.freq-row {
  display: flex;
  justify-content: space-between;
  padding: 2px 4px;
  font-size: 11px;
}
.freq-value {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  flex: 1;
}
.freq-count {
  font-family: monospace;
  color: var(--primary-color, #0078d4);
  margin-left: 8px;
}
</style>
