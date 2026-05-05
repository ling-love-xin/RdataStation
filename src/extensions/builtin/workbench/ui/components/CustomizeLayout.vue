<template>
  <div class="customize-layout">
    <div class="layout-header">
      <h3>Customize Layout</h3>
    </div>

    <div class="layout-section">
      <h4>显示/隐藏</h4>

      <div class="layout-option">
        <label class="checkbox-label">
          <input
            type="checkbox"
            :checked="layoutStore.menuBarVisible"
            @change="layoutStore.toggleMenuBar"
          />
          <span>Menu Bar</span>
        </label>
      </div>

      <div class="layout-option">
        <label class="checkbox-label">
          <input
            type="checkbox"
            :checked="layoutStore.leftActivityBarVisible"
            @change="layoutStore.toggleLeftActivityBar"
          />
          <span>Left Activity Bar</span>
        </label>
      </div>

      <div class="layout-option">
        <label class="checkbox-label">
          <input
            type="checkbox"
            :checked="layoutStore.rightActivityBarVisible"
            @change="layoutStore.toggleRightActivityBar"
          />
          <span>Right Activity Bar</span>
        </label>
      </div>

      <div class="layout-option">
        <label class="checkbox-label">
          <input
            type="checkbox"
            :checked="layoutStore.primarySideBarVisible"
            @change="layoutStore.togglePrimarySideBar"
          />
          <span>Primary Side Bar</span>
        </label>
      </div>

      <div class="layout-option">
        <label class="checkbox-label">
          <input
            type="checkbox"
            :checked="layoutStore.secondarySideBarVisible"
            @change="layoutStore.toggleSecondarySideBar"
          />
          <span>Secondary Side Bar</span>
        </label>
      </div>

      <div class="layout-option">
        <label class="checkbox-label">
          <input
            type="checkbox"
            :checked="layoutStore.panelVisible"
            @change="layoutStore.togglePanel"
          />
          <span>Panel</span>
        </label>
      </div>

      <div class="layout-option">
        <label class="checkbox-label">
          <input
            type="checkbox"
            :checked="layoutStore.statusBarVisible"
            @change="layoutStore.toggleStatusBar"
          />
          <span>Status Bar</span>
        </label>
      </div>
    </div>

    <div class="layout-section">
      <h4>尺寸设置</h4>

      <div class="size-option">
        <label>Primary Side Bar Width: {{ layoutStore.primarySideBarWidth }}px</label>
        <input
          type="range"
          :min="200"
          :max="600"
          :value="layoutStore.primarySideBarWidth"
          @input="(e) => layoutStore.setPrimarySideBarWidth(parseInt((e.target as HTMLInputElement).value))"
        />
      </div>

      <div class="size-option">
        <label>Secondary Side Bar Width: {{ layoutStore.secondarySideBarWidth }}px</label>
        <input
          type="range"
          :min="200"
          :max="600"
          :value="layoutStore.secondarySideBarWidth"
          @input="(e) => layoutStore.setSecondarySideBarWidth(parseInt((e.target as HTMLInputElement).value))"
        />
      </div>

      <div class="size-option">
        <label>Panel Height: {{ layoutStore.panelHeight }}px</label>
        <input
          type="range"
          :min="100"
          :max="600"
          :value="layoutStore.panelHeight"
          @input="(e) => layoutStore.setPanelHeight(parseInt((e.target as HTMLInputElement).value))"
        />
      </div>
    </div>

    <div class="layout-section">
      <h4>面板管理</h4>

      <div class="panel-list">
        <div v-if="allPanels.length === 0" class="no-panels">
          暂无面板
        </div>

        <div
          v-for="panel in allPanels"
          :key="panel.id"
          class="panel-item"
        >
          <div class="panel-info">
            <span class="panel-title">{{ panel.title }}</span>
            <span class="panel-location">{{ getLocationName(panel) }}</span>
          </div>
          <div class="panel-actions">
            <select
              class="location-select"
              :value="getPanelLocation(panel.id)"
              @change="(e) => handleMovePanel(panel.id, (e.target as HTMLSelectElement).value as any)"
            >
              <option value="center">Center</option>
              <option value="left">Left</option>
              <option value="right">Right</option>
              <option value="bottom">Bottom</option>
              <option value="floating">Floating</option>
            </select>
            <button
              class="action-btn floating-btn"
              @click="handleCreateFloating(panel.id)"
              title="创建浮动窗口"
            >
              <ExternalLink :size="14" />
            </button>
            <button
              class="action-btn close-btn"
              @click="handleClosePanel(panel.id)"
              title="关闭面板"
            >
              <X :size="14" />
            </button>
          </div>
        </div>
      </div>
    </div>

    <div class="layout-section">
      <h4>浮动窗口</h4>

      <div class="floating-list">
        <div v-if="floatingPanels.length === 0" class="no-panels">
          暂无浮动窗口
        </div>

        <div
          v-for="panel in floatingPanels"
          :key="panel.id"
          class="panel-item floating"
        >
          <div class="panel-info">
            <span class="panel-title">{{ panel.title }}</span>
            <span class="panel-location">Floating</span>
          </div>
          <div class="panel-actions">
            <button
              class="action-btn close-btn"
              @click="handleCloseFloating(panel.id)"
              title="关闭浮动窗口"
            >
              <X :size="14" />
            </button>
          </div>
        </div>
      </div>
    </div>

    <div class="layout-actions">
      <button class="reset-btn" @click="handleReset">
        重置布局
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { ExternalLink, X } from 'lucide-vue-next'
import { useLayoutStore, type PanelLocation } from '../stores/layout-store'
import type { IDockviewPanel } from '@/core/dockview-types'

const layoutStore = useLayoutStore()

const allPanels = computed(() => layoutStore.getAllPanels())
const floatingPanels = computed(() => layoutStore.getFloatingPanels())

function getLocationName(panel: IDockviewPanel): string {
  const location = layoutStore.getPanelConfig(panel.id)?.location || 'center'
  const locationNames: Record<PanelLocation, string> = {
    left: 'Left',
    right: 'Right',
    center: 'Center',
    bottom: 'Bottom',
    floating: 'Floating'
  }
  return locationNames[location] || 'Unknown'
}

function getPanelLocation(panelId: string): PanelLocation {
  return layoutStore.getPanelConfig(panelId)?.location || 'center'
}

function handleMovePanel(panelId: string, location: PanelLocation) {
  layoutStore.movePanelToLocation(panelId, location)
}

function handleCreateFloating(panelId: string) {
  layoutStore.createFloatingPanel(panelId)
}

function handleClosePanel(panelId: string) {
  const panel = layoutStore.dockviewApi?.getPanel(panelId)
  if (panel) {
    panel.api.close()
  }
}

function handleCloseFloating(panelId: string) {
  layoutStore.closeFloatingPanel(panelId)
}

function handleReset() {
  layoutStore.resetLayout()
}
</script>

<style scoped>
.customize-layout {
  padding: 16px;
  height: 100%;
  overflow-y: auto;
}

.layout-header {
  margin-bottom: 24px;
  padding-bottom: 12px;
  border-bottom: 1px solid var(--color-border);
}

.layout-header h3 {
  margin: 0;
  font-size: 14px;
  font-weight: 600;
  color: var(--color-text-primary);
}

.layout-section {
  margin-bottom: 24px;
}

.layout-section h4 {
  margin: 0 0 12px 0;
  font-size: 12px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  color: var(--color-text-secondary);
}

.layout-option {
  margin-bottom: 8px;
}

.checkbox-label {
  display: flex;
  align-items: center;
  gap: 8px;
  cursor: pointer;
  font-size: 13px;
  color: var(--color-text-primary);
  user-select: none;
}

.checkbox-label input[type="checkbox"] {
  width: 16px;
  height: 16px;
  cursor: pointer;
  accent-color: var(--color-accent);
}

.size-option {
  margin-bottom: 16px;
}

.size-option label {
  display: block;
  margin-bottom: 8px;
  font-size: 13px;
  color: var(--color-text-primary);
}

.size-option input[type="range"] {
  width: 100%;
  cursor: pointer;
  accent-color: var(--color-accent);
}

.panel-list,
.floating-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.no-panels {
  padding: 16px;
  text-align: center;
  color: var(--color-text-tertiary);
  font-size: 13px;
  background-color: var(--color-bg-secondary);
  border-radius: 6px;
}

.panel-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 10px 12px;
  background-color: var(--color-bg-secondary);
  border-radius: 6px;
  border: 1px solid var(--color-border);
}

.panel-item.floating {
  background-color: var(--color-bg-tertiary);
  border-color: var(--color-accent);
}

.panel-info {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.panel-title {
  font-size: 13px;
  font-weight: 500;
  color: var(--color-text-primary);
}

.panel-location {
  font-size: 11px;
  color: var(--color-text-tertiary);
}

.panel-actions {
  display: flex;
  align-items: center;
  gap: 6px;
}

.location-select {
  padding: 4px 8px;
  font-size: 12px;
  border: 1px solid var(--color-border);
  border-radius: 4px;
  background-color: var(--color-bg-tertiary);
  color: var(--color-text-primary);
  cursor: pointer;
}

.location-select:focus {
  outline: none;
  border-color: var(--color-accent);
}

.action-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 26px;
  height: 26px;
  border: none;
  border-radius: 4px;
  background-color: transparent;
  color: var(--color-text-tertiary);
  cursor: pointer;
  transition: all 0.15s ease;
}

.action-btn:hover {
  background-color: var(--color-bg-hover);
  color: var(--color-text-primary);
}

.action-btn.floating-btn:hover {
  color: var(--color-accent);
}

.action-btn.close-btn:hover {
  color: var(--color-error);
}

.layout-actions {
  padding-top: 16px;
  border-top: 1px solid var(--color-border);
}

.reset-btn {
  width: 100%;
  padding: 10px 16px;
  border: 1px solid var(--color-border);
  border-radius: 6px;
  background-color: var(--color-bg-tertiary);
  color: var(--color-text-primary);
  font-size: 13px;
  cursor: pointer;
  transition: all 0.2s ease;
}

.reset-btn:hover {
  background-color: var(--color-bg-hover);
  border-color: var(--color-border-hover);
}
</style>
