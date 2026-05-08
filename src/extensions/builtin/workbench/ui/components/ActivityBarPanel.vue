<template>
  <ActivityBar
    :items="panelParams.items"
    :position="panelParams.position"
    :show-toggle="panelParams.showToggle ?? true"
    :is-hidden="panelParams.isHidden ?? false"
  />
</template>

<script setup lang="ts">
import { computed } from 'vue'

import type {
  LeftActivityItem,
  RightActivityItem,
} from '@/extensions/builtin/workbench/ui/stores/layout-store'

import ActivityBar from './ActivityBar.vue'

interface ActivityBarPanelParams {
  items: LeftActivityItem[] | RightActivityItem[]
  position?: 'left' | 'right'
  showToggle?: boolean
  isHidden?: boolean
}

const props = defineProps<{
  params: {
    params: ActivityBarPanelParams
    api: unknown
    containerApi: unknown
  }
}>()

const panelParams = computed(() => props.params.params)
</script>
