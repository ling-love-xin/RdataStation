import * as monaco from "monaco-editor"
import { ref, shallowRef, computed, markRaw, type Ref, type ShallowRef, type ComputedRef } from "vue"

import { ShortcutManager } from "@/extensions/builtin/workbench/manager/ShortcutManager"
import {
  type OpenFileInfo,
  type OpenFileParams,
  type ResultSetCreateParams,
  type ResultSetMetadata,
} from "@/extensions/builtin/workbench/types/editor-types"
import { useEditorPersistence } from "@/extensions/builtin/workbench/ui/composables/useEditorPersistence"
import { useEditorRuntime } from "@/extensions/builtin/workbench/ui/stores/editor-runtime-store"

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

let dockviewApi: DockviewApiFacade | null = null

const { runtime, startExecution, finishExecution, cancelExecution: cancelRuntime } = useEditorRuntime()

const openFiles: ShallowRef<Map<string, OpenFileInfo>> = shallowRef(new Map())
const activeFilePath: Ref<string | null> = ref(null)
const editorRef: ShallowRef<monaco.editor.IStandaloneCodeEditor | null> = shallowRef(null)
const isInitialized: Ref<boolean> = ref(false)
const tabGroupId: Ref<string | null> = ref(null)

const MAX_RESULT_SETS = 5

const fileEditors = new Map<string, monaco.editor.IStandaloneCodeEditor>()
let resultIdCounter = 0

const activeFileInfo: ComputedRef<OpenFileInfo | null> = computed(() => {
  if (!activeFilePath.value) return null
  return openFiles.value.get(activeFilePath.value) ?? null
})

function sanitize(s: string): string { return s.replace(/[^a-zA-Z0-9_-]/g, "_") }

function filePanelId(filePath: string): string { return `panel_editor_${sanitize(filePath)}` }

function syncDirtyToTab(filePath: string, isDirty: boolean): void {
  const info = openFiles.value.get(filePath)
  if (!info) return
  const title = isDirty ? `\u2022 ${info.fileName}` : info.fileName
  try { dockviewApi?.getPanel(filePanelId(filePath))?.api.setTitle(title) } catch { /* */ }
}

function onKeydown(e: KeyboardEvent): void { ShortcutManager.handleKeydown(e) }

export const EditorManager = {
  get openFiles() { return openFiles.value },
  get activeFilePath() { return activeFilePath.value },
  get activeFileInfo() { return activeFileInfo.value },
  get editor() { return editorRef.value },
  get isExecuting() { return runtime.isExecuting },
  get lastExecutionTime() { return runtime.lastExecutionTime },
  get isInitialized() { return isInitialized.value },
  get dockviewApi() { return dockviewApi },

  setEditor(ed: monaco.editor.IStandaloneCodeEditor): void {
    editorRef.value = markRaw(ed)
  },

  registerFileEditor(filePath: string, ed: monaco.editor.IStandaloneCodeEditor): void {
    fileEditors.set(filePath, markRaw(ed))
  },

  unregisterFileEditor(filePath: string): void {
    const ed = fileEditors.get(filePath)
    if (ed) { try { ed.dispose() } catch { /* */ } }
    fileEditors.delete(filePath)
  },

  onPanelActivated(panelId: string): void {
    const fp = EditorManager.panelIdToFilePath(panelId)
    if (!fp) return
    activeFilePath.value = fp
    const ed = fileEditors.get(fp)
    if (ed) editorRef.value = ed
    for (const [path, info] of openFiles.value) {
      if (info.resultPanelIds.length === 0) continue
      for (const rpId of info.resultPanelIds) {
        try {
          dockviewApi?.getPanel(rpId)?.api.setVisible(path === fp)
        } catch { /* dockview */ }
      }
    }
  },

  setActiveResultIndex(filePath: string, index: number): void {
    const info = openFiles.value.get(filePath)
    if (!info) return
    if (info.resultSets.length === 0) { info.activeResultIndex = -1; return }
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
        floating: { x: 200, y: 200, width: 800, height: 400 },
      })
    } catch { /* dockview */ }

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
      } catch { /* dockview */ }
    }

    openFiles.value = new Map(openFiles.value)
  },

  createResultSet(filePath: string, data: ResultSetCreateParams): string {
    const info = openFiles.value.get(filePath)
    if (!info) { console.warn(`[EditorManager] File not found: ${filePath}`); return '' }

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
    } else { openFiles.value = new Map(openFiles.value) }
  },

  init(api: unknown): void {
    if (isInitialized.value) return
    dockviewApi = api as DockviewApiFacade
    isInitialized.value = true
    window.addEventListener("keydown", onKeydown)

    ShortcutManager.register('Ctrl+S', 'editor', () => EditorManager.saveCurrentFile(), '保存')
    ShortcutManager.register('Ctrl+Enter', 'editor', () => EditorManager.executeCurrentSQL(), '执行')
    ShortcutManager.register('Ctrl+/', 'editor', () => EditorManager.toggleComment(), '注释')
  },

  destroy(): void {
    window.removeEventListener("keydown", onKeydown)
    for (const [, ed] of fileEditors) { try { ed.dispose() } catch { /* */ } }
    fileEditors.clear()
    for (const [, info] of openFiles.value) { try { info.model.dispose() } catch { /* */ } }
    openFiles.value = new Map()
    activeFilePath.value = null
    editorRef.value = null
    tabGroupId.value = null
    dockviewApi = null
    isInitialized.value = false
  },

  openFile(params: OpenFileParams): void {
    if (!isInitialized.value) { console.warn("[EditorManager] Not initialized"); return }
    if (openFiles.value.has(params.filePath)) { EditorManager.switchToFile(params.filePath); return }

    let model: monaco.editor.ITextModel
    try { model = monaco.editor.createModel(params.sql, params.language) } catch (e) { console.error("[EditorManager] Model:", e); return }
    markRaw(model)

    const pid = filePanelId(params.filePath)
    const isFirstFile = openFiles.value.size === 0
    const refGroup = tabGroupId.value

    const info: OpenFileInfo = {
      model,
      filePath: params.filePath, fileName: params.fileName,
      language: params.language, type: params.type ?? "file",
      isDirty: false, connectionId: params.connectionId ?? "",
      databaseName: params.databaseName ?? "",
      resultSets: [], activeResultIndex: -1, resultPanelIds: [], detachedResultIds: [],
    }

    openFiles.value.set(params.filePath, info)

    dockviewApi?.addPanel({
      id: pid,
      component: "editorPanel",
      title: params.fileName,
      position: isFirstFile || !refGroup
        ? { direction: "right" }
        : { referenceGroup: refGroup, direction: "within" },
      params: { filePath: params.filePath, fileName: params.fileName, language: params.language },
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

    model.onDidChangeContent(() => {
      const fi = openFiles.value.get(params.filePath)
      if (!fi) return
      fi.isDirty = model.getValue() !== params.sql
      syncDirtyToTab(params.filePath, fi.isDirty)
      if (fi.isDirty) { const { draft } = useEditorPersistence(pid, params.filePath); draft.save(model.getValue()) }
      openFiles.value = new Map(openFiles.value)
    })
  },

  closeFile(filePath: string): void {
    const info = openFiles.value.get(filePath)
    if (!info) return

    const pid = filePanelId(filePath)

    for (const rpId of info.resultPanelIds) { dockviewApi?.getPanel(rpId)?.api.close() }
    try { info.model.dispose() } catch { /* */ }
    EditorManager.unregisterFileEditor(filePath)
    try {
      const { draft } = useEditorPersistence(pid, filePath)
      draft.remove()
    } catch { /* persistence */ }

    const wasActive = activeFilePath.value === filePath

    openFiles.value.delete(filePath)
    dockviewApi?.getPanel(pid)?.api.close()

    if (openFiles.value.size === 0) {
      openFiles.value = new Map()
      activeFilePath.value = null
      editorRef.value = null
      tabGroupId.value = null
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
    if (!info) { console.warn(`[EditorManager] File not found: ${filePath}`); return }
    if (activeFilePath.value === filePath) return

    activeFilePath.value = filePath
    const ed = fileEditors.get(filePath)
    if (ed) editorRef.value = ed
    const pid = filePanelId(filePath)
    dockviewApi?.getPanel(pid)?.focus()
  },

  async saveCurrentFile(): Promise<void> {
    const info = activeFileInfo.value
    if (!info) return
    const ed = fileEditors.get(info.filePath)
    if (!ed) return
    const model = ed.getModel()
    if (!model) return
    const content = model.getValue()
    try {
      const { saveScratchpadFile } = await import("@/extensions/builtin/scratchpad/infrastructure/api/scratchpad-api")
      await saveScratchpadFile(info.filePath, content)
    } catch (e) { console.warn("[EditorManager] Save:", e) }
    info.isDirty = false
    syncDirtyToTab(info.filePath, false)
    openFiles.value = new Map(openFiles.value)
    const pid = filePanelId(info.filePath)
    const { draft } = useEditorPersistence(pid, info.filePath)
    draft.remove()
  },

  async openNewQuery(connectionId?: string, databaseName?: string): Promise<void> {
    const { createScratchpadEntry, listScratchpadFiles } = await import("@/extensions/builtin/scratchpad/infrastructure/api/scratchpad-api")
    const ts = Date.now(); const fileName = `Untitled-${ts}.sql`
    try { await listScratchpadFiles() } catch { /* */ }
    const entry = await createScratchpadEntry(fileName, false)
    const path = (entry as { path?: string }).path || fileName
    EditorManager.openFile({ filePath: path, fileName, language: "sql", sql: "", type: "file", connectionId: connectionId ?? "", databaseName: databaseName ?? "" })
  },

  openAnalysisPanel(connectionId?: string, databaseName?: string): void {
    const ts = Date.now()
    const fileName = `分析-${ts}`
    const filePath = `__analysis__${ts}`
    EditorManager.openFile({
      filePath, fileName, language: "sql", sql: "",
      type: "analysis",
      connectionId: connectionId ?? "",
      databaseName: databaseName ?? "",
    })
  },

  executeCurrentSQL(): Promise<void> {
    const info = activeFileInfo.value
    if (!info) return Promise.resolve()
    const ed = fileEditors.get(info.filePath)
    if (!ed) return Promise.resolve()
    const model = ed.getModel()
    if (!model) return Promise.resolve()
    const sel = ed.getSelection()
    const sql = sel && !sel.isEmpty() ? model.getValueInRange(sel) : model.getValue()
    if (!sql.trim()) return Promise.resolve()
    startExecution()
    return (async () => {
      try {
        const { executeSql } = await import("@/extensions/builtin/query/ui/services/query")
        const result = await executeSql(sql, info.connectionId || undefined)
        const typed = result as Record<string, unknown>
        if (typed.success) {
          EditorManager.createResultSet(info.filePath, {
            columns: (typed.columns as string[]) ?? [],
            rows: (typed.rows as unknown[][]) ?? [],
            totalRows: (typed.totalRows as number) ?? (typed.rowCount as number) ?? 0,
            elapsedMs: (typed.elapsedMs as number) ?? (typed.elapsed_ms as number) ?? 0,
            affectedRows: (typed.affectedRows as number) ?? (typed.affected_rows as number) ?? 0,
            sql: sql.slice(0, 500), error: null,
          })
        } else {
          EditorManager.createResultSet(info.filePath, {
            columns: [], rows: [], totalRows: 0,
            elapsedMs: (typed.elapsedMs as number) ?? (typed.elapsed_ms as number) ?? 0,
            affectedRows: 0, sql: sql.slice(0, 500),
            error: (typed.error as string) ?? "Unknown error",
          })
        }
      } catch (e) {
        console.error("[EditorManager] Exec:", e)
        EditorManager.createResultSet(info.filePath, {
          columns: [], rows: [], totalRows: 0, elapsedMs: 0, affectedRows: 0,
          sql: sql.slice(0, 500), error: e instanceof Error ? e.message : "Unknown error",
        })
      } finally { finishExecution() }
    })()
  },

  async executeNewTabSQL(): Promise<void> {
    const info = activeFileInfo.value
    if (!info) { await EditorManager.executeCurrentSQL(); return }
    const prevIndex = info.activeResultIndex
    await EditorManager.executeCurrentSQL()
    if (prevIndex >= 0 && prevIndex !== info.activeResultIndex) {
      EditorManager.setActiveResultIndex(info.filePath, prevIndex)
    }
  },

  async executeDuckDBAccelerated(): Promise<void> {
    const info = activeFileInfo.value
    if (!info) return
    const ed = fileEditors.get(info.filePath)
    if (!ed) return
    const model = ed.getModel()
    if (!model) return
    const sel = ed.getSelection()
    const sql = sel && !sel.isEmpty() ? model.getValueInRange(sel) : model.getValue()
    if (!sql.trim()) return
    startExecution()
    try {
      const { executeDuckDBAccelerated, generateAttachName, rewriteDuckDBSQL } = await import("@/extensions/builtin/workbench/services/sql-editor-service")
      const { appDataDir } = await import("@tauri-apps/api/path")
      const attachName = generateAttachName(info.connectionId || "remote")
      await executeDuckDBAccelerated({ sql: rewriteDuckDBSQL(sql, attachName), connId: info.connectionId, dataDir: await appDataDir() })
    } catch (e) { console.error("[EditorManager] DuckDB:", e) }
    finally { finishExecution() }
  },

  cancelExecution(): void { cancelRuntime() },

  async formatSQL(): Promise<void> {
    const info = activeFileInfo.value
    if (!info) return
    const ed = fileEditors.get(info.filePath)
    if (!ed) return
    const model = ed.getModel()
    if (!model) return
    try {
      const { formatSql } = await import("@/extensions/builtin/workbench/services/sql-editor-service")
      const r = formatSql(model.getValue())
      if (r) model.setValue(r)
    } catch (e) { console.error("[EditorManager] Format:", e) }
  },

  async validateSQL(): Promise<void> {
    const info = activeFileInfo.value
    if (!info) return
    const ed = fileEditors.get(info.filePath)
    if (!ed) return
    const model = ed.getModel()
    if (!model) return
    try {
      const { validateSql } = await import("@/extensions/builtin/workbench/services/sql-editor-service")
      console.log("[EditorManager] Validate:", validateSql(model.getValue()))
    } catch (e) { console.error("[EditorManager] Validate:", e) }
  },

  toggleComment(): void {
    const info = activeFileInfo.value
    if (!info) return
    const ed = fileEditors.get(info.filePath)
    ed?.trigger("keyboard", "editor.action.commentLine", null)
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
      } catch { /* dockview */ }

      openFiles.value = new Map(openFiles.value)
      break
    }
  },
}