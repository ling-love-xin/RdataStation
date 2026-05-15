<template>
  <div class="code-editor-statusbar">
    <div class="status-left">
      <SaveStatusIndicator
        v-if="filePath"
        :status="saveStatus"
        :last-save-time="lastSaveTime"
      />
      <span v-if="filePath" class="status-divider" />
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
        <EditorSettingsPopup
          v-if="showSettings || activeDropdown === 'settings'"
          :settings="settings"
          :handlers="settingsHandlers"
          @close="closeSettings"
        />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { Settings } from 'lucide-vue-next'
import * as monaco from 'monaco-editor'
import { ref, computed, onMounted, onBeforeUnmount } from 'vue'

import type { EditorSettingsState, EditorSettingsHandlers } from '@/extensions/builtin/workbench/ui/composables/useEditorSettings'
import type { SaveStatus } from '@/extensions/builtin/workbench/ui/composables/useFileSave'

import EditorSettingsPopup from './EditorSettingsPopup.vue'
import SaveStatusIndicator from './SaveStatusIndicator.vue'

interface Props {
  filePath: string
  fileName: string
  saveStatus: SaveStatus
  lastSaveTime: number | null
  selectedTextInfo: string | null
  cursorPosition: string
  fileSizeDisplay: string
  initialLanguage: string
  editorModel: monaco.editor.ITextModel | null
  editor: monaco.editor.IStandaloneCodeEditor | null
  settings: EditorSettingsState
  settingsHandlers: EditorSettingsHandlers
}

const props = defineProps<Props>()

const emit = defineEmits<{
  'language-change': [langId: string]
}>()

const currentLanguage = ref(props.initialLanguage)
const currentEncoding = ref('UTF-8')
const currentEol = ref('LF')
const indentMode = ref<'spaces' | 'tabs'>('spaces')
const indentSize = ref(2)
const editMode = ref<'INS' | 'OVR'>('INS')
const activeDropdown = ref<string | null>(null)
const showSettings = ref(false)

const langDropdownRef = ref<HTMLElement>()
const encodingDropdownRef = ref<HTMLElement>()
const eolDropdownRef = ref<HTMLElement>()
const indentDropdownRef = ref<HTMLElement>()
const settingsDropdownRef = ref<HTMLElement>()

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
  if (props.editorModel) {
    monaco.editor.setModelLanguage(props.editorModel, langId)
  }
  emit('language-change', langId)
  closeDropdown()
}

function switchEncoding(enc: string) {
  currentEncoding.value = enc
  closeDropdown()
}

function switchEol(eol: string) {
  currentEol.value = eol
  if (props.editorModel) {
    const seq = eol === 'CRLF' ? monaco.editor.EndOfLineSequence.CRLF : monaco.editor.EndOfLineSequence.LF
    props.editorModel.pushEOL(seq)
  }
  closeDropdown()
}

function switchIndentMode(mode: 'spaces' | 'tabs') {
  indentMode.value = mode
  props.editor?.updateOptions({
    insertSpaces: mode === 'spaces',
  })
}

function switchIndentSize(size: number) {
  indentSize.value = size
  props.editor?.updateOptions({ tabSize: size })
}

function toggleEditMode() {
  editMode.value = editMode.value === 'INS' ? 'OVR' : 'INS'
}

function closeSettings() {
  showSettings.value = false
  closeDropdown()
}

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

onMounted(() => {
  document.addEventListener('click', handleClickOutside)
})

onBeforeUnmount(() => {
  document.removeEventListener('click', handleClickOutside)
})
</script>

<style scoped>
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
</style>