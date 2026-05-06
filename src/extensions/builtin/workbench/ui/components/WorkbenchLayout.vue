<template>
  <div :class="['workbench-layout', uiStore.isDark ? 'dark' : 'light']">
    <WorkbenchTitleBar />
    <DockviewLayout />
    <WorkbenchStatusBar />
  </div>
</template>

<script setup lang="ts">
import DockviewLayout from '@/extensions/builtin/workbench/ui/components/DockviewLayout.vue'
import WorkbenchStatusBar from '@/extensions/builtin/workbench/ui/components/WorkbenchStatusBar.vue'
import WorkbenchTitleBar from '@/extensions/builtin/workbench/ui/components/WorkbenchTitleBar.vue'
import { useUiStore } from '@/shared/stores/ui'

const uiStore = useUiStore()
</script>

<style scoped>
.workbench-layout {
  width: 100vw;
  height: 100vh;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  font-family: var(--font-sans, -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif);
  background-color: var(--bg-primary);
}

.workbench-layout > :deep(.dockview-fullscreen) {
  flex: 1;
  min-height: 0;
}
</style>
