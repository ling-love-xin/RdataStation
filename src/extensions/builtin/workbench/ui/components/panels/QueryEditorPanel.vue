<template>
  <div class="query-editor-panel">
    <QueryToolbar
      :is-executing="isExecuting"
      :execution-mode="executionMode"
      :connection-name="connectionName"
      :in-transaction="inTransaction"
      @execute="handleExecute"
      @execute-new="handleExecuteNew"
      @accelerate="handleAccelerate"
      @explain="handleExplain"
      @format="handleFormat"
      @transpile="handleTranspile"
      @mode-change="handleModeChange"
      @begin-transaction="handleBeginTransaction"
      @commit-transaction="handleCommitTransaction"
      @rollback-transaction="handleRollbackTransaction"
    />
    <EditorBody ref="editorBodyRef" :params="props.params" />
    <QueryStatusBar
      :connection-name="connectionName"
      :cursor-position="cursorPosition"
      :statement-count="statementCount"
      :in-transaction="inTransaction"
      :last-execution-time="lastExecutionTime"
      :editor-mode="editorMode"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'

import { EditorManager } from '@/extensions/builtin/workbench/manager/EditorManager'
import type { EditorPanelParams } from '@/extensions/builtin/workbench/types/editor-types'
import { useSqlExecution } from '@/extensions/builtin/workbench/ui/composables/useSqlExecution'

import EditorBody from './EditorBody.vue'
import QueryStatusBar from './QueryStatusBar.vue'
import QueryToolbar from './QueryToolbar.vue'

const props = defineProps<{
  params: EditorPanelParams
}>()

const editorBodyRef = ref<InstanceType<typeof EditorBody> | null>(null)

const runtimeConnId = ref<string>('')
const connectionName = computed(() => EditorManager.activeFileInfo?.connectionId ?? '')

const {
  executing: isExecuting,
  lastExecutionTime,
  inTransaction,
  statementCount,
  scheduleParse,
  executeSingleStatement,
  executeNewTab,
  executeDuckDBAccelerated,
  beginTransaction,
  commitTransaction,
  rollbackTransaction,
} = useSqlExecution({
  panelId: props.params.filePath,
  getEditorValue: () => editorBodyRef.value?.getEditorValue?.() ?? '',
  getSelectedText: () => editorBodyRef.value?.getSelectedText?.() ?? '',
  runtimeConnId,
  currentConnectionName: connectionName,
})

const cursorPosition = ref('Ln 1, Col 1')
const editorMode = computed(() => 'SQL')
const executionMode = ref<'normal' | 'analysis' | 'smart'>('normal')

function handleExecute() {
  executeSingleStatement()
}

function handleExecuteNew() {
  executeNewTab()
}

function handleAccelerate() {
  executeDuckDBAccelerated()
}

function handleExplain() {
  // TODO: 实现执行计划功能
}

function handleFormat() {
  EditorManager.formatSQL()
}

function handleTranspile(_dialect: string) {
  // TODO: 实现 SQL 方言转译功能
}

function handleModeChange(mode: 'normal' | 'analysis' | 'smart') {
  executionMode.value = mode
}

function handleBeginTransaction() {
  beginTransaction()
}

function handleCommitTransaction() {
  commitTransaction()
}

function handleRollbackTransaction() {
  rollbackTransaction()
}

onMounted(() => {
  const info = EditorManager.activeFileInfo
  if (info?.connectionId) {
    runtimeConnId.value = info.connectionId
  }
  scheduleParse()
})
</script>

<style scoped>
.query-editor-panel {
  display: flex;
  flex-direction: column;
  height: 100%;
  overflow: hidden;
}
</style>