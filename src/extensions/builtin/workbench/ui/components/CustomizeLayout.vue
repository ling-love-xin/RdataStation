<template>
  <div class="customize-layout">
    <div class="dialog-header">
      <h3>{{ t('workbench.layout') }}</h3>
      <button class="close-btn" @click="$emit('close')">
        <X :size="16" />
      </button>
    </div>

    <div class="dialog-body">
      <!-- 显示/隐藏 -->
      <div class="section">
        <div class="section-title">{{ t('workbench.showHideElements') }}</div>
        <div class="section-content">
          <label class="checkbox-item">
            <input
              v-model="layoutStore.primarySideBarVisible"
              type="checkbox"
              @change="layoutStore.togglePrimarySideBar()"
            />
            <span>{{ t('workbench.primarySideBar') }} ({{ t('workbench.alwaysShow') }})</span>
          </label>
          <label class="checkbox-item">
            <input
              v-model="layoutStore.secondarySideBarVisible"
              type="checkbox"
              @change="layoutStore.toggleSecondarySideBar()"
            />
            <span>{{ t('workbench.secondarySideBar') }} ({{ t('workbench.alwaysShow') }})</span>
          </label>
        </div>
      </div>

      <div class="divider" />

      <!-- 尺寸设置 -->
      <div class="section">
        <div class="section-title">{{ t('workbench.sizeSettings') }}</div>
        <div class="section-content">
          <div class="size-input">
            <span>{{ t('workbench.primarySideBar') }}</span>
            <input
              v-model.number="primarySideBarWidth"
              type="number"
              min="100"
              max="500"
              @change="layoutStore.setPrimarySideBarWidth(primarySideBarWidth)"
            />
          </div>
          <div class="size-input">
            <span>{{ t('workbench.secondarySideBar') }}</span>
            <input
              v-model.number="secondarySideBarWidth"
              type="number"
              min="100"
              max="500"
              @change="layoutStore.setSecondarySideBarWidth(secondarySideBarWidth)"
            />
          </div>
        </div>
      </div>

      <div class="divider" />

      <!-- 面板管理 -->
      <div class="section">
        <div class="section-title">{{ t('workbench.panelManagement') }}</div>
        <div class="section-content">
          <div v-if="allPanels.length === 0" class="empty-state">
            {{ t('workbench.noPanels') }}
          </div>
          <div v-for="panel in allPanels" :key="panel.id" class="panel-item">
            <span class="panel-title">{{ panel.title }}</span>
            <div class="panel-actions">
              <button
                v-if="isFloating(panel)"
                class="action-btn"
                :title="t('workbench.dockBack')"
                @click="dockPanel(panel)"
              >
                <ExternalLink :size="14" />
              </button>
              <button class="action-btn" :title="t('workbench.close')" @click="closePanel(panel)">
                <X :size="14" />
              </button>
            </div>
          </div>
        </div>
      </div>

      <div class="divider" />

      <!-- 浮动窗口 -->
      <div class="section">
        <div class="section-title">{{ t('workbench.floatingWindows') }}</div>
        <div class="section-content">
          <div v-if="floatingPanels.length === 0" class="empty-state">
            {{ t('workbench.noFloatingWindows') }}
          </div>
          <div v-for="panel in floatingPanels" :key="panel.id" class="panel-item">
            <span class="panel-title">{{ panel.title }}</span>
            <div class="panel-actions">
              <button class="action-btn" :title="t('workbench.dockBack')" @click="dockPanel(panel)">
                <ExternalLink :size="14" />
              </button>
              <button class="action-btn" :title="t('workbench.close')" @click="closePanel(panel)">
                <X :size="14" />
              </button>
            </div>
          </div>
        </div>
      </div>

      <div class="divider" />

      <!-- 重置 -->
      <div class="section">
        <button class="reset-btn" @click="layoutStore.resetLayout()">
          <RotateCcw :size="14" />
          {{ t('workbench.resetLayout') }}
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ExternalLink, X, RotateCcw } from 'lucide-vue-next'
import { ref, computed } from 'vue'
import { useI18n } from 'vue-i18n'

import type { IDockviewPanel } from '@/core/dockview-types'

import { useLayoutStore } from '../stores/layout-store'


const { t } = useI18n()
const layoutStore = useLayoutStore()

const primarySideBarWidth = ref(280)
const secondarySideBarWidth = ref(280)

const allPanels = computed(() => layoutStore.getAllPanels())
const floatingPanels = computed(() => layoutStore.getFloatingPanels())

const floatingPanelIds = computed(() => new Set(floatingPanels.value.map(p => p.id)))

function isFloating(panel: IDockviewPanel): boolean {
  return floatingPanelIds.value.has(panel.id)
}

function dockPanel(panel: IDockviewPanel) {
  const panelApi = layoutStore.dockviewApi?.getPanel(panel.id)
  if (panelApi) {
    layoutStore.dockviewApi?.addPanel({
      id: panel.id,
      component: panelApi.id,
      title: panelApi.title ?? panel.id,
      position: { referencePanel: panel.id }
    })
  }
}

function closePanel(panel: IDockviewPanel) {
  const panelApi = layoutStore.dockviewApi?.getPanel(panel.id)
  if (panelApi) {
    panelApi.api.close()
  }
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

.layout-option.locked {
  opacity: 0.5;
}

.layout-option.locked em {
  font-size: 11px;
  color: var(--color-text-tertiary);
  font-style: normal;
  margin-left: 4px;
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

.checkbox-label input[type='checkbox'] {
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

.size-option input[type='range'] {
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
