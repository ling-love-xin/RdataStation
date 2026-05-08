import { computed } from 'vue'

import { useResultStore } from '../stores/result-store'

import type { ResultTab } from '../types/result'

export function useResultTabs() {
  const store = useResultStore()

  const tabs = computed(() => store.tabs)
  const activeTabId = computed(() => store.activeTabId)
  const activeTab = computed(() => store.activeTab)
  const hasActiveTab = computed(() => activeTab.value !== null)

  function addTab(sql: string, connectionId: string): ResultTab {
    return store.addTab(sql, connectionId)
  }

  function removeTab(id: string): void {
    store.closeTab(id)
  }

  function activateTab(id: string): void {
    store.switchTab(id)
  }

  function renameTab(id: string, title: string): void {
    const tab = store.tabs.find(t => t.id === id)
    if (tab) tab.title = title
  }

  function tabHasDirty(tab: ResultTab): boolean {
    return tab.dirtyRows.size > 0
  }

  return {
    tabs,
    activeTabId,
    activeTab,
    hasActiveTab,
    addTab,
    removeTab,
    activateTab,
    renameTab,
    tabHasDirty,
  }
}
