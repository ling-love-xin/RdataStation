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
    <WorkbenchStatusBar />
    <SettingsDialog :show="showSettingsDialog" @update:show="showSettingsDialog = $event" />
    <AddDataSourceDialog
      v-model="showConnectionModal"
      :project-path="appStore.projectPath"
      @save="handleSaveConnection"
    />
  </div>
</template>

<script setup lang="ts">
import { getCurrentWindow } from '@tauri-apps/api/window'
import { onMounted, onUnmounted, ref } from 'vue'

import { useProjectStore } from '@/core/project/stores/project'
import AddDataSourceDialog from '@/extensions/builtin/connection/ui/components/AddDataSourceDialog.vue'
import * as connectionService from '@/extensions/builtin/connection/ui/services/connection'
import { useConnectionStore } from '@/extensions/builtin/connection/ui/stores/connection-store'
import SettingsDialog from '@/extensions/builtin/settings/ui/components/SettingsDialog.vue'
import WorkbenchStatusBar from '@/extensions/builtin/workbench/ui/components/WorkbenchStatusBar.vue'
import WorkbenchTitleBar from '@/extensions/builtin/workbench/ui/components/WorkbenchTitleBar.vue'
import {
  WorkbenchEvent,
  listenWorkbenchEvent,
} from '@/extensions/builtin/workbench/ui/constants/workbench-events'
import { useAppStore } from '@/stores/useAppStore'

const isMaximized = ref(false)
const showSettingsDialog = ref(false)
const showConnectionModal = ref(false)

const appStore = useAppStore()
const projectStore = useProjectStore()
const connectionStore = useConnectionStore()

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

const handleOpenSettings = () => {
  showSettingsDialog.value = true
}

const handleOpenConnectionModal = () => {
  showConnectionModal.value = true
}

const handleSaveConnection = async (data: Record<string, unknown>) => {
  try {
    const driver = data.driver as string | undefined
    if (!driver) return

    const url = data.url as string | undefined
    if (!url) return

    const saveToGlobal = Boolean(data.saveToGlobal)
    const saveToProject = Boolean(data.saveToProject)
    const name = (data.name as string) || ''

    if (!saveToGlobal && !saveToProject) return

    const connectOpts = {
      driverId: data.driverId as string | undefined,
      networkConfigId: (data.networkConfigId as string | null) ?? null,
      environmentId: data.environmentId as string | undefined,
      authConfigId: data.authConfigId as string | undefined,
      driverProperties: data.driverProps ? JSON.stringify(data.driverProps) : undefined,
      advancedOptions: data.advanced ? JSON.stringify(data.advanced) : undefined,
      description: data.description as string | undefined,
    }

    if (saveToGlobal) {
      await connectionService.connectDatabase(driver, url, name, 'global', undefined, connectOpts)
    }

    if (saveToProject) {
      const projectId = projectStore.currentProject?.id
      if (projectId) {
        await connectionService.connectDatabase(driver, url, name, 'project', projectId, connectOpts)
      }
    }

    await connectionStore.loadConnections()
    window.dispatchEvent(new CustomEvent('navigator-refresh'))
    showConnectionModal.value = false
  } catch (err) {
    console.error('保存连接失败:', err)
  }
}

let cleanupSettingsListener: (() => void) | null = null
let cleanupConnectionListener: (() => void) | null = null

onMounted(() => {
  cleanupSettingsListener = listenWorkbenchEvent(WorkbenchEvent.OpenSettings, handleOpenSettings)
  cleanupConnectionListener = listenWorkbenchEvent(WorkbenchEvent.NewConnection, handleOpenConnectionModal)
  window.addEventListener('open-connection-modal', handleOpenConnectionModal)
})

onUnmounted(() => {
  cleanupSettingsListener?.()
  cleanupConnectionListener?.()
  window.removeEventListener('open-connection-modal', handleOpenConnectionModal)
})
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
