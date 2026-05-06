<template>
  <div class="settings-panel">
    <div class="settings-header">
      <Settings :size="20" class="settings-icon" />
      <h2>设置</h2>
    </div>

    <div class="settings-content">
      <div class="settings-section">
        <h3>
          <Layout :size="16" />
          布局设置
        </h3>

        <div class="setting-item">
          <div class="setting-label">
            <span class="label-text">布局比例</span>
            <span class="label-hint">调整三栏布局的宽度比例</span>
          </div>
          <div class="ratio-display">
            <span class="ratio-value">左侧: {{ leftRatio }}%</span>
            <span class="ratio-separator">:</span>
            <span class="ratio-value">中间: {{ centerRatio }}%</span>
            <span class="ratio-separator">:</span>
            <span class="ratio-value">右侧: {{ rightRatio }}%</span>
          </div>
        </div>

        <div class="setting-item">
          <div class="setting-label">
            <span class="label-text">左侧宽度</span>
            <span class="label-value">{{ leftWidth }}px</span>
          </div>
          <input
            v-model.number="leftWidth"
            type="range"
            min="150"
            max="600"
            step="10"
            class="slider"
            @input="handleWidthChange"
          />
          <div class="slider-labels">
            <span>150px</span>
            <span>600px</span>
          </div>
        </div>

        <div class="setting-item">
          <div class="setting-label">
            <span class="label-text">右侧宽度</span>
            <span class="label-value">{{ rightWidth }}px</span>
          </div>
          <input
            v-model.number="rightWidth"
            type="range"
            min="150"
            max="600"
            step="10"
            class="slider"
            @input="handleWidthChange"
          />
          <div class="slider-labels">
            <span>150px</span>
            <span>600px</span>
          </div>
        </div>

        <div class="setting-item">
          <div class="setting-label">
            <span class="label-text">最小面板宽度</span>
          </div>
          <div class="min-width-buttons">
            <button
              v-for="preset in minWidthPresets"
              :key="preset.value"
              :class="['preset-btn', { active: minimumPanelWidth === preset.value }]"
              @click="setMinimumWidth(preset.value)"
            >
              {{ preset.label }}
            </button>
          </div>
        </div>

        <div class="setting-item">
          <div class="setting-label">
            <span class="label-text">左侧面板布局</span>
            <span class="label-hint">标签页或垂直分割</span>
          </div>
          <div class="layout-mode-buttons">
            <button
              v-for="mode in layoutModeOptions"
              :key="mode.value"
              :class="['mode-btn', { active: leftPanelLayoutMode === mode.value }]"
              @click="leftPanelLayoutMode = mode.value as 'tabs' | 'vertical-split'"
            >
              <component :is="mode.icon" :size="14" />
              {{ mode.label }}
            </button>
          </div>
        </div>

        <div class="setting-item">
          <div class="setting-label">
            <span class="label-text">右侧面板布局</span>
            <span class="label-hint">标签页或垂直分割</span>
          </div>
          <div class="layout-mode-buttons">
            <button
              v-for="mode in layoutModeOptions"
              :key="mode.value"
              :class="['mode-btn', { active: rightPanelLayoutMode === mode.value }]"
              @click="rightPanelLayoutMode = mode.value as 'tabs' | 'vertical-split'"
            >
              <component :is="mode.icon" :size="14" />
              {{ mode.label }}
            </button>
          </div>
        </div>
      </div>

      <div class="settings-section">
        <h3>
          <Grid3X3 :size="16" />
          布局预设
        </h3>

        <div class="preset-grid">
          <div
            v-for="layout in layoutPresets"
            :key="layout.id"
            :class="['preset-card', { active: currentPreset === layout.id }]"
            @click="applyPreset(layout)"
          >
            <div class="preset-preview">
              <div
                class="preset-bar left"
                :style="{ width: layout.leftWidth + '%' }"
              ></div>
              <div
                class="preset-bar center"
                :style="{ width: layout.centerWidth + '%' }"
              ></div>
              <div
                class="preset-bar right"
                :style="{ width: layout.rightWidth + '%' }"
              ></div>
            </div>
            <span class="preset-name">{{ layout.name }}</span>
            <span class="preset-ratio">{{ layout.leftWidth }}:{{ layout.centerWidth }}:{{ layout.rightWidth }}</span>
          </div>
        </div>
      </div>

      <div class="settings-section">
        <h3>
          <Maximize2 :size="16" />
          拖拽约束
        </h3>

        <div class="setting-item">
          <div class="setting-label">
            <span class="label-text">最大面板宽度</span>
            <span class="label-value">{{ maximumPanelWidth }}px</span>
          </div>
          <input
            v-model.number="maximumPanelWidth"
            type="range"
            min="300"
            max="1000"
            step="50"
            class="slider"
            @input="handleConstraintChange"
          />
          <div class="slider-labels">
            <span>300px</span>
            <span>1000px</span>
          </div>
        </div>

        <div class="setting-item">
          <div class="setting-label">
            <span class="label-text">启用最大宽度约束</span>
          </div>
          <label class="switch">
            <input
              v-model="enableMaxWidthConstraint"
              type="checkbox"
              @change="handleConstraintChange"
            />
            <span class="slider-switch"></span>
          </label>
        </div>
      </div>

      <div class="settings-section">
        <h3>
          <RefreshCw :size="16" />
          布局操作
        </h3>

        <div class="action-buttons">
          <button class="action-btn primary" @click="resetToDefault">
            <RotateCcw :size="16" />
            恢复默认布局
          </button>
          <button class="action-btn" @click="saveSettings">
            <Save :size="16" />
            保存当前布局
          </button>
          <button class="action-btn" @click="clearSavedLayout">
            <Trash2 :size="16" />
            清除保存的布局
          </button>
        </div>
      </div>

      <div class="settings-section">
        <h3>
          <Monitor :size="16" />
          响应式设置
        </h3>

        <div class="setting-item">
          <div class="setting-label">
            <span class="label-text">窗口调整时自动重算</span>
            <span class="label-hint">窗口大小改变时自动调整布局</span>
          </div>
          <label class="switch">
            <input
              v-model="autoResize"
              type="checkbox"
              @change="handleAutoResizeChange"
            />
            <span class="slider-switch"></span>
          </label>
        </div>

        <div class="setting-item">
          <div class="setting-label">
            <span class="label-text">防抖延迟</span>
            <span class="label-value">{{ debounceDelay }}ms</span>
          </div>
          <input
            v-model.number="debounceDelay"
            type="range"
            min="100"
            max="500"
            step="50"
            class="slider"
            @input="handleDebounceChange"
          />
          <div class="slider-labels">
            <span>100ms</span>
            <span>500ms</span>
          </div>
        </div>
      </div>

      <div class="settings-section info-section">
        <h3>
          <Info :size="16" />
          当前状态
        </h3>

        <div class="info-grid">
          <div class="info-item">
            <span class="info-label">容器宽度</span>
            <span class="info-value">{{ containerWidth }}px</span>
          </div>
          <div class="info-item">
            <span class="info-label">实际比例</span>
            <span class="info-value">{{ leftRatio }}:{{ centerRatio }}:{{ rightRatio }}</span>
          </div>
          <div class="info-item">
            <span class="info-label">保存状态</span>
            <span class="info-value">{{ isSaved ? '已保存' : '未保存' }}</span>
          </div>
          <div class="info-item">
            <span class="info-label">最后保存</span>
            <span class="info-value">{{ lastSavedTime || '无' }}</span>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import {
  Settings,
  Layout,
  Grid3X3,
  Maximize2,
  RefreshCw,
  RotateCcw,
  Save,
  Trash2,
  Monitor,
  Info,
  Layers,
  Columns
} from 'lucide-vue-next'
import { ref, computed, onMounted, onUnmounted, watch } from 'vue'

import { useLayoutStore } from '../../../workbench/ui/stores/layout-store'

const layoutStore = useLayoutStore()

const LAYOUT_STORAGE_KEY = 'rdata_station_layout_config'
const SETTINGS_STORAGE_KEY = 'rdata_station_layout_settings'

interface LayoutSettings {
  leftWidth: number
  rightWidth: number
  minimumPanelWidth: number
  maximumPanelWidth: number
  enableMaxWidthConstraint: boolean
  autoResize: boolean
  debounceDelay: number
  leftPanelLayoutMode: 'tabs' | 'vertical-split'
  rightPanelLayoutMode: 'tabs' | 'vertical-split'
  timestamp: number
}

const leftWidth = ref(300)
const rightWidth = ref(300)
const minimumPanelWidth = ref(200)
const maximumPanelWidth = ref(600)
const enableMaxWidthConstraint = ref(true)
const autoResize = ref(true)
const debounceDelay = ref(200)
const containerWidth = ref(window.innerWidth - 96)
const isSaved = ref(false)
const lastSavedTime = ref<string | null>(null)

const layoutModeOptions = [
  { value: 'tabs', label: '标签页', icon: Columns },
  { value: 'vertical-split', label: '垂直分割', icon: Layers }
]

const leftPanelLayoutMode = computed({
  get: () => layoutStore.leftPanelLayoutMode,
  set: (value) => {
    layoutStore.setLeftPanelLayoutMode(value)
    handleLayoutModeChange()
  }
})

const rightPanelLayoutMode = computed({
  get: () => layoutStore.rightPanelLayoutMode,
  set: (value) => {
    layoutStore.setRightPanelLayoutMode(value)
    handleLayoutModeChange()
  }
})

const layoutPresets = [
  { id: 'balanced', name: '平衡', leftWidth: 25, centerWidth: 50, rightWidth: 25 },
  { id: 'focus-left', name: '左侧为主', leftWidth: 35, centerWidth: 40, rightWidth: 25 },
  { id: 'focus-right', name: '右侧为主', leftWidth: 25, centerWidth: 40, rightWidth: 35 },
  { id: 'equal', name: '均分', leftWidth: 33, centerWidth: 34, rightWidth: 33 },
  { id: 'narrow-sides', name: '窄边距', leftWidth: 15, centerWidth: 70, rightWidth: 15 }
]

const minWidthPresets = [
  { label: '紧凑 (150px)', value: 150 },
  { label: '标准 (200px)', value: 200 },
  { label: '宽松 (300px)', value: 300 }
]

const currentPreset = computed(() => {
  const totalWidth = leftWidth.value + containerWidth.value - leftWidth.value - rightWidth.value + rightWidth.value
  const leftPercent = Math.round((leftWidth.value / containerWidth.value) * 100)
  const rightPercent = Math.round((rightWidth.value / containerWidth.value) * 100)
  const centerPercent = 100 - leftPercent - rightPercent

  const matched = layoutPresets.find(
    p => Math.abs(p.leftWidth - leftPercent) <= 5 &&
         Math.abs(p.centerWidth - centerPercent) <= 5 &&
         Math.abs(p.rightWidth - rightPercent) <= 5
  )
  return matched?.id || 'custom'
})

const leftRatio = computed(() => Math.round((leftWidth.value / containerWidth.value) * 100))
const centerRatio = computed(() => Math.round(((containerWidth.value - leftWidth.value - rightWidth.value) / containerWidth.value) * 100))
const rightRatio = computed(() => Math.round((rightWidth.value / containerWidth.value) * 100))

function updateContainerWidth() {
  containerWidth.value = window.innerWidth - 96
}

function loadSettings() {
  try {
    const stored = localStorage.getItem(SETTINGS_STORAGE_KEY)
    if (stored) {
      const settings: LayoutSettings = JSON.parse(stored)
      const age = Date.now() - settings.timestamp
      const maxAge = 30 * 24 * 60 * 60 * 1000

      if (age < maxAge) {
        leftWidth.value = settings.leftWidth
        rightWidth.value = settings.rightWidth
        minimumPanelWidth.value = settings.minimumPanelWidth
        maximumPanelWidth.value = settings.maximumPanelWidth
        enableMaxWidthConstraint.value = settings.enableMaxWidthConstraint
        autoResize.value = settings.autoResize
        debounceDelay.value = settings.debounceDelay
        if (settings.leftPanelLayoutMode) {
          layoutStore.setLeftPanelLayoutMode(settings.leftPanelLayoutMode)
        }
        if (settings.rightPanelLayoutMode) {
          layoutStore.setRightPanelLayoutMode(settings.rightPanelLayoutMode)
        }
        isSaved.value = true
        lastSavedTime.value = new Date(settings.timestamp).toLocaleString('zh-CN')
      }
    }

    const layoutStored = localStorage.getItem(LAYOUT_STORAGE_KEY)
    if (layoutStored) {
      const layout = JSON.parse(layoutStored)
      lastSavedTime.value = new Date(layout.timestamp).toLocaleString('zh-CN')
    }
  } catch (error) {
    console.error('[Settings] Failed to load settings:', error)
  }
}

function saveSettings() {
  try {
    const settings: LayoutSettings = {
      leftWidth: leftWidth.value,
      rightWidth: rightWidth.value,
      minimumPanelWidth: minimumPanelWidth.value,
      maximumPanelWidth: maximumPanelWidth.value,
      enableMaxWidthConstraint: enableMaxWidthConstraint.value,
      autoResize: autoResize.value,
      debounceDelay: debounceDelay.value,
      leftPanelLayoutMode: layoutStore.leftPanelLayoutMode,
      rightPanelLayoutMode: layoutStore.rightPanelLayoutMode,
      timestamp: Date.now()
    }
    localStorage.setItem(SETTINGS_STORAGE_KEY, JSON.stringify(settings))
    localStorage.setItem(LAYOUT_STORAGE_KEY, JSON.stringify({
      leftWidth: leftWidth.value,
      rightWidth: rightWidth.value,
      centerWidth: containerWidth.value - leftWidth.value - rightWidth.value,
      timestamp: Date.now()
    }))

    isSaved.value = true
    lastSavedTime.value = new Date().toLocaleString('zh-CN')
    console.log('[Settings] Layout saved:', settings)
  } catch (error) {
    console.error('[Settings] Failed to save settings:', error)
  }
}

function handleLayoutModeChange() {
  dispatchLayoutUpdate()
  saveSettings()
}

function handleWidthChange() {
  const maxLeft = containerWidth.value - rightWidth.value - 400
  if (leftWidth.value > maxLeft) {
    leftWidth.value = maxLeft
  }

  const maxRight = containerWidth.value - leftWidth.value - 400
  if (rightWidth.value > maxRight) {
    rightWidth.value = maxRight
  }

  dispatchLayoutUpdate()
}

function handleConstraintChange() {
  dispatchLayoutUpdate()
}

function handleAutoResizeChange() {
  dispatchLayoutUpdate()
}

function handleDebounceChange() {
  dispatchLayoutUpdate()
}

function setMinimumWidth(value: number) {
  minimumPanelWidth.value = value
  dispatchLayoutUpdate()
}

function dispatchLayoutUpdate() {
  window.dispatchEvent(new CustomEvent('workbench-layout-settings-update', {
    detail: {
      leftWidth: leftWidth.value,
      rightWidth: rightWidth.value,
      minimumWidth: minimumPanelWidth.value,
      maximumWidth: enableMaxWidthConstraint.value ? maximumPanelWidth.value : undefined,
      autoResize: autoResize.value,
      debounceDelay: debounceDelay.value
    }
  }))
}

function applyPreset(layout: typeof layoutPresets[0]) {
  const totalWidth = containerWidth.value
  leftWidth.value = Math.round(totalWidth * (layout.leftWidth / 100))
  rightWidth.value = Math.round(totalWidth * (layout.rightWidth / 100))
  dispatchLayoutUpdate()
}

function resetToDefault() {
  leftWidth.value = Math.round(containerWidth.value * 0.25)
  rightWidth.value = Math.round(containerWidth.value * 0.25)
  minimumPanelWidth.value = 200
  maximumPanelWidth.value = 600
  enableMaxWidthConstraint.value = true
  autoResize.value = true
  debounceDelay.value = 200

  localStorage.removeItem(LAYOUT_STORAGE_KEY)
  localStorage.removeItem(SETTINGS_STORAGE_KEY)
  isSaved.value = false
  lastSavedTime.value = null

  window.dispatchEvent(new CustomEvent('reset-workbench-layout'))
  console.log('[Settings] Layout reset to default')
}

function clearSavedLayout() {
  localStorage.removeItem(LAYOUT_STORAGE_KEY)
  localStorage.removeItem(SETTINGS_STORAGE_KEY)
  isSaved.value = false
  lastSavedTime.value = null
  console.log('[Settings] Saved layout cleared')
}

onMounted(() => {
  loadSettings()
  updateContainerWidth()
  window.addEventListener('resize', updateContainerWidth)
})

onUnmounted(() => {
  window.removeEventListener('resize', updateContainerWidth)
})

watch(
  [leftWidth, rightWidth, minimumPanelWidth, maximumPanelWidth, enableMaxWidthConstraint, autoResize, debounceDelay, leftPanelLayoutMode, rightPanelLayoutMode],
  () => {
    if (autoResize.value) {
      saveSettings()
    }
  },
  { deep: true }
)
</script>

<style scoped>
.settings-panel {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: var(--color-bg-primary, #1e1e1e);
  color: var(--color-text-primary, #cccccc);
}

.settings-header {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 16px 20px;
  border-bottom: 1px solid var(--color-border, #3c3c3c);
  background: var(--color-bg-secondary, #252526);
}

.settings-header h2 {
  margin: 0;
  font-size: 16px;
  font-weight: 600;
}

.settings-icon {
  color: var(--color-accent, #007acc);
}

.settings-content {
  flex: 1;
  overflow-y: auto;
  padding: 16px 20px;
}

.settings-section {
  margin-bottom: 24px;
}

.settings-section h3 {
  display: flex;
  align-items: center;
  gap: 8px;
  margin: 0 0 16px 0;
  font-size: 14px;
  font-weight: 600;
  color: var(--color-text-secondary, #9d9d9d);
}

.setting-item {
  margin-bottom: 20px;
}

.setting-label {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 8px;
}

.label-text {
  font-size: 13px;
  color: var(--color-text-primary, #cccccc);
}

.label-hint {
  font-size: 11px;
  color: var(--color-text-secondary, #808080);
  margin-left: 8px;
}

.label-value {
  font-size: 12px;
  color: var(--color-accent, #007acc);
  font-weight: 500;
}

.ratio-display {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 12px;
  background: var(--color-bg-secondary, #2d2d2d);
  border-radius: 4px;
  font-size: 12px;
}

.ratio-value {
  color: var(--color-accent, #007acc);
  font-weight: 500;
}

.ratio-separator {
  color: var(--color-text-secondary, #808080);
}

.slider {
  width: 100%;
  height: 4px;
  border-radius: 2px;
  background: var(--color-bg-secondary, #3c3c3c);
  outline: none;
  -webkit-appearance: none;
}

.slider::-webkit-slider-thumb {
  -webkit-appearance: none;
  appearance: none;
  width: 14px;
  height: 14px;
  border-radius: 50%;
  background: var(--color-accent, #007acc);
  cursor: pointer;
}

.slider-labels {
  display: flex;
  justify-content: space-between;
  margin-top: 4px;
  font-size: 10px;
  color: var(--color-text-secondary, #808080);
}

.min-width-buttons {
  display: flex;
  gap: 8px;
}

.layout-mode-buttons {
  display: flex;
  gap: 8px;
}

.preset-btn {
  padding: 6px 12px;
  border: 1px solid var(--color-border, #3c3c3c);
  border-radius: 4px;
  background: var(--color-bg-secondary, #2d2d2d);
  color: var(--color-text-primary, #cccccc);
  font-size: 12px;
  cursor: pointer;
  transition: all 0.2s;
}

.mode-btn {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 8px 14px;
  border: 1px solid var(--color-border, #3c3c3c);
  border-radius: 6px;
  background: var(--color-bg-secondary, #2d2d2d);
  color: var(--color-text-primary, #cccccc);
  font-size: 12px;
  cursor: pointer;
  transition: all 0.2s;
}

.preset-btn:hover,
.mode-btn:hover {
  background: var(--color-bg-hover, #3c3c3c);
}

.preset-btn.active,
.mode-btn.active {
  background: var(--color-accent, #007acc);
  border-color: var(--color-accent, #007acc);
  color: white;
}

.preset-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(140px, 1fr));
  gap: 12px;
}

.preset-card {
  display: flex;
  flex-direction: column;
  gap: 8px;
  padding: 12px;
  border: 1px solid var(--color-border, #3c3c3c);
  border-radius: 6px;
  background: var(--color-bg-secondary, #2d2d2d);
  cursor: pointer;
  transition: all 0.2s;
}

.preset-card:hover {
  border-color: var(--color-accent, #007acc);
  transform: translateY(-2px);
}

.preset-card.active {
  border-color: var(--color-accent, #007acc);
  background: rgba(0, 122, 204, 0.1);
}

.preset-preview {
  display: flex;
  height: 24px;
  border-radius: 4px;
  overflow: hidden;
  background: var(--color-bg-primary, #1e1e1e);
}

.preset-bar {
  height: 100%;
  transition: width 0.2s;
}

.preset-bar.left {
  background: var(--color-accent, #007acc);
}

.preset-bar.center {
  background: var(--color-bg-hover, #3c3c3c);
}

.preset-bar.right {
  background: var(--color-accent-secondary, #107c10);
}

.preset-name {
  font-size: 12px;
  font-weight: 500;
  color: var(--color-text-primary, #cccccc);
}

.preset-ratio {
  font-size: 10px;
  color: var(--color-text-secondary, #808080);
}

.switch {
  position: relative;
  display: inline-block;
  width: 36px;
  height: 20px;
}

.switch input {
  opacity: 0;
  width: 0;
  height: 0;
}

.slider-switch {
  position: absolute;
  cursor: pointer;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background-color: var(--color-bg-secondary, #3c3c3c);
  transition: 0.3s;
  border-radius: 20px;
}

.slider-switch:before {
  position: absolute;
  content: "";
  height: 14px;
  width: 14px;
  left: 3px;
  bottom: 3px;
  background-color: white;
  transition: 0.3s;
  border-radius: 50%;
}

.switch input:checked + .slider-switch {
  background-color: var(--color-accent, #007acc);
}

.switch input:checked + .slider-switch:before {
  transform: translateX(16px);
}

.action-buttons {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.action-btn {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 10px 16px;
  border: 1px solid var(--color-border, #3c3c3c);
  border-radius: 6px;
  background: var(--color-bg-secondary, #2d2d2d);
  color: var(--color-text-primary, #cccccc);
  font-size: 13px;
  cursor: pointer;
  transition: all 0.2s;
}

.action-btn:hover {
  background: var(--color-bg-hover, #3c3c3c);
}

.action-btn.primary {
  background: var(--color-accent, #007acc);
  border-color: var(--color-accent, #007acc);
  color: white;
}

.action-btn.primary:hover {
  background: #005a9e;
}

.info-grid {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 12px;
}

.info-item {
  display: flex;
  flex-direction: column;
  gap: 4px;
  padding: 10px 12px;
  background: var(--color-bg-secondary, #2d2d2d);
  border-radius: 6px;
}

.info-label {
  font-size: 11px;
  color: var(--color-text-secondary, #808080);
}

.info-value {
  font-size: 13px;
  font-weight: 500;
  color: var(--color-accent, #007acc);
}

.info-section {
  margin-top: 32px;
  padding-top: 24px;
  border-top: 1px solid var(--color-border, #3c3c3c);
}
</style>