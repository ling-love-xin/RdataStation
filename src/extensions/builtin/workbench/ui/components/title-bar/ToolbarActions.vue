<template>
  <div class="toolbar-wrapper">
    <!-- 已启用的工具按钮 -->
    <button
      v-for="tool in enabledTools"
      :key="tool.id"
      class="icon-btn toolbar-btn"
      :title="tool.name"
      @click="handleToolAction(tool)"
    >
      <component :is="tool.icon" :size="14" />
    </button>

    <!-- 自定义工具栏下拉 -->
    <div class="toolbar-section">
      <button class="icon-btn more-btn" :title="t('workbench.customizeToolbar')" @click="toggleDropdown">
        <MoreHorizontal :size="16" />
      </button>

      <Transition name="dropdown">
        <div v-if="showDropdown" class="dropdown-panel toolbar-dropdown">
          <div class="dropdown-header">{{ t('workbench.customizeToolbar') }}</div>
          <div class="dropdown-divider" />
          <div class="toolbar-options">
            <label v-for="tool in tools" :key="tool.id" class="toolbar-option">
              <input v-model="tool.enabled" type="checkbox" @change="handleToggleTool(tool)" />
              <component :is="tool.icon" :size="14" />
              <span>{{ tool.name }}</span>
            </label>
          </div>
          <div class="dropdown-divider" />
          <div class="dropdown-item" @click="handleResetToolbar">
            <RotateCcw :size="14" />
            <span>{{ t('workbench.resetToDefault') }}</span>
          </div>
        </div>
      </Transition>
    </div>
  </div>
</template>

<script setup lang="ts">
import { MoreHorizontal, RotateCcw } from 'lucide-vue-next'
import { computed, ref } from 'vue'
import { useI18n } from 'vue-i18n'

import type { Component } from 'vue'

export interface ToolbarTool {
  id: string
  name: string
  icon: Component
  enabled: boolean
  action: () => void
}

interface Props {
  tools: ToolbarTool[]
}

const props = defineProps<Props>()

const emit = defineEmits<{
  (e: 'tool-action', toolId: string): void
  (e: 'toggle-tool', toolId: string, enabled: boolean): void
  (e: 'reset-toolbar'): void
}>()

const { t } = useI18n()

const showDropdown = ref(false)

const enabledTools = computed(() => props.tools.filter(tool => tool.enabled))

function toggleDropdown() {
  showDropdown.value = !showDropdown.value
}

function handleToolAction(tool: ToolbarTool) {
  tool.action()
  emit('tool-action', tool.id)
}

function handleToggleTool(tool: ToolbarTool) {
  emit('toggle-tool', tool.id, tool.enabled)
}

function handleResetToolbar() {
  emit('reset-toolbar')
  showDropdown.value = false
}
</script>

<style scoped>
@import './title-bar.css';

.toolbar-wrapper {
  display: flex;
  align-items: center;
  gap: var(--spacing-xs);
}

.toolbar-dropdown {
  right: 0;
  min-width: 180px;
}

.dropdown-header {
  padding: 6px 12px;
  font-size: var(--font-size-md);
  font-weight: 600;
  color: var(--text-primary);
}
</style>
