import { ref, computed } from 'vue'

import {
  listScratchpadFiles,
  createScratchpadEntry,
  deleteScratchpadEntry,
  renameScratchpadEntry,
  readScratchpadFile,
  saveScratchpadFile,
  importExternalFile,
  addExternalReference,
  removeExternalReference,
  openInExplorer,
  checkFileSize,
  updateFileMeta,
  searchFileContent,
  listTrash,
  restoreFromTrash,
  emptyTrash,
  getAnalyzableFiles,
  watchScratchpad,
  unwatchScratchpad,
  promoteScratchpadToResource,
} from '../../infrastructure/api/scratchpad-api'

import type {
  ScratchpadResponse,
  ScratchpadEntry,
  ExternalReference,
  AnalyzableFile,
  PromoteResult,
  SearchResult,
} from '../../types'

export function useScratchpad() {
  const response = ref<ScratchpadResponse | null>(null)
  const isLoading = ref(false)
  const error = ref<string | null>(null)
  const searchQuery = ref('')
  const trashEntries = ref<ScratchpadEntry[]>([])
  const analyzableFiles = ref<AnalyzableFile[]>([])

  const localEntries = computed(() => {
    if (!response.value) return []
    const entries = response.value.local_entries
    if (!searchQuery.value) return entries
    const q = searchQuery.value.toLowerCase()
    return entries.filter(e => e.name.toLowerCase().includes(q))
  })

  const externalReferences = computed(() => {
    if (!response.value) return []
    const refs = response.value.external_references
    if (!searchQuery.value) return refs
    const q = searchQuery.value.toLowerCase()
    return refs.filter(r => r.alias.toLowerCase().includes(q) || r.path.toLowerCase().includes(q))
  })

  const scratchpadPath = computed(() => response.value?.scratchpad_path ?? '')

  const invalidReferences = computed(() =>
    externalReferences.value.filter(ref => {
      if (!ref.path) return true
      const pathPattern = /^([A-Za-z]:[\\/]|[/\\])/
      return !pathPattern.test(ref.path) || ref.path.includes('..')
    })
  )

  const validReferences = computed(() =>
    externalReferences.value.filter(ref => {
      if (!ref.path) return false
      const pathPattern = /^([A-Za-z]:[\\/]|[/\\])/
      return pathPattern.test(ref.path) && !ref.path.includes('..')
    })
  )

  async function loadFiles(): Promise<void> {
    isLoading.value = true
    error.value = null
    try {
      response.value = await listScratchpadFiles()
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e)
      response.value = null
    } finally {
      isLoading.value = false
    }
  }

  async function createEntry(name: string, isFolder: boolean): Promise<ScratchpadEntry | null> {
    try {
      const entry = await createScratchpadEntry(name, isFolder)
      await loadFiles()
      return entry
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e)
      return null
    }
  }

  async function deleteEntry(relativePath: string): Promise<boolean> {
    try {
      await deleteScratchpadEntry(relativePath)
      await loadFiles()
      return true
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e)
      return false
    }
  }

  async function renameEntry(
    relativePath: string,
    newName: string
  ): Promise<ScratchpadEntry | null> {
    try {
      const entry = await renameScratchpadEntry(relativePath, newName)
      await loadFiles()
      return entry
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e)
      return null
    }
  }

  async function loadFileContent(relativePath: string): Promise<string | null> {
    try {
      return await readScratchpadFile(relativePath)
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e)
      return null
    }
  }

  async function saveFile(relativePath: string, content: string): Promise<boolean> {
    try {
      await saveScratchpadFile(relativePath, content)
      return true
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e)
      return false
    }
  }

  async function importFile(sourcePath: string): Promise<ScratchpadEntry | null> {
    try {
      const entry = await importExternalFile(sourcePath)
      await loadFiles()
      return entry
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e)
      return null
    }
  }

  async function addReference(alias: string, path: string): Promise<ExternalReference | null> {
    try {
      const ref = await addExternalReference(alias, path)
      await loadFiles()
      return ref
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e)
      return null
    }
  }

  async function removeReference(alias: string): Promise<boolean> {
    try {
      await removeExternalReference(alias)
      await loadFiles()
      return true
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e)
      return false
    }
  }

  function isRefValid(ref: ExternalReference): boolean {
    if (!ref.path) return false
    const pathPattern = /^([A-Za-z]:[\\/]|[/\\])/
    return pathPattern.test(ref.path) && !ref.path.includes('..')
  }

  function findEntry(entryPath: string): ScratchpadEntry | undefined {
    return localEntries.value.find(e => e.path === entryPath)
  }

  async function openInExplorerAction(path: string): Promise<boolean> {
    try {
      await openInExplorer(path)
      return true
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e)
      return false
    }
  }

  async function getFileSize(relativePath: string): Promise<number | null> {
    try {
      return await checkFileSize(relativePath)
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e)
      return null
    }
  }

  function isRefInvalid(ref: ExternalReference): boolean {
    return !isRefValid(ref)
  }

  function clearError(): void {
    error.value = null
  }

  async function loadTrashEntries(): Promise<void> {
    try {
      trashEntries.value = await listTrash()
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e)
    }
  }

  async function restoreTrashEntry(trashName: string): Promise<boolean> {
    try {
      await restoreFromTrash(trashName)
      await loadTrashEntries()
      await loadFiles()
      return true
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e)
      return false
    }
  }

  async function emptyTrashBin(): Promise<boolean> {
    try {
      await emptyTrash()
      trashEntries.value = []
      return true
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e)
      return false
    }
  }

  async function loadAnalyzableFiles(): Promise<void> {
    try {
      analyzableFiles.value = await getAnalyzableFiles()
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e)
    }
  }

  async function saveFileMeta(
    relativePath: string,
    connectionId?: string
  ): Promise<boolean> {
    try {
      await updateFileMeta(relativePath, connectionId)
      return true
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e)
      return false
    }
  }

  async function searchContent(query: string, caseSensitive = false): Promise<SearchResult | null> {
    try {
      return await searchFileContent(query, caseSensitive)
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e)
      return null
    }
  }

  async function startWatching(): Promise<void> {
    try {
      await watchScratchpad()
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e)
    }
  }

  async function stopWatching(): Promise<void> {
    try {
      await unwatchScratchpad()
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e)
    }
  }

  async function promoteToResource(
    relativePath: string,
    removeAfter: boolean
  ): Promise<PromoteResult | null> {
    try {
      const result = await promoteScratchpadToResource(relativePath, removeAfter)
      if (removeAfter) {
        await loadFiles()
      }
      return result
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e)
      return null
    }
  }

  return {
    response,
    isLoading,
    error,
    searchQuery,
    localEntries,
    externalReferences,
    scratchpadPath,
    invalidReferences,
    validReferences,
    loadFiles,
    createEntry,
    deleteEntry,
    renameEntry,
    loadFileContent,
    saveFile,
    importFile,
    addReference,
    removeReference,
    isRefValid,
    isRefInvalid,
    findEntry,
    openInExplorerAction,
    getFileSize,
    clearError,
    saveFileMeta,
    searchContent,
    trashEntries,
    loadTrashEntries,
    restoreTrashEntry,
    emptyTrashBin,
    analyzableFiles,
    loadAnalyzableFiles,
    startWatching,
    stopWatching,
    promoteToResource,
  }
}
