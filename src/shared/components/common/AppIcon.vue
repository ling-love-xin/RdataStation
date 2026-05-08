<template>
  <component :is="iconComponent" v-bind="iconProps" />
</template>

<script setup lang="ts">
import { computed } from 'vue'

import { useAppStore } from '@/stores/useAppStore'

import type { LucideIcon } from 'lucide-vue-next'

// 使用动态导入避免 namespace 问题
const iconModules = import.meta.glob('lucide-vue-next', { eager: true })
const icons = iconModules['lucide-vue-next'] as Record<string, LucideIcon>

interface Props {
  name: string
  size?: number
  strokeWidth?: number
  accent?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  size: 16,
  strokeWidth: 2,
  accent: false,
})

const appStore = useAppStore()

const iconComponent = computed<LucideIcon>(() => {
  return icons[props.name] as LucideIcon
})

const iconProps = computed(() => {
  const color = props.accent ? '#E17055' : appStore.isDark ? '#E5E7EB' : '#1F2937'
  return {
    size: props.size,
    'stroke-width': props.strokeWidth,
    color,
  }
})
</script>
