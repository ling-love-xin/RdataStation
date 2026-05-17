import { EditorManager } from '@/extensions/builtin/workbench/manager/EditorManager'
import type { IEditorManager } from '@/extensions/builtin/workbench/types/editor-types'

export function useEditorManager(): IEditorManager {
  return EditorManager
}