import type { OpenFileInfo, OpenFileParams } from '@/extensions/builtin/workbench/types/editor-types'
import { PANEL_PREFIX_EDITOR } from '@/extensions/builtin/workbench/types/editor-types'
import { useEditorPersistence } from '@/extensions/builtin/workbench/ui/composables/useEditorPersistence'

import {
  openFiles,
  activeFilePath,
  editorRef,
  tabGroupId,
  dockviewApi,
  savedStates,
  notifyOpenFilesChanged,
} from './editor-state'
import {
  getEditorView,
  saveEditorState,
  panelIdToFilePath,
  removeInstancesForFile,
  isFileOpenElsewhere,
} from './instance-service'

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
    console.warn('[FileManager] dockview setTitle failed')
  }
}

export function openFile(params: OpenFileParams): void {
  if (openFiles.value.has(params.filePath)) {
    switchToFile(params.filePath)
    if (isFileOpenElsewhere(params.filePath)) {
      // eslint-disable-next-line no-console
      console.debug(`[FileManager] File already open in another group: ${params.filePath}`)
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

  notifyOpenFilesChanged()
}

export function closeFile(filePath: string): void {
  const info = openFiles.value.get(filePath)
  if (!info) return

  const pid = filePanelId(filePath)

  for (const rpId of info.resultPanelIds) {
    dockviewApi?.getPanel(rpId)?.api.close()
  }

  const idToRemove = removeInstancesForFile(filePath)
  for (const id of idToRemove) {
    const removed = info.states.get(id)
    if (removed) {
      info.states.set(id, removed)
    }
  }

  try {
    const { draft } = useEditorPersistence(pid, filePath)
    draft.remove()
  } catch {
    console.warn('[FileManager] draft.remove failed during closeFile')
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

  notifyOpenFilesChanged()

  if (wasActive) {
    const next = openFiles.value.keys().next().value as string | undefined
    if (next) switchToFile(next)
    else activeFilePath.value = null
  }
}

export function switchToFile(filePath: string): void {
  const info = openFiles.value.get(filePath)
  if (!info) {
    console.warn(`[FileManager] File not found: ${filePath}`)
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
}

export function onPanelActivated(panelId: string): void {
  const fp = panelIdToFilePath(panelId)
  if (!fp) return
  activeFilePath.value = fp
  const ed = getEditorView(fp)
  if (ed) editorRef.value = ed

  for (const [path, fileInfo] of openFiles.value) {
    if (fileInfo.resultPanelIds.length === 0) continue
    for (const rpId of fileInfo.resultPanelIds) {
      try {
        dockviewApi?.getPanel(rpId)?.api.setVisible(path === fp)
      } catch {
        console.warn('[FileManager] result panel visibility update failed')
      }
    }
  }
}

export function findCenterGroup(): string | undefined {
  return tabGroupId.value ?? undefined
}

export function syncDirty(filePath: string, isDirty: boolean): void {
  syncDirtyToTab(filePath, isDirty)
}