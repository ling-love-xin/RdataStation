<template>
  <div class="panel-header-actions">
    <NTooltip v-if="!isFloating" trigger="hover" placement="bottom">
      <template #trigger>
        <NButton size="tiny" quaternary @click="handleMaximize">
          <Maximize :size="14" />
        </NButton>
      </template>
      <span>最大化（将当前组铺满中心区域，有多个组时效果更明显）</span>
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
  </div>
</template>

<script setup lang="ts">
import { ExternalLink, Maximize, ArrowLeftToLine } from 'lucide-vue-next'
import { NButton, NTooltip } from 'naive-ui'
import { watch, ref, computed } from 'vue'

const p = defineProps<{ params: any }>()

const lastGroup = ref<any>(null)
const lastApi = ref<any>(null)
const lastAccessor = ref<any>(null)
const locationType = ref<string>('grid')
const isFloating = computed(() => locationType.value === 'floating')

watch(() => p.params, (params) => {
  if (!params) return
  if (params.group) {
    lastGroup.value = params.group
    lastAccessor.value = params.group.model?.accessor
  }
  if (params.api) {
    lastApi.value = params.api
    locationType.value = params.api.location?.type || 'grid'
  }
}, { immediate: true, deep: true })

function handleMaximize() {
  lastApi.value?.maximize?.()
}

function handleFloat() {
  if (isFloating.value) {
    lastApi.value?.moveTo?.({ position: 'center' })
  } else {
    lastAccessor.value?.addFloatingGroup?.(lastGroup.value)
  }
}
</script>
