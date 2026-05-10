<template>
  <Teleport to="body">
    <Transition name="modal">
      <div v-show="visible" class="command-palette-overlay" @click.self="handleClose">
        <div class="command-palette-container">
          <!-- 搜索输入 -->
          <div class="command-palette-input-wrapper">
            <Search :size="18" class="search-icon" />
            <input
              ref="inputRef"
              v-model="searchQuery"
              type="text"
              class="command-palette-input"
              :placeholder="t('commandPalette.placeholder')"
              @keydown.down.prevent="handleArrowDown"
              @keydown.up.prevent="handleArrowUp"
              @keydown.enter.prevent="handleEnter"
              @keydown.esc.prevent="handleClose"
            />
          </div>

          <!-- 结果列表 -->
          <div
            v-if="filteredCommands.length > 0"
            class="command-palette-results"
            role="listbox"
            :aria-label="t('commandPalette.title')"
          >
            <div
              v-for="(cmd, index) in filteredCommands"
              :key="cmd.id"
              class="command-item"
              :class="{ active: index === selectedIndex }"
              role="option"
              :aria-selected="index === selectedIndex"
              tabindex="-1"
              @click="handleSelect(cmd)"
              @mouseenter="selectedIndex = index"
            >
              <div class="command-item-left">
                <component :is="getCommandIcon(cmd.icon)" :size="16" aria-hidden="true" />
                <span class="command-label">{{ cmd.label }}</span>
              </div>
              <div class="command-item-right">
                <span class="command-category">{{ cmd.category }}</span>
                <span v-if="cmd.shortcut" class="command-shortcut">{{ cmd.shortcut }}</span>
              </div>
            </div>
          </div>

          <!-- 空状态 -->
          <div v-else-if="searchQuery" class="command-palette-empty">
            {{ t('commandPalette.noResults') }}
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<script setup lang="ts">
import { Search, Terminal, FileText, Settings, Database, Play, HelpCircle } from 'lucide-vue-next'
import { computed, nextTick, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'

import { useCommandStore } from '../../stores/command-store'

import type { Command } from '../../stores/command-store'
import type { Component } from 'vue'

interface Props {
  visible: boolean
}

const props = defineProps<Props>()

const emit = defineEmits<{
  close: []
}>()

const { t } = useI18n()
const commandStore = useCommandStore()

const searchQuery = ref('')
const selectedIndex = ref(0)
const inputRef = ref<HTMLInputElement | null>(null)

const filteredCommands = computed(() => {
  return commandStore.search(searchQuery.value)
})

function getCommandIcon(iconName?: string): Component {
  const iconMap: Record<string, Component> = {
    terminal: Terminal,
    file: FileText,
    settings: Settings,
    database: Database,
    play: Play,
    help: HelpCircle,
  }
  return iconMap[iconName || ''] || Terminal
}

function handleArrowDown() {
  if (selectedIndex.value < filteredCommands.value.length - 1) {
    selectedIndex.value++
  }
}

function handleArrowUp() {
  if (selectedIndex.value > 0) {
    selectedIndex.value--
  }
}

function handleEnter() {
  const cmd = filteredCommands.value[selectedIndex.value]
  if (cmd) {
    handleSelect(cmd)
  }
}

function handleSelect(cmd: Command) {
  commandStore.execute(cmd.id)
  handleClose()
}

function handleClose() {
  searchQuery.value = ''
  selectedIndex.value = 0
  emit('close')
}

watch(
  () => props.visible,
  visible => {
    if (visible) {
      nextTick(() => {
        inputRef.value?.focus()
      })
    }
  }
)

watch(searchQuery, () => {
  selectedIndex.value = 0
})
</script>

<style scoped>
.command-palette-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: var(--overlay-bg);
  backdrop-filter: blur(2px);
  display: flex;
  align-items: flex-start;
  justify-content: center;
  padding-top: 15vh;
  z-index: 2000;
}

.command-palette-container {
  background: var(--bg-secondary);
  border: 1px solid var(--border-color);
  border-radius: var(--border-radius-md);
  width: 100%;
  max-width: 600px;
  overflow: hidden;
  box-shadow: var(--shadow-lg);
}

.command-palette-input-wrapper {
  display: flex;
  align-items: center;
  gap: var(--spacing-sm);
  padding: var(--spacing-md);
  border-bottom: 1px solid var(--border-color);
}

.search-icon {
  color: var(--text-tertiary);
  flex-shrink: 0;
}

.command-palette-input {
  flex: 1;
  border: none;
  background: transparent;
  color: var(--text-primary);
  font-size: var(--font-size-lg);
  outline: none;
}

.command-palette-input::placeholder {
  color: var(--text-tertiary);
}

.command-palette-results {
  max-height: 400px;
  overflow-y: auto;
  padding: var(--spacing-xs) 0;
}

.command-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: var(--spacing-sm) var(--spacing-md);
  cursor: pointer;
  transition: background 0.1s;
  color: var(--text-secondary);
}

.command-item:hover,
.command-item.active {
  background: var(--primary-light);
  color: var(--text-primary);
}

.command-item-left {
  display: flex;
  align-items: center;
  gap: var(--spacing-sm);
}

.command-label {
  font-size: var(--font-size-md);
}

.command-item-right {
  display: flex;
  align-items: center;
  gap: var(--spacing-sm);
}

.command-category {
  font-size: var(--font-size-sm);
  color: var(--text-tertiary);
  text-transform: capitalize;
}

.command-shortcut {
  font-size: var(--font-size-sm);
  color: var(--text-tertiary);
  background: var(--bg-tertiary);
  padding: 2px 6px;
  border-radius: var(--border-radius-sm);
  font-family: var(--font-mono);
}

.command-palette-empty {
  padding: var(--spacing-lg);
  text-align: center;
  color: var(--text-tertiary);
  font-size: var(--font-size-md);
}

/* 过渡动画 */
.modal-enter-active,
.modal-leave-active {
  transition: all 0.2s ease;
}

.modal-enter-from,
.modal-leave-to {
  opacity: 0;
}

.modal-enter-from .command-palette-container,
.modal-leave-to .command-palette-container {
  transform: translateY(-8px);
}
</style>
