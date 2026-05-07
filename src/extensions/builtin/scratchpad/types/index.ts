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
  added_at: string
}

export interface ScratchpadResponse {
  local_entries: ScratchpadEntry[]
  external_references: ExternalReference[]
  scratchpad_path: string
}
