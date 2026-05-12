<template>
  <Teleport to="body">
    <Transition name="modal">
      <div v-if="visible" class="modal-overlay" @click.self="handleClose">
        <div class="modal-container">
          <header class="modal-header">
            <h2 class="header-title">{{ t('workbench.customizeLayout') }}</h2>
            <button class="btn-close" @click="handleClose">
              <X :size="18" />
            </button>
          </header>

          <div class="modal-body">
            <n-tabs type="line" animated>
              <n-tab-pane name="edgeGroups" :tab="t('workbench.customizeLayoutDialog.edgeGroups')">
                <div class="edge-groups-tab">
                  <div class="edge-group-card">
                    <div class="card-row">
                      <span class="card-label">{{ t('workbench.customizeLayoutDialog.leftEdgeGroup') }}</span>
                      <n-switch
                        :value="!layoutStore.leftEdgeGroupCollapsed"
                        size="small"
                        @update:value="handleLeftEdgeToggle"
                      />
                      <span class="switch-label">
                        {{ layoutStore.leftEdgeGroupCollapsed ? t('workbench.customizeLayoutDialog.hidden') : t('workbench.customizeLayoutDialog.visible') }}
                      </span>
                    </div>
                    <div class="card-row">
                      <span class="card-label width-label">{{ t('workbench.customizeLayoutDialog.width') }}</span>
                      <n-input-number
                        :value="layoutStore.primarySideBarWidth"
                        :min="MIN_WIDTH"
                        :max="MAX_WIDTH"
                        :step="10"
                        size="small"
                        style="width: 100px"
                        @update:value="handleLeftWidthChange"
                      />
                      <span class="unit-label">px</span>
                    </div>
                  </div>

                  <div class="edge-group-card">
                    <div class="card-row">
                      <span class="card-label">{{ t('workbench.customizeLayoutDialog.rightEdgeGroup') }}</span>
                      <n-switch
                        :value="!layoutStore.rightEdgeGroupCollapsed"
                        size="small"
                        @update:value="handleRightEdgeToggle"
                      />
                      <span class="switch-label">
                        {{ layoutStore.rightEdgeGroupCollapsed ? t('workbench.customizeLayoutDialog.hidden') : t('workbench.customizeLayoutDialog.visible') }}
                      </span>
                    </div>
                    <div class="card-row">
                      <span class="card-label width-label">{{ t('workbench.customizeLayoutDialog.width') }}</span>
                      <n-input-number
                        :value="layoutStore.secondarySideBarWidth"
                        :min="MIN_WIDTH"
                        :max="MAX_WIDTH"
                        :step="10"
                        size="small"
                        style="width: 100px"
                        @update:value="handleRightWidthChange"
                      />
                      <span class="unit-label">px</span>
                    </div>
                  </div>

                  <div class="chrome-section">
                    <span class="section-title">{{ t('workbench.customizeLayoutDialog.chrome') }}</span>
                    <div class="card-row">
                      <span class="card-label">{{ t('workbench.customizeLayoutDialog.menuBarVisible') }}</span>
                      <n-switch
                        :value="layoutStore.menuBarVisible"
                        size="small"
                        @update:value="handleMenuBarToggle"
                      />
                    </div>
                    <div class="card-row">
                      <span class="card-label">{{ t('workbench.customizeLayoutDialog.statusBarVisible') }}</span>
                      <n-switch
                        :value="layoutStore.statusBarVisible"
                        size="small"
                        @update:value="handleStatusBarToggle"
                      />
                    </div>
                  </div>
                </div>
              </n-tab-pane>

              <n-tab-pane name="presets" :tab="t('workbench.customizeLayoutDialog.presets')">
                <div class="presets-tab">
                  <n-radio-group
                    :value="selectedPreset"
                    name="layout-preset"
                    @update:value="selectedPreset = $event"
                  >
                    <div
                      v-for="preset in presetOptions"
                      :key="preset.key"
                      class="preset-option"
                      @click="selectedPreset = preset.key"
                    >
                      <n-radio :value="preset.key">
                        <span class="preset-name">{{ preset.label }}</span>
                      </n-radio>
                      <span class="preset-desc">{{ preset.desc }}</span>
                    </div>
                  </n-radio-group>
                  <n-button
                    type="primary"
                    size="small"
                    :disabled="!selectedPreset"
                    class="apply-preset-btn"
                    @click="handleApplyPreset"
                  >
                    {{ t('workbench.customizeLayoutDialog.apply') }}
                  </n-button>
                </div>
              </n-tab-pane>
            </n-tabs>
          </div>

          <footer class="modal-footer">
            <n-button size="small" @click="handleReset">
              {{ t('workbench.customizeLayoutDialog.resetDefault') }}
            </n-button>
            <n-button type="primary" size="small" @click="handleClose">
              {{ t('workbench.customizeLayoutDialog.done') }}
            </n-button>
          </footer>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { X } from 'lucide-vue-next'

import { useLayoutStore } from '@/extensions/builtin/workbench/ui/stores/layout-store'

const { t } = useI18n()
const layoutStore = useLayoutStore()

const MIN_WIDTH = 200
const MAX_WIDTH = 600

const visible = computed(() => layoutStore.showCustomizeLayoutDialog)

interface PresetOption {
  key: string
  label: string
  desc: string
}

const selectedPreset = ref<string | null>(null)

const presetOptions = computed<PresetOption[]>(() => [
  { key: 'default', label: '默认布局', desc: '标准三栏布局，A:B:C = 1:2:1' },
  { key: 'compact', label: '紧凑布局', desc: '收起两侧面板，保留主编辑区' },
  { key: 'analysis', label: '分析布局', desc: '展开右侧面板，适合数据分析场景' },
])

function handleLeftEdgeToggle(value: boolean) {
  if (value) {
    layoutStore.expandLeftEdgeGroup()
  } else {
    layoutStore.collapseLeftEdgeGroup()
  }
}

function handleRightEdgeToggle(value: boolean) {
  if (value) {
    layoutStore.expandRightEdgeGroup()
  } else {
    layoutStore.collapseRightEdgeGroup()
  }
}

function handleLeftWidthChange(value: number | null) {
  if (value !== null && value >= MIN_WIDTH && value <= MAX_WIDTH) {
    layoutStore.setPrimarySideBarWidth(value)
    const api = layoutStore.dockviewApi
    if (api) {
      const group = api.getEdgeGroup('left')
      if (group) {
        group.setSize({ width: value })
      }
    }
  }
}

function handleRightWidthChange(value: number | null) {
  if (value !== null && value >= MIN_WIDTH && value <= MAX_WIDTH) {
    layoutStore.setSecondarySideBarWidth(value)
    const api = layoutStore.dockviewApi
    if (api) {
      const group = api.getEdgeGroup('right')
      if (group) {
        group.setSize({ width: value })
      }
    }
  }
}

function handleMenuBarToggle(value: boolean) {
  if (value !== layoutStore.menuBarVisible) {
    layoutStore.toggleMenuBar()
  }
}

function handleStatusBarToggle(value: boolean) {
  if (value !== layoutStore.statusBarVisible) {
    layoutStore.toggleStatusBar()
  }
}

function handleApplyPreset() {
  if (!selectedPreset.value) return

  if (selectedPreset.value === 'compact') {
    layoutStore.collapseLeftEdgeGroup()
    layoutStore.collapseRightEdgeGroup()
  } else if (selectedPreset.value === 'analysis') {
    layoutStore.expandLeftEdgeGroup()
    layoutStore.expandRightEdgeGroup()
    const api = layoutStore.dockviewApi
    if (api) {
      const rightGroup = api.getEdgeGroup('right')
      if (rightGroup) {
        rightGroup.setSize({ width: 360 })
        layoutStore.setSecondarySideBarWidth(360)
      }
    }
  } else {
    layoutStore.expandLeftEdgeGroup()
    layoutStore.expandRightEdgeGroup()
    const api = layoutStore.dockviewApi
    if (api) {
      const leftGroup = api.getEdgeGroup('left')
      const rightGroup = api.getEdgeGroup('right')
      if (leftGroup) {
        leftGroup.setSize({ width: 300 })
        layoutStore.setPrimarySideBarWidth(300)
      }
      if (rightGroup) {
        rightGroup.setSize({ width: 300 })
        layoutStore.setSecondarySideBarWidth(300)
      }
    }
    layoutStore.menuBarVisible = true
    layoutStore.statusBarVisible = true
  }
}

function handleReset() {
  layoutStore.expandLeftEdgeGroup()
  layoutStore.expandRightEdgeGroup()
  const api = layoutStore.dockviewApi
  if (api) {
    const leftGroup = api.getEdgeGroup('left')
    const rightGroup = api.getEdgeGroup('right')
    if (leftGroup) {
      leftGroup.setSize({ width: 300 })
      layoutStore.setPrimarySideBarWidth(300)
    }
    if (rightGroup) {
      rightGroup.setSize({ width: 300 })
      layoutStore.setSecondarySideBarWidth(300)
    }
  }
  layoutStore.menuBarVisible = true
  layoutStore.statusBarVisible = true
  selectedPreset.value = null
}

function handleClose() {
  layoutStore.closeCustomizeLayoutDialog()
}

function handleKeydown(e: KeyboardEvent) {
  if (e.key === 'Escape' && visible.value) {
    handleClose()
  }
}

onMounted(() => {
  window.addEventListener('keydown', handleKeydown)
})

onUnmounted(() => {
  window.removeEventListener('keydown', handleKeydown)
})
</script>

<style scoped>
.modal-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.6);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 10000;
  backdrop-filter: blur(4px);
}

.modal-container {
  width: 520px;
  max-width: 90vw;
  background: var(--color-bg-primary, var(--bg-primary));
  border-radius: var(--border-radius-md, 6px);
  box-shadow: 0 20px 60px rgba(0, 0, 0, 0.3);
  display: flex;
  flex-direction: column;
  overflow: hidden;
  font-family: var(--font-sans);
}

.modal-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 14px 20px;
  border-bottom: 1px solid var(--color-border, var(--border-color));
  background: var(--color-bg-secondary, var(--bg-secondary));
}

.header-title {
  font-size: 16px;
  font-weight: 600;
  color: var(--color-text-primary, var(--text-primary));
  margin: 0;
}

.btn-close {
  width: 28px;
  height: 28px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: transparent;
  border: none;
  border-radius: var(--border-radius-sm, 4px);
  color: var(--color-text-secondary, var(--text-secondary));
  cursor: pointer;
  transition: all 0.2s;
}

.btn-close:hover {
  background: var(--color-hover, var(--bg-hover));
  color: var(--color-text-primary, var(--text-primary));
}

.modal-body {
  flex: 1;
  overflow-y: auto;
  padding: 16px 20px;
  background: var(--color-bg-primary, var(--bg-primary));
}

.edge-groups-tab {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.edge-group-card {
  display: flex;
  flex-direction: column;
  gap: 10px;
  padding: 14px;
  background: var(--color-bg-secondary, var(--bg-secondary));
  border-radius: var(--border-radius-sm, 4px);
  border: 1px solid var(--color-border-subtle, var(--border-color));
}

.card-row {
  display: flex;
  align-items: center;
  gap: 8px;
}

.card-label {
  font-size: 14px;
  color: var(--color-text-primary, var(--text-primary));
  min-width: 90px;
}

.width-label {
  min-width: 36px;
}

.switch-label {
  font-size: 12px;
  color: var(--color-text-muted, var(--text-muted));
}

.unit-label {
  font-size: 12px;
  color: var(--color-text-muted, var(--text-muted));
}

.chrome-section {
  display: flex;
  flex-direction: column;
  gap: 10px;
  padding: 14px;
  background: var(--color-bg-secondary, var(--bg-secondary));
  border-radius: var(--border-radius-sm, 4px);
  border: 1px solid var(--color-border-subtle, var(--border-color));
}

.section-title {
  font-size: 14px;
  font-weight: 600;
  color: var(--color-text-primary, var(--text-primary));
  margin-bottom: 4px;
}

.presets-tab {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.preset-option {
  display: flex;
  flex-direction: column;
  gap: 4px;
  padding: 6px 0;
}

.preset-name {
  font-size: 14px;
  color: var(--color-text-primary, var(--text-primary));
}

.preset-desc {
  font-size: 12px;
  color: var(--color-text-muted, var(--text-muted));
  padding-left: 24px;
}

.apply-preset-btn {
  align-self: flex-start;
  margin-top: 8px;
}

.modal-footer {
  display: flex;
  align-items: center;
  justify-content: flex-end;
  gap: 8px;
  padding: 12px 20px;
  border-top: 1px solid var(--color-border, var(--border-color));
  background: var(--color-bg-secondary, var(--bg-secondary));
}

.modal-enter-active,
.modal-leave-active {
  transition: opacity 0.25s ease;
}

.modal-enter-from,
.modal-leave-to {
  opacity: 0;
}
</style>
