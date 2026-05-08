import * as monaco from 'monaco-editor'
import { ref, shallowRef, watch, onBeforeUnmount, type Ref } from 'vue'

import { useEditorPersistence } from '@/extensions/builtin/workbench/ui/composables/useEditorPersistence'

export interface MonacoEditorOptions {
  containerRef: Ref<HTMLElement | undefined>
  panelId: string
  initialValue?: string
  language?: string
  theme?: string
}

export function useMonacoEditor(options: MonacoEditorOptions) {
  const {
    containerRef,
    panelId,
    initialValue = '',
    language = 'sql',
    theme = 'rdata-dark',
  } = options

  const editor = shallowRef<monaco.editor.IStandaloneCodeEditor | null>(null)
  const editorModel = shallowRef<monaco.editor.ITextModel | null>(null)
  const showWelcome = ref(true)
  const cursorPosition = ref('Ln 1, Col 1')
  const selectedTextInfo = ref('')
  const editorReady = ref(false)
  const monacoDisposables: monaco.IDisposable[] = []

  const { draft } = useEditorPersistence(panelId)

  function createEditor(): void {
    const el = containerRef.value
    if (!el) return

    const restoredValue = draft.load() || initialValue
    if (restoredValue) {
      showWelcome.value = false
    }

    editor.value = monaco.editor.create(el, {
      value: restoredValue,
      language,
      theme,
      automaticLayout: true,
      minimap: { enabled: true, scale: 1, showSlider: 'mouseover' },
      fontSize: 14,
      fontFamily: '"JetBrains Mono", "Fira Code", Consolas, monospace',
      fontLigatures: true,
      wordWrap: 'on',
      lineNumbers: 'on',
      renderWhitespace: 'selection',
      folding: true,
      foldingStrategy: 'auto',
      showFoldingControls: 'always',
      matchBrackets: 'always',
      autoIndent: 'full',
      formatOnPaste: true,
      suggestOnTriggerCharacters: true,
      snippetSuggestions: 'top',
      tabSize: 2,
      insertSpaces: true,
      scrollBeyondLastLine: false,
      colorDecorators: false,
      links: false,
      renderValidationDecorations: 'off',
      overviewRulerLanes: 0,
      hideCursorInOverviewRuler: true,
      overviewRulerBorder: false,
      padding: { top: 8, bottom: 8 },
    })

    editorModel.value = editor.value.getModel()
    editorReady.value = true
  }

  function setupEditorEvents(
    onContentChange?: (value: string) => void,
    onSelectionChange?: (info: { lines: number; chars: number } | null) => void
  ): void {
    const ed = editor.value
    if (!ed) return

    monacoDisposables.push(
      ed.onDidChangeModelContent(() => {
        const value = ed.getValue()
        showWelcome.value = value.trim().length === 0
        draft.save(value)
        onContentChange?.(value)
      })
    )

    monacoDisposables.push(
      ed.onDidChangeCursorPosition(e => {
        cursorPosition.value = `Ln ${e.position.lineNumber}, Col ${e.position.column}`
      })
    )

    monacoDisposables.push(
      ed.onDidChangeCursorSelection(e => {
        const model = ed.getModel()
        if (!model) return

        const selection = e.selection
        if (selection.isEmpty()) {
          selectedTextInfo.value = ''
          onSelectionChange?.(null)
          return
        }

        const selectedText = model.getValueInRange(selection)
        const lines = selection.endLineNumber - selection.startLineNumber + 1
        const chars = selectedText.length

        const linesText = lines === 1 ? '1 line' : `${lines} lines`
        const charsText = chars === 1 ? '1 char' : `${chars} chars`

        if (lines === 1) {
          selectedTextInfo.value = `${linesText} selected (${charsText})`
        } else {
          selectedTextInfo.value = `${linesText} selected`
        }

        onSelectionChange?.({ lines, chars })
      })
    )
  }

  function setupEditorCommands(handlers: Record<string, () => void>): void {
    const ed = editor.value
    if (!ed) return

    Object.entries(handlers).forEach(([keybinding, handler]) => {
      try {
        const keyCode = Number(keybinding)
        if (!Number.isNaN(keyCode)) {
          ed.addAction({
            id: `custom-action-${keybinding}`,
            label: `Custom action ${keybinding}`,
            keybindings: [keyCode],
            run: handler,
          })
        }
      } catch (error) {
        console.warn(`[useMonacoEditor] Failed to register command ${keybinding}:`, error)
      }
    })
  }

  function getValue(): string {
    return editor.value?.getValue() ?? ''
  }

  function setValue(value: string): void {
    editor.value?.setValue(value)
    showWelcome.value = value.trim().length === 0
  }

  function getSelectedText(): string {
    const ed = editor.value
    if (!ed) return ''
    const selection = ed.getSelection()
    if (!selection || selection.isEmpty()) return ''
    const model = ed.getModel()
    if (!model) return ''
    return model.getValueInRange(selection)
  }

  function insertText(text: string): void {
    const ed = editor.value
    if (!ed) return

    const selection = ed.getSelection()
    if (selection) {
      ed.executeEdits('insert', [
        {
          range: selection.isEmpty()
            ? new monaco.Range(
                selection.startLineNumber,
                selection.startColumn,
                selection.startLineNumber,
                selection.startColumn
              )
            : selection,
          text,
        },
      ])
    }
  }

  function focus(): void {
    editor.value?.focus()
  }

  function updateLanguage(newLanguage: string): void {
    if (editorModel.value) {
      monaco.editor.setModelLanguage(editorModel.value, newLanguage)
    }
  }

  function disposeMonacoDisposables(): void {
    monacoDisposables.forEach(d => d.dispose())
    monacoDisposables.length = 0
  }

  function disposeEditor(): void {
    if (editor.value) {
      const model = editor.value.getModel()
      editor.value.dispose()
      if (model) {
        model.dispose()
      }
      editor.value = null
      editorModel.value = null
    }
  }

  watch(
    () => theme,
    newTheme => {
      editor.value?.updateOptions({ theme: newTheme })
    }
  )

  onBeforeUnmount(() => {
    disposeEditor()
    disposeMonacoDisposables()
  })

  return {
    editor,
    editorModel,
    editorReady,
    showWelcome,
    cursorPosition,
    selectedTextInfo,
    createEditor,
    setupEditorEvents,
    setupEditorCommands,
    getValue,
    setValue,
    getSelectedText,
    insertText,
    focus,
    updateLanguage,
    disposeEditor,
    disposeMonacoDisposables,
    monacoDisposables,
  }
}
