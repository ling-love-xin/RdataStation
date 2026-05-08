<template>
  <div v-if="visible" class="editor-watermark">
    <div class="watermark-text">
      <div class="watermark-title">{{ $t('sqlEditor.title') }}</div>
      <div class="watermark-shortcuts">
        <span class="shortcut-hint">{{ $t('sqlEditor.welcomeShortcuts') }}</span>
        <span class="shortcut-hint"
          ><kbd>Ctrl</kbd>+<kbd>Enter</kbd> {{ $t('sqlEditor.shortcutExecute') }}</span
        >
        <span class="shortcut-hint"
          ><kbd>Ctrl</kbd>+<kbd>Shift</kbd>+<kbd>F</kbd> {{ $t('sqlEditor.shortcutFormat') }}</span
        >
        <span class="shortcut-hint"
          ><kbd>Ctrl</kbd>+<kbd>/</kbd> {{ $t('sqlEditor.shortcutComment') }}</span
        >
        <span class="shortcut-hint"><kbd>F5</kbd> {{ $t('sqlEditor.shortcutExecuteAll') }}</span>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
interface Props {
  visible: boolean
}

defineProps<Props>()
</script>

<style scoped>
.editor-watermark {
  position: absolute;
  top: 40px;
  left: 0;
  right: 0;
  bottom: 24px;
  display: flex;
  align-items: center;
  justify-content: center;
  pointer-events: none;
  z-index: 1;
}

.watermark-text {
  text-align: center;
  opacity: 0.35;
  user-select: none;
}

.watermark-title {
  font-size: 20px;
  font-weight: 600;
  color: var(--text-primary, #cccccc);
  margin-bottom: 16px;
}

.watermark-shortcuts {
  display: flex;
  flex-direction: column;
  gap: 6px;
  align-items: center;
}

.shortcut-hint {
  font-size: 12px;
  color: var(--text-secondary, #858585);
  line-height: 1.6;
}

.shortcut-hint kbd {
  display: inline-block;
  padding: 1px 5px;
  margin: 0 2px;
  background: var(--bg-tertiary, #2d2d30);
  border: 1px solid var(--border-color, #3e3e42);
  border-radius: 3px;
  font-family: 'JetBrains Mono', monospace;
  font-size: 11px;
  color: var(--text-secondary, #858585);
}
</style>
