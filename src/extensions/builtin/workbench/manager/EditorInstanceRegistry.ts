import { EditorState } from '@codemirror/state'
import { EditorView } from '@codemirror/view'
import { markRaw } from 'vue'

import type { EditorInstance } from '@/extensions/builtin/workbench/types/editor-types'

export class EditorInstanceRegistry {
  private instances = new Map<string, EditorInstance>()
  private savedStates = new Map<string, EditorState>()
  private panelGroupMap = new Map<string, string>()

  constructor(private sanitize: (s: string) => string) {}

  instanceId(filePath: string, groupId: string): string {
    return `${groupId}::${filePath}`
  }

  register(filePath: string, instanceId: string, groupId: string, view: EditorView, writable: boolean): void {
    const inst: EditorInstance = {
      instanceId,
      filePath,
      groupId,
      view: markRaw(view),
      state: null,
      writable,
    }
    this.instances.set(instanceId, inst)
  }

  unregister(instanceId: string): void {
    const inst = this.instances.get(instanceId)
    if (inst) {
      this.saveState(inst.filePath, inst.view.state)
      try {
        inst.view.destroy()
      } catch {
        console.warn('[EditorInstanceRegistry] destroy failed for', instanceId)
      }
    }
    this.instances.delete(instanceId)
  }

  getView(filePath: string): EditorView | undefined {
    for (const [, inst] of this.instances) {
      if (inst.filePath === filePath && inst.writable) return inst.view
    }
    return undefined
  }

  getViewForPanel(panelId: string, filePath: string): EditorView | undefined {
    const gid = this.panelGroupMap.get(panelId)
    if (!gid) return undefined
    const id = this.instanceId(filePath, gid)
    return this.instances.get(id)?.view
  }

  mapPanelToGroup(panelId: string, groupId: string): void {
    this.panelGroupMap.set(panelId, groupId)
  }

  isPrimaryInstance(filePath: string, instanceId?: string): boolean {
    for (const [id, inst] of this.instances) {
      if (inst.filePath === filePath && inst.writable) {
        if (instanceId === undefined) return false
        if (id !== instanceId) return false
      }
    }
    return true
  }

  isFileOpenElsewhere(filePath: string, excludeGroupId?: string): boolean {
    for (const [, inst] of this.instances) {
      if (inst.filePath === filePath) {
        if (excludeGroupId && inst.groupId === excludeGroupId) continue
        return true
      }
    }
    return false
  }

  saveState(filePath: string, state: EditorState): void {
    this.savedStates.set(filePath, state)
  }

  getSavedState(filePath: string): EditorState | undefined {
    return this.savedStates.get(filePath)
  }

  clearAll(): void {
    for (const [, inst] of this.instances) {
      try {
        inst.view.destroy()
      } catch {
        console.warn('[EditorInstanceRegistry] destroy failed during clearAll')
      }
    }
    this.instances.clear()
    this.savedStates.clear()
    this.panelGroupMap.clear()
  }
}