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
        <!-- Visibility Section -->
        <div class="section">
          <div class="section-item" @click="toggleVisibility('menuBar')">
            <div class="item-left">
              <Menu :size="16" class="item-icon" />
              <span class="item-label">Menu Bar</span>
            </div>
            <div class="item-right">
              <span class="item-action">Visibility</span>
              <div class="checkbox" :class="{ checked: visibility.menuBar }">
                <Check v-if="visibility.menuBar" :size="12" />
              </div>
            </div>
          </div>

          <div class="section-item" @click="toggleVisibility('activityBar')">
            <div class="item-left">
              <LayoutGrid :size="16" class="item-icon" />
              <span class="item-label">Activity Bar</span>
            </div>
            <div class="item-right">
              <div class="checkbox" :class="{ checked: visibility.activityBar }">
                <Check v-if="visibility.activityBar" :size="12" />
              </div>
            </div>
          </div>

          <div class="section-item" @click="toggleVisibility('primarySideBar')">
            <div class="item-left">
              <PanelLeft :size="16" class="item-icon" />
              <span class="item-label">Primary Side Bar</span>
            </div>
            <div class="item-right">
              <div class="shortcut">
                <kbd>Ctrl</kbd>
                <span>+</span>
                <kbd>B</kbd>
              </div>
              <div class="checkbox" :class="{ checked: visibility.primarySideBar }">
                <Check v-if="visibility.primarySideBar" :size="12" />
              </div>
            </div>
          </div>

          <div class="section-item" @click="toggleVisibility('secondarySideBar')">
            <div class="item-left">
              <PanelRight :size="16" class="item-icon" />
              <span class="item-label">Secondary Side Bar</span>
            </div>
            <div class="item-right">
              <div class="shortcut">
                <kbd>Ctrl</kbd>
                <span>+</span>
                <kbd>Alt</kbd>
                <span>+</span>
                <kbd>B</kbd>
              </div>
              <div class="checkbox" :class="{ checked: visibility.secondarySideBar }">
                <Check v-if="visibility.secondarySideBar" :size="12" />
              </div>
            </div>
          </div>

          <div class="section-item" @click="toggleVisibility('panel')">
            <div class="item-left">
              <PanelBottom :size="16" class="item-icon" />
              <span class="item-label">Panel</span>
            </div>
            <div class="item-right">
              <div class="shortcut">
                <kbd>Ctrl</kbd>
                <span>+</span>
                <kbd>J</kbd>
              </div>
              <div class="checkbox" :class="{ checked: visibility.panel }">
                <Check v-if="visibility.panel" :size="12" />
              </div>
            </div>
          </div>

          <div class="section-item" @click="toggleVisibility('statusBar')">
            <div class="item-left">
              <Minus :size="16" class="item-icon" />
              <span class="item-label">Status Bar</span>
            </div>
            <div class="item-right">
              <div class="checkbox" :class="{ checked: visibility.statusBar }">
                <Check v-if="visibility.statusBar" :size="12" />
              </div>
            </div>
          </div>
        </div>

        <div class="divider"></div>

        <!-- Primary Side Bar Position -->
        <div class="section">
          <div class="section-item" @click="setPrimarySideBarPosition('left')">
            <div class="item-left">
              <PanelLeft :size="16" class="item-icon" />
              <span class="item-label">Left</span>
            </div>
            <div class="item-right">
              <span class="item-hint">Primary Side Bar Position</span>
              <div class="radio" :class="{ checked: primarySideBarPosition === 'left' }">
                <div v-if="primarySideBarPosition === 'left'" class="radio-dot"></div>
              </div>
            </div>
          </div>

          <div class="section-item" @click="setPrimarySideBarPosition('right')">
            <div class="item-left">
              <PanelRight :size="16" class="item-icon" />
              <span class="item-label">Right</span>
            </div>
            <div class="item-right">
              <div class="radio" :class="{ checked: primarySideBarPosition === 'right' }">
                <div v-if="primarySideBarPosition === 'right'" class="radio-dot"></div>
              </div>
            </div>
          </div>
        </div>

        <div class="divider"></div>

        <!-- Panel Alignment -->
        <div class="section">
          <div class="section-item" @click="setPanelAlignment('left')">
            <div class="item-left">
              <AlignLeft :size="16" class="item-icon" />
              <span class="item-label">Left</span>
            </div>
            <div class="item-right">
              <span class="item-hint">Panel Alignment</span>
              <div class="radio" :class="{ checked: panelAlignment === 'left' }">
                <div v-if="panelAlignment === 'left'" class="radio-dot"></div>
              </div>
            </div>
          </div>

          <div class="section-item" @click="setPanelAlignment('right')">
            <div class="item-left">
              <AlignRight :size="16" class="item-icon" />
              <span class="item-label">Right</span>
            </div>
            <div class="item-right">
              <div class="radio" :class="{ checked: panelAlignment === 'right' }">
                <div v-if="panelAlignment === 'right'" class="radio-dot"></div>
              </div>
            </div>
          </div>

          <div class="section-item" @click="setPanelAlignment('center')">
            <div class="item-left">
              <AlignCenter :size="16" class="item-icon" />
              <span class="item-label">Center</span>
            </div>
            <div class="item-right">
              <div class="radio" :class="{ checked: panelAlignment === 'center' }">
                <div v-if="panelAlignment === 'center'" class="radio-dot"></div>
              </div>
            </div>
          </div>

          <div class="section-item" @click="setPanelAlignment('justify')">
            <div class="item-left">
              <AlignStartVertical :size="16" class="item-icon" />
              <span class="item-label">Justify</span>
            </div>
            <div class="item-right">
              <div class="radio" :class="{ checked: panelAlignment === 'justify' }">
                <div v-if="panelAlignment === 'justify'" class="radio-dot"></div>
              </div>
            </div>
          </div>
        </div>

        <div class="divider"></div>

        <!-- Quick Input Position -->
        <div class="section">
          <div class="section-item" @click="setQuickInputPosition('top')">
            <div class="item-left">
              <ArrowUp :size="16" class="item-icon" />
              <span class="item-label">Top</span>
            </div>
            <div class="item-right">
              <span class="item-hint">Quick Input Position</span>
              <div class="radio" :class="{ checked: quickInputPosition === 'top' }">
                <div v-if="quickInputPosition === 'top'" class="radio-dot"></div>
              </div>
            </div>
          </div>

          <div class="section-item" @click="setQuickInputPosition('center')">
            <div class="item-left">
              <Circle :size="16" class="item-icon" />
              <span class="item-label">Center</span>
            </div>
            <div class="item-right">
              <div class="radio" :class="{ checked: quickInputPosition === 'center' }">
                <div v-if="quickInputPosition === 'center'" class="radio-dot"></div>
              </div>
            </div>
          </div>
        </div>

        <div class="divider"></div>

        <!-- Full Screen -->
        <div class="section">
          <div class="section-item" @click="toggleFullScreen">
            <div class="item-left">
              <Maximize :size="16" class="item-icon" />
              <span class="item-label">Full Screen</span>
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
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import {
  X,
  Menu,
  LayoutGrid,
  PanelLeft,
  PanelRight,
  PanelBottom,
  Minus,
  Check,
  AlignLeft,
  AlignRight,
  AlignCenter,
  AlignStartVertical,
  ArrowUp,
  Circle,
  Maximize
} from 'lucide-vue-next'
import { ref, onMounted, onUnmounted } from 'vue'

import { useLayoutStore } from '../stores/layout-store'

const layoutStore = useLayoutStore()

const emit = defineEmits<{
  close: []
}>()

const visibility = ref({
  menuBar: layoutStore.menuBarVisible,
  activityBar: layoutStore.leftActivityBarVisible && layoutStore.rightActivityBarVisible,
  primarySideBar: layoutStore.primarySideBarVisible,
  secondarySideBar: layoutStore.secondarySideBarVisible,
  panel: layoutStore.panelVisible,
  statusBar: layoutStore.statusBarVisible
})

const primarySideBarPosition = ref<'left' | 'right'>('left')
const panelAlignment = ref<'left' | 'right' | 'center' | 'justify'>('center')
const quickInputPosition = ref<'top' | 'center'>('top')
const fullScreen = ref(false)

function handleClose() {
  emit('close')
}

function toggleVisibility(key: keyof typeof visibility.value) {
  visibility.value[key] = !visibility.value[key]

  switch (key) {
    case 'menuBar':
      layoutStore.toggleMenuBar()
      break
    case 'activityBar':
      layoutStore.toggleLeftActivityBar()
      layoutStore.toggleRightActivityBar()
      break
    case 'primarySideBar':
      layoutStore.togglePrimarySideBar()
      break
    case 'secondarySideBar':
      layoutStore.toggleSecondarySideBar()
      break
    case 'panel':
      layoutStore.togglePanel()
      break
    case 'statusBar':
      layoutStore.toggleStatusBar()
      break
  }
}

function setPrimarySideBarPosition(pos: 'left' | 'right') {
  primarySideBarPosition.value = pos
}

function setPanelAlignment(align: 'left' | 'right' | 'center' | 'justify') {
  panelAlignment.value = align
}

function setQuickInputPosition(pos: 'top' | 'center') {
  quickInputPosition.value = pos
}

function toggleFullScreen() {
  fullScreen.value = !fullScreen.value
  if (fullScreen.value) {
    document.documentElement.requestFullscreen?.()
  } else {
    document.exitFullscreen?.()
  }
}

function handleKeyDown(e: KeyboardEvent) {
  if (e.key === 'Escape') {
    handleClose()
  }
}

onMounted(() => {
  document.addEventListener('keydown', handleKeyDown)
})

onUnmounted(() => {
  document.removeEventListener('keydown', handleKeyDown)
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
</style>
