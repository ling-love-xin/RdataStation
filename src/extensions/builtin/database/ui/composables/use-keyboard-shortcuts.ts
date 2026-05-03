import { ref, computed, onMounted, onUnmounted } from 'vue'

export interface ShortcutAction {
  id: string
  name: string
  description: string
  shortcut: string
  category: 'navigation' | 'connection' | 'search' | 'editor'
  handler: () => void
}

export function useKeyboardShortcuts() {
  const isPanelOpen = ref(false)
  const registeredActions = ref<ShortcutAction[]>([])

  const navigationActions = computed(() =>
    registeredActions.value.filter(a => a.category === 'navigation')
  )

  const connectionActions = computed(() =>
    registeredActions.value.filter(a => a.category === 'connection')
  )

  const searchActions = computed(() =>
    registeredActions.value.filter(a => a.category === 'search')
  )

  const editorActions = computed(() =>
    registeredActions.value.filter(a => a.category === 'editor')
  )

  function registerAction(action: ShortcutAction) {
    registeredActions.value.push(action)
  }

  function unregisterAction(id: string) {
    registeredActions.value = registeredActions.value.filter(a => a.id !== id)
  }

  function togglePanel() {
    isPanelOpen.value = !isPanelOpen.value
  }

  function closePanel() {
    isPanelOpen.value = false
  }

  function executeAction(id: string) {
    const action = registeredActions.value.find(a => a.id === id)
    if (action) {
      action.handler()
      closePanel()
    }
  }

  function handleGlobalShortcuts(event: KeyboardEvent) {
    if (event.ctrlKey && event.key === 'p') {
      event.preventDefault()
      togglePanel()
      return
    }

    if (event.key === 'F5') {
      event.preventDefault()
      const refreshAction = registeredActions.value.find(a => a.id === 'refresh')
      if (refreshAction) {
        refreshAction.handler()
      }
      return
    }

    if (event.ctrlKey && event.key === 'd') {
      event.preventDefault()
      const disconnectAction = registeredActions.value.find(a => a.id === 'disconnect')
      if (disconnectAction) {
        disconnectAction.handler()
      }
      return
    }

    if (event.ctrlKey && event.shiftKey && event.key === 'p') {
      event.preventDefault()
      togglePanel()
      return
    }

    for (const action of registeredActions.value) {
      if (matchesShortcut(event, action.shortcut)) {
        event.preventDefault()
        action.handler()
        break
      }
    }
  }

  function matchesShortcut(event: KeyboardEvent, shortcut: string): boolean {
    const parts = shortcut.toLowerCase().split('+')
    const ctrl = parts.includes('ctrl')
    const shift = parts.includes('shift')
    const alt = parts.includes('alt')
    const key = parts[parts.length - 1]

    return (
      event.ctrlKey === ctrl &&
      event.shiftKey === shift &&
      event.altKey === alt &&
      event.key.toLowerCase() === key
    )
  }

  function registerDefaultActions(handlers: {
    onRefresh: () => void
    onDisconnect: () => void
    onFocusSearch: () => void
    onTogglePanel: () => void
  }) {
    registerAction({
      id: 'refresh',
      name: '刷新',
      description: '刷新当前节点元数据',
      shortcut: 'F5',
      category: 'navigation',
      handler: handlers.onRefresh
    })

    registerAction({
      id: 'disconnect',
      name: '断开连接',
      description: '断开当前数据库连接',
      shortcut: 'Ctrl+D',
      category: 'connection',
      handler: handlers.onDisconnect
    })

    registerAction({
      id: 'focus-search',
      name: '聚焦搜索',
      description: '聚焦到搜索输入框',
      shortcut: 'Ctrl+P',
      category: 'search',
      handler: handlers.onFocusSearch
    })

    registerAction({
      id: 'toggle-panel',
      name: '快捷面板',
      description: '打开快捷操作面板',
      shortcut: 'Ctrl+Shift+P',
      category: 'navigation',
      handler: handlers.onTogglePanel
    })
  }

  onMounted(() => {
    window.addEventListener('keydown', handleGlobalShortcuts)
  })

  onUnmounted(() => {
    window.removeEventListener('keydown', handleGlobalShortcuts)
  })

  return {
    isPanelOpen,
    registeredActions,
    navigationActions,
    connectionActions,
    searchActions,
    editorActions,
    registerAction,
    unregisterAction,
    togglePanel,
    closePanel,
    executeAction,
    registerDefaultActions
  }
}
