import { EditorState } from '@codemirror/state'
import { EditorView } from '@codemirror/view'
import { markRaw } from 'vue'

import { ShortcutManager } from '@/extensions/builtin/workbench/manager/ShortcutManager'
import {
  type OpenFileParams,
  type ResultSetCreateParams,
} from '@/extensions/builtin/workbench/types/editor-types'
import { useEditorRecovery } from '@/extensions/builtin/workbench/ui/composables/useEditorRecovery'

import {
  setupCrossWindowListeners as setupCrossWindowListenersImpl,
  popoutActiveFile as popoutActiveFileImpl,
  onPanelUndocked as onPanelUndockedImpl,
} from './cross-window-service'
import {
  openFiles,
  activeFilePath,
  activeFileInfo,
  editorRef,
  isInitialized,
  dockviewApi,
  crossWindowUnlisteners,
  tabGroupId,
  editorInstances,
  runtimeState,
  setDockviewApi,
  clearDockviewApi,
  resetResultIdCounter,
  type DockviewApiFacade,
} from './editor-state'
import {
  openFile as openFileImpl,
  closeFile as closeFileImpl,
  closeFileChecked as closeFileCheckedImpl,
  newFile as newFileImpl,
  saveCurrentFileToDisk,
  saveFileAs as saveFileAsImpl,
  checkExternalFileChanges,
  switchToFile as switchToFileImpl,
  onPanelActivated as onPanelActivatedImpl,
  findCenterGroup as findCenterGroupImpl,
} from './file-manager'
import {
  getEditorView,
  getSavedState,
  saveEditorState,
  registerFileEditor,
  unregisterFileEditor,
  isPrimaryInstance,
  isFileOpenElsewhere,
  updatePanelGroup,
  panelIdToFilePath,
  clearAllInstances,
} from './instance-service'
import {
  createResultSet as createResultSetImpl,
  removeResultSet as removeResultSetImpl,
  setActiveResultIndex as setActiveResultIndexImpl,
  detachResultPanel as detachResultPanelImpl,
  attachResultPanel as attachResultPanelImpl,
  renameResultSet as renameResultSetImpl,
} from './result-set-manager'

type CodeMirrorStateJSON = Record<string, unknown>
type ApiResponseJSON = Record<string, unknown>

const SQL_LOG_TRUNCATE_LENGTH = 500

function onKeydown(e: KeyboardEvent): void {
  ShortcutManager.handleKeydown(e)
}

function onCanvasClick(): void {
  ShortcutManager.setActiveScope('editor')
}

export const EditorManager = {
  get openFiles() {
    return openFiles.value
  },
  get activeFilePath() {
    return activeFilePath.value
  },
  get activeFileInfo() {
    return activeFileInfo.value
  },
  get editor() {
    return editorRef.value
  },
  get isExecuting() {
    return runtimeState.isExecuting
  },
  get lastExecutionTime() {
    return runtimeState.lastExecutionTime
  },
  get isInitialized() {
    return isInitialized.value
  },
  get dockviewApi() {
    return dockviewApi
  },

  getSavedStateForFile(filePath: string): EditorState | undefined {
    return getSavedState(filePath)
  },

  saveEditorStateForFile(filePath: string, state: EditorState): void {
    saveEditorState(filePath, state)
  },

  hasRecoveryData(): boolean {
    const { hasRecoveryData } = useEditorRecovery()
    return hasRecoveryData()
  },

  loadRecoverySnapshots(): {
    filePath: string
    fileName: string
    language: string
    isDirty: boolean
  }[] {
    const { loadSnapshots } = useEditorRecovery()
    return loadSnapshots().map(s => ({
      filePath: s.filePath,
      fileName: s.fileName,
      language: s.language,
      isDirty: s.isDirty,
    }))
  },

  clearRecovery(): void {
    const { clearRecovery } = useEditorRecovery()
    clearRecovery()
  },

  setEditor(ed: EditorView): void {
    editorRef.value = markRaw(ed)
  },

  registerFileEditor(filePath: string, ed: EditorView): void {
    registerFileEditor(filePath, ed)
  },

  unregisterFileEditor(filePath: string): void {
    unregisterFileEditor(filePath)
  },

  isPrimaryInstance(filePath: string, instanceId?: string): boolean {
    return isPrimaryInstance(filePath, instanceId)
  },

  isFileOpenElsewhere(filePath: string, excludeGroupId?: string): boolean {
    return isFileOpenElsewhere(filePath, excludeGroupId)
  },

  onPanelActivated(panelId: string): void {
    onPanelActivatedImpl(panelId)
  },

  updatePanelGroup(panelId: string, groupId: string): void {
    updatePanelGroup(panelId, groupId)
  },

  onPanelUndocked(panelId: string): void {
    onPanelUndockedImpl(panelId)
  },

  setActiveResultIndex(filePath: string, index: number): void {
    setActiveResultIndexImpl(filePath, index)
  },

  detachResultPanel(panelId: string): void {
    detachResultPanelImpl(panelId)
  },

  attachResultPanel(panelId: string, filePath: string): void {
    attachResultPanelImpl(panelId, filePath)
  },

  createResultSet(filePath: string, data: ResultSetCreateParams): string {
    return createResultSetImpl(filePath, data)
  },

  removeResultSet(filePath: string, resultSetId: string): void {
    removeResultSetImpl(filePath, resultSetId)
  },

  init(api: unknown): void {
    if (isInitialized.value) return
    setDockviewApi(api as DockviewApiFacade)
    isInitialized.value = true
    window.addEventListener('keydown', onKeydown)
    const canvas = document.querySelector('.dv-canvas')
    if (canvas) canvas.addEventListener('click', onCanvasClick)

    ShortcutManager.register('Ctrl+S', 'editor', () => EditorManager.saveCurrentFile(), '保存')
    ShortcutManager.register(
      'Ctrl+Enter',
      'editor',
      () => EditorManager.executeCurrentSQL(),
      '执行'
    )
    ShortcutManager.register('Ctrl+/', 'editor', () => EditorManager.toggleComment(), '注释')
  },

  destroy(): void {
    window.removeEventListener('keydown', onKeydown)
    const canvas = document.querySelector('.dv-canvas')
    if (canvas) canvas.removeEventListener('click', onCanvasClick)

    const { saveSnapshot } = useEditorRecovery()
    for (const [_id, inst] of editorInstances) {
      try {
        const stateJSON = inst.view.state.toJSON() as unknown as CodeMirrorStateJSON
        const info = openFiles.value.get(inst.filePath)
        saveSnapshot(inst.filePath, stateJSON, {
          fileName: info?.fileName ?? inst.filePath.split(/[/\\]/).pop() ?? inst.filePath,
          language: info?.language ?? 'plaintext',
          isDirty: info?.isDirty ?? false,
          scrollTop: inst.view.scrollDOM.scrollTop,
          scrollLeft: inst.view.scrollDOM.scrollLeft,
        })
      } catch {
        console.warn('[EditorManager] recovery saveSnapshot failed during destroy')
      }
    }
    clearAllInstances()
    openFiles.value = new Map()
    activeFilePath.value = null
    editorRef.value = null
    tabGroupId.value = null
    resetResultIdCounter()
    for (const unlisten of crossWindowUnlisteners) {
      try {
        unlisten()
      } catch {
        console.warn('[EditorManager] unlisten failed during destroy')
      }
    }
    crossWindowUnlisteners.length = 0
    clearDockviewApi()
    isInitialized.value = false
  },

  openFile(params: OpenFileParams): void {
    openFileImpl(params)
    // 记录文件初始修改时间，用于外部变更检测
    import('@tauri-apps/plugin-fs')
      .then(({ stat }) =>
        stat(params.filePath)
          .then(m => {
            const info = openFiles.value.get(params.filePath)
            if (info) {
              info.lastModifiedAt = (m as { mtime?: { ms: number } }).mtime?.ms ?? Date.now()
            }
          })
          .catch(() => {
            /* 不可达文件，不影响流程 */
          })
      )
      .catch(() => {
        /* plugin-fs 不可用 */
      })
  },

  closeFile(filePath: string): void {
    closeFileImpl(filePath)
  },

  async closeFileChecked(filePath: string): Promise<boolean> {
    return closeFileCheckedImpl(filePath)
  },

  newFile(language?: string, content?: string): string {
    return newFileImpl(language, content)
  },

  async saveFileAs(filePath: string): Promise<void> {
    return saveFileAsImpl(filePath)
  },

  async checkExternalFileChanges(): Promise<void> {
    return checkExternalFileChanges()
  },

  panelIdToFilePath(panelId: string): string | null {
    return panelIdToFilePath(panelId)
  },

  findCenterGroup(): string | undefined {
    return findCenterGroupImpl()
  },

  switchToFile(filePath: string): void {
    switchToFileImpl(filePath)
  },

  async saveCurrentFile(): Promise<void> {
    const info = activeFileInfo.value
    if (!info) return
    try {
      await saveCurrentFileToDisk(info.filePath)
    } catch (e) {
      console.warn('[EditorManager] Save:', e)
    }
    const { removeSnapshot } = useEditorRecovery()
    removeSnapshot(info.filePath)
  },

  async openNewQuery(connectionId?: string, databaseName?: string): Promise<void> {
    const { createScratchpadEntry, listScratchpadFiles } =
      await import('@/extensions/builtin/scratchpad/infrastructure/api/scratchpad-api')
    const ts = Date.now()
    const fileName = `Untitled-${ts}.sql`
    try {
      await listScratchpadFiles()
    } catch {
      console.warn('[EditorManager] listScratchpadFiles failed')
    }
    const entry = await createScratchpadEntry(fileName, false)
    const path = (entry as { path?: string }).path || fileName
    openFileImpl({
      filePath: path,
      fileName,
      language: 'sql',
      sql: '',
      type: 'file',
      connectionId: connectionId ?? '',
      databaseName: databaseName ?? '',
    })
  },

  async openAnalysisPanel(connectionId?: string, databaseName?: string): Promise<void> {
    const { createScratchpadEntry, listScratchpadFiles } =
      await import('@/extensions/builtin/scratchpad/infrastructure/api/scratchpad-api')
    const ts = Date.now()
    const fileName = `分析-${ts}.sql`
    try {
      await listScratchpadFiles()
    } catch {
      console.warn('[EditorManager] listScratchpadFiles failed')
    }
    const entry = await createScratchpadEntry(fileName, false)
    const path = (entry as { path?: string }).path || fileName
    openFileImpl({
      filePath: path,
      fileName,
      language: 'sql',
      sql: '',
      type: 'analysis',
      connectionId: connectionId ?? '',
      databaseName: databaseName ?? '',
    })
  },

  async popoutActiveFile(): Promise<void> {
    popoutActiveFileImpl()
  },

  setupCrossWindowListeners(): void {
    setupCrossWindowListenersImpl()
  },

  executeCurrentSQL(): Promise<void> {
    const info = activeFileInfo.value
    if (!info) return Promise.resolve()
    const ed = getEditorView(info.filePath)
    if (!ed) return Promise.resolve()
    const sel = ed.state.selection.main
    const sql = sel.empty ? ed.state.doc.toString() : ed.state.doc.sliceString(sel.from, sel.to)
    if (!sql.trim()) return Promise.resolve()
    runtimeState.startExecution()
    return (async () => {
      try {
        const { executeSql } = await import('@/extensions/builtin/query/ui/services/query')
        const result = await executeSql(sql, info.connectionId || undefined)
        const typed = result as unknown as ApiResponseJSON
        if (typed.success) {
          createResultSetImpl(info.filePath, {
            columns: (typed.columns as string[]) ?? [],
            rows: (typed.rows as unknown[][]) ?? [],
            totalRows: (typed.totalRows as number) ?? (typed.rowCount as number) ?? 0,
            elapsedMs: (typed.elapsedMs as number) ?? (typed.elapsed_ms as number) ?? 0,
            affectedRows: (typed.affectedRows as number) ?? (typed.affected_rows as number) ?? 0,
            sql: sql.slice(0, SQL_LOG_TRUNCATE_LENGTH),
            error: null,
          })
        } else {
          createResultSetImpl(info.filePath, {
            columns: [],
            rows: [],
            totalRows: 0,
            elapsedMs: (typed.elapsedMs as number) ?? (typed.elapsed_ms as number) ?? 0,
            affectedRows: 0,
            sql: sql.slice(0, SQL_LOG_TRUNCATE_LENGTH),
            error: (typed.error as string) ?? 'Unknown error',
          })
        }
      } catch (e) {
        console.error('[EditorManager] Exec:', e)
        createResultSetImpl(info.filePath, {
          columns: [],
          rows: [],
          totalRows: 0,
          elapsedMs: 0,
          affectedRows: 0,
          sql: sql.slice(0, SQL_LOG_TRUNCATE_LENGTH),
          error: e instanceof Error ? e.message : 'Unknown error',
        })
      } finally {
        runtimeState.finishExecution()
      }
    })()
  },

  async executeNewTabSQL(): Promise<void> {
    const info = activeFileInfo.value
    if (!info) {
      await EditorManager.executeCurrentSQL()
      return
    }
    const prevIndex = info.activeResultIndex
    await EditorManager.executeCurrentSQL()
    if (prevIndex >= 0 && prevIndex !== info.activeResultIndex) {
      setActiveResultIndexImpl(info.filePath, prevIndex)
    }
  },

  async executeDuckDBAccelerated(): Promise<void> {
    const info = activeFileInfo.value
    if (!info) return
    const ed = getEditorView(info.filePath)
    if (!ed) return
    const sel = ed.state.selection.main
    const sql = sel.empty ? ed.state.doc.toString() : ed.state.doc.sliceString(sel.from, sel.to)
    if (!sql.trim()) return
    runtimeState.startExecution()
    try {
      const { executeDuckDBAccelerated, generateAttachName, rewriteDuckDBSQL } =
        await import('@/extensions/builtin/workbench/services/sql-editor-service')
      const { appDataDir } = await import('@tauri-apps/api/path')
      const attachName = generateAttachName(info.connectionId || 'remote')
      await executeDuckDBAccelerated({
        sql: rewriteDuckDBSQL(sql, attachName),
        connId: info.connectionId,
        dataDir: await appDataDir(),
      })
    } catch (e) {
      console.error('[EditorManager] DuckDB:', e)
    } finally {
      runtimeState.finishExecution()
    }
  },

  cancelExecution(): void {
    runtimeState.cancelExecution()
  },

  async formatSQL(): Promise<void> {
    const info = activeFileInfo.value
    if (!info) return
    const ed = getEditorView(info.filePath)
    if (!ed) return
    try {
      const { formatSql } =
        await import('@/extensions/builtin/workbench/services/sql-editor-service')
      const r = await formatSql(ed.state.doc.toString())
      if (r) {
        ed.dispatch({ changes: { from: 0, to: ed.state.doc.length, insert: r } })
      }
    } catch (e) {
      console.error('[EditorManager] Format:', e)
    }
  },

  async validateSQL(): Promise<void> {
    const info = activeFileInfo.value
    if (!info) return
    const ed = getEditorView(info.filePath)
    if (!ed) return
    try {
      const { validateSql } =
        await import('@/extensions/builtin/workbench/services/sql-editor-service')
      const { setEditorDiagnostics } =
        await import('@/extensions/builtin/workbench/services/cm-sql-extensions')
      const markers = await validateSql(ed.state.doc.toString())
      setEditorDiagnostics(
        ed,
        markers.map(m => ({
          from: 0,
          to: 0,
          severity: m.severity as 'error' | 'warning' | 'info',
          message: m.message,
        }))
      )
    } catch {
      console.warn('[EditorManager] SQL validation failed')
    }
  },

  toggleComment(): void {
    const info = activeFileInfo.value
    if (!info) return
    const ed = getEditorView(info.filePath)
    if (!ed) return
    ed.focus()
    import('@codemirror/commands')
      .then(({ toggleComment }) => {
        toggleComment(ed)
      })
      .catch(() => {
        console.warn('[EditorManager] toggleComment extension unavailable')
      })
  },

  renameResultSet(panelId: string, newTitle: string): void {
    renameResultSetImpl(panelId, newTitle)
  },
}