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
import ActivityBar from './ActivityBar.vue'

const props = defineProps<{
  params: {
    params: Record<string, unknown>
    api: unknown
    containerApi: unknown
    tabLocation?: string
  }
}>()

const panelParams = computed(() => (props.params.params || {}) as Record<string, unknown>)
</script>
