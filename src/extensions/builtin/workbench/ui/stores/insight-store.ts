/**
 * 洞察状态管理
 *
 * 管理列洞察面板的数据加载、缓存和状态
 */
import { defineStore } from 'pinia'
import { ref, computed } from 'vue'

import { getColumnInsightFull } from '../services/result-analysis'

import type { ColumnInsightFull } from '../services/result-analysis'

export const useInsightStore = defineStore('insight', () => {
  const currentColumn = ref<string | null>(null)
  const currentTempTable = ref<string | null>(null)
  const insightData = ref<ColumnInsightFull | null>(null)
  const isLoading = ref(false)
  const error = ref<string | null>(null)

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

  async function loadColumnInsight(tempTable: string, column: string): Promise<void> {
    if (isLoading.value) return

    isLoading.value = true
    error.value = null

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

  function clear(): void {
    insightData.value = null
    currentColumn.value = null
    currentTempTable.value = null
    error.value = null
    isLoading.value = false
  }

  return {
    currentColumn,
    currentTempTable,
    insightData,
    isLoading,
    error,
    statsKind,
    hasData,
    nullRatePercent,
    nullRateDisplay,
    totalCountDisplay,
    loadColumnInsight,
    clear
  }
})
