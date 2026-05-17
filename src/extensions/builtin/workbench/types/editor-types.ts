import type * as monaco from 'monaco-editor'

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
  model: monaco.editor.ITextModel
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
  setEditor(ed: monaco.editor.IStandaloneCodeEditor): void
  setActiveResultIndex(filePath: string, index: number): void
  readonly openFiles: Map<string, OpenFileInfo>
  readonly activeFilePath: string | null
  readonly activeFileInfo: OpenFileInfo | null
  readonly editor: monaco.editor.IStandaloneCodeEditor | null
  readonly isExecuting: boolean
  readonly lastExecutionTime: number | null
  readonly isInitialized: boolean
  readonly dockviewApi: unknown
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