<template>
  <div class="panel-header-actions">
    <NTooltip v-if="!isFloating" trigger="hover" placement="bottom">
      <template #trigger>
        <NButton size="tiny" quaternary @click="handleMaximize">
          <Maximize2 v-if="!isMaximized" :size="14" />
          <Minimize2 v-else :size="14" />
        </NButton>
      </template>
      <span>{{ isMaximized ? '还原' : '最大化（将当前组铺满中心区域，有多个组时效果更明显）' }}</span>
    </NTooltip>
    <NTooltip trigger="hover" placement="bottom">
      <template #trigger>
        <NButton size="tiny" quaternary @click="handleFloat">
          <ExternalLink v-if="!isFloating" :size="14" />
          <ArrowLeftToLine v-else :size="14" />
        </NButton>
      </template>
      <span>{{ isFloating ? '放回主网格' : '浮动窗口' }}</span>
    </NTooltip>
    <NTooltip trigger="hover" placement="bottom">
      <template #trigger>
        <NButton size="tiny" quaternary :class="{ 'is-pinned': isPinned }" @click="handlePin">
          <Pin v-if="!isPinned" :size="14" />
          <PinOff v-else :size="14" />
        </NButton>
      </template>
      <span>{{ isPinned ? '取消钉住' : '钉住（防止面板被关闭）' }}</span>
    </NTooltip>
  </div>
</template>

<script setup lang="ts">
import { ExternalLink, Maximize2, Minimize2, ArrowLeftToLine, Pin, PinOff } from 'lucide-vue-next'
import { NButton, NTooltip } from 'naive-ui'
import { computed, ref, watch } from 'vue'

import { useLayoutStore } from '@/extensions/builtin/workbench/ui/stores/layout-store'

const p = defineProps<{ params: any }>()

const layoutStore = useLayoutStore()

const localMaximized = ref(false)

watch(() => p.params, (params) => {
  if (params?.group?.api) {
    const groupApi = params.group.api
    localMaximized.value = groupApi.isMaximized?.() ?? false
    
    groupApi.onDidMaximizedChange?.((isMax: boolean) => {
      localMaximized.value = isMax
    })
  }
}, { immediate: true })

function getGroup() {
  return p.params?.group
}

function getAccessor() {
  return p.params?.group?.model?.accessor
}

function getApi() {
  return p.params?.api
}

function getGroupApi() {
  return p.params?.group?.api
}

function getPanelId() {
  return p.params?.panel?.id
}

const isFloating = computed(() => {
  const api = getApi()
  return api?.location?.type === 'floating'
})

const isMaximized = computed(() => {
  return localMaximized.value
})

const isPinned = computed(() => {
  const panelId = getPanelId()
  if (!panelId) return false
  return layoutStore.isPanelPinned(panelId)
})

function handleMaximize() {
  const groupApi = getGroupApi()
  if (isMaximized.value) {
    groupApi?.exitMaximized?.()
  } else {
    groupApi?.maximize?.()
  }
}

function handleFloat() {
  const group = getGroup()
  const accessor = getAccessor()
  const api = getApi()
  
  if (isFloating.value) {
    api?.moveTo?.({ position: 'center' })
  } else {
    accessor?.addFloatingGroup?.(group)
  }
}

function handlePin() {
  const panelId = getPanelId()
  if (panelId) {
    layoutStore.togglePanelPinned(panelId)
  }
}
</script>

<style scoped>
.is-pinned {
  color: var(--primary-color, #165DFF);
}
</style>
