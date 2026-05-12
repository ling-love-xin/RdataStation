<template>
  <div class="icon-tab">
    <component :is="iconComponent" :size="14" class="icon-tab-icon" />
    <span class="icon-tab-title">{{ title }}</span>
  </div>
</template>

<script setup lang="ts">
import {
  Database,
  BarChart3,
  Puzzle,
  FileText,
  Sparkles,
  StickyNote,
  Dices,
  Layout,
} from 'lucide-vue-next'
import { computed } from 'vue'

import type { Component } from 'vue'

const props = defineProps<{
  params: {
    api: { id: string; title?: string }
    title?: string
    params?: Record<string, unknown>
    containerApi?: unknown
    groupApi?: unknown
  }
}>()

const PANEL_ICONS: Record<string, Component> = {
  scratchpad: StickyNote,
  databaseNavigator: Database,
  'analytics-resource-manager': BarChart3,
  plugins: Puzzle,
  sqlHistory: FileText,
  mockPanel: Dices,
  columnInsights: Sparkles,
  emptyWorkbench: Layout,
}

const componentId = computed(() => {
  const panelId = props.params.api?.id || ''
  return panelId.replace(/^panel_/, '')
})

const iconComponent = computed(() => {
  return PANEL_ICONS[componentId.value] || Layout
})

const title = computed(() => {
  return props.params.title || props.params.api?.title || ''
})
</script>

<style scoped>
.icon-tab {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 0 4px;
  min-width: 0;
  overflow: hidden;
}

.icon-tab-icon {
  flex-shrink: 0;
  opacity: 0.8;
}

.icon-tab-title {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  font-size: 12px;
}
</style>