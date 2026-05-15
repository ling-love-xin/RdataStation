import * as monaco from 'monaco-editor'
import { ref, computed, type ShallowRef } from 'vue'

export interface EditorSettingsState {
  fontSize: number
  fontFamily: string
  wordWrap: boolean
  minimap: boolean
  lineNumbers: boolean
  fontLigatures: boolean
  folding: boolean
  matchBrackets: boolean
  autoIndent: boolean
  formatOnPaste: boolean
  scrollBeyond: boolean
  renderWhitespace: string
  cursorStyle: string
  cursorBlinking: boolean
  ruler: number
  renderIndentGuides: boolean
}

export interface EditorSettingsHandlers {
  setFontSize: (size: number) => void
  setFontFamily: (family: string) => void
  setTabSize: (size: number) => void
  setWordWrap: (wrap: boolean) => void
  setMinimap: (show: boolean) => void
  setLineNumbers: (show: boolean) => void
  setFontLigatures: (enable: boolean) => void
  setFolding: (enable: boolean) => void
  setMatchBrackets: (enable: boolean) => void
  setAutoIndent: (enable: boolean) => void
  setFormatOnPaste: (enable: boolean) => void
  setScrollBeyond: (enable: boolean) => void
  setRenderWhitespace: (mode: string) => void
  setCursorStyle: (style: string) => void
  setCursorBlinking: (enable: boolean) => void
  setRuler: (column: number) => void
  setIndentGuides: (enable: boolean) => void
}

export interface MonacoEditorSetters {
  setFontSize: (size: number) => void
  setWordWrap: (wrap: boolean) => void
  setMinimap: (show: boolean) => void
  setFontFamily: (family: string) => void
  setTabSize: (size: number) => void
}

export function useEditorSettings(
  editor: ShallowRef<monaco.editor.IStandaloneCodeEditor | null>,
  monacoSetters: MonacoEditorSetters
) {
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

  const settingsState = computed<EditorSettingsState>(() => ({
    fontSize: editorFontSize.value,
    fontFamily: editorFontFamily.value,
    wordWrap: editorWordWrap.value,
    minimap: editorMinimap.value,
    lineNumbers: editorLineNumbers.value,
    fontLigatures: editorFontLigatures.value,
    folding: editorFolding.value,
    matchBrackets: editorMatchBrackets.value,
    autoIndent: editorAutoIndent.value,
    formatOnPaste: editorFormatOnPaste.value,
    scrollBeyond: editorScrollBeyond.value,
    renderWhitespace: editorRenderWhitespace.value,
    cursorStyle: editorCursorStyle.value,
    cursorBlinking: editorCursorBlinking.value,
    ruler: editorRuler.value,
    renderIndentGuides: editorRenderIndentGuides.value,
  }))

  const handlers: EditorSettingsHandlers = {
    setFontSize(size: number) {
      editorFontSize.value = size
      monacoSetters.setFontSize(size)
    },
    setFontFamily(family: string) {
      editorFontFamily.value = family
      monacoSetters.setFontFamily(family)
    },
    setTabSize(size: number) {
      monacoSetters.setTabSize(size)
      editor.value?.updateOptions({ tabSize: size })
    },
    setWordWrap(wrap: boolean) {
      editorWordWrap.value = wrap
      monacoSetters.setWordWrap(wrap)
    },
    setMinimap(show: boolean) {
      editorMinimap.value = show
      monacoSetters.setMinimap(show)
    },
    setLineNumbers(show: boolean) {
      editorLineNumbers.value = show
      editor.value?.updateOptions({ lineNumbers: show ? 'on' : 'off' })
    },
    setFontLigatures(enable: boolean) {
      editorFontLigatures.value = enable
      editor.value?.updateOptions({ fontLigatures: enable })
    },
    setFolding(enable: boolean) {
      editorFolding.value = enable
      editor.value?.updateOptions({ folding: enable })
    },
    setMatchBrackets(enable: boolean) {
      editorMatchBrackets.value = enable
      editor.value?.updateOptions({ matchBrackets: enable ? 'always' : 'never' })
    },
    setAutoIndent(enable: boolean) {
      editorAutoIndent.value = enable
      editor.value?.updateOptions({ autoIndent: enable ? 'full' : 'none' })
    },
    setFormatOnPaste(enable: boolean) {
      editorFormatOnPaste.value = enable
      editor.value?.updateOptions({ formatOnPaste: enable })
    },
    setScrollBeyond(enable: boolean) {
      editorScrollBeyond.value = enable
      editor.value?.updateOptions({ scrollBeyondLastLine: enable })
    },
    setRenderWhitespace(mode: string) {
      editorRenderWhitespace.value = mode
      editor.value?.updateOptions({
        renderWhitespace: mode as 'selection' | 'none' | 'all' | 'boundary' | 'trailing',
      })
    },
    setCursorStyle(style: string) {
      editorCursorStyle.value = style
      editor.value?.updateOptions({
        cursorStyle: style as 'line' | 'block' | 'underline',
      })
    },
    setCursorBlinking(enable: boolean) {
      editorCursorBlinking.value = enable
      editor.value?.updateOptions({ cursorBlinking: enable ? 'blink' : 'solid' })
    },
    setRuler(column: number) {
      editorRuler.value = column
      editor.value?.updateOptions({ rulers: column > 0 ? [column] : [] })
    },
    setIndentGuides(enable: boolean) {
      editorRenderIndentGuides.value = enable
      editor.value?.updateOptions({ renderIndentGuides: enable })
    },
  }

  return {
    settingsState,
    handlers,
  }
}