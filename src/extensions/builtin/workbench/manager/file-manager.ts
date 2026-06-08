import type { OpenFileInfo, OpenFileParams } from '@/extensions/builtin/workbench/types/editor-types'
import { PANEL_PREFIX_EDITOR } from '@/extensions/builtin/workbench/types/editor-types'
import { useEditorPersistence } from '@/extensions/builtin/workbench/ui/composables/useEditorPersistence'
import { confirmUnsavedClose, confirmExternalChange, confirmFileConflict } from '@/extensions/builtin/workbench/ui/composables/useFileDialogs'

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

let untitledCounter = 0

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

function makeFileInfo(params: OpenFileParams): OpenFileInfo {
  return {
    filePath: params.filePath,
    fileName: params.fileName,
    language: params.language,
    type: params.type ?? 'file',
    isDirty: false,
    exists: true,
    lastModifiedAt: null,
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

  const info = makeFileInfo(params)

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

/**
 * 创建新建未保存文件（Ctrl+N 场景）
 * filePath 为虚拟路径 "untitled:N" 不指向磁盘文件
 */
export function newFile(language?: string, content?: string): string {
  untitledCounter++
  const filePath = `untitled:${untitledCounter}`
  const lang = language ?? 'sql'
  const fileName = `Untitled-${untitledCounter}.${lang}`

  const pid = filePanelId(filePath)
  const isFirstFile = openFiles.value.size === 0
  const refGroup = tabGroupId.value

  const params: OpenFileParams = {
    filePath,
    fileName,
    language: lang,
    sql: content ?? '',
    type: 'file',
  }

  const info = makeFileInfo(params)
  info.exists = false

  openFiles.value.set(filePath, info)

  dockviewApi?.addPanel({
    id: pid,
    component: 'editorPanel',
    title: fileName,
    position:
      isFirstFile || !refGroup
        ? { direction: 'right' }
        : { referenceGroup: refGroup, direction: 'within' },
    params: {
      filePath,
      fileName,
      language: lang,
      content: content ?? '',
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
  return filePath
}

/**
 * 关闭文件前检查脏状态，如果修改未保存则弹出确认对话框
 * 返回 false 表示用户取消了关闭
 */
export async function closeFileChecked(filePath: string): Promise<boolean> {
  const info = openFiles.value.get(filePath)
  if (!info) return true

  if (info.isDirty) {
    const result = await confirmUnsavedClose(info.fileName)
    if (result === 'cancel') return false
    if (result === 'save') {
      try {
        await saveCurrentFileToDisk(filePath)
      } catch {
        // 保存失败，询问是否仍然关闭
        const forceResult = await confirmUnsavedClose(info.fileName)
        if (forceResult !== 'discard') return false
      }
    }
  }

  closeFile(filePath)
  return true
}

/**
 * 批量关闭文件时检查脏状态
 */
export async function closeFilesChecked(filePaths: string[]): Promise<void> {
  for (const fp of filePaths) {
    const closed = await closeFileChecked(fp)
    if (!closed) break
  }
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

// ========== 保存 / 另存为 ==========

/**
 * 保存当前文件到磁盘
 * 如果是未保存文件 (exists=false)，弹出"另存为"对话框
 */
export async function saveCurrentFileToDisk(filePath: string): Promise<void> {
  const info = openFiles.value.get(filePath)
  if (!info) throw new Error('File not open')

  const ed = getEditorView(filePath)
  if (!ed) throw new Error('No editor instance')

  // 检查外部文件变更
  if (info.exists && info.lastModifiedAt) {
    const currentMtime = await getFileModifiedAt(filePath)
    if (currentMtime && currentMtime > info.lastModifiedAt) {
      const result = await confirmExternalChange(info.fileName)
      if (result === 'cancel') return
      // 'reload' -> discard local changes and reload disk content
      // 'overwrite' -> continue saving
    }
  }

  const content = ed.state.doc.toString()

  // 未保存文件 → 另存为
  if (!info.exists) {
    const { writeTextFile, save } = await import('@tauri-apps/plugin-dialog')
    const path = await save({
      title: '另存为',
      filters: [{ name: 'SQL Files', extensions: ['sql'] }],
    })
    if (!path) throw new Error('Save cancelled')
    await writeTextFile(path, content)
    info.filePath = path
    info.fileName = path.split(/[/\\]/).pop() ?? path
    info.exists = true
    info.isDirty = false
    info.lastModifiedAt = Date.now()
    syncDirtyToTab(filePath, false)
    notifyOpenFilesChanged()
    return
  }

  // 已存在文件 → 直接写入
  const { writeTextFile } = await import('@tauri-apps/plugin-fs')
  await writeTextFile(filePath, content)
  info.isDirty = false
  info.lastModifiedAt = Date.now()
  syncDirtyToTab(filePath, false)
  notifyOpenFilesChanged()
}

/**
 * 将文件"另存为"到新路径
 */
export async function saveFileAs(filePath: string): Promise<void> {
  const info = openFiles.value.get(filePath)
  if (!info) throw new Error('File not open')

  const ed = getEditorView(filePath)
  if (!ed) throw new Error('No editor instance')

  const { writeTextFile, save } = await import('@tauri-apps/plugin-dialog')
  const newPath = await save({
    title: '另存为',
    filters: [{ name: 'All Files', extensions: ['*'] }],
  })
  if (!newPath) throw new Error('Save cancelled')

  const content = ed.state.doc.toString()
  await writeTextFile(newPath, content)

  info.filePath = newPath
  info.fileName = newPath.split(/[/\\]/).pop() ?? newPath
  info.exists = true
  info.isDirty = false
  info.lastModifiedAt = Date.now()
  syncDirtyToTab(filePath, false)
  notifyOpenFilesChanged()
}

// ========== 外部文件变更检测 ==========

async function getFileModifiedAt(filePath: string): Promise<number | null> {
  try {
    const { stat } = await import('@tauri-apps/plugin-fs')
    const metadata = await stat(filePath)
    return (metadata as { mtime?: { ms: number } }).mtime?.ms ?? Date.now()
  } catch {
    return null
  }
}

/**
 * 检查磁盘文件是否已被外部修改
 * 应在编辑器获得焦点时调用
 */
export async function checkExternalFileChanges(): Promise<void> {
  for (const [filePath, info] of openFiles.value) {
    if (!info.exists) continue
    const diskMtime = await getFileModifiedAt(filePath)
    if (diskMtime && info.lastModifiedAt && diskMtime > info.lastModifiedAt) {
      const result = await confirmExternalChange(info.fileName)
      if (result === 'reload') {
        const ed = getEditorView(filePath)
        if (ed) {
          const { readTextFile } = await import('@tauri-apps/plugin-fs')
          const content = await readTextFile(filePath)
          ed.dispatch({
            changes: { from: 0, to: ed.state.doc.length, insert: content },
          })
          info.isDirty = false
          info.lastModifiedAt = diskMtime
          syncDirtyToTab(filePath, false)
          notifyOpenFilesChanged()
        }
      }
      // 'keep' -> 保持本地版本，更新 lastModifiedAt 避免重复弹窗
      else {
        info.lastModifiedAt = diskMtime
      }
    }
  }
}

// ========== 跨窗口冲突解决 ==========

/**
 * 多窗口编辑同一文件时处理冲突
 * 调用方应传入远端和本地的内容，让用户选择保留哪个版本
 */
export async function reconcileCrossWindowConflict(
  filePath: string,
  remoteContent: string,
  remoteIsDirty: boolean
): Promise<void> {
  const info = openFiles.value.get(filePath)
  if (!info) return

  const ed = getEditorView(filePath)
  if (!ed) return

  const localContent = ed.state.doc.toString()

  if (remoteContent === localContent) {
    // 内容相同，同步 dirty 状态
    info.isDirty = remoteIsDirty
    syncDirtyToTab(filePath, remoteIsDirty)
    notifyOpenFilesChanged()
    return
  }

  const result = await confirmFileConflict(info.fileName)

  if (result === 'keep-local') return

  if (result === 'keep-remote') {
    ed.dispatch({
      changes: { from: 0, to: ed.state.doc.length, insert: remoteContent },
    })
    info.isDirty = remoteIsDirty
    syncDirtyToTab(filePath, remoteIsDirty)
  }

  // merge -> 保留本地 + 远端追加为注释（最简策略）
  if (result === 'merge') {
    const merged = `${localContent}\n/* ===== Remote =====\n${remoteContent}\n*/`
    ed.dispatch({
      changes: { from: 0, to: ed.state.doc.length, insert: merged },
    })
    info.isDirty = true
    syncDirtyToTab(filePath, true)
  }

  notifyOpenFilesChanged()
}