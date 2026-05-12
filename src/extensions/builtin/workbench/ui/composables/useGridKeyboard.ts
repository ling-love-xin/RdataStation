import { onMounted, onUnmounted, type ComputedRef } from 'vue'

import type { ResultTab } from '../types/result'
import type { GridApi } from 'ag-grid-community'

export interface KeyboardBinding {
  key: string
  ctrlKey?: boolean
  shiftKey?: boolean
  handler: (event: KeyboardEvent) => void
}

export function useGridKeyboard(
  activeTab: ComputedRef<ResultTab | null>,
  gridApi: { value: GridApi | null },
  callbacks: {
    onRefresh: (tab: ResultTab) => void
    onSave: (tab: ResultTab) => void
    onCancel: (tab: ResultTab) => void
    onCopy: () => void
  }
) {
  const bindings: KeyboardBinding[] = [
    {
      key: 'Enter',
      ctrlKey: true,
      handler: () => {
        const tab = activeTab.value
        if (tab) callbacks.onRefresh(tab)
      },
    },
    {
      key: 's',
      ctrlKey: true,
      handler: event => {
        event.preventDefault()
        const tab = activeTab.value
        if (tab) callbacks.onSave(tab)
      },
    },
    {
      key: 'r',
      ctrlKey: true,
      handler: event => {
        event.preventDefault()
        const tab = activeTab.value
        if (tab) callbacks.onRefresh(tab)
      },
    },
    {
      key: 'c',
      ctrlKey: true,
      handler: () => {
        callbacks.onCopy()
      },
    },
  ]

  function handleKeydown(event: KeyboardEvent): void {
    for (const binding of bindings) {
      const ctrlMatch = binding.ctrlKey ? event.ctrlKey || event.metaKey : true
      const shiftMatch = binding.shiftKey ? event.shiftKey : true
      if (event.key === binding.key && ctrlMatch && shiftMatch) {
        binding.handler(event)
        return
      }
    }
  }

  onMounted(() => {
    document.addEventListener('keydown', handleKeydown)
  })

  onUnmounted(() => {
    document.removeEventListener('keydown', handleKeydown)
  })

  return {
    handleKeydown,
    bindings,
  }
}
