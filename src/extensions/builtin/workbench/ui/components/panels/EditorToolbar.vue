<template>
  <div class="editor-toolbar" :class="`toolbar-${toolbarPosition}`">
    <div class="toolbar-group">
      <NTooltip trigger="hover">
        <template #trigger>
          <NButton quaternary size="small" class="toolbar-btn" @click="$emit('execute')">
            <Play :size="16" />
          </NButton>
        </template>
        {{ $t('sqlEditor.execute') }}{{ $t('sqlEditor.shortcutCtrlEnter') }}
      </NTooltip>

      <NTooltip trigger="hover">
        <template #trigger>
          <NButton quaternary size="small" class="toolbar-btn" @click="$emit('executeNew')">
            <Plus :size="16" />
          </NButton>
        </template>
        {{ $t('sqlEditor.executeNew') }}
      </NTooltip>

      <NTooltip v-if="props.showAdvanced" trigger="hover">
        <template #trigger>
          <NButton quaternary size="small" class="toolbar-btn" @click="$emit('executeBatch')">
            <ListChecks :size="16" />
          </NButton>
        </template>
        {{ $t('sqlEditor.executeBatch') }}
      </NTooltip>

      <NTooltip v-if="isDuckDb && props.showAdvanced" trigger="hover">
        <template #trigger>
          <NButton quaternary size="small" class="toolbar-btn" @click="$emit('duckdbExecute')">
            <Zap :size="16" />
          </NButton>
        </template>
        {{ $t('sqlEditor.duckdbAccelerate') }}
      </NTooltip>

      <div v-if="props.showAdvanced" class="toolbar-divider" />

      <NTooltip v-if="props.showAdvanced" trigger="hover">
        <template #trigger>
          <NButton quaternary size="small" class="toolbar-btn" @click="$emit('format')">
            <AlignLeft :size="16" />
          </NButton>
        </template>
        {{ $t('sqlEditor.format') }}{{ $t('sqlEditor.shortcutCtrlShiftF') }}
      </NTooltip>

      <NTooltip v-if="props.showAdvanced" trigger="hover">
        <template #trigger>
          <NButton quaternary size="small" class="toolbar-btn" @click="$emit('validate')">
            <Sparkles :size="16" />
          </NButton>
        </template>
        {{ $t('sqlEditor.validate') }}
      </NTooltip>

      <NTooltip v-if="props.showAdvanced" trigger="hover">
        <template #trigger>
          <NButton quaternary size="small" class="toolbar-btn" @click="$emit('transpile')">
            <ArrowLeftRight :size="16" />
          </NButton>
        </template>
        {{ $t('sqlEditor.transpile') }}
      </NTooltip>

      <NTooltip v-if="props.showAdvanced" trigger="hover">
        <template #trigger>
          <NButton quaternary size="small" class="toolbar-btn" @click="$emit('explain')">
            <FileSearch :size="16" />
          </NButton>
        </template>
        {{ $t('sqlEditor.explain') }}
      </NTooltip>

      <NTooltip v-if="props.showAdvanced" trigger="hover">
        <template #trigger>
          <NButton quaternary size="small" class="toolbar-btn" @click="$emit('saveSnippet')">
            <Star :size="16" />
          </NButton>
        </template>
        {{ $t('sqlEditor.saveSnippet') }}
      </NTooltip>

      <div v-if="props.showAdvanced" class="toolbar-divider" />

      <NTooltip trigger="hover">
        <template #trigger>
          <NButton quaternary size="small" class="toolbar-btn" @click="$emit('toggleMinimap')">
            <Map :size="16" />
          </NButton>
        </template>
        {{ $t('sqlEditor.toggleMinimap') }}
      </NTooltip>

      <NTooltip trigger="hover">
        <template #trigger>
          <NButton quaternary size="small" class="toolbar-btn" @click="$emit('toggleSettings')">
            <Settings :size="16" />
          </NButton>
        </template>
        {{ $t('sqlEditor.editorSettings') }}
      </NTooltip>
    </div>
  </div>
</template>

<script setup lang="ts">
import {
  Play,
  Plus,
  Zap,
  Sparkles,
  ArrowLeftRight,
  AlignLeft,
  ListChecks,
  FileSearch,
  Star,
  Map,
  Settings,
} from 'lucide-vue-next'
import { NButton, NTooltip } from 'naive-ui'

interface Props {
  toolbarPosition: 'top' | 'left' | 'right'
  isDuckDb: boolean
  showAdvanced?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  showAdvanced: true,
})

interface Emits {
  (e: 'execute'): void
  (e: 'executeNew'): void
  (e: 'executeBatch'): void
  (e: 'duckdbExecute'): void
  (e: 'format'): void
  (e: 'validate'): void
  (e: 'transpile'): void
  (e: 'explain'): void
  (e: 'saveSnippet'): void
  (e: 'toggleMinimap'): void
  (e: 'toggleSettings'): void
}

defineEmits<Emits>()
</script>

<style scoped>
.editor-toolbar {
  display: flex;
  align-items: center;
  padding: 4px 8px;
  background: var(--bg-secondary, #252526);
  border-bottom: 1px solid var(--border-color, #3e3e42);
  gap: 2px;
}

.editor-toolbar.toolbar-left,
.editor-toolbar.toolbar-right {
  flex-direction: column;
  border-bottom: none;
  padding: 8px 4px;
}

.editor-toolbar.toolbar-left {
  border-right: 1px solid var(--border-color, #3e3e42);
}

.editor-toolbar.toolbar-right {
  border-left: 1px solid var(--border-color, #3e3e42);
}

.toolbar-group {
  display: flex;
  align-items: center;
  gap: 2px;
}

.toolbar-left .toolbar-group,
.toolbar-right .toolbar-group {
  flex-direction: column;
}

.toolbar-btn {
  color: var(--text-secondary, #858585);
}

.toolbar-btn:hover {
  color: var(--text-primary, #cccccc);
  background: var(--bg-hover, #2d2d30);
}

.toolbar-divider {
  width: 1px;
  height: 16px;
  background: var(--border-color, #3e3e42);
  margin: 0 6px;
}

.toolbar-left .toolbar-divider,
.toolbar-right .toolbar-divider {
  width: 16px;
  height: 1px;
  margin: 6px 0;
}
</style>
