import { defineStore } from 'pinia'
import { ref, computed } from 'vue'

export type Theme = 'light' | 'dark' | 'system'

export const useUiStore = defineStore('ui', () => {
  // State
  const theme = ref<Theme>('dark')
  const sidebarCollapsed = ref(false)
  const sidebarWidth = ref(280)
  const showHistoryPanel = ref(false)
  const showConnectionPanel = ref(true)

  // Getters
  const isDark = computed(() => {
    if (theme.value === 'system') {
      return window.matchMedia('(prefers-color-scheme: dark)').matches
    }
    return theme.value === 'dark'
  })

  const effectiveTheme = computed(() => isDark.value ? 'dark' : 'light')

  // Actions
  function setTheme(newTheme: Theme) {
    theme.value = newTheme
    applyTheme()
  }

  function toggleTheme() {
    theme.value = isDark.value ? 'light' : 'dark'
    applyTheme()
  }

  function applyTheme() {
    const html = document.documentElement
    if (isDark.value) {
      html.classList.add('dark')
    } else {
      html.classList.remove('dark')
    }
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

  // 初始化主题
  function initTheme() {
    applyTheme()
    // 监听系统主题变化
    window.matchMedia('(prefers-color-scheme: dark)').addEventListener('change', () => {
      if (theme.value === 'system') {
        applyTheme()
      }
    })
  }

  return {
    // State
    theme,
    sidebarCollapsed,
    sidebarWidth,
    showHistoryPanel,
    showConnectionPanel,
    // Getters
    isDark,
    effectiveTheme,
    // Actions
    setTheme,
    toggleTheme,
    toggleSidebar,
    setSidebarWidth,
    toggleHistoryPanel,
    toggleConnectionPanel,
    initTheme
  }
})
