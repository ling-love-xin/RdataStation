<template>
  <div class="code-editor-panel">
    <div class="code-editor-body">
      <div
        ref="editorContainer"
        class="monaco-container"
        @mousedown="focusEditor"
        @keydown.enter.stop
      />
      <div v-if="showWelcome" class="editor-welcome-overlay">
        <div class="welcome-content">
          <div class="welcome-icon">
            <Code :size="48" />
          </div>
          <h3>{{ welcomeTitle }}</h3>
          <p>{{ welcomeSubtitle }}</p>
        </div>
      </div>
    </div>

    <div class="code-editor-statusbar">
      <div class="status-left">
        <span v-if="fileName" class="status-item status-file">
          {{ fileName }}
        </span>
        <span v-if="fileName" class="status-divider" />

        <div ref="langDropdownRef" class="status-dropdown">
          <span class="status-item status-clickable" @click.stop="toggleDropdown('lang')">
            {{ currentLanguage }}
          </span>
          <div v-if="activeDropdown === 'lang'" class="dropdown-menu" @click.stop>
            <div class="dropdown-scroll">
              <div
                v-for="lang in languageOptions"
                :key="lang.id"
                class="dropdown-option"
                :class="{ active: currentLanguage === lang.id }"
                @click="switchLanguage(lang.id)"
              >
                <span class="option-check">{{ currentLanguage === lang.id ? '✓' : '' }}</span>
                <span>{{ lang.label }}</span>
              </div>
            </div>
          </div>
        </div>
        <span class="status-divider" />

        <div ref="encodingDropdownRef" class="status-dropdown">
          <span class="status-item status-clickable" @click.stop="toggleDropdown('encoding')">
            {{ currentEncoding }}
          </span>
          <div v-if="activeDropdown === 'encoding'" class="dropdown-menu" @click.stop>
            <div class="dropdown-scroll">
              <div
                v-for="enc in encodingOptions"
                :key="enc"
                class="dropdown-option"
                :class="{ active: currentEncoding === enc }"
                @click="switchEncoding(enc)"
              >
                <span class="option-check">{{ currentEncoding === enc ? '✓' : '' }}</span>
                <span>{{ enc }}</span>
              </div>
            </div>
          </div>
        </div>
        <span class="status-divider" />

        <div ref="eolDropdownRef" class="status-dropdown">
          <span class="status-item status-clickable" @click.stop="toggleDropdown('eol')">
            {{ currentEol }}
          </span>
          <div v-if="activeDropdown === 'eol'" class="dropdown-menu" @click.stop>
            <div
              v-for="eol in eolOptions"
              :key="eol.id"
              class="dropdown-option"
              :class="{ active: currentEol === eol.id }"
              @click="switchEol(eol.id)"
            >
              <span class="option-check">{{ currentEol === eol.id ? '✓' : '' }}</span>
              <span>{{ eol.label }}</span>
            </div>
          </div>
        </div>
        <span class="status-divider" />

        <span class="status-item status-clickable" @click.stop="toggleEditMode">
          {{ editMode }}
        </span>
        <span class="status-divider" />

        <div ref="indentDropdownRef" class="status-dropdown">
          <span class="status-item status-clickable" @click.stop="toggleDropdown('indent')">
            {{ indentLabel }}
          </span>
          <div v-if="activeDropdown === 'indent'" class="dropdown-menu" @click.stop>
            <div class="dropdown-section-header">缩进方式</div>
            <div
              v-for="opt in indentModeOptions"
              :key="opt.id"
              class="dropdown-option"
              :class="{ active: indentMode === opt.id }"
              @click="switchIndentMode(opt.id)"
            >
              <span class="option-check">{{ indentMode === opt.id ? '✓' : '' }}</span>
              <span>{{ opt.label }}</span>
            </div>
            <div class="dropdown-divider" />
            <div class="dropdown-section-header">缩进大小</div>
            <div
              v-for="size in indentSizeOptions"
              :key="size"
              class="dropdown-option"
              :class="{ active: indentSize === size }"
              @click="switchIndentSize(size)"
            >
              <span class="option-check">{{ indentSize === size ? '✓' : '' }}</span>
              <span>{{ size }}</span>
            </div>
          </div>
        </div>
      </div>

      <div class="status-right">
        <span v-if="selectedTextInfo" class="status-item status-selection">
          {{ selectedTextInfo }}
        </span>
        <span class="status-item status-cursor">
          {{ cursorPosition }}
        </span>
        <span class="status-divider" />
        <span class="status-item status-file-size">
          {{ fileSizeDisplay }}
        </span>
        <span class="status-divider" />

        <div ref="settingsDropdownRef" class="status-dropdown">
          <span
            class="status-item status-clickable status-gear"
            :class="{ active: showSettings || activeDropdown === 'settings' }"
            @click.stop="toggleDropdown('settings')"
          >
            <Settings :size="13" />
          </span>
          <div v-if="showSettings || activeDropdown === 'settings'" class="settings-popup" @click.stop>
            <div class="settings-header">
              <span class="settings-title">编辑器设置</span>
              <span class="settings-close" @click="showSettings = false; closeDropdown()">✕</span>
            </div>
            <div class="settings-scroll">
              <div class="settings-section">
                <div class="settings-row">
                  <label class="settings-label">字号</label>
                  <select class="settings-select" :value="editorFontSize" @change="onFontSizeChange">
                    <option v-for="s in [10,11,12,13,14,15,16,18,20,22,24,28,30]" :key="s" :value="s">{{ s }}</option>
                  </select>
                </div>
                <div class="settings-row">
                  <label class="settings-label">字体</label>
                  <input
                    class="settings-input"
                    :value="editorFontFamily"
                    @blur="onFontFamilyChange"
                    @keydown.enter="($event.target as HTMLInputElement).blur()"
                  />
                </div>
                <div class="settings-row">
                  <label class="settings-label">制表符大小</label>
                  <select class="settings-select" :value="indentSize" @change="onTabSizeChange">
                    <option v-for="s in [1,2,4,8]" :key="s" :value="s">{{ s }}</option>
                  </select>
                </div>
              </div>

              <div class="settings-section">
                <div class="settings-row">
                  <label class="settings-toggle">
                    <input type="checkbox" :checked="editorWordWrap" @change="onWordWrapToggle" />
                    <span>自动换行</span>
                  </label>
                </div>
                <div class="settings-row">
                  <label class="settings-toggle">
                    <input type="checkbox" :checked="editorMinimap" @change="onMinimapToggle" />
                    <span>Minimap</span>
                  </label>
                </div>
                <div class="settings-row">
                  <label class="settings-toggle">
                    <input type="checkbox" :checked="editorLineNumbers" @change="onLineNumbersToggle" />
                    <span>行号</span>
                  </label>
                </div>
                <div class="settings-row">
                  <label class="settings-toggle">
                    <input type="checkbox" :checked="editorFontLigatures" @change="onFontLigaturesToggle" />
                    <span>字体连字</span>
                  </label>
                </div>
                <div class="settings-row">
                  <label class="settings-toggle">
                    <input type="checkbox" :checked="editorFolding" @change="onFoldingToggle" />
                    <span>代码折叠</span>
                  </label>
                </div>
                <div class="settings-row">
                  <label class="settings-toggle">
                    <input type="checkbox" :checked="editorMatchBrackets" @change="onMatchBracketsToggle" />
                    <span>括号匹配</span>
                  </label>
                </div>
                <div class="settings-row">
                  <label class="settings-toggle">
                    <input type="checkbox" :checked="editorAutoIndent" @change="onAutoIndentToggle" />
                    <span>自动缩进</span>
                  </label>
                </div>
                <div class="settings-row">
                  <label class="settings-toggle">
                    <input type="checkbox" :checked="editorFormatOnPaste" @change="onFormatOnPasteToggle" />
                    <span>粘贴时格式化</span>
                  </label>
                </div>
                <div class="settings-row">
                  <label class="settings-toggle">
                    <input type="checkbox" :checked="editorScrollBeyond" @change="onScrollBeyondToggle" />
                    <span>滚动超过最后一行</span>
                  </label>
                </div>
              </div>

              <div class="settings-section">
                <div class="settings-row">
                  <label class="settings-label">空白符渲染</label>
                  <select class="settings-select" :value="editorRenderWhitespace" @change="onRenderWhitespaceChange">
                    <option value="selection">仅选中</option>
                    <option value="none">无</option>
                    <option value="all">全部</option>
                    <option value="boundary">边界</option>
                    <option value="trailing">尾部</option>
                  </select>
                </div>
                <div class="settings-row">
                  <label class="settings-label">光标样式</label>
                  <select class="settings-select" :value="editorCursorStyle" @change="onCursorStyleChange">
                    <option value="line">竖线</option>
                    <option value="block">块状</option>
                    <option value="underline">下划线</option>
                  </select>
                </div>
                <div class="settings-row">
                  <label class="settings-toggle">
                    <input type="checkbox" :checked="editorCursorBlinking" @change="onCursorBlinkingToggle" />
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
                    :value="editorRuler"
                    @change="onRulerChange"
                  />
                </div>
                <div class="settings-row">
                  <label class="settings-toggle">
                    <input type="checkbox" :checked="editorRenderIndentGuides" @change="onIndentGuidesToggle" />
                    <span>缩进参考线</span>
                  </label>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import 'monaco-editor/esm/vs/basic-languages/sql/sql.contribution'
import { Code, Settings } from 'lucide-vue-next'
import * as monaco from 'monaco-editor'
import { ref, computed, watch, onMounted, onBeforeUnmount, nextTick } from 'vue'

import { useMonacoEditor } from '@/extensions/builtin/workbench/ui/composables/useMonacoEditor'
import { useUiStore } from '@/shared/stores/ui'
import { rdataDark, rdataLight } from '@/shared/styles/monaco-theme'

interface CodeEditorParams {
  language?: string
  initialValue?: string
  fileName?: string
  filePath?: string
  panelId?: string
}

const props = defineProps<{
  params: Record<string, unknown>
}>()

const editorParams = computed<CodeEditorParams>(() => {
  const raw = props.params
  const nested = (raw.params as Record<string, unknown> | undefined) || raw
  return nested as CodeEditorParams
})

const panelId = computed(() => String(editorParams.value.panelId || 'code_editor'))
const editorLanguage = computed(() => String(editorParams.value.language || 'plaintext'))
const initialValue = computed(() => String(editorParams.value.initialValue || ''))
const fileName = computed(() => String(editorParams.value.fileName || ''))

const welcomeTitle = computed(() => fileName.value || 'Untitled')
const welcomeSubtitle = computed(
  () => String(editorParams.value.filePath || 'Start editing...')
)

const uiStore = useUiStore()
const currentTheme = computed(() => {
  return uiStore.isDark ? 'rdata-dark' : 'rdata-light'
})

const editorContainer = ref<HTMLElement>()
const editorCreated = ref(false)

const {
  editor,
  editorModel,
  showWelcome,
  cursorPosition,
  selectedTextInfo,
  createEditor,
  setupEditorEvents,
  setValue,
  getValue,
  updateLanguage,
  disposeEditor,
  disposeMonacoDisposables,
  setMinimap,
  setFontSize,
  setWordWrap,
  setTabSize: setMonacoTabSize,
  setFontFamily,
} = useMonacoEditor({
  containerRef: editorContainer,
  panelId: panelId.value,
  initialValue: initialValue.value,
  language: editorLanguage.value,
  theme: currentTheme.value,
})

const currentLanguage = ref(editorLanguage.value)
const currentEncoding = ref('UTF-8')
const currentEol = ref('LF')
const indentMode = ref<'spaces' | 'tabs'>('spaces')
const indentSize = ref(2)
const fileSizeDisplay = ref('0 B')
const activeDropdown = ref<string | null>(null)
const editMode = ref<'INS' | 'OVR'>('INS')
const showSettings = ref(false)

const langDropdownRef = ref<HTMLElement>()
const encodingDropdownRef = ref<HTMLElement>()
const eolDropdownRef = ref<HTMLElement>()
const indentDropdownRef = ref<HTMLElement>()
const settingsDropdownRef = ref<HTMLElement>()

const editorFontSize = ref(14)
const editorFontFamily = ref('"JetBrains Mono", "Fira Code", Consolas, monospace')
const editorWordWrap = ref(true)
const editorMinimap = ref(true)
const editorLineNumbers = ref(true)
const editorFontLigatures = ref(true)
const editorFolding = ref(true)
const editorMatchBrackets = ref(true)
const editorAutoIndent = ref(true)
const editorFormatOnPaste = ref(true)
const editorScrollBeyond = ref(false)
const editorRenderWhitespace = ref('selection')
const editorCursorStyle = ref('line')
const editorCursorBlinking = ref(true)
const editorRuler = ref(0)
const editorRenderIndentGuides = ref(true)

const languageOptions = [
  { id: 'plaintext', label: 'Plain Text' },
  { id: 'sql', label: 'SQL' },
  { id: 'javascript', label: 'JavaScript' },
  { id: 'typescript', label: 'TypeScript' },
  { id: 'json', label: 'JSON' },
  { id: 'html', label: 'HTML' },
  { id: 'css', label: 'CSS' },
  { id: 'python', label: 'Python' },
  { id: 'java', label: 'Java' },
  { id: 'c', label: 'C' },
  { id: 'cpp', label: 'C++' },
  { id: 'csharp', label: 'C#' },
  { id: 'go', label: 'Go' },
  { id: 'rust', label: 'Rust' },
  { id: 'ruby', label: 'Ruby' },
  { id: 'php', label: 'PHP' },
  { id: 'xml', label: 'XML' },
  { id: 'yaml', label: 'YAML' },
  { id: 'markdown', label: 'Markdown' },
  { id: 'shell', label: 'Shell Script' },
  { id: 'graphql', label: 'GraphQL' },
]

const encodingOptions = [
  'UTF-8',
  'UTF-8 with BOM',
  'UTF-16 LE',
  'UTF-16 BE',
  'GBK',
  'GB2312',
  'ISO-8859-1',
  'Windows-1252',
]

const eolOptions = [
  { id: 'LF', label: 'LF (Unix)' },
  { id: 'CRLF', label: 'CRLF (Windows)' },
]

const indentModeOptions = [
  { id: 'spaces', label: '空格缩进' },
  { id: 'tabs', label: '制表符缩进' },
]

const indentSizeOptions = [1, 2, 4, 8]

const indentLabel = computed(() =>
  indentMode.value === 'spaces' ? `Spaces: ${indentSize.value}` : `Tab Size: ${indentSize.value}`
)

function toggleDropdown(name: string) {
  if (name === 'settings') {
    showSettings.value = !showSettings.value
    if (name !== activeDropdown.value) {
      activeDropdown.value = name
    } else if (activeDropdown.value === name) {
      activeDropdown.value = null
    }
    return
  }
  activeDropdown.value = activeDropdown.value === name ? null : name
}

function closeDropdown() {
  activeDropdown.value = null
}

function switchLanguage(langId: string) {
  currentLanguage.value = langId
  if (editorModel.value) {
    monaco.editor.setModelLanguage(editorModel.value, langId)
  }
  closeDropdown()
}

function switchEncoding(enc: string) {
  currentEncoding.value = enc
  closeDropdown()
}

function switchEol(eol: string) {
  currentEol.value = eol
  if (editorModel.value) {
    const seq = eol === 'CRLF' ? monaco.editor.EndOfLineSequence.CRLF : monaco.editor.EndOfLineSequence.LF
    editorModel.value.pushEOL(seq)
  }
  closeDropdown()
}

function switchIndentMode(mode: 'spaces' | 'tabs') {
  indentMode.value = mode
  editor.value?.updateOptions({
    insertSpaces: mode === 'spaces',
  })
}

function switchIndentSize(size: number) {
  indentSize.value = size
  editor.value?.updateOptions({ tabSize: size })
}

function toggleEditMode() {
  editMode.value = editMode.value === 'INS' ? 'OVR' : 'INS'
}

function updateFileSize() {
  const value = getValue()
  const bytes = new TextEncoder().encode(value).length
  if (bytes < 1024) {
    fileSizeDisplay.value = `${bytes} B`
  } else if (bytes < 1024 * 1024) {
    fileSizeDisplay.value = `${(bytes / 1024).toFixed(1)} KB`
  } else {
    fileSizeDisplay.value = `${(bytes / (1024 * 1024)).toFixed(1)} MB`
  }
}

function onFontSizeChange(e: Event) {
  const val = Number((e.target as HTMLSelectElement).value)
  editorFontSize.value = val
  setFontSize(val)
}

function onFontFamilyChange(e: FocusEvent) {
  const val = (e.target as HTMLInputElement).value.trim()
  if (val) {
    editorFontFamily.value = val
    setFontFamily(val)
  }
}

function onTabSizeChange(e: Event) {
  const val = Number((e.target as HTMLSelectElement).value)
  indentSize.value = val
  setMonacoTabSize(val)
  editor.value?.updateOptions({ tabSize: val })
}

function onWordWrapToggle(e: Event) {
  const val = (e.target as HTMLInputElement).checked
  editorWordWrap.value = val
  setWordWrap(val)
}

function onMinimapToggle(e: Event) {
  const val = (e.target as HTMLInputElement).checked
  editorMinimap.value = val
  setMinimap(val)
}

function onLineNumbersToggle(e: Event) {
  const val = (e.target as HTMLInputElement).checked
  editorLineNumbers.value = val
  editor.value?.updateOptions({ lineNumbers: val ? 'on' : 'off' })
}

function onFontLigaturesToggle(e: Event) {
  const val = (e.target as HTMLInputElement).checked
  editorFontLigatures.value = val
  editor.value?.updateOptions({ fontLigatures: val })
}

function onFoldingToggle(e: Event) {
  const val = (e.target as HTMLInputElement).checked
  editorFolding.value = val
  editor.value?.updateOptions({ folding: val })
}

function onMatchBracketsToggle(e: Event) {
  const val = (e.target as HTMLInputElement).checked
  editorMatchBrackets.value = val
  editor.value?.updateOptions({ matchBrackets: val ? 'always' : 'never' })
}

function onAutoIndentToggle(e: Event) {
  const val = (e.target as HTMLInputElement).checked
  editorAutoIndent.value = val
  editor.value?.updateOptions({ autoIndent: val ? 'full' : 'none' })
}

function onFormatOnPasteToggle(e: Event) {
  const val = (e.target as HTMLInputElement).checked
  editorFormatOnPaste.value = val
  editor.value?.updateOptions({ formatOnPaste: val })
}

function onScrollBeyondToggle(e: Event) {
  const val = (e.target as HTMLInputElement).checked
  editorScrollBeyond.value = val
  editor.value?.updateOptions({ scrollBeyondLastLine: val })
}

function onRenderWhitespaceChange(e: Event) {
  const val = (e.target as HTMLSelectElement).value as 'selection' | 'none' | 'all' | 'boundary' | 'trailing'
  editorRenderWhitespace.value = val
  editor.value?.updateOptions({ renderWhitespace: val })
}

function onCursorStyleChange(e: Event) {
  const val = (e.target as HTMLSelectElement).value as 'line' | 'block' | 'underline'
  editorCursorStyle.value = val
  editor.value?.updateOptions({ cursorStyle: val })
}

function onCursorBlinkingToggle(e: Event) {
  const val = (e.target as HTMLInputElement).checked
  editorCursorBlinking.value = val
  editor.value?.updateOptions({ cursorBlinking: val ? 'blink' : 'solid' })
}

function onRulerChange(e: Event) {
  const val = Number((e.target as HTMLInputElement).value) || 0
  editorRuler.value = val
  editor.value?.updateOptions({ rulers: val > 0 ? [val] : [] })
}

function onIndentGuidesToggle(e: Event) {
  const val = (e.target as HTMLInputElement).checked
  editorRenderIndentGuides.value = val
  editor.value?.updateOptions({ renderIndentGuides: val })
}

watch(editorParams, (newParams) => {
  if (editorCreated.value) {
    if (newParams.initialValue !== undefined) {
      setValue(String(newParams.initialValue))
      updateFileSize()
    }
    if (newParams.language && newParams.language !== currentLanguage.value) {
      currentLanguage.value = String(newParams.language)
      updateLanguage(String(newParams.language))
    }
  }
})

watch(currentTheme, (newTheme) => {
  editor.value?.updateOptions({ theme: newTheme })
})

function handleClickOutside(e: MouseEvent) {
  if (activeDropdown.value || showSettings.value) {
    const target = e.target as Node
    const refs: Record<string, HTMLElement | undefined> = {
      lang: langDropdownRef.value,
      encoding: encodingDropdownRef.value,
      eol: eolDropdownRef.value,
      indent: indentDropdownRef.value,
      settings: settingsDropdownRef.value,
    }
    const key = activeDropdown.value
    if (key) {
      const activeRef = refs[key]
      if (activeRef && !activeRef.contains(target)) {
        closeDropdown()
        showSettings.value = false
      }
    }
  }
}

function focusEditor() {
  const ed = editor.value
  if (ed) {
    requestAnimationFrame(() => {
      ed.focus()
    })
  }
}

onMounted(async () => {
  await nextTick()

  monaco.editor.defineTheme(
    'rdata-dark',
    rdataDark as Parameters<typeof monaco.editor.defineTheme>[1]
  )
  monaco.editor.defineTheme(
    'rdata-light',
    rdataLight as Parameters<typeof monaco.editor.defineTheme>[1]
  )

  createEditor()
  editorCreated.value = true

  focusEditor()

  setupEditorEvents(
    () => {
      updateFileSize()
    },
    (info) => {
      if (info) {
        const linesText = info.lines === 1 ? '1 line' : `${info.lines} lines`
        const charsText = info.chars === 1 ? '1 char' : `${info.chars} chars`
        if (info.lines === 1) {
          selectedTextInfo.value = `${linesText} selected (${charsText})`
        } else {
          selectedTextInfo.value = `${linesText} selected`
        }
      }
    }
  )

  updateFileSize()

  document.addEventListener('click', handleClickOutside)
})

onBeforeUnmount(() => {
  document.removeEventListener('click', handleClickOutside)
  disposeEditor()
  disposeMonacoDisposables()
})

defineExpose({
  getValue,
  setValue,
  focus: focusEditor,
  focusEditor,
  updateLanguage,
})
</script>

<style scoped>
.code-editor-panel {
  display: flex;
  flex-direction: column;
  height: 100%;
  width: 100%;
  background: var(--bg-primary, #1e1f22);
  color: var(--text-primary, #e5e7eb);
  overflow: hidden;
}

.code-editor-body {
  flex: 1;
  position: relative;
  min-height: 0;
}

.monaco-container {
  width: 100%;
  height: 100%;
}

.editor-welcome-overlay {
  position: absolute;
  inset: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  pointer-events: none;
  z-index: 1;
}

.welcome-content {
  text-align: center;
  opacity: 0.45;
}

.welcome-icon {
  margin-bottom: 16px;
  color: var(--text-color-secondary, #888);
}

.welcome-content h3 {
  margin: 0 0 8px 0;
  font-size: 20px;
  font-weight: 600;
  color: var(--text-color-primary, #ccc);
}

.welcome-content p {
  margin: 0;
  font-size: 13px;
  color: var(--text-color-tertiary, #666);
}

.code-editor-statusbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  height: 22px;
  padding: 0 12px;
  background: var(--brand-accent, #007acc);
  color: #fff;
  font-size: 12px;
  user-select: none;
  flex-shrink: 0;
}

.status-left,
.status-right {
  display: flex;
  align-items: center;
  gap: 6px;
}

.status-item {
  white-space: nowrap;
  opacity: 0.9;
}

.status-clickable {
  cursor: pointer;
  padding: 0 4px;
  border-radius: 2px;
}

.status-clickable:hover {
  background: rgba(255, 255, 255, 0.12);
}

.status-clickable.active {
  background: rgba(255, 255, 255, 0.2);
}

.status-gear {
  display: flex;
  align-items: center;
  padding: 0 3px;
  border-radius: 2px;
}

.status-divider {
  width: 1px;
  height: 14px;
  background: rgba(255, 255, 255, 0.3);
  margin: 0 2px;
}

.status-cursor {
  cursor: pointer;
  padding: 0 4px;
  border-radius: 2px;
}

.status-cursor:hover {
  background: rgba(255, 255, 255, 0.12);
}

.status-file-size {
  opacity: 0.75;
}

.status-dropdown {
  position: relative;
}

.dropdown-menu {
  position: absolute;
  bottom: 100%;
  left: 0;
  margin-bottom: 4px;
  min-width: 180px;
  max-height: 320px;
  background: var(--bg-secondary, #2b2d30);
  border: 1px solid var(--border-color, #4a5458);
  border-radius: 4px;
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.36);
  z-index: 1000;
  overflow: hidden;
}

.dropdown-scroll {
  max-height: 300px;
  overflow-y: auto;
  padding: 4px 0;
}

.dropdown-option {
  display: flex;
  align-items: center;
  padding: 4px 12px;
  font-size: 12px;
  color: var(--text-primary, #e5e7eb);
  cursor: pointer;
  transition: background 0.1s;
}

.dropdown-option:hover {
  background: var(--hover-bg, rgba(255, 255, 255, 0.08));
}

.dropdown-option.active {
  background: var(--brand-accent, #007acc);
  color: #fff;
}

.option-check {
  width: 18px;
  flex-shrink: 0;
  font-size: 11px;
}

.dropdown-section-header {
  padding: 6px 12px 2px 12px;
  font-size: 11px;
  font-weight: 600;
  color: var(--text-muted, #6b7280);
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.dropdown-divider {
  height: 1px;
  background: var(--border-color, #4a5458);
  margin: 4px 0;
}

.dropdown-scroll::-webkit-scrollbar {
  width: 6px;
}

.dropdown-scroll::-webkit-scrollbar-thumb {
  background: rgba(255, 255, 255, 0.15);
  border-radius: 3px;
}

.dropdown-scroll::-webkit-scrollbar-track {
  background: transparent;
}

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

.settings-toggle input[type="checkbox"] {
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