<template>
  <div class="insight-panel" :class="{ dark: uiStore.isDark }">
    <NTabs type="segment" size="small" :default-value="activeTab" animated class="insight-tabs">
      <NTabPane name="column" :tab="t('resultPanel.columnInsight')">
        <div
          v-if="!insightStore.hasData && !insightStore.isLoading && !insightStore.error"
          class="insight-empty"
        >
          <Search :size="28" class="empty-icon" />
          <span class="empty-text">{{ t('resultPanel.clickColumnHeader') }}</span>
          <span class="empty-hint">{{ t('resultPanel.orRightClickColumn') }}</span>
        </div>

        <div v-else-if="insightStore.isLoading" class="insight-loading">
          <NSpin size="small" />
          <span class="loading-text"
            >{{ t('resultPanel.analyzing', { column: insightStore.currentColumn }) }}...</span
          >
          <div class="skeleton-block">
            <div v-for="n in 6" :key="n" class="sk-row">
              <div class="sk-bar" :style="{ width: skeletonWidth(n) }"></div>
            </div>
          </div>
        </div>

        <div v-else-if="insightStore.error" class="insight-error">
          <AlertTriangle :size="20" class="error-icon" />
          <span class="error-text">{{ insightStore.error }}</span>
          <NButton size="tiny" quaternary @click="retry">{{ t('navigator.retry') }}</NButton>
        </div>

        <div v-else-if="insightStore.insightData" class="insight-content">
          <div class="insight-header">
            <span class="insight-title"
              >{{ t('resultPanel.columnInsight') }}: {{ insightStore.currentColumn }}</span
            >
            <div class="header-actions">
              <NButton
                size="tiny"
                quaternary
                :title="t('resultPanel.exportJson')"
                @click="exportJSON"
              >
                <Download :size="11" />
              </NButton>
              <NButton
                size="tiny"
                quaternary
                :title="t('resultPanel.exportMarkdown')"
                @click="exportMarkdown"
              >
                <FileText :size="11" />
              </NButton>
              <NButton
                v-if="!insightStore.savedVersionId"
                size="tiny"
                quaternary
                :loading="insightStore.isSaving"
                @click="insightStore.saveCurrentInsight()"
              >
                <Save :size="11" />
              </NButton>
              <NTag v-else :bordered="false" size="tiny" type="success">{{
                t('resultPanel.saved')
              }}</NTag>
            </div>
          </div>

          <div v-if="insightStore.qualityScore" class="quality-score-section">
            <div class="quality-badge" :class="scoreLevelClass">
              <span class="quality-score-num">{{
                Math.round(insightStore.qualityScore.overall_score)
              }}</span>
              <span class="quality-level">{{ insightStore.qualityScore.level }}</span>
            </div>
            <div class="quality-summary">{{ insightStore.qualityScore.summary }}</div>
            <div class="quality-dimensions">
              <div
                v-for="dim in insightStore.qualityScore.dimensions"
                :key="dim.name"
                class="quality-dim"
              >
                <div class="dim-header">
                  <span class="dim-name">{{ dim.name }}</span>
                  <span class="dim-score">{{ Math.round(dim.score) }}</span>
                </div>
                <div class="dim-bar-track">
                  <div
                    class="dim-bar-fill"
                    :style="{ width: dim.score + '%' }"
                    :class="dimScoreBarClass(dim.score)"
                  />
                </div>
                <span class="dim-detail">{{ dim.detail }}</span>
              </div>
            </div>
          </div>

          <div class="insight-body">
            <NCollapse :default-expanded-names="['basic', 'dist', 'quality', 'sample']">
              <NCollapseItem name="basic" :title="t('resultPanel.basicStats')">
                <div class="stat-grid">
                  <div class="stat-row">
                    <span class="stat-label">{{ t('resultPanel.countLabel') }}</span>
                    <span class="stat-value">{{ insightStore.totalCountDisplay }}</span>
                  </div>
                  <div class="stat-row">
                    <span class="stat-label">{{ t('resultPanel.typeLabel') }}</span>
                    <NTag :bordered="false" size="tiny">{{
                      insightStore.insightData.stats.data_type
                    }}</NTag>
                  </div>
                  <div class="stat-row">
                    <span class="stat-label">{{ t('resultPanel.nullRateLabel') }}</span>
                    <span
                      class="stat-value"
                      :class="{ 'text-warning': insightStore.insightData.stats.null_rate > 0.05 }"
                    >
                      {{ insightStore.nullRateDisplay }}
                      <span class="stat-sub"
                        >({{
                          t('resultPanel.nullRateDetail', {
                            count: insightStore.insightData.stats.null_count,
                          })
                        }})</span
                      >
                    </span>
                  </div>
                  <div v-if="insightStore.insightData.stats.unique_count != null" class="stat-row">
                    <span class="stat-label">{{ t('resultPanel.uniqueLabel') }}</span>
                    <span class="stat-value">{{
                      insightStore.insightData.stats.unique_count.toLocaleString()
                    }}</span>
                  </div>

                  <template v-if="insightStore.statsKind === 'Numeric'">
                    <div class="stat-row">
                      <span class="stat-label">{{ t('resultPanel.meanLabel') }}</span>
                      <span class="stat-value mono">{{
                        fmtNum(
                          (insightStore.insightData.stats.stats_detail as NumericStatsDetail).avg
                        )
                      }}</span>
                    </div>
                    <div class="stat-row">
                      <span class="stat-label">{{ t('resultPanel.medianLabel') }}</span>
                      <span class="stat-value mono">{{
                        fmtNum(
                          (insightStore.insightData.stats.stats_detail as NumericStatsDetail).median
                        )
                      }}</span>
                    </div>
                    <div class="stat-row">
                      <span class="stat-label">{{ t('resultPanel.minLabel') }}</span>
                      <span class="stat-value mono">{{
                        fmtNum(
                          (insightStore.insightData.stats.stats_detail as NumericStatsDetail).min
                        )
                      }}</span>
                    </div>
                    <div class="stat-row">
                      <span class="stat-label">{{ t('resultPanel.maxLabel') }}</span>
                      <span class="stat-value mono">{{
                        fmtNum(
                          (insightStore.insightData.stats.stats_detail as NumericStatsDetail).max
                        )
                      }}</span>
                    </div>
                    <div class="stat-row">
                      <span class="stat-label">{{ t('resultPanel.percentileLabel') }}</span>
                      <span class="stat-value mono"
                        >{{
                          fmtNum(
                            (insightStore.insightData.stats.stats_detail as NumericStatsDetail).p25
                          )
                        }}
                        /
                        {{
                          fmtNum(
                            (insightStore.insightData.stats.stats_detail as NumericStatsDetail).p75
                          )
                        }}</span
                      >
                    </div>
                    <div
                      v-if="
                        (insightStore.insightData.stats.stats_detail as NumericStatsDetail)
                          .stddev != null
                      "
                      class="stat-row"
                    >
                      <span class="stat-label">{{ t('resultPanel.stddevLabel') }}</span>
                      <span class="stat-value mono">{{
                        fmtNum(
                          (insightStore.insightData.stats.stats_detail as NumericStatsDetail)
                            .stddev ?? 0
                        )
                      }}</span>
                    </div>
                    <div
                      v-if="
                        (insightStore.insightData.stats.stats_detail as NumericStatsDetail)
                          .skewness != null
                      "
                      class="stat-row"
                    >
                      <span class="stat-label">{{ t('resultPanel.skewnessLabel') }}</span>
                      <span class="stat-value mono"
                        >{{
                          fmtNum(
                            (insightStore.insightData.stats.stats_detail as NumericStatsDetail)
                              .skewness ?? 0
                          )
                        }}
                        <span class="stat-sub">{{
                          skewDesc(
                            (insightStore.insightData.stats.stats_detail as NumericStatsDetail)
                              .skewness ?? 0
                          )
                        }}</span>
                      </span>
                    </div>
                  </template>

                  <template v-if="insightStore.statsKind === 'Text'">
                    <div class="stat-row">
                      <span class="stat-label">{{ t('resultPanel.lengthRangeLabel') }}</span>
                      <span class="stat-value"
                        >{{
                          (insightStore.insightData.stats.stats_detail as TextStatsDetail)
                            .min_length
                        }}
                        ~
                        {{
                          (insightStore.insightData.stats.stats_detail as TextStatsDetail)
                            .max_length
                        }}</span
                      >
                    </div>
                  </template>

                  <template v-if="insightStore.statsKind === 'DateTime'">
                    <div class="stat-row">
                      <span class="stat-label">{{ t('resultPanel.earliestLabel') }}</span>
                      <span class="stat-value mono small">{{
                        (insightStore.insightData.stats.stats_detail as DateTimeStatsDetail)
                          .earliest
                      }}</span>
                    </div>
                    <div class="stat-row">
                      <span class="stat-label">{{ t('resultPanel.latestLabel') }}</span>
                      <span class="stat-value mono small">{{
                        (insightStore.insightData.stats.stats_detail as DateTimeStatsDetail).latest
                      }}</span>
                    </div>
                    <div class="stat-row">
                      <span class="stat-label">{{ t('resultPanel.spanLabel') }}</span>
                      <span class="stat-value">{{
                        t('resultPanel.spanDays', {
                          days: (insightStore.insightData.stats.stats_detail as DateTimeStatsDetail)
                            .span_days,
                        })
                      }}</span>
                    </div>
                  </template>

                  <template v-if="insightStore.statsKind === 'Boolean'">
                    <div class="stat-row">
                      <span class="stat-label">{{ t('resultPanel.trueLabel') }}</span>
                      <span class="stat-value"
                        >{{
                          (
                            insightStore.insightData.stats.stats_detail as BooleanStatsDetail
                          ).true_count.toLocaleString()
                        }}
                        <span class="stat-sub"
                          >({{
                            t('resultPanel.trueRatio', {
                              ratio: (
                                (insightStore.insightData.stats.stats_detail as BooleanStatsDetail)
                                  .true_ratio * 100
                              ).toFixed(1),
                            })
                          }})</span
                        >
                      </span>
                    </div>
                    <div class="stat-row">
                      <span class="stat-label">{{ t('resultPanel.falseLabel') }}</span>
                      <span class="stat-value">{{
                        (
                          insightStore.insightData.stats.stats_detail as BooleanStatsDetail
                        ).false_count.toLocaleString()
                      }}</span>
                    </div>
                  </template>
                </div>
              </NCollapseItem>

              <NCollapseItem
                v-if="hasDistribution"
                name="dist"
                :title="t('resultPanel.dataDistribution')"
              >
                <template v-if="insightStore.statsKind === 'Numeric'">
                  <div class="histogram">
                    <div
                      v-for="bin in insightStore.insightData.histogram"
                      :key="bin.label"
                      class="histo-row"
                    >
                      <span class="histo-label">{{ bin.label }}</span>
                      <div class="histo-bar-wrap">
                        <div
                          class="histo-bar"
                          :style="{ width: (bin.ratio * 100).toFixed(1) + '%' }"
                        ></div>
                      </div>
                      <span class="histo-ratio">{{ (bin.ratio * 100).toFixed(1) }}%</span>
                    </div>
                  </div>
                </template>

                <template v-if="insightStore.statsKind === 'Text'">
                  <div class="freq-list">
                    <button
                      v-for="(item, i) in (
                        insightStore.insightData.stats.stats_detail as TextStatsDetail
                      ).top_values"
                      :key="i"
                      class="freq-item freq-clickable"
                      @click="filterByValue(item.value)"
                    >
                      <span class="freq-label" :title="item.value">[{{ item.value }}]</span>
                      <div class="freq-bar-wrap">
                        <div
                          class="freq-bar"
                          :style="{ width: (item.ratio * 100).toFixed(1) + '%' }"
                        ></div>
                      </div>
                      <span class="freq-ratio">{{ (item.ratio * 100).toFixed(1) }}%</span>
                    </button>
                  </div>
                </template>

                <template v-if="insightStore.statsKind === 'DateTime'">
                  <div class="freq-list">
                    <button
                      v-for="(item, i) in (
                        insightStore.insightData.stats.stats_detail as DateTimeStatsDetail
                      ).monthly_distribution"
                      :key="i"
                      class="freq-item freq-clickable"
                      @click="filterByValue(item.value)"
                    >
                      <span class="freq-label">{{ item.value }}</span>
                      <div class="freq-bar-wrap">
                        <div
                          class="freq-bar freq-bar-datetime"
                          :style="{ width: (item.ratio * 100).toFixed(1) + '%' }"
                        ></div>
                      </div>
                      <span class="freq-ratio">{{ (item.ratio * 100).toFixed(1) }}%</span>
                    </button>
                  </div>
                </template>

                <template v-if="insightStore.statsKind === 'Boolean'">
                  <div class="bool-dist">
                    <div class="histo-row">
                      <span class="histo-label">True</span>
                      <div class="histo-bar-wrap">
                        <div
                          class="histo-bar histo-bar-bool"
                          :style="{
                            width:
                              (
                                (insightStore.insightData.stats.stats_detail as BooleanStatsDetail)
                                  .true_ratio * 100
                              ).toFixed(1) + '%',
                          }"
                        ></div>
                      </div>
                      <span class="histo-ratio"
                        >{{
                          (
                            (insightStore.insightData.stats.stats_detail as BooleanStatsDetail)
                              .true_ratio * 100
                          ).toFixed(1)
                        }}%</span
                      >
                    </div>
                  </div>
                </template>
                <div class="viz-action">
                  <NButton size="tiny" quaternary @click="openVisualization">
                    <template #icon>
                      <BarChart3 :size="14" />
                    </template>
                    {{ t('resultPanel.openChart') }}
                  </NButton>
                </div>
              </NCollapseItem>

              <NCollapseItem name="quality" :title="t('resultPanel.dataQuality')">
                <div class="quality-list">
                  <div
                    v-if="insightStore.insightData.stats.null_rate === 0"
                    class="quality-item quality-ok"
                  >
                    ✅ {{ t('resultPanel.nullRateOk') }}
                  </div>
                  <div
                    v-else-if="insightStore.insightData.stats.null_rate > 0.1"
                    class="quality-item quality-warn"
                  >
                    ⚠️ {{ t('resultPanel.highNullRate', { rate: insightStore.nullRateDisplay }) }}
                  </div>

                  <div v-if="insightStore.statsKind === 'Numeric'">
                    <div
                      v-for="(ext, i) in (
                        insightStore.insightData.stats.stats_detail as NumericStatsDetail
                      ).is_extreme"
                      :key="i"
                      class="quality-item quality-warn"
                    >
                      ⚠️ {{ t('resultPanel.extremeValue', { value: ext.value }) }}
                    </div>
                    <div
                      v-if="
                        (insightStore.insightData.stats.stats_detail as NumericStatsDetail)
                          .skewness != null
                      "
                    >
                      <div
                        v-if="
                          Math.abs(
                            (insightStore.insightData.stats.stats_detail as NumericStatsDetail)
                              .skewness ?? 0
                          ) > 1
                        "
                        class="quality-item quality-info"
                      >
                        ℹ️
                        {{
                          skewDesc(
                            (insightStore.insightData.stats.stats_detail as NumericStatsDetail)
                              .skewness ?? 0
                          )
                        }}
                      </div>
                    </div>
                  </div>

                  <div
                    v-if="
                      insightStore.statsKind === 'Text' &&
                      (insightStore.insightData.stats.stats_detail as TextStatsDetail).top_values
                        .length >= 5
                    "
                    class="quality-item quality-info"
                  >
                    ℹ️
                    {{
                      t('resultPanel.categoryCount', {
                        count: (insightStore.insightData.stats.stats_detail as TextStatsDetail)
                          .top_values.length,
                      })
                    }}
                  </div>

                  <div
                    v-if="
                      insightStore.statsKind === 'Boolean' &&
                      (insightStore.insightData.stats.stats_detail as BooleanStatsDetail)
                        .true_ratio > 0.95
                    "
                    class="quality-item quality-info"
                  >
                    ℹ️
                    {{
                      t('resultPanel.highlyImbalanced', {
                        ratio: (
                          (insightStore.insightData.stats.stats_detail as BooleanStatsDetail)
                            .true_ratio * 100
                        ).toFixed(1),
                      })
                    }}
                  </div>
                </div>
              </NCollapseItem>

              <NCollapseItem name="sample" :title="t('resultPanel.sampleData')">
                <div class="sample-list">
                  <div
                    v-for="(val, i) in insightStore.insightData.sample"
                    :key="i"
                    class="sample-item"
                  >
                    <span class="sample-idx">{{ i + 1 }}</span>
                    <span class="sample-val mono">{{ formatCellValue(val) }}</span>
                  </div>
                </div>
              </NCollapseItem>
            </NCollapse>
          </div>

          <div v-if="insightStore.storageStats" class="insight-storage-footer">
            <span class="storage-text">{{
              t('resultPanel.storageStats', {
                snapshots: insightStore.storageStats.total_snapshots,
                columns: insightStore.storageStats.unique_columns,
                size: insightStore.storageStats.total_size_display,
              })
            }}</span>
            <NButton
              size="tiny"
              quaternary
              :loading="insightStore.isCleaning"
              class="cleanup-btn"
              @click="handleCleanup"
            >
              {{ t('resultPanel.cleanupOldData') }}
            </NButton>
          </div>

          <div v-if="insightStore.columnRules.length > 0" class="insight-rules-footer">
            <span class="rules-label">{{ t('resultPanel.applicableRules') }}:</span>
            <div class="rules-tags">
              <NTag
                v-for="rule in insightStore.columnRules"
                :key="rule.id"
                size="tiny"
                :bordered="false"
                type="info"
                class="rule-tag"
              >
                {{ rule.name }}
              </NTag>
            </div>
          </div>
        </div>
      </NTabPane>
      <NTabPane name="multi" :tab="t('resultPanel.multiColumnAnalysis')">
        <MultiColumnView
          v-if="availableColumns.length > 0"
          :temp-table="insightStore.currentTempTable ?? ''"
          :all-columns="availableColumns"
        />
        <div v-else class="insight-empty">
          <BarChart3 :size="28" class="empty-icon" />
          <span class="empty-text">{{ t('resultPanel.multiColumnHint') }}</span>
        </div>
      </NTabPane>

      <NTabPane name="history" :tab="t('resultPanel.history')">
        <div v-if="insightStore.history.length === 0" class="insight-empty">
          <Clock :size="28" class="empty-icon" />
          <span class="empty-text">{{ t('resultPanel.saveHistoryHint') }}</span>
        </div>
        <div v-else class="history-content">
          <div class="history-header">
            <span class="history-title">{{
              t('resultPanel.versionsCount', { count: insightStore.history.length })
            }}</span>
            <NButton
              v-if="insightStore.diffVersionId"
              size="tiny"
              quaternary
              @click="insightStore.clearDiff()"
            >
              {{ t('resultPanel.cancelDiff') }}
            </NButton>
          </div>
          <div class="history-list">
            <div
              v-for="entry in insightStore.history"
              :key="entry.version_id"
              class="history-item"
              :class="{ active: insightStore.diffVersionId === entry.version_id }"
              @click="insightStore.loadVersionDetail(entry.version_id)"
            >
              <div class="history-item-main">
                <span class="history-date">{{ formatDate(entry.created_at) }}</span>
                <NTag
                  :bordered="false"
                  size="tiny"
                  :type="insightStore.diffVersionId === entry.version_id ? 'primary' : 'default'"
                >
                  {{ entry.checksum.substring(0, 8) }}
                </NTag>
              </div>
            </div>
          </div>

          <div v-if="insightStore.diffData && insightStore.insightData" class="diff-panel">
            <div class="diff-header">{{ t('resultPanel.versionCompare') }}</div>
            <div class="diff-grid">
              <div class="diff-row">
                <span class="diff-label">{{ t('resultPanel.nullRateCompare') }}</span>
                <span class="diff-old"
                  >{{ (insightStore.diffData.stats.null_rate * 100).toFixed(1) }}%</span
                >
                <span class="diff-arrow">→</span>
                <span
                  class="diff-new"
                  :class="
                    diffClass(
                      insightStore.diffData.stats.null_rate,
                      insightStore.insightData.stats.null_rate
                    )
                  "
                >
                  {{ (insightStore.insightData.stats.null_rate * 100).toFixed(1) }}%
                </span>
              </div>
              <div class="diff-row">
                <span class="diff-label">{{ t('resultPanel.totalCountCompare') }}</span>
                <span class="diff-old">{{
                  insightStore.diffData.stats.total_count.toLocaleString()
                }}</span>
                <span class="diff-arrow">→</span>
                <span
                  class="diff-new"
                  :class="
                    diffClass(
                      insightStore.diffData.stats.total_count,
                      insightStore.insightData.stats.total_count
                    )
                  "
                >
                  {{ insightStore.insightData.stats.total_count.toLocaleString() }}
                </span>
              </div>
              <div
                v-if="
                  insightStore.diffData.stats.unique_count != null &&
                  insightStore.insightData.stats.unique_count != null
                "
                class="diff-row"
              >
                <span class="diff-label">{{ t('resultPanel.uniqueCountCompare') }}</span>
                <span class="diff-old">{{
                  insightStore.diffData.stats.unique_count.toLocaleString()
                }}</span>
                <span class="diff-arrow">→</span>
                <span
                  class="diff-new"
                  :class="
                    diffClass(
                      insightStore.diffData.stats.unique_count,
                      insightStore.insightData.stats.unique_count
                    )
                  "
                >
                  {{ insightStore.insightData.stats.unique_count.toLocaleString() }}
                </span>
              </div>
              <div class="diff-row">
                <span class="diff-label">{{ t('resultPanel.nullCountCompare') }}</span>
                <span class="diff-old">{{ insightStore.diffData.stats.null_count }}</span>
                <span class="diff-arrow">→</span>
                <span
                  class="diff-new"
                  :class="
                    diffClass(
                      insightStore.diffData.stats.null_count,
                      insightStore.insightData.stats.null_count
                    )
                  "
                >
                  {{ insightStore.insightData.stats.null_count }}
                </span>
              </div>
            </div>
          </div>

          <div v-if="insightStore.isDiffLoading" class="diff-loading">
            <NSpin size="small" />
            <span>{{ t('resultPanel.loadVersionData') }}</span>
          </div>
        </div>
      </NTabPane>
    </NTabs>
  </div>
</template>

<script setup lang="ts">
import { AlertTriangle, Clock, Download, Save, Search, BarChart3, FileText } from 'lucide-vue-next'
import { NCollapse, NCollapseItem, NButton, NSpin, NTag, NTabs, NTabPane } from 'naive-ui'
import { onMounted, computed, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'

import { useUiStore } from '@/shared/stores/ui'

import MultiColumnView from './MultiColumnView.vue'
import { useInsightStore } from '../../stores/insight-store'
import { useResultStore } from '../../stores/result-store'

import type {
  NumericStatsDetail,
  TextStatsDetail,
  DateTimeStatsDetail,
  BooleanStatsDetail,
} from '../../services/result-analysis'

const { t } = useI18n()
const uiStore = useUiStore()
const insightStore = useInsightStore()
const resultStore = useResultStore()

const activeTab = ref('column')
const availableColumns = ref<string[]>([])

const hasDistribution = computed(() => {
  if (!insightStore.insightData) return false
  const kind = insightStore.statsKind
  if (kind === 'Numeric') {
    return (
      insightStore.insightData.histogram != null && insightStore.insightData.histogram.length > 0
    )
  }
  if (kind === 'Text') {
    return (insightStore.insightData.stats.stats_detail as TextStatsDetail).top_values.length > 0
  }
  if (kind === 'DateTime') {
    return (
      (insightStore.insightData.stats.stats_detail as DateTimeStatsDetail).monthly_distribution
        .length > 0
    )
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
  if (s > 1) return t('resultPanel.rightSkew')
  if (s < -1) return t('resultPanel.leftSkew')
  return t('resultPanel.approxSymmetric')
}

function formatCellValue(val: unknown): string {
  if (val === null || val === undefined) return 'NULL'
  if (typeof val === 'object') {
    try {
      return JSON.stringify(val)
    } catch {
      return String(val)
    }
  }
  const str = String(val)
  if (str.length > 200) return str.substring(0, 200) + '...'
  return str
}

function openVisualization(): void {
  const d = insightStore.insightData
  if (!d) return

  const kind = insightStore.statsKind
  let data: Record<string, unknown>[] = []
  let columns: string[] = []
  let title = ''

  const extractors: Record<
    string,
    () => { data: Record<string, unknown>[]; columns: string[]; title: string }
  > = {
    Numeric: () => {
      const chart = insightStore.histogramForChart(d.histogram)
      return {
        data: chart.labels.map((label, i) => ({ label, count: chart.data[i] })),
        columns: ['label', 'count'],
        title: t('resultPanel.distribution', { column: d.stats.column_name }),
      }
    },
    Text: () => {
      const tv = (d.stats.stats_detail as TextStatsDetail).top_values
      return {
        data: tv.map(item => ({ value: item.value, count: item.count })),
        columns: ['value', 'count'],
        title: t('resultPanel.topValues', { column: d.stats.column_name }),
      }
    },
    DateTime: () => {
      const md = (d.stats.stats_detail as DateTimeStatsDetail).monthly_distribution
      return {
        data: (md as unknown as Array<Record<string, unknown>>).map(item => ({ month: item.month as string, count: item.count as number })),
        columns: ['month', 'count'],
        title: t('resultPanel.monthlyDistribution', { column: d.stats.column_name }),
      }
    },
    Boolean: () => {
      const bd = d.stats.stats_detail as BooleanStatsDetail
      return {
        data: [
          { value: 'True', count: bd.true_count },
          { value: 'False', count: d.stats.total_count - bd.true_count },
        ],
        columns: ['value', 'count'],
        title: t('resultPanel.booleanDistribution', { column: d.stats.column_name }),
      }
    },
  }

  const extractor = extractors[kind]
  if (extractor) {
    const result = extractor()
    data = result.data
    columns = result.columns
    title = result.title
  }

  if (data.length > 0) {
    insightStore.requestVisualization({ data, columns, title })
  }
}

function filterByValue(value: string): void {
  if (insightStore.currentColumn) {
    const activeTab = resultStore.activeTab
    if (activeTab) {
      const col = insightStore.currentColumn
      const expr = `${col} = ${typeof value === 'string' ? `'${value.replace(/'/g, "''")}'` : value}`
      resultStore.applyQuickFilter(activeTab.id, expr)
      window.dispatchEvent(
        new CustomEvent('show-notification', {
          detail: { message: t('resultPanel.filterApplied', { expr }), type: 'info' },
        })
      )
    }
  }
}

function retry(): void {
  if (insightStore.currentTempTable && insightStore.currentColumn) {
    insightStore.loadColumnInsight(insightStore.currentTempTable, insightStore.currentColumn)
  }
}

function skeletonWidth(n: number): string {
  const widths = ['70%', '55%', '60%', '45%', '65%', '50%']
  return widths[(n - 1) % widths.length]
}

function handleCleanup(): void {
  insightStore.cleanupOldSnapshots(90)
}

function exportJSON(): void {
  if (!insightStore.insightData) return
  const blob = new Blob([JSON.stringify(insightStore.insightData, null, 2)], {
    type: 'application/json',
  })
  const url = URL.createObjectURL(blob)
  const a = document.createElement('a')
  const col = insightStore.currentColumn ?? 'column'
  a.href = url
  a.download = `insight-${col}-${new Date().toISOString().slice(0, 10)}.json`
  a.click()
  URL.revokeObjectURL(url)
}

function exportMarkdown(): void {
  if (!insightStore.insightData) return
  const d = insightStore.insightData
  const lines: string[] = [
    `# ${t('resultPanel.columnInsight')}: ${insightStore.currentColumn ?? '-'}`,
    '',
    `| ${t('resultPanel.countLabel')} | ${t('resultPanel.typeLabel')} |`,
    '|------|----|',
    `| ${t('resultPanel.countLabel')} | ${d.stats.total_count.toLocaleString()} |`,
    `| ${t('resultPanel.typeLabel')} | ${d.stats.data_type} |`,
    `| ${t('resultPanel.nullRateLabel')} | ${(d.stats.null_rate * 100).toFixed(1)}% (${t('resultPanel.nullRateDetail', { count: d.stats.null_count })}) |`,
  ]
  if (d.stats.unique_count != null) {
    lines.push(`| ${t('resultPanel.uniqueLabel')} | ${d.stats.unique_count.toLocaleString()} |`)
  }
  const blob = new Blob([lines.join('\n')], { type: 'text/markdown' })
  const url = URL.createObjectURL(blob)
  const a = document.createElement('a')
  const col = insightStore.currentColumn ?? 'column'
  a.href = url
  a.download = `insight-${col}-${new Date().toISOString().slice(0, 10)}.md`
  a.click()
  URL.revokeObjectURL(url)
}

function formatDate(isoString: string | undefined): string {
  if (!isoString) return '-'
  try {
    const d = new Date(isoString)
    return d.toLocaleDateString(undefined, {
      month: 'short',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit',
    })
  } catch {
    return isoString.slice(0, 19)
  }
}

function diffClass(oldVal: number, newVal: number): string {
  if (newVal > oldVal) return 'diff-increase'
  if (newVal < oldVal) return 'diff-decrease'
  return 'diff-same'
}

const scoreLevelClass = computed(() => {
  if (!insightStore.qualityScore) return ''
  const s = insightStore.qualityScore.overall_score
  if (s >= 85) return 'score-excellent'
  if (s >= 70) return 'score-good'
  if (s >= 50) return 'score-fair'
  if (s >= 30) return 'score-poor'
  return 'score-bad'
})

function dimScoreBarClass(score: number): string {
  if (score >= 80) return 'bar-good'
  if (score >= 50) return 'bar-fair'
  return 'bar-poor'
}

watch(
  () => insightStore.insightData,
  data => {
    if (data) {
      insightStore.loadQualityScore()
    } else {
      insightStore.qualityScore = null
    }
  }
)

watch(
  () => [insightStore.insightData, insightStore.autoOpenVisualization] as const,
  ([data, flag]) => {
    if (data && flag) {
      openVisualization()
      insightStore.autoOpenVisualization = false
    }
  }
)

onMounted(() => {
  insightStore.loadStorageStats()
  insightStore.loadMultiRules()
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
.empty-icon {
  opacity: 0.35;
}
.empty-text {
  font-size: 12px;
  font-weight: 500;
}
.empty-hint {
  font-size: 10px;
  opacity: 0.6;
}

.insight-loading {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100%;
  gap: 10px;
  color: var(--text-secondary);
}
.loading-text {
  font-size: 11px;
}

.skeleton-block {
  margin-top: 12px;
  display: flex;
  flex-direction: column;
  gap: 8px;
  padding: 0 12px;
}
.sk-row {
  height: 14px;
  border-radius: 3px;
  background: var(--bg-tertiary);
  position: relative;
  overflow: hidden;
}
.sk-bar {
  height: 100%;
  border-radius: 3px;
  background: var(--border-color);
  animation: sk-pulse 1.5s ease-in-out infinite;
}
@keyframes sk-pulse {
  0%,
  100% {
    opacity: 0.4;
  }
  50% {
    opacity: 0.8;
  }
}

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
.error-icon {
  opacity: 0.7;
}
.error-text {
  font-size: 11px;
  word-break: break-all;
}

.insight-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 6px 8px;
  flex-shrink: 0;
  border-bottom: 1px solid var(--border-color);
  background: var(--bg-secondary);
}
.insight-title {
  font-size: 11px;
  font-weight: 600;
  color: var(--text-primary);
}
.header-actions {
  display: flex;
  align-items: center;
  gap: 2px;
}

.insight-body {
  flex: 1;
  overflow-y: auto;
  overflow-x: hidden;
}

.stat-grid {
  display: flex;
  flex-direction: column;
  gap: 0;
}
.stat-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 3px 0;
  min-height: 22px;
  border-bottom: 1px solid var(--border-color);
}
.stat-row:last-child {
  border-bottom: none;
}
.stat-label {
  font-size: 10px;
  color: var(--text-tertiary);
  flex-shrink: 0;
}
.stat-value {
  font-size: 11px;
  color: var(--text-primary);
  text-align: right;
}
.stat-value.mono {
  font-family: monospace;
  font-size: 10px;
}
.stat-value.small {
  font-size: 9px;
}
.stat-sub {
  color: var(--text-tertiary);
  font-size: 9px;
  margin-left: 2px;
}
.text-warning {
  color: var(--warning-color, #ff7d00);
}

.histogram {
  display: flex;
  flex-direction: column;
  gap: 3px;
}
.histo-row {
  display: flex;
  align-items: center;
  gap: 4px;
  height: 20px;
}
.histo-label {
  font-size: 9px;
  color: var(--text-tertiary);
  width: 60px;
  flex-shrink: 0;
  text-align: right;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.histo-bar-wrap {
  flex: 1;
  min-width: 0;
  height: 12px;
  background: var(--bg-tertiary);
  border-radius: 2px;
  overflow: hidden;
}
.histo-bar {
  height: 100%;
  background: var(--primary-color);
  border-radius: 2px;
  min-width: 2px;
  transition: width 0.3s ease;
}
.histo-bar-bool {
  background: #52c41a;
}
.histo-ratio {
  font-size: 9px;
  color: var(--text-secondary);
  width: 38px;
  text-align: right;
  flex-shrink: 0;
}

.viz-action {
  margin-top: 8px;
  display: flex;
  justify-content: flex-end;
}

.freq-list {
  display: flex;
  flex-direction: column;
  gap: 3px;
}
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
.freq-item.freq-clickable {
  cursor: pointer;
}
.freq-item.freq-clickable:hover {
  background: var(--bg-hover, rgba(0, 0, 0, 0.03));
  border-radius: 2px;
}
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
.freq-bar-wrap {
  flex: 1;
  min-width: 0;
  height: 10px;
  background: var(--bg-tertiary);
  border-radius: 2px;
  overflow: hidden;
}
.freq-bar {
  height: 100%;
  background: var(--primary-color);
  border-radius: 2px;
  min-width: 2px;
  opacity: 0.7;
  transition: width 0.3s ease;
}
.freq-bar-datetime {
  background: #722ed1;
}
.freq-ratio {
  font-size: 9px;
  color: var(--text-secondary);
  width: 38px;
  flex-shrink: 0;
  text-align: right;
  font-family: monospace;
}

.bool-dist {
  display: flex;
  flex-direction: column;
  gap: 3px;
}

.sample-list {
  display: flex;
  flex-direction: column;
  gap: 2px;
}
.sample-item {
  display: flex;
  align-items: flex-start;
  gap: 6px;
  padding: 2px 0;
  font-size: 10px;
}
.sample-idx {
  color: var(--text-tertiary);
  min-width: 14px;
  flex-shrink: 0;
}
.sample-val {
  color: var(--text-primary);
  font-family: monospace;
  word-break: break-all;
  line-height: 1.5;
  font-size: 10px;
}

.quality-list {
  display: flex;
  flex-direction: column;
  gap: 4px;
}
.quality-item {
  font-size: 10px;
  padding: 2px 0;
  line-height: 1.5;
}
.quality-ok {
  color: #52c41a;
}
.quality-warn {
  color: var(--warning-color, #ff7d00);
}
.quality-info {
  color: var(--text-secondary);
}

.quality-score-section {
  padding: 6px 8px;
  border-bottom: 1px solid var(--border-color);
  background: var(--bg-tertiary);
}
.quality-badge {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  padding: 2px 8px;
  border-radius: 10px;
  margin-bottom: 4px;
}
.quality-badge.score-excellent {
  background: #e6f7e6;
  color: #1a7a1a;
}
.quality-badge.score-good {
  background: #e6f7ff;
  color: #1a6db5;
}
.quality-badge.score-fair {
  background: #fff7e6;
  color: #b57a1a;
}
.quality-badge.score-poor {
  background: #fff2e6;
  color: #b54a1a;
}
.quality-badge.score-bad {
  background: #ffe6e6;
  color: #b51a1a;
}
.quality-score-num {
  font-size: 18px;
  font-weight: 700;
  line-height: 1;
}
.quality-level {
  font-size: 11px;
  font-weight: 600;
}
.quality-summary {
  font-size: 10px;
  color: var(--text-secondary);
  margin-bottom: 6px;
  line-height: 1.4;
}
.quality-dimensions {
  display: flex;
  flex-direction: column;
  gap: 4px;
}
.quality-dim {
  display: flex;
  flex-direction: column;
  gap: 1px;
}
.dim-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}
.dim-name {
  font-size: 9px;
  color: var(--text-secondary);
}
.dim-score {
  font-size: 9px;
  color: var(--text-primary);
  font-weight: 600;
}
.dim-bar-track {
  height: 3px;
  background: var(--border-color);
  border-radius: 2px;
  overflow: hidden;
}
.dim-bar-fill {
  height: 100%;
  border-radius: 2px;
  transition: width 0.3s ease;
}
.dim-bar-fill.bar-good {
  background: #52c41a;
}
.dim-bar-fill.bar-fair {
  background: var(--warning-color, #ff7d00);
}
.dim-bar-fill.bar-poor {
  background: #f5222d;
}
.dim-detail {
  font-size: 8px;
  color: var(--text-tertiary);
}

.insight-tabs :deep(.n-tab-pane) {
  animation: tab-fade-in 0.18s ease;
}
@keyframes tab-fade-in {
  from {
    opacity: 0.6;
    transform: translateY(3px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

.insight-storage-footer {
  flex-shrink: 0;
  padding: 4px 8px;
  border-top: 1px solid var(--border-color);
  background: var(--bg-secondary);
  display: flex;
  align-items: center;
  justify-content: space-between;
}
.storage-text {
  font-size: 9px;
  color: var(--text-tertiary);
}
.cleanup-btn {
  font-size: 9px;
}

.insight-rules-footer {
  flex-shrink: 0;
  padding: 4px 8px;
  border-top: 1px solid var(--border-color);
  background: var(--bg-secondary);
  display: flex;
  align-items: center;
  gap: 4px;
}
.rules-label {
  font-size: 9px;
  color: var(--text-tertiary);
  flex-shrink: 0;
}
.rules-tags {
  display: flex;
  flex-wrap: wrap;
  gap: 3px;
}
.rule-tag {
  cursor: default;
  font-size: 9px;
}

.insight-panel.dark .freq-item.freq-clickable:hover {
  background: rgba(255, 255, 255, 0.04);
}

/* 历史 / 版本对比 */
.history-content {
  display: flex;
  flex-direction: column;
  height: 100%;
  overflow: hidden;
}
.history-header {
  flex-shrink: 0;
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 8px 4px 8px;
}
.history-title {
  font-size: 11px;
  color: var(--text-secondary);
}
.history-list {
  flex-shrink: 0;
  max-height: 50%;
  overflow-y: auto;
  padding: 0 4px;
}
.history-item {
  padding: 5px 6px;
  border-radius: 3px;
  cursor: pointer;
  margin: 2px 0;
  transition: background 0.15s;
}
.history-item:hover {
  background: var(--bg-secondary);
}
.history-item.active {
  background: var(--bg-tertiary);
}
.history-item-main {
  display: flex;
  align-items: center;
  justify-content: space-between;
}
.history-date {
  font-size: 10px;
  color: var(--text-secondary);
}

.diff-panel {
  flex-shrink: 0;
  margin: 8px 4px;
  border: 1px solid var(--border-color);
  border-radius: 4px;
  overflow: hidden;
}
.diff-header {
  padding: 4px 8px;
  background: var(--bg-secondary);
  font-size: 11px;
  font-weight: 600;
  color: var(--text-primary);
  border-bottom: 1px solid var(--border-color);
}
.diff-grid {
  padding: 6px 8px;
}
.diff-row {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 3px 0;
  font-size: 11px;
}
.diff-label {
  width: 48px;
  flex-shrink: 0;
  color: var(--text-tertiary);
  font-size: 10px;
}
.diff-old {
  color: var(--text-secondary);
  font-family: monospace;
  font-size: 10px;
}
.diff-arrow {
  color: var(--text-tertiary);
  font-size: 10px;
}
.diff-new {
  font-family: monospace;
  font-size: 10px;
  font-weight: 600;
}
.diff-increase {
  color: var(--success-color);
}
.diff-decrease {
  color: var(--danger-color);
}
.diff-same {
  color: var(--text-primary);
}

.diff-loading {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
  padding: 12px;
  font-size: 10px;
  color: var(--text-tertiary);
}
</style>
