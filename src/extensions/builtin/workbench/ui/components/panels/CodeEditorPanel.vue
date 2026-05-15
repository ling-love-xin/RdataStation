<template>
  <div class="code-editor-panel">
    <div class="code-editor-body">
      <div
        ref="editorContainer"
        class="monaco-container"
        @mousedown="focusEditor"
        @keydown.enter.stop
        @keydown.ctrl.s.prevent="handleCtrlS"
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

    <CodeEditorStatusbar
      :file-path="filePath"
      :file-name="fileName"
      :save-status="saveStatus"
      :last-save-time="lastSaveTime"
      :selected-text-info="selectedTextInfo"
      :cursor-position="cursorPosition"
      :file-size-display="fileSizeDisplay"
      :initial-language="editorLanguage"
      :editor-model="editorModel"
      :editor="editor"
      :settings="settingsState"
      :settings-handlers="settingsHandlers"
      @language-change="onLanguageChange"
    />
  </div>
</template>

<script setup lang="ts">
import 'monaco-editor/esm/vs/basic-languages/sql/sql.contribution'
import { Code } from 'lucide-vue-next'
import * as monaco from 'monaco-editor'
import { ref, computed, watch, onMounted, onBeforeUnmount, nextTick } from 'vue'

import { useEditorSettings } from '@/extensions/builtin/workbench/ui/composables/useEditorSettings'
import { useFileSave } from '@/extensions/builtin/workbench/ui/composables/useFileSave'
import { useMonacoEditor } from '@/extensions/builtin/workbench/ui/composables/useMonacoEditor'
import { useTabDirtyState } from '@/extensions/builtin/workbench/ui/composables/useTabDirtyState'
import { useUiStore } from '@/shared/stores/ui'
import { rdataDark, rdataLight } from '@/shared/styles/monaco-theme'

import CodeEditorStatusbar from './CodeEditorStatusbar.vue'

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
const filePath = computed(() => String(editorParams.value.filePath || ''))

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

let editorSaveHandler: (() => void) | null = null

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

const { settingsState, handlers: settingsHandlers } = useEditorSettings(editor, {
  setFontSize,
  setWordWrap,
  setMinimap,
  setFontFamily,
  setTabSize: setMonacoTabSize,
})

const {
  saveStatus,
  lastSaveTime,
  saveError: _saveError,
  isDirty,
  manualSave,
  markDirty,
  _setAutoSaveInterval: _setSaveInterval,
} = useFileSave({
  filePath,
  getContent: () => getValue(),
  autoSaveInterval: 30000,
  maxRetries: 3,
  retryDelay: 2000,
  onSaveSuccess: () => {
    updateFileSize()
  },
  onSaveError: (error) => {
    console.warn('[CodeEditorPanel] Save failed:', error)
  },
})

const { setDirty: setTabDirty, clearPanel: clearTabDirty } = useTabDirtyState()

watch([isDirty, panelId], ([dirty, pid]) => {
  if (pid) {
    setTabDirty(pid, dirty)
  }
})

const fileSizeDisplay = ref('0 B')

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

function onLanguageChange(langId: string) {
  updateLanguage(langId)
}

watch(editorParams, (newParams) => {
  if (editorCreated.value) {
    if (newParams.initialValue !== undefined) {
      setValue(String(newParams.initialValue))
      updateFileSize()
    }
    if (newParams.language) {
      updateLanguage(String(newParams.language))
    }
  }
})

watch(currentTheme, (newTheme) => {
  editor.value?.updateOptions({ theme: newTheme })
})

function focusEditor() {
  const ed = editor.value
  if (ed) {
    requestAnimationFrame(() => {
      ed.focus()
    })
  }
}

async function handleCtrlS() {
  if (filePath.value) {
    await manualSave()
  }
}

function handleBeforeUnload(e: BeforeUnloadEvent) {
  if (isDirty.value && filePath.value) {
    e.preventDefault()
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
      markDirty()
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

  window.addEventListener('beforeunload', handleBeforeUnload)

  const handleEditorSave = () => {
    if (filePath.value) {
      manualSave()
    }
  }
  window.addEventListener('sql-editor-save', handleEditorSave)
  editorSaveHandler = handleEditorSave
})

onBeforeUnmount(() => {
  clearTabDirty(panelId.value)
  window.removeEventListener('beforeunload', handleBeforeUnload)
  if (editorSaveHandler) {
    window.removeEventListener('sql-editor-save', editorSaveHandler)
    editorSaveHandler = null
  }
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
</style>