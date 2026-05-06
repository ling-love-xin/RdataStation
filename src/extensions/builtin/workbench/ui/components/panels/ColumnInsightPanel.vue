<template>
  <div class="insight-panel" :class="{ dark: uiStore.isDark }">
    <div v-if="!insightStore.hasData && !insightStore.isLoading && !insightStore.error" class="insight-empty">
      <Search :size="28" class="empty-icon" />
      <span class="empty-text">点击结果集列头查看洞察</span>
      <span class="empty-hint">或右键点击列 → "查看列洞察"</span>
    </div>

    <div v-else-if="insightStore.isLoading" class="insight-loading">
      <NSpin size="small" />
      <span class="loading-text">正在分析...</span>
    </div>

    <div v-else-if="insightStore.error" class="insight-error">
      <AlertTriangle :size="20" class="error-icon" />
      <span class="error-text">{{ insightStore.error }}</span>
      <NButton size="tiny" quaternary @click="retry">重试</NButton>
    </div>

    <template v-else-if="insightStore.insightData">
      <div class="insight-header">
        <span class="insight-title">列洞察: {{ insightStore.currentColumn }}</span>
        <NButton size="tiny" quaternary @click="insightStore.clear()">
          <X :size="12" />
        </NButton>
      </div>

      <div class="insight-body">
        <NCollapse :default-expanded-names="['basic', 'dist', 'quality', 'sample']">
          <NCollapseItem name="basic" title="基础统计">
            <div class="stat-grid">
              <div class="stat-row">
                <span class="stat-label">计数</span>
                <span class="stat-value">{{ insightStore.totalCountDisplay }}</span>
              </div>
              <div class="stat-row">
                <span class="stat-label">类型</span>
                <NTag :bordered="false" size="tiny">{{ insightStore.insightData.stats.data_type }}</NTag>
              </div>
              <div class="stat-row">
                <span class="stat-label">空值率</span>
                <span class="stat-value" :class="{ 'text-warning': insightStore.insightData.stats.null_rate > 0.05 }">
                  {{ insightStore.nullRateDisplay }}
                  <span class="stat-sub">({{ insightStore.insightData.stats.null_count }} 行)</span>
                </span>
              </div>
              <div v-if="insightStore.insightData.stats.unique_count != null" class="stat-row">
                <span class="stat-label">去重数</span>
                <span class="stat-value">{{ insightStore.insightData.stats.unique_count.toLocaleString() }}</span>
              </div>

              <template v-if="insightStore.statsKind === 'Numeric'">
                <div class="stat-row">
                  <span class="stat-label">均值</span>
                  <span class="stat-value mono">{{ fmtNum((insightStore.insightData.stats.stats_detail as NumericStatsDetail).avg) }}</span>
                </div>
                <div class="stat-row">
                  <span class="stat-label">中位数</span>
                  <span class="stat-value mono">{{ fmtNum((insightStore.insightData.stats.stats_detail as NumericStatsDetail).median) }}</span>
                </div>
                <div class="stat-row">
                  <span class="stat-label">最小值</span>
                  <span class="stat-value mono">{{ fmtNum((insightStore.insightData.stats.stats_detail as NumericStatsDetail).min) }}</span>
                </div>
                <div class="stat-row">
                  <span class="stat-label">最大值</span>
                  <span class="stat-value mono">{{ fmtNum((insightStore.insightData.stats.stats_detail as NumericStatsDetail).max) }}</span>
                </div>
                <div class="stat-row">
                  <span class="stat-label">P25 / P75</span>
                  <span class="stat-value mono">{{ fmtNum((insightStore.insightData.stats.stats_detail as NumericStatsDetail).p25) }} / {{ fmtNum((insightStore.insightData.stats.stats_detail as NumericStatsDetail).p75) }}</span>
                </div>
                <div v-if="(insightStore.insightData.stats.stats_detail as NumericStatsDetail).stddev != null" class="stat-row">
                  <span class="stat-label">标准差</span>
                  <span class="stat-value mono">{{ fmtNum((insightStore.insightData.stats.stats_detail as NumericStatsDetail).stddev!) }}</span>
                </div>
                <div v-if="(insightStore.insightData.stats.stats_detail as NumericStatsDetail).skewness != null" class="stat-row">
                  <span class="stat-label">偏度</span>
                  <span class="stat-value mono">{{ fmtNum((insightStore.insightData.stats.stats_detail as NumericStatsDetail).skewness!) }}
                    <span class="stat-sub">{{ skewDesc((insightStore.insightData.stats.stats_detail as NumericStatsDetail).skewness!) }}</span>
                  </span>
                </div>
              </template>

              <template v-if="insightStore.statsKind === 'Text'">
                <div class="stat-row">
                  <span class="stat-label">长度范围</span>
                  <span class="stat-value">{{ (insightStore.insightData.stats.stats_detail as TextStatsDetail).min_length }} ~ {{ (insightStore.insightData.stats.stats_detail as TextStatsDetail).max_length }}</span>
                </div>
              </template>

              <template v-if="insightStore.statsKind === 'DateTime'">
                <div class="stat-row">
                  <span class="stat-label">最早</span>
                  <span class="stat-value mono small">{{ (insightStore.insightData.stats.stats_detail as DateTimeStatsDetail).earliest }}</span>
                </div>
                <div class="stat-row">
                  <span class="stat-label">最晚</span>
                  <span class="stat-value mono small">{{ (insightStore.insightData.stats.stats_detail as DateTimeStatsDetail).latest }}</span>
                </div>
                <div class="stat-row">
                  <span class="stat-label">跨度</span>
                  <span class="stat-value">{{ (insightStore.insightData.stats.stats_detail as DateTimeStatsDetail).span_days }} 天</span>
                </div>
              </template>

              <template v-if="insightStore.statsKind === 'Boolean'">
                <div class="stat-row">
                  <span class="stat-label">True</span>
                  <span class="stat-value">{{ (insightStore.insightData.stats.stats_detail as BooleanStatsDetail).true_count.toLocaleString() }}
                    <span class="stat-sub">({{ ((insightStore.insightData.stats.stats_detail as BooleanStatsDetail).true_ratio * 100).toFixed(1) }}%)</span>
                  </span>
                </div>
                <div class="stat-row">
                  <span class="stat-label">False</span>
                  <span class="stat-value">{{ (insightStore.insightData.stats.stats_detail as BooleanStatsDetail).false_count.toLocaleString() }}</span>
                </div>
              </template>
            </div>
          </NCollapseItem>

          <NCollapseItem v-if="hasDistribution" name="dist" title="数据分布">
            <template v-if="insightStore.statsKind === 'Numeric'">
              <div class="histogram">
                <div v-for="bin in insightStore.insightData.histogram" :key="bin.label" class="histo-row">
                  <span class="histo-label">{{ bin.label }}</span>
                  <div class="histo-bar-wrap">
                    <div class="histo-bar" :style="{ width: (bin.ratio * 100).toFixed(1) + '%' }"></div>
                  </div>
                  <span class="histo-ratio">{{ (bin.ratio * 100).toFixed(1) }}%</span>
                </div>
              </div>
            </template>

            <template v-if="insightStore.statsKind === 'Text'">
              <div class="freq-list">
                <button
                  v-for="(item, i) in (insightStore.insightData.stats.stats_detail as TextStatsDetail).top_values"
                  :key="i"
                  class="freq-item freq-clickable"
                  @click="filterByValue(item.value)"
                >
                  <span class="freq-label" :title="item.value">[{{ item.value }}]</span>
                  <div class="freq-bar-wrap">
                    <div class="freq-bar" :style="{ width: (item.ratio * 100).toFixed(1) + '%' }"></div>
                  </div>
                  <span class="freq-ratio">{{ (item.ratio * 100).toFixed(1) }}%</span>
                </button>
              </div>
            </template>

            <template v-if="insightStore.statsKind === 'DateTime'">
              <div class="freq-list">
                <div
                  v-for="(item, i) in (insightStore.insightData.stats.stats_detail as DateTimeStatsDetail).monthly_distribution"
                  :key="i"
                  class="freq-item"
                >
                  <span class="freq-label">{{ item.value }}</span>
                  <div class="freq-bar-wrap">
                    <div class="freq-bar freq-bar-datetime" :style="{ width: (item.ratio * 100).toFixed(1) + '%' }"></div>
                  </div>
                  <span class="freq-ratio">{{ (item.ratio * 100).toFixed(1) }}%</span>
                </div>
              </div>
            </template>

            <template v-if="insightStore.statsKind === 'Boolean'">
              <div class="bool-dist">
                <div class="histo-row">
                  <span class="histo-label">True</span>
                  <div class="histo-bar-wrap">
                    <div class="histo-bar histo-bar-bool" :style="{ width: ((insightStore.insightData.stats.stats_detail as BooleanStatsDetail).true_ratio * 100).toFixed(1) + '%' }"></div>
                  </div>
                  <span class="histo-ratio">{{ ((insightStore.insightData.stats.stats_detail as BooleanStatsDetail).true_ratio * 100).toFixed(1) }}%</span>
                </div>
              </div>
            </template>
          </NCollapseItem>

          <NCollapseItem name="quality" title="数据质量">
            <div class="quality-list">
              <div v-if="insightStore.insightData.stats.null_rate === 0" class="quality-item quality-ok">
                ✅ 空值率低于阈值
              </div>
              <div v-else-if="insightStore.insightData.stats.null_rate > 0.1" class="quality-item quality-warn">
                ⚠️ 高空值率 ({{ insightStore.nullRateDisplay }})
              </div>

              <div v-if="insightStore.statsKind === 'Numeric'">
                <div
                  v-for="(ext, i) in (insightStore.insightData.stats.stats_detail as NumericStatsDetail).is_extreme"
                  :key="i"
                  class="quality-item quality-warn"
                >
                  ⚠️ 存在极端值 ({{ ext.value }})
                </div>
                <div v-if="(insightStore.insightData.stats.stats_detail as NumericStatsDetail).skewness != null">
                  <div
v-if="Math.abs((insightStore.insightData.stats.stats_detail as NumericStatsDetail).skewness!) > 1"
                    class="quality-item quality-info">
                    ℹ️ {{ skewDesc((insightStore.insightData.stats.stats_detail as NumericStatsDetail).skewness!) }}
                  </div>
                </div>
              </div>

              <div
v-if="insightStore.statsKind === 'Text' && (insightStore.insightData.stats.stats_detail as TextStatsDetail).top_values.length >= 5"
                class="quality-item quality-info">
                ℹ️ 分类数为 {{ (insightStore.insightData.stats.stats_detail as TextStatsDetail).top_values.length }}
              </div>

              <div
v-if="insightStore.statsKind === 'Boolean' && (insightStore.insightData.stats.stats_detail as BooleanStatsDetail).true_ratio > 0.95"
                class="quality-item quality-info">
                ℹ️ 高度不平衡 (True 占比 {{ ((insightStore.insightData.stats.stats_detail as BooleanStatsDetail).true_ratio * 100).toFixed(1) }}%)
              </div>
            </div>
          </NCollapseItem>

          <NCollapseItem name="sample" title="样例数据">
            <div class="sample-list">
              <div v-for="(val, i) in insightStore.insightData.sample" :key="i" class="sample-item">
                <span class="sample-idx">{{ i + 1 }}</span>
                <span class="sample-val mono">{{ formatCellValue(val) }}</span>
              </div>
            </div>
          </NCollapseItem>
        </NCollapse>
      </div>
    </template>
  </div>
</template>

<script setup lang="ts">
import { AlertTriangle, Search, X } from 'lucide-vue-next'
import { NCollapse, NCollapseItem, NButton, NSpin, NTag } from 'naive-ui'
import { onMounted, onUnmounted, computed } from 'vue'

import { useUiStore } from '@/shared/stores/ui'

import { useInsightStore } from '../../stores/insight-store'

import type {
  NumericStatsDetail,
  TextStatsDetail,
  DateTimeStatsDetail,
  BooleanStatsDetail
} from '../../services/result-analysis'

const uiStore = useUiStore()
const insightStore = useInsightStore()

const hasDistribution = computed(() => {
  if (!insightStore.insightData) return false
  const kind = insightStore.statsKind
  if (kind === 'Numeric') {
    return insightStore.insightData.histogram != null && insightStore.insightData.histogram.length > 0
  }
  if (kind === 'Text') {
    return (insightStore.insightData.stats.stats_detail as TextStatsDetail).top_values.length > 0
  }
  if (kind === 'DateTime') {
    return (insightStore.insightData.stats.stats_detail as DateTimeStatsDetail).monthly_distribution.length > 0
  }
  if (kind === 'Boolean') return true
  return false
})

function fmtNum(n: number): string {
  if (!isFinite(n)) return 'N/A'
  if (Number.isInteger(n)) return n.toLocaleString()
  return parseFloat(n.toFixed(4)).toString()
}

function skewDesc(s: number): string {
  if (s > 1) return '右偏'
  if (s < -1) return '左偏'
  return '近似对称'
}

function formatCellValue(val: unknown): string {
  if (val === null || val === undefined) return 'NULL'
  if (typeof val === 'object') {
    try { return JSON.stringify(val) } catch { return String(val) }
  }
  const str = String(val)
  if (str.length > 200) return str.substring(0, 200) + '...'
  return str
}

function filterByValue(value: string): void {
  if (insightStore.currentColumn) {
    window.dispatchEvent(new CustomEvent('insight-filter-by-value', {
      detail: { column: insightStore.currentColumn, value }
    }))
  }
}

function retry(): void {
  if (insightStore.currentTempTable && insightStore.currentColumn) {
    insightStore.loadColumnInsight(insightStore.currentTempTable, insightStore.currentColumn)
  }
}

function handleOpenInsight(event: Event): void {
  const detail = (event as CustomEvent).detail as { column: string; tempTable: string }
  if (detail?.column && detail?.tempTable) {
    insightStore.loadColumnInsight(detail.tempTable, detail.column)
  }
}

onMounted(() => {
  window.addEventListener('open-column-insight', handleOpenInsight)
})

onUnmounted(() => {
  window.removeEventListener('open-column-insight', handleOpenInsight)
})
</script>

<style scoped>
.insight-panel {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: var(--bg-primary);
  overflow: hidden;
}

.insight-empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100%;
  gap: 8px;
  color: var(--text-tertiary);
  padding: 24px 16px;
  text-align: center;
}
.empty-icon { opacity: 0.35; }
.empty-text { font-size: 12px; font-weight: 500; }
.empty-hint { font-size: 10px; opacity: 0.6; }

.insight-loading {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100%;
  gap: 10px;
  color: var(--text-secondary);
}
.loading-text { font-size: 11px; }

.insight-error {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100%;
  gap: 8px;
  color: var(--danger-color);
  padding: 24px 16px;
  text-align: center;
}
.error-icon { opacity: 0.7; }
.error-text { font-size: 11px; word-break: break-all; }

/* Header */
.insight-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 6px 8px;
  flex-shrink: 0;
  border-bottom: 1px solid var(--border-color);
  background: var(--bg-secondary);
}
.insight-title { font-size: 11px; font-weight: 600; color: var(--text-primary); }

/* Body */
.insight-body {
  flex: 1;
  overflow-y: auto;
  overflow-x: hidden;
}

/* Stats grid */
.stat-grid { display: flex; flex-direction: column; gap: 0; }
.stat-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 3px 0;
  min-height: 22px;
  border-bottom: 1px solid var(--border-color);
}
.stat-row:last-child { border-bottom: none; }
.stat-label { font-size: 10px; color: var(--text-tertiary); flex-shrink: 0; }
.stat-value { font-size: 11px; color: var(--text-primary); text-align: right; }
.stat-value.mono { font-family: monospace; font-size: 10px; }
.stat-value.small { font-size: 9px; }
.stat-sub { color: var(--text-tertiary); font-size: 9px; margin-left: 2px; }
.text-warning { color: var(--warning-color, #FF7D00); }

/* Histogram */
.histogram { display: flex; flex-direction: column; gap: 3px; }
.histo-row { display: flex; align-items: center; gap: 4px; height: 20px; }
.histo-label { font-size: 9px; color: var(--text-tertiary); width: 60px; flex-shrink: 0; text-align: right; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
.histo-bar-wrap { flex: 1; min-width: 0; height: 12px; background: var(--bg-tertiary); border-radius: 2px; overflow: hidden; }
.histo-bar { height: 100%; background: var(--primary-color); border-radius: 2px; min-width: 2px; transition: width 0.3s ease; }
.histo-bar-bool { background: #52c41a; }
.histo-ratio { font-size: 9px; color: var(--text-secondary); width: 38px; flex-shrink: 0; text-align: right; font-family: monospace; }

/* Frequency list */
.freq-list { display: flex; flex-direction: column; gap: 3px; }
.freq-item {
  display: flex;
  align-items: center;
  gap: 4px;
  height: 20px;
  background: none;
  border: none;
  padding: 0;
  cursor: default;
  width: 100%;
}
.freq-item.freq-clickable { cursor: pointer; }
.freq-item.freq-clickable:hover { background: var(--bg-hover, rgba(0,0,0,0.03)); border-radius: 2px; }
.freq-label {
  font-size: 10px;
  color: var(--primary-color);
  width: 56px;
  flex-shrink: 0;
  text-align: right;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  font-weight: 500;
}
.freq-bar-wrap { flex: 1; min-width: 0; height: 10px; background: var(--bg-tertiary); border-radius: 2px; overflow: hidden; }
.freq-bar { height: 100%; background: var(--primary-color); border-radius: 2px; min-width: 2px; opacity: 0.7; transition: width 0.3s ease; }
.freq-bar-datetime { background: #722ed1; }
.freq-ratio { font-size: 9px; color: var(--text-secondary); width: 38px; flex-shrink: 0; text-align: right; font-family: monospace; }

/* Boolean */
.bool-dist { display: flex; flex-direction: column; gap: 3px; }

/* Sample list */
.sample-list { display: flex; flex-direction: column; gap: 2px; }
.sample-item { display: flex; align-items: flex-start; gap: 6px; padding: 2px 0; font-size: 10px; }
.sample-idx { color: var(--text-tertiary); min-width: 14px; flex-shrink: 0; }
.sample-val { color: var(--text-primary); font-family: monospace; word-break: break-all; line-height: 1.5; }
.sample-val.mono { font-size: 10px; }

/* Quality */
.quality-list { display: flex; flex-direction: column; gap: 4px; }
.quality-item { font-size: 10px; padding: 2px 0; line-height: 1.5; }
.quality-ok { color: #52c41a; }
.quality-warn { color: var(--warning-color, #FF7D00); }
.quality-info { color: var(--text-secondary); }

/* Dark theme overrides */
.insight-panel.dark .freq-item.freq-clickable:hover { background: rgba(255,255,255,0.04); }
</style>
