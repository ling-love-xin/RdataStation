/**
 * @deprecated 由 ShortcutManager + 'global' scope 快捷键替代。
 * Ctrl+B / Ctrl+Shift+B / Escape 功能已在 WorkbenchView.onReady 中
 * 通过 ShortcutManager.register() 注册为全局快捷键。
 *
 * @see {@link import('@/extensions/builtin/workbench/manager/ShortcutManager').ShortcutManager}
 */
import { onMounted, onUnmounted } from 'vue'

import type { DockviewApi } from 'dockview-vue'

interface DockviewKeyboardOptions {
  layoutStore: {
    collapseLeftEdgeGroup: () => void
    expandLeftEdgeGroup: () => void
    collapseRightEdgeGroup: () => void
    expandRightEdgeGroup: () => void
    dockviewApi: DockviewApi | null
  }
}

export function useDockviewKeyboard(options: DockviewKeyboardOptions) {
  const { layoutStore } = options

  function handleKeydown(e: KeyboardEvent) {
    const mod = e.ctrlKey || e.metaKey

    if (mod && e.key === 'b' && !e.shiftKey) {
      e.preventDefault()
      const leftGroup = layoutStore.dockviewApi?.getEdgeGroup('left')
      if (leftGroup?.isCollapsed()) {
        layoutStore.expandLeftEdgeGroup()
      } else {
        layoutStore.collapseLeftEdgeGroup()
      }
      return
    }

    if (mod && e.key === 'B') {
      e.preventDefault()
      const rightGroup = layoutStore.dockviewApi?.getEdgeGroup('right')
      if (rightGroup?.isCollapsed()) {
        layoutStore.expandRightEdgeGroup()
      } else {
        layoutStore.collapseRightEdgeGroup()
      }
      return
    }

    if (e.key === 'Escape' && !mod && !e.shiftKey) {
      const activeEl = document.activeElement
      if (activeEl && (activeEl.tagName === 'INPUT' || activeEl.tagName === 'TEXTAREA')) {
        return
      }
      const leftGroup = layoutStore.dockviewApi?.getEdgeGroup('left')
      if (leftGroup && leftGroup.isMaximized()) {
        leftGroup.exitMaximized()
        return
      }
      const rightGroup = layoutStore.dockviewApi?.getEdgeGroup('right')
      if (rightGroup && rightGroup.isMaximized()) {
        rightGroup.exitMaximized()
        return
      }
    }
  }

  onMounted(() => {
    window.addEventListener('keydown', handleKeydown)
  })

  onUnmounted(() => {
    window.removeEventListener('keydown', handleKeydown)
  })
}
