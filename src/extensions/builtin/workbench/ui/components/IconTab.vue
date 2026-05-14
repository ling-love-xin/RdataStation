<template>
  <div
    class="icon-tab"
    :class="{ 'icon-tab--edge': isEdgeGroup }"
    :title="isEdgeGroup ? titleText : undefined"
  >
    <component :is="iconComponent" :size="14" class="icon-tab-icon" />
    <span v-if="!isEdgeGroup" class="icon-tab-title">{{ titleText }}</span>
  </div>
</template>

<script setup lang="ts">
import {
  Database,
  BarChart3,
  Puzzle,
  FileText,
  FileCode,
  Sparkles,
  StickyNote,
  Dices,
  Layout,
} from 'lucide-vue-next'
import { computed, onBeforeUnmount, onMounted, ref } from 'vue'

import type { Component } from 'vue'

interface TabApi {
  id: string
  title?: string
  onDidTitleChange?: (listener: (e: { title: string }) => void) => { dispose: () => void }
}

const props = defineProps<{
  params: {
    api: TabApi
    title?: string
    params?: Record<string, unknown>
    containerApi?: unknown
    groupApi?: unknown
    tabLocation?: string
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
  sqlEditor: Database,
  codeEditor: FileCode,
  queryResult: BarChart3,
  multiTabResult: BarChart3,
  dynamicObjectProperties: FileText,
}

const componentId = computed(() => {
  const panelId = props.params.api?.id || ''
  return panelId.replace(/^panel_/, '').replace(/_\d+$/, '')
})

const iconComponent = computed(() => {
  return PANEL_ICONS[componentId.value] || Layout
})

const isEdgeGroup = computed(() => {
  const groupApi = props.params.groupApi as { id?: string } | undefined
  const groupId = groupApi?.id || ''
  return groupId === 'left-edge' || groupId === 'right-edge'
})

const titleText = ref('')

let titleDisposable: { dispose: () => void } | null = null

onMounted(() => {
  const api = props.params.api
  titleText.value = api?.title || ''

  if (api?.onDidTitleChange) {
    titleDisposable = api.onDidTitleChange((e: { title: string }) => {
      titleText.value = e.title
    })
  }
})

onBeforeUnmount(() => {
  titleDisposable?.dispose()
  titleDisposable = null
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
  color: inherit;
}

.icon-tab--edge {
  justify-content: center;
  gap: 0;
}

.icon-tab--edge .icon-tab-icon {
  opacity: 0.9;
}
</style>