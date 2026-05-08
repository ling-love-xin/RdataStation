import { invoke } from '@tauri-apps/api/core'

import type {
  ScratchpadResponse,
  ScratchpadEntry,
  ExternalReference,
  AnalyzableFile,
  PromoteResult,
  SearchMatch,
} from '../../types'

export async function listScratchpadFiles(): Promise<ScratchpadResponse> {
  return invoke<ScratchpadResponse>('list_scratchpad_files')
}

export async function createScratchpadEntry(
  name: string,
  isFolder: boolean
): Promise<ScratchpadEntry> {
  return invoke<ScratchpadEntry>('create_scratchpad_entry', {
    name,
    isFolder,
  })
}

export async function deleteScratchpadEntry(relativePath: string): Promise<void> {
  return invoke<void>('delete_scratchpad_entry', {
    relativePath,
  })
}

export async function renameScratchpadEntry(
  relativePath: string,
  newName: string
): Promise<ScratchpadEntry> {
  return invoke<ScratchpadEntry>('rename_scratchpad_entry', {
    relativePath,
    newName,
  })
}

export async function readScratchpadFile(relativePath: string): Promise<string> {
  return invoke<string>('read_scratchpad_file', {
    relativePath,
  })
}

export async function saveScratchpadFile(relativePath: string, content: string): Promise<void> {
  return invoke<void>('save_scratchpad_file', {
    relativePath,
    content,
  })
}

export async function importExternalFile(sourcePath: string): Promise<ScratchpadEntry> {
  return invoke<ScratchpadEntry>('import_external_file', {
    sourcePath,
  })
}

export async function addExternalReference(
  alias: string,
  path: string
): Promise<ExternalReference> {
  return invoke<ExternalReference>('add_external_reference', {
    alias,
    path,
  })
}

export async function removeExternalReference(alias: string): Promise<void> {
  return invoke<void>('remove_external_reference', {
    alias,
  })
}

export async function openInExplorer(path: string): Promise<void> {
  return invoke<void>('open_scratchpad_in_explorer', {
    path,
  })
}

export async function checkFileSize(relativePath: string): Promise<number> {
  return invoke<number>('check_scratchpad_file_size', {
    relativePath,
  })
}

export async function updateFileMeta(
  relativePath: string,
  connectionId?: string
): Promise<void> {
  return invoke<void>('update_scratchpad_file_meta', {
    relativePath,
    connectionId: connectionId ?? null,
  })
}

export async function searchFileContent(query: string): Promise<SearchMatch[]> {
  return invoke<SearchMatch[]>('search_scratchpad_content', { query })
}

export async function listTrash(): Promise<ScratchpadEntry[]> {
  return invoke<ScratchpadEntry[]>('list_scratchpad_trash')
}

export async function restoreFromTrash(trashName: string): Promise<void> {
  return invoke<void>('restore_scratchpad_from_trash', { trashName })
}

export async function emptyTrash(): Promise<void> {
  return invoke<void>('empty_scratchpad_trash')
}

export async function getAnalyzableFiles(): Promise<AnalyzableFile[]> {
  return invoke<AnalyzableFile[]>('get_analyzable_files')
}

export async function watchScratchpad(): Promise<void> {
  return invoke<void>('watch_scratchpad')
}

export async function unwatchScratchpad(): Promise<void> {
  return invoke<void>('unwatch_scratchpad')
}

export async function promoteScratchpadToResource(
  relativePath: string,
  removeAfter: boolean
): Promise<PromoteResult> {
  return invoke<PromoteResult>('promote_scratchpad_to_resource', {
    relativePath,
    removeAfter,
  })
}
