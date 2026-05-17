import { reactive } from 'vue'

interface EditorRuntimeState {
  isExecuting: boolean
  lastError: string | null
  lastExecutionTime: number | null
  resultVersion: number
}

const runtime = reactive<EditorRuntimeState>({
  isExecuting: false,
  lastError: null,
  lastExecutionTime: null,
  resultVersion: 0,
})

export function useEditorRuntime() {
  function startExecution() {
    runtime.isExecuting = true
    runtime.lastError = null
  }

  function finishExecution(error?: string) {
    runtime.isExecuting = false
    if (error) runtime.lastError = error
    runtime.lastExecutionTime = Date.now()
    runtime.resultVersion++
  }

  function cancelExecution() {
    runtime.isExecuting = false
  }

  function clearError() {
    runtime.lastError = null
  }

  function incrementResultVersion() {
    runtime.resultVersion++
  }

  return {
    runtime,
    startExecution,
    finishExecution,
    cancelExecution,
    clearError,
    incrementResultVersion,
  }
}