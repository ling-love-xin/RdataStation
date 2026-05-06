<template>
  <div class="customize-layout-dialog" @click.self="handleClose">
    <div class="dialog-content">
      <div class="dialog-header">
        <h3>Customize Layout</h3>
        <button class="close-btn" @click="handleClose">
          <X :size="16" />
        </button>
      </div>

      <div class="dialog-body">
        <!-- Edge Group 控制 -->
        <div class="section">
          <div class="section-title">侧边栏</div>

          <div class="section-item" @click="toggleLeftEdgeGroup">
            <div class="item-left">
              <PanelLeft :size="16" class="item-icon" />
              <span class="item-label">左侧边栏</span>
            </div>
            <div class="item-right">
              <span class="item-hint">{{ layoutStore.leftEdgeGroupCollapsed ? '已收起' : '已展开' }}</span>
              <div class="checkbox" :class="{ checked: !layoutStore.leftEdgeGroupCollapsed }">
                <Check v-if="!layoutStore.leftEdgeGroupCollapsed" :size="12" />
              </div>
            </div>
          </div>

          <div class="section-item" @click="toggleRightEdgeGroup">
            <div class="item-left">
              <PanelRight :size="16" class="item-icon" />
              <span class="item-label">右侧边栏</span>
            </div>
            <div class="item-right">
              <span class="item-hint">{{ layoutStore.rightEdgeGroupCollapsed ? '已收起' : '已展开' }}</span>
              <div class="checkbox" :class="{ checked: !layoutStore.rightEdgeGroupCollapsed }">
                <Check v-if="!layoutStore.rightEdgeGroupCollapsed" :size="12" />
              </div>
            </div>
          </div>
        </div>

        <div class="divider"></div>

        <!-- 界面元素可见性 -->
        <div class="section">
          <div class="section-title">界面元素</div>

          <div class="section-item" @click="toggleVisibility('menuBar')">
            <div class="item-left">
              <Menu :size="16" class="item-icon" />
              <span class="item-label">菜单栏</span>
            </div>
            <div class="item-right">
              <div class="checkbox" :class="{ checked: layoutStore.menuBarVisible }">
                <Check v-if="layoutStore.menuBarVisible" :size="12" />
              </div>
            </div>
          </div>

          <div class="section-item" @click="toggleVisibility('statusBar')">
            <div class="item-left">
              <Minus :size="16" class="item-icon" />
              <span class="item-label">状态栏</span>
            </div>
            <div class="item-right">
              <div class="checkbox" :class="{ checked: layoutStore.statusBarVisible }">
                <Check v-if="layoutStore.statusBarVisible" :size="12" />
              </div>
            </div>
          </div>
        </div>

        <div class="divider"></div>

        <!-- 全屏 -->
        <div class="section">
          <div class="section-title">窗口</div>

          <div class="section-item" @click="toggleFullScreen">
            <div class="item-left">
              <Maximize :size="16" class="item-icon" />
              <span class="item-label">全屏</span>
            </div>
            <div class="item-right">
              <div class="shortcut">
                <kbd>F11</kbd>
              </div>
              <div class="checkbox" :class="{ checked: fullScreen }">
                <Check v-if="fullScreen" :size="12" />
              </div>
            </div>
          </div>
        </div>

        <div class="divider"></div>

        <!-- 重置 -->
        <div class="section">
          <div class="reset-item" @click="handleResetLayout">
            <RotateCcw :size="16" class="item-icon" />
            <span class="item-label">重置布局</span>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import {
  X,
  Menu,
  PanelLeft,
  PanelRight,
  Minus,
  Check,
  Maximize,
  RotateCcw
} from 'lucide-vue-next'
import { ref, onMounted, onUnmounted } from 'vue'

import { useLayoutStore } from '../stores/layout-store'

const layoutStore = useLayoutStore()

const emit = defineEmits<{
  close: []
}>()

const fullScreen = ref(false)

function handleClose() {
  emit('close')
}

function getGroupApi(panelId: string) {
  const panel = layoutStore.dockviewApi?.getPanel(panelId)
  return (panel as any)?.group?.api
}

function toggleLeftEdgeGroup() {
  if (layoutStore.leftEdgeGroupCollapsed) {
    layoutStore.expandLeftEdgeGroup()
  } else {
    layoutStore.collapseLeftEdgeGroup()
  }
}

function toggleRightEdgeGroup() {
  layoutStore.rightEdgeGroupCollapsed = !layoutStore.rightEdgeGroupCollapsed
  const groupApi = getGroupApi('panel_rightActivityBar')
  if (layoutStore.rightEdgeGroupCollapsed) {
    groupApi?.collapse?.()
  } else {
    groupApi?.expand?.()
  }
}

function toggleVisibility(key: string) {
  switch (key) {
    case 'menuBar':
      layoutStore.toggleMenuBar()
      break
    case 'statusBar':
      layoutStore.toggleStatusBar()
      break
  }
}

function toggleFullScreen() {
  fullScreen.value = !fullScreen.value
  if (fullScreen.value) {
    document.documentElement.requestFullscreen?.()
  } else {
    document.exitFullscreen?.()
  }
}

function handleResetLayout() {
  layoutStore.resetLayout()
}

function onFullScreenChange() {
  fullScreen.value = !!document.fullscreenElement
}

function handleKeyDown(e: KeyboardEvent) {
  if (e.key === 'Escape') {
    handleClose()
  }
}

onMounted(() => {
  document.addEventListener('keydown', handleKeyDown)
  document.addEventListener('fullscreenchange', onFullScreenChange)
})

onUnmounted(() => {
  document.removeEventListener('keydown', handleKeyDown)
  document.removeEventListener('fullscreenchange', onFullScreenChange)
})
</script>

<style scoped>
.customize-layout-dialog {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  z-index: 9999;
  display: flex;
  align-items: flex-start;
  justify-content: center;
  padding-top: 60px;
  background: rgba(0, 0, 0, 0.5);
}

.dialog-content {
  width: 480px;
  max-height: 80vh;
  background: var(--color-bg-primary, #1e1e1e);
  border: 1px solid var(--color-border, #3c3c3c);
  border-radius: 8px;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.4);
  overflow: hidden;
  display: flex;
  flex-direction: column;
}

.dialog-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 16px;
  border-bottom: 1px solid var(--color-border, #3c3c3c);
  background: var(--color-bg-secondary, #252526);
}

.dialog-header h3 {
  margin: 0;
  font-size: 13px;
  font-weight: 600;
  color: var(--color-text-primary, #cccccc);
}

.close-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  border: none;
  border-radius: 4px;
  background: transparent;
  color: var(--color-text-secondary, #808080);
  cursor: pointer;
  transition: all 0.15s;
}

.close-btn:hover {
  background: var(--color-bg-hover, #3c3c3c);
  color: var(--color-text-primary, #cccccc);
}

.dialog-body {
  flex: 1;
  overflow-y: auto;
  padding: 8px 0;
}

.section {
  padding: 4px 0;
}

.section-title {
  padding: 4px 16px 8px;
  font-size: 11px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  color: var(--color-text-tertiary, #606060);
}

.section-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 6px 16px;
  cursor: pointer;
  transition: background 0.1s;
}

.section-item:hover {
  background: var(--color-bg-hover, #2a2d2e);
}

.item-left {
  display: flex;
  align-items: center;
  gap: 10px;
}

.item-icon {
  color: var(--color-text-secondary, #808080);
}

.item-label {
  font-size: 13px;
  color: var(--color-text-primary, #cccccc);
}

.item-right {
  display: flex;
  align-items: center;
  gap: 12px;
}

.item-action {
  font-size: 12px;
  color: var(--color-text-secondary, #808080);
}

.item-hint {
  font-size: 11px;
  color: var(--color-text-tertiary, #606060);
}

.shortcut {
  display: flex;
  align-items: center;
  gap: 2px;
}

.shortcut kbd {
  padding: 2px 6px;
  font-size: 11px;
  font-family: var(--font-mono, monospace);
  color: var(--color-text-secondary, #808080);
  background: var(--color-bg-tertiary, #3c3c3c);
  border: 1px solid var(--color-border, #505050);
  border-radius: 3px;
}

.shortcut span {
  font-size: 10px;
  color: var(--color-text-tertiary, #606060);
}

.checkbox {
  width: 18px;
  height: 18px;
  border: 1px solid var(--color-border, #505050);
  border-radius: 3px;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.15s;
}

.checkbox.checked {
  background: var(--color-accent, #007acc);
  border-color: var(--color-accent, #007acc);
}

.checkbox svg {
  color: white;
}

.radio {
  width: 18px;
  height: 18px;
  border: 1px solid var(--color-border, #505050);
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.15s;
}

.radio.checked {
  border-color: var(--color-accent, #007acc);
}

.radio-dot {
  width: 10px;
  height: 10px;
  border-radius: 50%;
  background: var(--color-accent, #007acc);
}

.divider {
  height: 1px;
  background: var(--color-border, #3c3c3c);
  margin: 4px 16px;
}

.reset-item {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 8px 16px;
  cursor: pointer;
  transition: background 0.1s;
  color: var(--color-text-primary, #cccccc);
}

.reset-item:hover {
  background: var(--color-bg-hover, #2a2d2e);
}
</style>
