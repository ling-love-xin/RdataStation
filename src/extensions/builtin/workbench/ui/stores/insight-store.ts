/**
 * 洞察状态管理
 *
 * 管理列洞察面板的数据加载、缓存、保存、历史查询和存储统计
 */
import { defineStore } from 'pinia'
import { ref, computed } from 'vue'

import {
  getColumnInsightFull,
  saveColumnInsightSnapshot,
  getColumnInsightHistory,
  getInsightStorageStats,
  executeInsightRule,
  listInsightRules,
  cleanupInsightSnapshots,
  getInsightVersionDetail,
  profileColumnFromTable,
  getColumnQuality,
} from '../services/result-analysis'

import type {
  ColumnInsightFull,
  InsightVersionEntry,
  InsightStorageStats,
  RuleMeta,
  CleanupResult,
  QualityScore,
} from '../services/result-analysis'

export const useInsightStore = defineStore('insight', () => {
  const currentColumn = ref<string | null>(null)
  const currentTempTable = ref<string | null>(null)
  const insightData = ref<ColumnInsightFull | null>(null)
  const isLoading = ref(false)
  const isSaving = ref(false)
  const error = ref<string | null>(null)

  const history = ref<InsightVersionEntry[]>([])
  const storageStats = ref<InsightStorageStats | null>(null)
  const savedVersionId = ref<string | null>(null)

  // === 多列分析 ===
  const availableColumns = ref<string[]>([])
  const multiRules = ref<RuleMeta[]>([])
  const multiResult = ref<unknown>(null)
  const isMultiExecuting = ref(false)

  // === 清理 ===
  const isCleaning = ref(false)
  const cleanupResult = ref<CleanupResult | null>(null)

  // === 版本对比 ===
  const diffVersionId = ref<string | null>(null)
  const diffData = ref<ColumnInsightFull | null>(null)
  const isDiffLoading = ref(false)

  // === 质量评分 ===
  const qualityScore = ref<QualityScore | null>(null)
  const isQualityLoading = ref(false)

  const statsKind = computed<string>(() => {
    return insightData.value?.stats?.stats_detail?.kind ?? 'Unknown'
  })

  const hasData = computed(() => insightData.value !== null)

  const nullRatePercent = computed(() => {
    if (!insightData.value) return '0.0'
    return (insightData.value.stats.null_rate * 100).toFixed(1)
  })

  const nullRateDisplay = computed(() => {
    if (!insightData.value) return '0.0%'
    return `${nullRatePercent.value}%`
  })

  const totalCountDisplay = computed(() => {
    if (!insightData.value) return '0'
    return insightData.value.stats.total_count.toLocaleString()
  })

  const historyCount = computed(() => history.value.length)

  const columnRules = computed<RuleMeta[]>(() => {
    if (!statsKind.value || statsKind.value === 'Unknown') return []
    return multiRules.value.filter((r) =>
      r.category === 'column' || r.applies_to.includes(statsKind.value) || r.applies_to.includes('Any')
    )
  })

  const multiColumnRules = computed<RuleMeta[]>(() =>
    multiRules.value.filter((r) => r.category === 'multi')
  )

  async function loadColumnInsight(tempTable: string, column: string): Promise<void> {
    if (isLoading.value) return

    isLoading.value = true
    error.value = null
    savedVersionId.value = null

    try {
      const result = await getColumnInsightFull(tempTable, column)
      insightData.value = result
      currentColumn.value = column
      currentTempTable.value = tempTable
    } catch (e: unknown) {
      const msg = e instanceof Error ? e.message : String(e)
      error.value = msg
      insightData.value = null
    } finally {
      isLoading.value = false
    }
  }

  async function saveCurrentInsight(): Promise<void> {
    if (!currentTempTable.value || !currentColumn.value || isSaving.value) return

    isSaving.value = true
    error.value = null

    try {
      const versionId = await saveColumnInsightSnapshot({
        temp_table: currentTempTable.value,
        column_name: currentColumn.value
      })
      savedVersionId.value = versionId
      await loadHistory(currentColumn.value)
      await loadStorageStats()
    } catch (e: unknown) {
      const msg = e instanceof Error ? e.message : String(e)
      error.value = `保存失败: ${msg}`
    } finally {
      isSaving.value = false
    }
  }

  async function loadHistory(columnName?: string): Promise<void> {
    const col = columnName ?? currentColumn.value
    if (!col) return

    try {
      history.value = await getColumnInsightHistory(col)
    } catch {
      history.value = []
    }
  }

  async function loadStorageStats(): Promise<void> {
    try {
      storageStats.value = await getInsightStorageStats()
    } catch {
      storageStats.value = null
    }
  }

  function clear(): void {
    insightData.value = null
    currentColumn.value = null
    currentTempTable.value = null
    error.value = null
    isLoading.value = false
    isSaving.value = false
    savedVersionId.value = null
    multiResult.value = null
  }

  function clearMultiResult() {
    multiResult.value = null
    isMultiExecuting.value = false
  }

  // === 多列分析 actions ===

  async function loadMultiRules(): Promise<void> {
    try {
      const rules = await listInsightRules()
      multiRules.value = rules
    } catch {
      multiRules.value = []
    }
  }

  async function executeMultiRule(
    ruleId: string,
    params: Record<string, string>
  ): Promise<void> {
    if (!currentTempTable.value || isMultiExecuting.value) return

    isMultiExecuting.value = true
    multiResult.value = null
    error.value = null

    try {
      const result = await executeInsightRule({
        rule_id: ruleId,
        params,
        temp_table: currentTempTable.value,
      })
      multiResult.value = result
    } catch (e: unknown) {
      const msg = e instanceof Error ? e.message : String(e)
      error.value = `多列分析失败: ${msg}`
    } finally {
      isMultiExecuting.value = false
    }
  }

  // === 清理 actions ===

  async function cleanupOldSnapshots(days: number): Promise<void> {
    if (isCleaning.value) return

    isCleaning.value = true
    cleanupResult.value = null
    error.value = null

    try {
      const result = await cleanupInsightSnapshots(days)
      cleanupResult.value = result
      await loadStorageStats()
    } catch (e: unknown) {
      const msg = e instanceof Error ? e.message : String(e)
      error.value = `清理失败: ${msg}`
    } finally {
      isCleaning.value = false
    }
  }

  // === 版本对比 actions ===

  async function loadVersionDetail(versionId: string): Promise<void> {
    if (isDiffLoading.value || diffVersionId.value === versionId) return

    isDiffLoading.value = true
    diffVersionId.value = versionId
    diffData.value = null

    try {
      const result = await getInsightVersionDetail(versionId)
      diffData.value = result
    } catch {
      diffData.value = null
    } finally {
      isDiffLoading.value = false
    }
  }

  function clearDiff(): void {
    diffVersionId.value = null
    diffData.value = null
    isDiffLoading.value = false
  }

  // === 质量评分 action ===

  async function loadQualityScore(): Promise<void> {
    if (!currentColumn.value || !currentTempTable.value || isQualityLoading.value) return

    isQualityLoading.value = true
    qualityScore.value = null

    try {
      const score = await getColumnQuality({
        column_name: currentColumn.value,
        temp_table: currentTempTable.value,
      })
      qualityScore.value = score
    } catch {
      qualityScore.value = null
    } finally {
      isQualityLoading.value = false
    }
  }

  // === 表列探查 action ===

  async function loadColumnFromTable(input: {
    connId: string
    database: string
    schema: string
    table: string
    column: string
  }): Promise<void> {
    if (isLoading.value) return

    isLoading.value = true
    insightData.value = null
    currentColumn.value = input.column
    currentTempTable.value = null
    savedVersionId.value = null
    history.value = []
    error.value = null

    try {
      const result = await profileColumnFromTable({
        conn_id: input.connId,
        database: input.database,
        schema: input.schema,
        table: input.table,
        column_name: input.column,
      })
      insightData.value = result
    } catch (e: unknown) {
      const msg = e instanceof Error ? e.message : String(e)
      error.value = `列探查失败: ${msg}`
    } finally {
      isLoading.value = false
    }
  }

  return {
    currentColumn,
    currentTempTable,
    insightData,
    isLoading,
    isSaving,
    error,
    history,
    storageStats,
    savedVersionId,
    statsKind,
    hasData,
    nullRatePercent,
    nullRateDisplay,
    totalCountDisplay,
    historyCount,
    columnRules,
    multiColumnRules,

    availableColumns,
    multiRules,
    multiResult,
    isMultiExecuting,
    clearMultiResult,
    loadMultiRules,
    executeMultiRule,

    isCleaning,
    cleanupResult,
    cleanupOldSnapshots,

    diffVersionId,
    diffData,
    isDiffLoading,
    loadVersionDetail,
    clearDiff,
    loadColumnFromTable,

    qualityScore,
    isQualityLoading,
    loadQualityScore,

    loadColumnInsight,
    saveCurrentInsight,
    loadHistory,
    loadStorageStats,
    clear
  }
})
