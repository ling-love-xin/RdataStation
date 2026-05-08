import { computed, type ComputedRef } from 'vue'

import { useResultStore } from '../stores/result-store'

import type { FilterMode, ResultTab } from '../types/result'

export function useFilterModes(activeTab: ComputedRef<ResultTab | null>) {
  const store = useResultStore()

  const currentMode = computed<FilterMode>(() => activeTab.value?.filterMode ?? 'quick')
  const isQuickActive = computed(() => currentMode.value === 'quick')
  const isSqlActive = computed(() => currentMode.value === 'sql')
  const isDuckdbActive = computed(() => currentMode.value === 'duckdb')

  function setMode(mode: FilterMode): void {
    const tab = activeTab.value
    if (!tab) return
    tab.filterMode = mode
  }

  function applyQuickFilter(expression: string): void {
    const tab = activeTab.value
    if (!tab) return
    tab.quickFilterExpression = expression
  }

  async function applySqlFilter(clause: string): Promise<void> {
    const tab = activeTab.value
    if (!tab) return
    await store.executeSqlFilter(tab.id, clause)
  }

  async function applyDuckdbAnalysis(sql: string): Promise<void> {
    const tab = activeTab.value
    if (!tab) return
    await store.executeDuckdbAnalysis(tab.id, sql)
  }

  function clearCurrentFilter(): void {
    const tab = activeTab.value
    if (!tab) return
    store.clearFilter(tab.id)
  }

  function modeLabel(): string {
    const labels: Record<FilterMode, string> = {
      quick: 'Quick',
      sql: 'SQL',
      duckdb: 'DuckDB',
    }
    return labels[currentMode.value]
  }

  return {
    currentMode,
    isQuickActive,
    isSqlActive,
    isDuckdbActive,
    setMode,
    applyQuickFilter,
    applySqlFilter,
    applyDuckdbAnalysis,
    clearCurrentFilter,
    modeLabel,
  }
}
