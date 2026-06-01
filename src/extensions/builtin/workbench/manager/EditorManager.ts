import { EditorState } from '@codemirror/state'
import { EditorView } from '@codemirror/view'
import {
  ref,
  shallowRef,
  computed,
  markRaw,
  type Ref,
  type ShallowRef,
  type ComputedRef,
} from 'vue'

import { ShortcutManager } from '@/extensions/builtin/workbench/manager/ShortcutManager'
import {
  PANEL_PREFIX_EDITOR,
  type OpenFileInfo,
  type OpenFileParams,
  type ResultSetCreateParams,
  type ResultSetMetadata,
  type EditorInstance,
} from '@/extensions/builtin/workbench/types/editor-types'
import {
  sendPopoutTransfer,
  listenMergeTransfer,
  listenWindowReady,
  listenStateSync,
  type StateSyncPayload,
} from '@/extensions/builtin/workbench/ui/composables/useCrossWindow'
import { useEditorPersistence } from '@/extensions/builtin/workbench/ui/composables/useEditorPersistence'
import { useEditorRecovery } from '@/extensions/builtin/workbench/ui/composables/useEditorRecovery'
import { useEditorRuntime } from '@/extensions/builtin/workbench/ui/stores/editor-runtime-store'

interface DockviewPanelHandle {
  api: { close(): void; setTitle(t: string): void; setVisible(v: boolean): void }
  focus(): void
  id: string
  group?: { id: string }
}

interface DockviewGroupHandle {
  api: { close(): void; setVisible(v: boolean): void; moveTo(p: { group: string }): void }
  id: string
  panels: Array<{ id: string }>
}

interface DockviewApiFacade {
  addPanel(opts: Record<string, unknown>): void
  getPanel(id: string): DockviewPanelHandle | undefined
  getGroup(id: string): DockviewGroupHandle | undefined
  movePanelOrGroup(panelId: string, opts: Record<string, unknown>): void
}

type CodeMirrorStateJSON = Record<string, unknown>
type ApiResponseJSON = Record<string, unknown>

let dockviewApi: DockviewApiFacade | null = null

const {
  runtime,
  startExecution,
  finishExecution,
  cancelExecution: cancelRuntime,
} = useEditorRuntime()

const openFiles: ShallowRef<Map<string, OpenFileInfo>> = shallowRef(new Map())
const activeFilePath: Ref<string | null> = ref(null)
const editorRef: ShallowRef<EditorView | null> = shallowRef(null)
const isInitialized: Ref<boolean> = ref(false)
const crossWindowUnlisteners: (() => void)[] = []
const tabGroupId: Ref<string | null> = ref(null)

const MAX_RESULT_SETS = 5
const SQL_LOG_TRUNCATE_LENGTH = 500
const DEFAULT_POPOUT_GEOMETRY = { x: 200, y: 200, width: 800, height: 400 } as const

const editorInstances = new Map<string, EditorInstance>()
const panelGroupMap = new Map<string, string>()
const savedStates = new Map<string, EditorState>()
let resultIdCounter = 0

const activeFileInfo: ComputedRef<OpenFileInfo | null> = computed(() => {
  if (!activeFilePath.value) return null
  return openFiles.value.get(activeFilePath.value) ?? null
})

function sanitize(s: string): string {
  return s.replace(/[^a-zA-Z0-9_-]/g, '_')
}

function filePanelId(filePath: string): string {
  return `${PANEL_PREFIX_EDITOR}${sanitize(filePath)}`
}

function syncDirtyToTab(filePath: string, isDirty: boolean): void {
  const info = openFiles.value.get(filePath)
  if (!info) return
  const title = isDirty ? `\u2022 ${info.fileName}` : info.fileName
  try {
    dockviewApi?.getPanel(filePanelId(filePath))?.api.setTitle(title)
  } catch {
    console.warn('[EditorManager] dockview setTitle failed')
  }
}

function instanceId(filePath: string, groupId: string): string {
  return `${groupId}::${filePath}`
}

function getEditorView(filePath: string): EditorView | undefined {
  for (const [, inst] of editorInstances) {
    if (inst.filePath === filePath && inst.writable) return inst.view
  }
  return undefined
}

function getEditorViewForPanel(panelId: string): EditorView | undefined {
  const gid = panelGroupMap.get(panelId)
  if (!gid) return undefined
  const fp = EditorManager.panelIdToFilePath(panelId)
  if (!fp) return undefined
  const id = instanceId(fp, gid)
  const inst = editorInstances.get(id)
  return inst?.view
}

function saveEditorState(filePath: string, state: EditorState): void {
  savedStates.set(filePath, state)
}

function getSavedState(filePath: string): EditorState | undefined {
  return savedStates.get(filePath)
}

function onKeydown(e: KeyboardEvent): void {
  ShortcutManager.handleKeydown(e)
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
    return runtime.isExecuting
  },
  get lastExecutionTime() {
    return runtime.lastExecutionTime
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
    const pid = filePanelId(filePath)
    const groupId = dockviewApi?.getPanel(pid)?.group?.id ?? filePath
    const id = instanceId(filePath, groupId)
    panelGroupMap.set(pid, groupId)
    const writable = EditorManager.isPrimaryInstance(filePath, id)
    const inst: EditorInstance = {
      instanceId: id,
      filePath,
      groupId,
      view: markRaw(ed),
      state: null,
      writable,
    }
    editorInstances.set(id, inst)

    const info = openFiles.value.get(filePath)
    if (info) {
      if (!info.primaryInstanceId) info.primaryInstanceId = id
      else if (id !== info.primaryInstanceId && !info.readonlyInstanceIds.includes(id)) {
        info.readonlyInstanceIds.push(id)
      }
      if (inst.state) {
        info.states.set(id, inst.state)
      }
    }
  },

  unregisterFileEditor(filePath: string): void {
    const pid = filePanelId(filePath)
    const groupId = panelGroupMap.get(pid) ?? filePath
    const id = instanceId(filePath, groupId)
    const inst = editorInstances.get(id)
    if (inst) {
      saveEditorState(filePath, inst.view.state)
      try {
        inst.view.destroy()
      } catch {
        console.warn('[EditorManager] view.destroy failed during unregister')
      }
    }
    editorInstances.delete(id)
    panelGroupMap.delete(pid)
  },

  isPrimaryInstance(filePath: string, instanceId?: string): boolean {
    for (const [id, inst] of editorInstances) {
      if (inst.filePath === filePath && inst.writable) {
        if (instanceId === undefined) return false
        if (id !== instanceId) return false
      }
    }
    return true
  },

  isFileOpenElsewhere(filePath: string, excludeGroupId?: string): boolean {
    for (const [, inst] of editorInstances) {
      if (inst.filePath === filePath) {
        if (excludeGroupId && inst.groupId === excludeGroupId) continue
        return true
      }
    }
    return false
  },

  onPanelActivated(panelId: string): void {
    const fp = EditorManager.panelIdToFilePath(panelId)
    if (!fp) return
    activeFilePath.value = fp
    const ed = getEditorViewForPanel(panelId) ?? getEditorView(fp)
    if (ed) editorRef.value = ed
    for (const [path, info] of openFiles.value) {
      if (info.resultPanelIds.length === 0) continue
      for (const rpId of info.resultPanelIds) {
        try {
          dockviewApi?.getPanel(rpId)?.api.setVisible(path === fp)
        } catch {
          console.warn('[EditorManager] result panel visibility update failed')
        }
      }
    }
  },

  updatePanelGroup(panelId: string, groupId: string): void {
    panelGroupMap.set(panelId, groupId)
  },

  onPanelUndocked(panelId: string): void {
    const fp = EditorManager.panelIdToFilePath(panelId)
    if (!fp) return
    const info = openFiles.value.get(fp)
    if (!info) return
    const inst = editorInstances.get(instanceId(fp, ''))
    if (inst?.view) {
      try {
        const stateJSON = inst.view.state.toJSON() as Record<string, unknown>
        sendPopoutTransfer({
          filePath: fp,
          fileName: info.fileName,
          language: info.language,
          content: inst.view.state.doc.toString(),
          stateJSON,
          connectionId: info.connectionId,
          databaseName: info.databaseName,
        })
      } catch {
        console.warn('[EditorManager] serialization failed during popout')
      }
    }
  },

  setActiveResultIndex(filePath: string, index: number): void {
    const info = openFiles.value.get(filePath)
    if (!info) return
    if (info.resultSets.length === 0) {
      info.activeResultIndex = -1
      return
    }
    info.activeResultIndex = Math.max(0, Math.min(index, info.resultSets.length - 1))
    openFiles.value = new Map(openFiles.value)
  },

  detachResultPanel(panelId: string): void {
    let filePath: string | null = null
    for (const [fp, info] of openFiles.value) {
      if (info.resultPanelIds.includes(panelId)) {
        filePath = fp
        info.resultPanelIds = info.resultPanelIds.filter(id => id !== panelId)
        info.detachedResultIds.push(panelId)
        if (info.activeResultIndex >= info.resultSets.length) {
          info.activeResultIndex = Math.max(-1, info.resultSets.length - 1)
        }
        break
      }
    }
    if (!filePath) return

    try {
      dockviewApi?.movePanelOrGroup(panelId, {
        group: `detached_${panelId}`,
        position: { direction: 'right' },
        floating: DEFAULT_POPOUT_GEOMETRY,
      })
    } catch {
      console.warn('[EditorManager] dockview movePanelOrGroup failed during detach')
    }

    openFiles.value = new Map(openFiles.value)
  },

  attachResultPanel(panelId: string, filePath: string): void {
    const info = openFiles.value.get(filePath)
    if (!info) return

    info.resultPanelIds.push(panelId)
    info.detachedResultIds = info.detachedResultIds.filter(id => id !== panelId)

    const gid = tabGroupId.value
    if (gid) {
      try {
        dockviewApi?.movePanelOrGroup(panelId, {
          group: gid,
          position: { direction: 'below' },
        })
      } catch {
        console.warn('[EditorManager] dockview movePanelOrGroup failed during attach')
      }
    }

    openFiles.value = new Map(openFiles.value)
  },

  createResultSet(filePath: string, data: ResultSetCreateParams): string {
    const info = openFiles.value.get(filePath)
    if (!info) {
      console.warn(`[EditorManager] File not found: ${filePath}`)
      return ''
    }

    while (info.resultSets.length >= MAX_RESULT_SETS) {
      const oldest = info.resultSets[0]
      if (oldest) {
        EditorManager.removeResultSet(filePath, oldest.id)
      } else {
        break
      }
    }

    resultIdCounter++
    const resultId = `result_${sanitize(filePath)}_${resultIdCounter}`

    const metadata: ResultSetMetadata = {
      id: resultId,
      title: data.error ? '错误' : `结果${info.resultSets.length + 1}`,
      columns: data.columns,
      totalRowCount: data.totalRows,
      elapsedMs: data.elapsedMs,
      affectedRows: data.affectedRows,
      messages: data.error ? `错误: ${data.error}` : `${data.totalRows} 行, ${data.elapsedMs}ms`,
      sql: data.sql,
      timestamp: Date.now(),
      dataSource: 'memory',
      duckdbTable: null,
      rows: data.rows,
    }

    info.resultSets.push(metadata)

    const panelIdentifier = `panel_${resultId}`
    const gid = tabGroupId.value
    dockviewApi?.addPanel({
      id: panelIdentifier,
      component: 'fileResultPanel',
      title: metadata.title,
      position: { referenceGroup: gid, direction: 'below' },
      params: { resultSetId: metadata.id, columns: metadata.columns, rows: data.rows, filePath },
    })
    info.resultPanelIds.push(panelIdentifier)

    EditorManager.setActiveResultIndex(filePath, info.resultSets.length - 1)
    return resultId
  },

  removeResultSet(filePath: string, resultSetId: string): void {
    const info = openFiles.value.get(filePath)
    if (!info) return
    const idx = info.resultSets.findIndex(rs => rs.id === resultSetId)
    if (idx < 0) return
    const panelId = `panel_${resultSetId}`
    dockviewApi?.getPanel(panelId)?.api.close()
    info.resultSets.splice(idx, 1)
    info.resultPanelIds = info.resultPanelIds.filter(id => id !== panelId)
    if (info.activeResultIndex >= info.resultSets.length) {
      EditorManager.setActiveResultIndex(filePath, Math.max(-1, info.resultSets.length - 1))
    } else {
      openFiles.value = new Map(openFiles.value)
    }
  },

  init(api: unknown): void {
    if (isInitialized.value) return
    dockviewApi = api as DockviewApiFacade
    isInitialized.value = true
    window.addEventListener('keydown', onKeydown)

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
    for (const [, inst] of editorInstances) {
      try {
        inst.view.destroy()
      } catch {
        console.warn('[EditorManager] view.destroy failed during destroy')
      }
    }
    editorInstances.clear()
    panelGroupMap.clear()
    savedStates.clear()
    openFiles.value = new Map()
    activeFilePath.value = null
    editorRef.value = null
    tabGroupId.value = null
    for (const unlisten of crossWindowUnlisteners) {
      try {
        unlisten()
      } catch {
        console.warn('[EditorManager] unlisten failed during destroy')
      }
    }
    crossWindowUnlisteners.length = 0
    dockviewApi = null
    isInitialized.value = false
  },

  openFile(params: OpenFileParams): void {
    if (!isInitialized.value) {
      console.warn('[EditorManager] Not initialized')
      return
    }
    if (openFiles.value.has(params.filePath)) {
      EditorManager.switchToFile(params.filePath)
      if (EditorManager.isFileOpenElsewhere(params.filePath)) {
        // eslint-disable-next-line no-console
        console.debug(`[EditorManager] File already open in another group: ${params.filePath}`)
      }
      return
    }

    const pid = filePanelId(params.filePath)
    const isFirstFile = openFiles.value.size === 0
    const refGroup = tabGroupId.value

    const info: OpenFileInfo = {
      filePath: params.filePath,
      fileName: params.fileName,
      language: params.language,
      type: params.type ?? 'file',
      isDirty: false,
      connectionId: params.connectionId ?? '',
      databaseName: params.databaseName ?? '',
      resultSets: [],
      activeResultIndex: -1,
      resultPanelIds: [],
      detachedResultIds: [],
      states: new Map(),
      primaryInstanceId: null,
      readonlyInstanceIds: [],
    }

    openFiles.value.set(params.filePath, info)

    dockviewApi?.addPanel({
      id: pid,
      component: 'editorPanel',
      title: params.fileName,
      position:
        isFirstFile || !refGroup
          ? { direction: 'right' }
          : { referenceGroup: refGroup, direction: 'within' },
      params: {
        filePath: params.filePath,
        fileName: params.fileName,
        language: params.language,
        content: params.sql,
      },
    })

    if (isFirstFile) {
      const captureGroup = (retry: number) => {
        if (retry > 20) return
        const panel = dockviewApi?.getPanel(pid)
        if (panel?.group?.id) {
          tabGroupId.value = panel.group.id
        } else {
          setTimeout(() => captureGroup(retry + 1), 50)
        }
      }
      setTimeout(() => captureGroup(0), 50)
    }

    openFiles.value = new Map(openFiles.value)
  },

  closeFile(filePath: string): void {
    const info = openFiles.value.get(filePath)
    if (!info) return

    const pid = filePanelId(filePath)

    for (const rpId of info.resultPanelIds) {
      dockviewApi?.getPanel(rpId)?.api.close()
    }

    const idToRemove: string[] = []
    for (const [id, inst] of editorInstances) {
      if (inst.filePath === filePath) {
        inst.state = inst.view.state
        info.states.set(id, inst.view.state)
        try {
          inst.view.destroy()
        } catch {
          console.warn('[EditorManager] view.destroy failed during closeFile')
        }
        idToRemove.push(id)
      }
    }
    for (const id of idToRemove) {
      editorInstances.delete(id)
    }
    try {
      const { draft } = useEditorPersistence(pid, filePath)
      draft.remove()
    } catch {
      console.warn('[EditorManager] draft.remove failed during closeFile')
    }

    const wasActive = activeFilePath.value === filePath

    openFiles.value.delete(filePath)
    dockviewApi?.getPanel(pid)?.api.close()

    if (openFiles.value.size === 0) {
      openFiles.value = new Map()
      activeFilePath.value = null
      editorRef.value = null
      tabGroupId.value = null
      savedStates.clear()
      return
    }

    openFiles.value = new Map(openFiles.value)

    if (wasActive) {
      const next = openFiles.value.keys().next().value as string | undefined
      if (next) EditorManager.switchToFile(next)
      else activeFilePath.value = null
    }
  },

  panelIdToFilePath(panelId: string): string | null {
    for (const [fp] of openFiles.value) {
      if (filePanelId(fp) === panelId) return fp
    }
    return null
  },

  findCenterGroup(): string | undefined {
    return tabGroupId.value ?? undefined
  },

  switchToFile(filePath: string): void {
    const info = openFiles.value.get(filePath)
    if (!info) {
      console.warn(`[EditorManager] File not found: ${filePath}`)
      return
    }
    if (activeFilePath.value === filePath) return

    if (activeFilePath.value) {
      const prevEd = getEditorView(activeFilePath.value)
      if (prevEd) {
        const prevState = prevEd.state
        saveEditorState(activeFilePath.value, prevState)
        const prevInfo = openFiles.value.get(activeFilePath.value)
        if (prevInfo?.primaryInstanceId) {
          prevInfo.states.set(prevInfo.primaryInstanceId, prevState)
        }
      }
    }

    activeFilePath.value = filePath
    const ed = getEditorView(filePath)
    if (ed) {
      const saved = info.states.get(info.primaryInstanceId ?? '')
      if (saved) ed.setState(saved)
      editorRef.value = ed
    }
    const pid = filePanelId(filePath)
    dockviewApi?.getPanel(pid)?.focus()
  },

  async saveCurrentFile(): Promise<void> {
    const info = activeFileInfo.value
    if (!info) return
    const ed = getEditorView(info.filePath)
    if (!ed) return
    const content = ed.state.doc.toString()
    try {
      const { saveScratchpadFile } =
        await import('@/extensions/builtin/scratchpad/infrastructure/api/scratchpad-api')
      await saveScratchpadFile(info.filePath, content)
    } catch (e) {
      console.warn('[EditorManager] Save:', e)
    }
    info.isDirty = false
    syncDirtyToTab(info.filePath, false)
    openFiles.value = new Map(openFiles.value)
    const pid = filePanelId(info.filePath)
    const { draft } = useEditorPersistence(pid, info.filePath)
    draft.remove()
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
    EditorManager.openFile({
      filePath: path,
      fileName,
      language: 'sql',
      sql: '',
      type: 'file',
      connectionId: connectionId ?? '',
      databaseName: databaseName ?? '',
    })
  },

  openAnalysisPanel(connectionId?: string, databaseName?: string): void {
    const ts = Date.now()
    const fileName = `分析-${ts}`
    const filePath = `__analysis__${ts}`
    EditorManager.openFile({
      filePath,
      fileName,
      language: 'sql',
      sql: '',
      type: 'analysis',
      connectionId: connectionId ?? '',
      databaseName: databaseName ?? '',
    })
  },

  async popoutActiveFile(): Promise<void> {
    const info = activeFileInfo.value
    if (!info) return
    const ed = getEditorView(info.filePath)
    if (!ed) return
    try {
      const stateJSON = ed.state.toJSON() as unknown as CodeMirrorStateJSON
      const content = ed.state.doc.toString()
      sendPopoutTransfer({
        filePath: info.filePath,
        fileName: info.fileName,
        language: info.language,
        content,
        stateJSON,
        connectionId: info.connectionId,
        databaseName: info.databaseName,
      })
    } catch {
      console.warn('[EditorManager] popout transfer failed')
    }
  },

  setupCrossWindowListeners(): void {
    listenWindowReady(() => {
      /* popout window is ready */
    })
      .then(unlisten => {
        crossWindowUnlisteners.push(unlisten)
      })
      .catch(() => {
        console.warn('[EditorManager] listenStateSync setup failed')
      })

    listenMergeTransfer(payload => {
      EditorManager.openFile({
        filePath: payload.filePath,
        fileName: payload.filePath.split(/[/\\]/).pop() ?? payload.filePath,
        language: 'sql',
        sql: payload.content,
        type: 'file',
      })
    })
      .then(unlisten => {
        crossWindowUnlisteners.push(unlisten)
      })
      .catch(() => {
        console.warn('[EditorManager] listenMergeTransfer setup failed')
      })

    listenStateSync((payload: StateSyncPayload) => {
      const ed = getEditorView(payload.filePath)
      if (!ed) return
      const currentDoc = ed.state.doc.toString()
      if (payload.content !== currentDoc) {
        ed.dispatch({
          changes: { from: 0, to: ed.state.doc.length, insert: payload.content },
        })
      }
      const info = openFiles.value.get(payload.filePath)
      if (info) {
        info.isDirty = payload.isDirty
        openFiles.value = new Map(openFiles.value)
      }
    })
      .then(unlisten => {
        crossWindowUnlisteners.push(unlisten)
      })
      .catch(() => {
        console.warn('[EditorManager] listenWindowReady setup failed')
      })
  },

  executeCurrentSQL(): Promise<void> {
    const info = activeFileInfo.value
    if (!info) return Promise.resolve()
    const ed = getEditorView(info.filePath)
    if (!ed) return Promise.resolve()
    const sel = ed.state.selection.main
    const sql = sel.empty ? ed.state.doc.toString() : ed.state.doc.sliceString(sel.from, sel.to)
    if (!sql.trim()) return Promise.resolve()
    startExecution()
    return (async () => {
      try {
        const { executeSql } = await import('@/extensions/builtin/query/ui/services/query')
        const result = await executeSql(sql, info.connectionId || undefined)
        const typed = result as unknown as ApiResponseJSON
        if (typed.success) {
          EditorManager.createResultSet(info.filePath, {
            columns: (typed.columns as string[]) ?? [],
            rows: (typed.rows as unknown[][]) ?? [],
            totalRows: (typed.totalRows as number) ?? (typed.rowCount as number) ?? 0,
            elapsedMs: (typed.elapsedMs as number) ?? (typed.elapsed_ms as number) ?? 0,
            affectedRows: (typed.affectedRows as number) ?? (typed.affected_rows as number) ?? 0,
            sql: sql.slice(0, SQL_LOG_TRUNCATE_LENGTH),
            error: null,
          })
        } else {
          EditorManager.createResultSet(info.filePath, {
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
        EditorManager.createResultSet(info.filePath, {
          columns: [],
          rows: [],
          totalRows: 0,
          elapsedMs: 0,
          affectedRows: 0,
          sql: sql.slice(0, SQL_LOG_TRUNCATE_LENGTH),
          error: e instanceof Error ? e.message : 'Unknown error',
        })
      } finally {
        finishExecution()
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
      EditorManager.setActiveResultIndex(info.filePath, prevIndex)
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
    startExecution()
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
      finishExecution()
    }
  },

  cancelExecution(): void {
    cancelRuntime()
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
    for (const [, info] of openFiles.value) {
      const idx = info.resultPanelIds.indexOf(panelId)
      const detIdx = info.detachedResultIds.indexOf(panelId)
      if (idx < 0 && detIdx < 0) continue

      const rsIdx = info.resultSets.findIndex(rs => `panel_${rs.id}` === panelId)
      if (rsIdx >= 0) {
        info.resultSets[rsIdx] = { ...info.resultSets[rsIdx], title: newTitle }
      }

      try {
        dockviewApi?.getPanel(panelId)?.api.setTitle(newTitle)
      } catch {
        console.warn('[EditorManager] dockview setTitle failed during rename')
      }

      openFiles.value = new Map(openFiles.value)
      break
    }
  },
}
