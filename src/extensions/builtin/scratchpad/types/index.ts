export type ScratchpadEntryKind = 'file' | 'folder'

export interface ScratchpadEntry {
  name: string
  path: string
  kind: ScratchpadEntryKind
  size: number
  modified_at: string
  extension: string
  is_external_ref: boolean
}

export interface ExternalReference {
  alias: string
  path: string
  created_at: string
}

export interface FileMeta {
  last_connection_id?: string
  last_executed_at?: string
}

export interface AnalyzableFile {
  name: string
  relative_path: string
  file_type: string
  size_bytes: number
  duckdb_query_hint: string
}

export interface ScratchpadResponse {
  local_entries: ScratchpadEntry[]
  external_references: ExternalReference[]
  scratchpad_path: string
  file_meta: Record<string, FileMeta>
}
