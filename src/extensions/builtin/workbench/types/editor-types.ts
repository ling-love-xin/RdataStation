import type { EditorState } from '@codemirror/state'
import type { EditorView } from '@codemirror/view'

export const PANEL_PREFIX_EDITOR = 'panel_editor_'
export const PANEL_PREFIX_RESULT = 'panel_result_'
export const PANEL_ID_EMPTY_WORKBENCH = 'panel_emptyWorkbench'
export const PANEL_ID_SCRATCHPAD = 'scratchpad'

export function isEditorPanel(panelId: string): boolean {
  return panelId.startsWith(PANEL_PREFIX_EDITOR)
}

export function isResultPanel(panelId: string): boolean {
  return panelId.startsWith(PANEL_PREFIX_RESULT)
}

export interface EditorInstance {
  instanceId: string
  filePath: string
  groupId: string
  view: EditorView
  state: EditorState | null
  writable: boolean
}

export interface GridStateSnapshot {
  columnState?: Record<string, unknown>[]
  filterModel?: Record<string, unknown>
  sortModel?: Record<string, unknown>[]
}

export type ShortcutScope = 'global' | 'editor' | 'scratchpad' | 'result' | 'none'

export interface ShortcutRegistration {
  key: string
  scope: ShortcutScope
  handler: () => void
  description: string
}

export type DataSource = 'memory' | 'duckdb'

export type FileType = 'file' | 'analysis'

export interface ResultSetMetadata {
  id: string
  title: string
  columns: string[]
  totalRowCount: number
  elapsedMs: number
  affectedRows: number
  messages: string
  sql: string
  timestamp: number
  dataSource: DataSource
  duckdbTable: string | null
  gridState?: GridStateSnapshot
  rows?: unknown[][]
}

export interface OpenFileInfo {
  filePath: string
  fileName: string
  language: string
  type: FileType
  isDirty: boolean
  connectionId: string
  databaseName: string
  resultSets: ResultSetMetadata[]
  activeResultIndex: number
  resultPanelIds: string[]
  detachedResultIds: string[]
  states: Map<string, EditorState>
  primaryInstanceId: string | null
  readonlyInstanceIds: string[]
}

export interface OpenFileParams {
  filePath: string
  fileName: string
  language: string
  sql: string
  connectionId?: string
  databaseName?: string
  type?: FileType
}

export interface ResultSetCreateParams {
  columns: string[]
  rows: unknown[][]
  totalRows: number
  elapsedMs: number
  affectedRows: number
  sql: string
  error: string | null
}

export interface IEditorManager {
  init(dockviewApi: unknown): void
  destroy(): void
  openFile(params: OpenFileParams): void
  closeFile(filePath: string): void
  panelIdToFilePath(panelId: string): string | null
  switchToFile(filePath: string): void
  onPanelActivated(panelId: string): void
  saveCurrentFile(): Promise<void>
  openNewQuery(connectionId?: string, databaseName?: string): Promise<void>
  openAnalysisPanel(connectionId?: string, databaseName?: string): void
  executeCurrentSQL(): Promise<void>
  executeNewTabSQL(): Promise<void>
  executeDuckDBAccelerated(): Promise<void>
  cancelExecution(): void
  formatSQL(): void
  validateSQL(): void
  toggleComment(): void
  renameResultSet(panelId: string, newTitle: string): void
  setEditor(ed: EditorView): void
  setActiveResultIndex(filePath: string, index: number): void
  readonly openFiles: Map<string, OpenFileInfo>
  readonly activeFilePath: string | null
  readonly activeFileInfo: OpenFileInfo | null
  readonly editor: EditorView | null
  readonly isExecuting: boolean
  readonly lastExecutionTime: number | null
  readonly isInitialized: boolean
  readonly dockviewApi: unknown

  registerFileEditor(filePath: string, ed: EditorView): void
  unregisterFileEditor(filePath: string): void
  isPrimaryInstance(filePath: string, instanceId?: string): boolean
  isFileOpenElsewhere(filePath: string, excludeGroupId?: string): boolean
  getSavedStateForFile(filePath: string): EditorState | undefined
  saveEditorStateForFile(filePath: string, state: EditorState): void
  hasRecoveryData(): boolean
  loadRecoverySnapshots(): { filePath: string; fileName: string; language: string; isDirty: boolean }[]
  clearRecovery(): void

  createResultSet(filePath: string, data: ResultSetCreateParams): string
  removeResultSet(filePath: string, resultSetId: string): void
  findCenterGroup(): string | undefined
  detachResultPanel(panelId: string): void
  attachResultPanel(panelId: string, filePath: string): void

  updatePanelGroup(panelId: string, groupId: string): void
  onPanelUndocked(panelId: string): void
  setupCrossWindowListeners(): void
  popoutActiveFile(): void
}

export interface IShortcutManager {
  register(key: string, scope: ShortcutScope, handler: () => void, desc: string): void
  unregister(key: string): void
  setActiveScope(scope: ShortcutScope): void
  handleKeydown(e: KeyboardEvent): void
}

export interface IResultPanelManager {
  addResultSet(filePath: string, data: ResultSetCreateParams): string
  removeResultSet(filePath: string, resultSetId: string): void
  getResultSetRows(filePath: string, resultSetId: string): unknown[][]
  createResultPanel(
    filePath: string,
    panelId: string,
    metadata: ResultSetMetadata,
    rows: unknown[][]
  ): void
  detachResultPanel(panelId: string): void
  attachResultPanel(panelId: string, filePath: string): void
  getAllResultSets(filePath: string): ResultSetMetadata[]
  getActiveResultSet(filePath: string): ResultSetMetadata | null
}