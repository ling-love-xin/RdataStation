<template>
  <div class="main-layout">
    <WorkbenchTitleBar
      :is-maximized="isMaximized"
      @minimize="handleMinimize"
      @maximize="handleMaximize"
      @close="handleClose"
    />
    <div class="main-content">
      <router-view />
    </div>
  </div>
</template>

<script setup lang="ts">
import { getCurrentWindow } from '@tauri-apps/api/window'
import { ref } from 'vue'

import WorkbenchTitleBar from '@/extensions/builtin/workbench/ui/components/WorkbenchTitleBar.vue'

const isMaximized = ref(false)

const handleMinimize = async () => {
  const window = getCurrentWindow()
  await window.minimize()
}

const handleMaximize = async () => {
  const window = getCurrentWindow()
  await window.toggleMaximize()
  isMaximized.value = !isMaximized.value
}

const handleClose = async () => {
  const window = getCurrentWindow()
  await window.close()
}
</script>

<style scoped>
.main-layout {
  width: 100vw;
  height: 100vh;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.main-content {
  flex: 1;
  overflow: hidden;
}
</style>
