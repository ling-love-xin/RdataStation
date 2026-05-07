import { defineStore } from 'pinia'
import { ref, computed } from 'vue'

import type { Theme } from '@/stores/config'
import { useAppStore } from '@/stores/useAppStore'


export type { Theme }

export const useUiStore = defineStore('ui', () => {
  const sidebarCollapsed = ref(false)
  const sidebarWidth = ref(280)
  const showHistoryPanel = ref(false)
  const showConnectionPanel = ref(true)

  const isDark = computed(() => {
    const appStore = useAppStore()
    return appStore.isDark
  })

  const theme = computed<Theme>({
    get: () => {
      const appStore = useAppStore()
      return appStore.effectiveTheme
    },
    set: (value: Theme) => {
      const appStore = useAppStore()
      appStore.setTheme(value)
    },
  })

  const effectiveTheme = computed(() => (isDark.value ? 'dark' : 'light'))

  function setTheme(newTheme: Theme) {
    theme.value = newTheme
    applyTheme()
  }

  function toggleTheme() {
    theme.value = isDark.value ? 'light' : 'dark'
    applyTheme()
  }

  function applyTheme() {
    const appStore = useAppStore()
    appStore.applyTheme()
  }

  function toggleSidebar() {
    sidebarCollapsed.value = !sidebarCollapsed.value
  }

  function setSidebarWidth(width: number) {
    sidebarWidth.value = Math.max(200, Math.min(400, width))
  }

  function toggleHistoryPanel() {
    showHistoryPanel.value = !showHistoryPanel.value
  }

  function toggleConnectionPanel() {
    showConnectionPanel.value = !showConnectionPanel.value
  }

  function initTheme() {
    applyTheme()
  }

  return {
    theme,
    sidebarCollapsed,
    sidebarWidth,
    showHistoryPanel,
    showConnectionPanel,
    isDark,
    effectiveTheme,
    setTheme,
    toggleTheme,
    toggleSidebar,
    setSidebarWidth,
    toggleHistoryPanel,
    toggleConnectionPanel,
    initTheme,
    applyTheme,
  }
})
