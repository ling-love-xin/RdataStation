<template>
  <div class="settings-popup">
    <div class="settings-header">
      <span class="settings-title">编辑器设置</span>
      <span class="settings-close" @click="emit('close')">✕</span>
    </div>
    <div class="settings-scroll">
      <div class="settings-section">
        <div class="settings-row">
          <label class="settings-label">字号</label>
          <select
            class="settings-select"
            :value="settings.fontSize"
            @change="handlers.setFontSize(Number(($event.target as HTMLSelectElement).value))"
          >
            <option v-for="s in fontSizes" :key="s" :value="s">{{ s }}</option>
          </select>
        </div>
        <div class="settings-row">
          <label class="settings-label">字体</label>
          <input
            class="settings-input"
            :value="settings.fontFamily"
            @blur="onFontFamilyBlur"
            @keydown.enter="($event.target as HTMLInputElement).blur()"
          />
        </div>
        <div class="settings-row">
          <label class="settings-label">制表符大小</label>
          <select
            class="settings-select"
            :value="settings.tabSize"
            @change="handlers.setTabSize(Number(($event.target as HTMLSelectElement).value))"
          >
            <option v-for="s in tabSizes" :key="s" :value="s">{{ s }}</option>
          </select>
        </div>
      </div>

      <div class="settings-section">
        <div class="settings-row">
          <label class="settings-toggle">
            <input
              type="checkbox"
              :checked="settings.wordWrap"
              @change="handlers.setWordWrap(($event.target as HTMLInputElement).checked)"
            />
            <span>自动换行</span>
          </label>
        </div>
        <div class="settings-row">
          <label class="settings-toggle">
            <input
              type="checkbox"
              :checked="settings.minimap"
              @change="handlers.setMinimap(($event.target as HTMLInputElement).checked)"
            />
            <span>Minimap</span>
          </label>
        </div>
        <div class="settings-row">
          <label class="settings-toggle">
            <input
              type="checkbox"
              :checked="settings.lineNumbers"
              @change="handlers.setLineNumbers(($event.target as HTMLInputElement).checked)"
            />
            <span>行号</span>
          </label>
        </div>
        <div class="settings-row">
          <label class="settings-toggle">
            <input
              type="checkbox"
              :checked="settings.fontLigatures"
              @change="handlers.setFontLigatures(($event.target as HTMLInputElement).checked)"
            />
            <span>字体连字</span>
          </label>
        </div>
        <div class="settings-row">
          <label class="settings-toggle">
            <input
              type="checkbox"
              :checked="settings.folding"
              @change="handlers.setFolding(($event.target as HTMLInputElement).checked)"
            />
            <span>代码折叠</span>
          </label>
        </div>
        <div class="settings-row">
          <label class="settings-toggle">
            <input
              type="checkbox"
              :checked="settings.matchBrackets"
              @change="handlers.setMatchBrackets(($event.target as HTMLInputElement).checked)"
            />
            <span>括号匹配</span>
          </label>
        </div>
        <div class="settings-row">
          <label class="settings-toggle">
            <input
              type="checkbox"
              :checked="settings.autoIndent"
              @change="handlers.setAutoIndent(($event.target as HTMLInputElement).checked)"
            />
            <span>自动缩进</span>
          </label>
        </div>
        <div class="settings-row">
          <label class="settings-toggle">
            <input
              type="checkbox"
              :checked="settings.formatOnPaste"
              @change="handlers.setFormatOnPaste(($event.target as HTMLInputElement).checked)"
            />
            <span>粘贴时格式化</span>
          </label>
        </div>
        <div class="settings-row">
          <label class="settings-toggle">
            <input
              type="checkbox"
              :checked="settings.scrollBeyond"
              @change="handlers.setScrollBeyond(($event.target as HTMLInputElement).checked)"
            />
            <span>滚动超过最后一行</span>
          </label>
        </div>
      </div>

      <div class="settings-section">
        <div class="settings-row">
          <label class="settings-label">空白符渲染</label>
          <select
            class="settings-select"
            :value="settings.renderWhitespace"
            @change="handlers.setRenderWhitespace(($event.target as HTMLSelectElement).value)"
          >
            <option value="selection">仅选中</option>
            <option value="none">无</option>
            <option value="all">全部</option>
            <option value="boundary">边界</option>
            <option value="trailing">尾部</option>
          </select>
        </div>
        <div class="settings-row">
          <label class="settings-label">光标样式</label>
          <select
            class="settings-select"
            :value="settings.cursorStyle"
            @change="handlers.setCursorStyle(($event.target as HTMLSelectElement).value)"
          >
            <option value="line">竖线</option>
            <option value="block">块状</option>
            <option value="underline">下划线</option>
          </select>
        </div>
        <div class="settings-row">
          <label class="settings-toggle">
            <input
              type="checkbox"
              :checked="settings.cursorBlinking"
              @change="handlers.setCursorBlinking(($event.target as HTMLInputElement).checked)"
            />
            <span>光标闪烁</span>
          </label>
        </div>
      </div>

      <div class="settings-section">
        <div class="settings-row">
          <label class="settings-label">标尺列数</label>
          <input
            class="settings-input settings-input-narrow"
            type="number"
            min="0"
            max="200"
            :value="settings.ruler"
            @change="handlers.setRuler(Number(($event.target as HTMLInputElement).value) || 0)"
          />
        </div>
        <div class="settings-row">
          <label class="settings-toggle">
            <input
              type="checkbox"
              :checked="settings.renderIndentGuides"
              @change="handlers.setIndentGuides(($event.target as HTMLInputElement).checked)"
            />
            <span>缩进参考线</span>
          </label>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import type {
  EditorSettingsState,
  EditorSettingsHandlers,
} from '@/extensions/builtin/workbench/ui/composables/useEditorSettings'

interface Props {
  settings: EditorSettingsState
  handlers: EditorSettingsHandlers
}

const props = defineProps<Props>()

const emit = defineEmits<{
  close: []
}>()

const fontSizes = [10, 11, 12, 13, 14, 15, 16, 18, 20, 22, 24, 28, 30]
const tabSizes = [1, 2, 4, 8]

function onFontFamilyBlur(e: FocusEvent) {
  const val = (e.target as HTMLInputElement).value.trim()
  if (val) {
    props.handlers.setFontFamily(val)
  }
}
</script>

<style scoped>
.settings-popup {
  position: absolute;
  bottom: 100%;
  right: 0;
  margin-bottom: 6px;
  width: 280px;
  max-height: 420px;
  background: var(--bg-secondary, #2b2d30);
  border: 1px solid var(--border-color, #4a5458);
  border-radius: 6px;
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.48);
  z-index: 1000;
  overflow: hidden;
}

.settings-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 12px;
  border-bottom: 1px solid var(--border-color, #4a5458);
}

.settings-title {
  font-size: 13px;
  font-weight: 600;
  color: var(--text-primary, #e5e7eb);
}

.settings-close {
  cursor: pointer;
  font-size: 14px;
  color: var(--text-muted, #6b7280);
  padding: 0 4px;
  border-radius: 2px;
}

.settings-close:hover {
  color: var(--text-primary, #e5e7eb);
  background: var(--hover-bg, rgba(255, 255, 255, 0.08));
}

.settings-scroll {
  max-height: 370px;
  overflow-y: auto;
  padding: 6px 0;
}

.settings-section {
  padding: 4px 0;
}

.settings-section + .settings-section {
  border-top: 1px solid var(--border-color, #4a5458);
}

.settings-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 4px 12px;
  min-height: 28px;
}

.settings-label {
  font-size: 12px;
  color: var(--text-primary, #e5e7eb);
  flex-shrink: 0;
}

.settings-toggle {
  display: flex;
  align-items: center;
  gap: 8px;
  cursor: pointer;
  font-size: 12px;
  color: var(--text-primary, #e5e7eb);
  width: 100%;
}

.settings-toggle input[type='checkbox'] {
  accent-color: var(--brand-accent, #007acc);
  cursor: pointer;
}

.settings-select {
  background: var(--bg-tertiary, #3c3c3c);
  color: var(--text-primary, #e5e7eb);
  border: 1px solid var(--border-color, #4a5458);
  border-radius: 3px;
  padding: 2px 6px;
  font-size: 12px;
  cursor: pointer;
  min-width: 80px;
}

.settings-select:focus {
  outline: 1px solid var(--brand-accent, #007acc);
  outline-offset: -1px;
}

.settings-input {
  background: var(--bg-tertiary, #3c3c3c);
  color: var(--text-primary, #e5e7eb);
  border: 1px solid var(--border-color, #4a5458);
  border-radius: 3px;
  padding: 2px 6px;
  font-size: 12px;
  width: 160px;
}

.settings-input:focus {
  outline: 1px solid var(--brand-accent, #007acc);
  outline-offset: -1px;
}

.settings-input-narrow {
  width: 60px;
  text-align: center;
}

.settings-scroll::-webkit-scrollbar {
  width: 6px;
}

.settings-scroll::-webkit-scrollbar-thumb {
  background: rgba(255, 255, 255, 0.15);
  border-radius: 3px;
}

.settings-scroll::-webkit-scrollbar-track {
  background: transparent;
}
</style>
