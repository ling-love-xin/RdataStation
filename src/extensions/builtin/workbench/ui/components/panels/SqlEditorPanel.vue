<template>
  <div :class="['sql-editor-panel', `toolbar-${toolbarPosition}`]">
    <EditorToolbar
      v-if="isSqlLanguage"
      :toolbar-position="toolbarPosition"
      :is-duck-db="isDuckDbConnection"
      :show-advanced="!isScratchpadMode"
      @execute="handleExecute"
      @execute-new="handleExecuteNew"
      @execute-batch="handleBatchExecute"
      @duckdb-execute="handleDuckDbExecute"
      @format="handleFormat"
      @validate="handleValidate"
      @transpile="showTranspileMenu = true"
    />

    <div class="editor-result-split">
      <div
        ref="editorAndResultContainer"
        class="editor-and-result"
        :style="{ flexDirection: 'column' }"
      >
        <div
          class="editor-container"
          :style="{
            flex: hasResults ? `${splitRatio}` : '1 1 auto',
            minHeight: hasResults ? `${splitRatio * 100}%` : 'auto',
          }"
        >
          <div ref="editorContainer" class="monaco-container" />
          <EditorWelcome :visible="showWelcome" />
        </div>

        <div v-if="hasResults" class="split-handle" @mousedown="startSplitDrag" />
        <div
          v-if="hasResults"
          class="result-container"
          :style="{
            flex: `calc(1 - ${splitRatio})`,
            maxHeight: `calc((1 - ${splitRatio}) * 100%)`,
          }"
        >
          <QueryResultPanel :result-data="currentResultData" :compact="true" />
        </div>
      </div>
    </div>

    <EditorStatusbar
      :cursor-position="cursorPosition"
      :selected-text-info="selectedTextInfo"
      :editor-mode="editorMode"
      :executing="executing"
      :can-cancel="true"
      :last-execution-time="lastExecutionTime"
      :connection-info-text="connectionInfoText"
      :popselect-options="popselectOptions"
      :selected-connection="selectedConnection"
      :in-transaction="inTransaction"
      :statement-count="statementCount"
      :is-dirty="isScratchpadMode && isDirty"
      @connection-change="onConnectionSelected"
      @cancel="cancelExecution"
      @commit="commitTransaction"
      @rollback="rollbackTransaction"
    />

    <TranspileModal
      :visible="showTranspileMenu"
      :dialect-options="dialectOptions"
      @close="showTranspileMenu = false"
      @transpile="handleTranspile"
      @explain="handleExplain"
      @save-snippet="handleSaveSnippet"
    />

    <ParamBindingModal
      :visible="showParamBinding"
      :params="detectedParams"
      @confirm="handleParamConfirm"
      @cancel="handleParamCancel"
    />
  </div>
</template>

<script setup lang="ts">
import { invoke } from '@tauri-apps/api/core'
import * as monaco from 'monaco-editor'
import { darkTheme, lightTheme } from 'naive-ui'
import { ref, computed, watch, onMounted, onBeforeUnmount, createDiscreteApi } from 'vue'
import { useI18n } from 'vue-i18n'
import 'monaco-editor/esm/vs/basic-languages/sql/sql.contribution'

import * as queryService from '@/extensions/builtin/query/ui/services/query'
import {
  transpileSql,
  formatSql,
  validateSql,
  registerDatabaseCompletionProvider,
  registerSqlFoldingProvider,
} from '@/extensions/builtin/workbench/services/sql-editor-service'
import { addCustomSnippet } from '@/extensions/builtin/workbench/services/sql-snippets'
import { useConnectionBinding } from '@/extensions/builtin/workbench/ui/composables/useConnectionBinding'
import { useDialectSync } from '@/extensions/builtin/workbench/ui/composables/useDialectSync'
import { useMonacoEditor } from '@/extensions/builtin/workbench/ui/composables/useMonacoEditor'
import { useSqlExecution } from '@/extensions/builtin/workbench/ui/composables/useSqlExecution'
import { useResultStore } from '@/extensions/builtin/workbench/ui/stores/result-store'
import { useSqlExecutionStore } from '@/extensions/builtin/workbench/ui/stores/sql-execution-store'
import { useUiStore } from '@/shared/stores/ui'
import { rdataDark, rdataLight } from '@/shared/styles/monaco-theme'
import type { SqlDialect } from '@/shared/types/sql'

import EditorStatusbar from './EditorStatusbar.vue'
import EditorToolbar from './EditorToolbar.vue'
import EditorWelcome from './EditorWelcome.vue'
import ParamBindingModal from './ParamBindingModal.vue'
import QueryResultPanel from './QueryResultPanel.vue'
import TranspileModal from './TranspileModal.vue'

interface Props {
  modelValue?: string
  language?: string
  theme?: string
  connectionId?: string
  connections?: Array<{
    id: string
    name: string
    type: string
    status: 'connected' | 'disconnected'
  }>
  params?: Record<string, unknown>
}

const props = withDefaults(defineProps<Props>(), {
  modelValue: '',
  language: 'sql',
  theme: 'vs-dark',
  connections: () => [],
  params: undefined,
})

const emit = defineEmits<{
  'update:modelValue': [value: string]
  execute: [sql: string, connectionId: string]
  format: [sql: string]
}>()

const uiStore = useUiStore()
const { t } = useI18n()

const configProviderPropsRef = ref({
  theme: uiStore.isDark ? darkTheme : lightTheme,
})
const { message } = createDiscreteApi(['message'], {
  configProviderProps: configProviderPropsRef,
})

const panelId = computed(() => {
  const raw = props.params
  if (!raw) return `sql-editor-${Date.now()}`
  const nested = raw.params as Record<string, unknown> | undefined
  return (nested?.panelId as string) || (raw.panelId as string) || `sql-editor-${Date.now()}`
})

const paramsConnectionId = computed(() => {
  const raw = props.params
  if (!raw) return ''
  const nested = raw.params as Record<string, unknown> | undefined
  return (nested?.connectionId as string) || (raw.connectionId as string) || ''
})

const scratchpadRelativePath = computed(() => {
  const raw = props.params
  if (!raw) return ''
  const nested = raw.params as Record<string, unknown> | undefined
  return (nested?.scratchpadRelativePath as string) || (raw.scratchpadRelativePath as string) || ''
})

const isScratchpadMode = computed(() => !!scratchpadRelativePath.value)

const editorLanguage = computed(() => {
  const raw = props.params
  if (!raw) return props.language
  const nested = raw.params as Record<string, unknown> | undefined
  return (nested?.language as string) || (raw.language as string) || props.language
})

const isSqlLanguage = computed(() => editorLanguage.value === 'sql')

const showTranspileMenu = ref(false)
const toolbarPosition = ref<'top' | 'left' | 'right'>('top')
const editorMode = ref('SQL')
const isDirty = ref(false)

const showParamBinding = ref(false)
const detectedParams = ref<string[]>([])
const pendingParamSql = ref('')

const {
  selectedConnection,
  runtimeConnId,
  popselectOptions,
  connectionInfoText,
  isDuckDbConnection,
  currentDatabaseType,
  currentDatabase,
  currentConnectionName,
  ensureConnection,
  onConnectionSelected,
} = useConnectionBinding({
  initialConnectionId: props.connectionId,
})

const editorContainerRef = ref<HTMLElement | undefined>()
const editorAndResultContainer = ref<HTMLElement | undefined>()

const {
  showWelcome,
  cursorPosition,
  selectedTextInfo,
  editorReady,
  editorModel,
  createEditor,
  setupEditorEvents,
  setupEditorCommands,
  getValue,
  setValue,
  getSelectedText,
  insertText,
  disposeEditor,
  disposeMonacoDisposables,
} = useMonacoEditor({
  containerRef: editorContainerRef,
  panelId: panelId.value,
  initialValue: props.modelValue,
  language: editorLanguage.value,
  theme: uiStore.isDark ? 'rdata-dark' : 'rdata-light',
})

const { getCurrentDialect, startSync, stopSync } = useDialectSync({
  dbType: computed(() => currentDatabaseType.value || null),
  editorReady,
})

const sqlExecutionStore = useSqlExecutionStore()

// async function onRefreshResult(): Promise<void> {
//   await handleExecute()
// }

const {
  executing,
  lastExecutionTime,
  hasResults,
  currentResultData,
  executeSingleStatement,
  cancelExecution,
  executeNewTab,
  inTransaction,
  statementCount,
  scheduleParse,
  beginTransaction,
  commitTransaction,
  rollbackTransaction,
  executeDuckDBAccelerated,
  executeBatch,
  checkForParams,
  buildBoundSql,
} = useSqlExecution({
  panelId: panelId.value,
  getEditorValue: () => getValue(),
  getSelectedText: () => getSelectedText(),
  runtimeConnId,
  currentDatabaseType,
  currentConnectionName,
})

// provider('sqlEditorRefresh', onRefreshResult)

const splitRatio = ref(0.55)
let splitDragging = false

function startSplitDrag(e: MouseEvent) {
  splitDragging = true
  const startY = e.clientY
  const startRatio = splitRatio.value
  const container = (e.target as HTMLElement).parentElement
  if (!container) return
  const containerHeight = container.getBoundingClientRect().height

  const onMove = (ev: MouseEvent) => {
    if (!splitDragging) return
    const delta = ev.clientY - startY
    const newRatio = startRatio + delta / containerHeight
    splitRatio.value = Math.max(0.2, Math.min(0.85, newRatio))
  }
  const onUp = () => {
    splitDragging = false
    document.removeEventListener('mousemove', onMove)
    document.removeEventListener('mouseup', onUp)
  }
  document.addEventListener('mousemove', onMove)
  document.addEventListener('mouseup', onUp)
}

async function handleExecute(): Promise<void> {
  const connId = selectedConnection.value
  if (!connId) {
    message.warning(t('sqlEditor.noConnection'))
    return
  }
  await ensureConnection(connId)
  if (isScratchpadMode.value) {
    invoke('update_scratchpad_file_meta', {
      relativePath: scratchpadRelativePath.value,
      connectionId: connId,
    }).catch(() => {})
  }

  const sql = getSelectedText() || getValue()
  const params = checkForParams(sql)
  if (params.length > 0) {
    detectedParams.value = params.map(p => p.name)
    pendingParamSql.value = sql
    showParamBinding.value = true
    return
  }

  await executeSingleStatement()
}

function handleInsertSnippet(e: CustomEvent<{ text: string }>): void {
  const { text } = e.detail
  if (text) {
    focus()
    insertText(text)
  }
}

async function handleExecuteNew(): Promise<void> {
  const connId = selectedConnection.value
  if (!connId) {
    message.warning(t('sqlEditor.noConnection'))
    return
  }
  await ensureConnection(connId)
  await executeNewTab()
}

async function handleParamConfirm(values: Record<string, string>): Promise<void> {
  showParamBinding.value = false
  const boundSql = buildBoundSql(pendingParamSql.value, values)
  pendingParamSql.value = ''
  detectedParams.value = []

  const connId = runtimeConnId.value
  if (!connId) {
    message.warning('No active connection')
    return
  }

  executing.value = true
  try {
    const result = await queryService.executeSql(boundSql, connId)
    const qr = (result as Record<string, unknown>).result || result as Record<string, unknown>

    lastExecutionTime.value = (result as Record<string, unknown>).elapsed_ms as number ?? 0

    if ((qr as Record<string, unknown>).error) {
      currentResultData.value = {
        columns: [],
        rows: [],
        totalRows: 0,
        elapsedMs: lastExecutionTime.value,
        affectedRows: 0,
        error: (qr as Record<string, unknown>).error as string,
      }
      hasResults.value = true
      message.error((qr as Record<string, unknown>).error as string)
    } else {
      currentResultData.value = {
        columns: ((qr as Record<string, unknown>).columns as string[]) ?? [],
        rows: ((qr as Record<string, unknown>).rows as unknown[][]) ?? [],
        totalRows: (qr as Record<string, unknown>).total_rows as number ?? ((qr as Record<string, unknown>).rows as unknown[][])?.length ?? 0,
        elapsedMs: lastExecutionTime.value,
        affectedRows: (qr as Record<string, unknown>).affected_rows as number ?? 0,
        error: null,
      }
      hasResults.value = true

      const storeResult = useSqlExecutionStore()
      const executionResult = {
        panelId: panelId.value,
        result: {
          columns: ((qr as Record<string, unknown>).columns as string[]) ?? [],
          rows: ((qr as Record<string, unknown>).rows as unknown[][]) ?? [],
          rowCount: (qr as Record<string, unknown>).total_rows as number ?? ((qr as Record<string, unknown>).rows as unknown[][])?.length ?? 0,
          executionTime: lastExecutionTime.value,
          affectedRows: (qr as Record<string, unknown>).affected_rows as number ?? 0,
        },
        error: null,
        timestamp: Date.now(),
      }
      storeResult.executionResults.set(panelId.value, executionResult)
      storeResult.executionResults = new Map(storeResult.executionResults)
      storeResult.setActiveEditorPanelId(panelId.value)

      message.success(`${currentResultData.value.totalRows} rows returned in ${lastExecutionTime.value}ms`)
    }
  } catch (error) {
    message.error(error instanceof Error ? error.message : String(error))
  } finally {
    executing.value = false
  }
}

function handleParamCancel(): void {
  showParamBinding.value = false
  pendingParamSql.value = ''
  detectedParams.value = []
}

async function handleDuckDbExecute(): Promise<void> {
  const connId = selectedConnection.value
  if (!connId) {
    message.warning(t('sqlEditor.noConnection'))
    return
  }
  await ensureConnection(connId)
  await executeDuckDBAccelerated()
}

async function handleBatchExecute(): Promise<void> {
  const connId = selectedConnection.value
  if (!connId) {
    message.warning(t('sqlEditor.noConnection'))
    return
  }
  await ensureConnection(connId)
  await executeBatch()
}

async function handleExplain(): Promise<void> {
  const connId = selectedConnection.value
  if (!connId) {
    message.warning(t('sqlEditor.noConnection'))
    return
  }

  const sql = getSelectedText() || getValue()
  if (!sql.trim()) return

  await ensureConnection(connId)

  const explainSql = `EXPLAIN ${sql}`
  const resultStore = useResultStore()

  try {
    const result = await invoke('execute_sql', {
      input: {
        conn_id: runtimeConnId.value,
        sql: explainSql,
        timeout_ms: null,
      },
    })

    const tab = resultStore.addTab(explainSql, connId)
    tab.title = t('sqlEditor.explain')
    resultStore.setTabResult(tab.id, {
      columns: result.result.columns || [],
      rows: result.result.rows || [],
      rowCount: result.result.rowCount || 0,
      elapsedMs: result.result.elapsedMs || 0,
    })
  } catch (error) {
    message.error(error instanceof Error ? error.message : String(error))
  }
}

async function handleSaveSnippet(): Promise<void> {
  const sql = getSelectedText() || getValue()
  if (!sql.trim()) {
    message.warning('No SQL to save')
    return
  }

  const preview = sql.replace(/\s+/g, ' ').trim().slice(0, 60)
  const label = `${preview}${sql.trim().length > 60 ? '...' : ''}`

  addCustomSnippet({
    label,
    detail: preview,
    insertText: sql.trim(),
    category: t('sqlEditor.favorites') || '收藏',
  })

  message.success(t('sqlEditor.snippetSaved'))
}

async function handleFormat(): Promise<void> {
  const sql = getValue()
  if (!sql.trim()) return
  try {
    const result = await formatSql(sql, getCurrentDialect())
    if (result) {
      setValue(result)
      message.success(t('sqlEditor.formatSuccess'))
    }
  } catch {
    message.error(t('sqlEditor.formatFailed'))
  }
}

async function handleValidate(): Promise<void> {
  const sql = getValue()
  if (!sql.trim()) return
  try {
    const markers = await validateSql(sql, getCurrentDialect())
    if (editorModel.value) {
      monaco.editor.setModelMarkers(editorModel.value, 'sql-validator', markers)
    }
    const errorCount = markers.filter(m => m.severity === monaco.MarkerSeverity.Error).length
    if (errorCount > 0) {
      message.warning(`${errorCount} error(s) found`)
    } else {
      message.success(t('sqlEditor.validateSuccess'))
    }
  } catch {
    message.error(t('sqlEditor.validateFailed'))
  }
}

async function handleTranspile(targetDialect: SqlDialect): Promise<void> {
  const sql = getValue()
  if (!sql.trim()) return
  try {
    const sourceDialect = getCurrentDialect()
    const result = await transpileSql(sql, sourceDialect, targetDialect)
    if (result && result !== sql) {
      setValue(result)
      message.success(t('sqlEditor.transpileSuccess', { dialect: targetDialect }))
    } else {
      message.info(t('sqlEditor.transpileSame'))
    }
  } catch {
    message.error(t('sqlEditor.transpileFailed'))
  } finally {
    showTranspileMenu.value = false
  }
}

async function handleScratchpadSave(): Promise<void> {
  if (!isScratchpadMode.value) return
  const content = getValue()
  try {
    await invoke('save_scratchpad_file', {
      relativePath: scratchpadRelativePath.value,
      content,
    })
    if (selectedConnection.value) {
      await invoke('update_scratchpad_file_meta', {
        relativePath: scratchpadRelativePath.value,
        connectionId: selectedConnection.value,
      })
    }
    isDirty.value = false
    message.success(t('common.saved'))
  } catch (e) {
    message.error(t('common.saveFailed') + ': ' + String(e))
  }
}

function markDirty() {
  if (isScratchpadMode.value) {
    isDirty.value = true
    scheduleAutoSave()
  }
}

let foldingDisposable: monaco.IDisposable | null = null
let autoSaveTimer: ReturnType<typeof setTimeout> | null = null

function scheduleAutoSave(): void {
  if (autoSaveTimer) clearTimeout(autoSaveTimer)
  autoSaveTimer = setTimeout(() => {
    if (isDirty.value) {
      handleScratchpadSave()
    }
  }, 2000)
}

const dialectOptions = computed(() => {
  const current = getCurrentDialect()
  const allDialects: Array<{ label: string; key: SqlDialect }> = [
    { label: 'MySQL', key: 'mysql' },
    { label: 'PostgreSQL', key: 'postgres' },
    { label: 'SQLite', key: 'sqlite' },
    { label: 'DuckDB', key: 'duckdb' },
    { label: 'SQL Server', key: 'mssql' },
    { label: 'Oracle', key: 'oracle' },
    { label: 'Snowflake', key: 'snowflake' },
    { label: 'BigQuery', key: 'bigquery' },
    { label: 'Redshift', key: 'redshift' },
  ]
  return allDialects.filter(d => d.key !== current)
})

onMounted(async () => {
  monaco.editor.defineTheme('rdata-dark', rdataDark)
  monaco.editor.defineTheme('rdata-light', rdataLight)

  createEditor()

  setupEditorEvents(
    value => {
      emit('update:modelValue', value)
      scheduleParse()
      markDirty()
    },
    _info => {
      // selection info handled internally
    }
  )

  setupEditorCommands({
    [monaco.KeyMod.CtrlCmd | monaco.KeyCode.Enter]: () => {
      if (isSqlLanguage.value) handleExecute()
    },
    [monaco.KeyMod.CtrlCmd | monaco.KeyMod.Shift | monaco.KeyCode.Enter]: () => {
      if (isSqlLanguage.value) handleExecuteNew()
    },
    [monaco.KeyMod.CtrlCmd | monaco.KeyMod.Shift | monaco.KeyCode.KeyF]: () => {
      if (isSqlLanguage.value) handleFormat()
    },
    [monaco.KeyMod.CtrlCmd | monaco.KeyCode.KeyR]: () => {
      if (isSqlLanguage.value) handleExecute()
    },
    [monaco.KeyCode.F5]: () => {
      if (isSqlLanguage.value) handleExecute()
    },
    [monaco.KeyMod.CtrlCmd | monaco.KeyCode.KeyS]: handleScratchpadSave,
  })

  if (paramsConnectionId.value && selectedConnection.value) {
    await ensureConnection(selectedConnection.value)
  }

  if (!isScratchpadMode.value) {
    const dbType = currentDatabaseType.value ?? undefined
    registerDatabaseCompletionProvider(
      runtimeConnId.value,
      currentDatabase.value,
      undefined,
      dbType,
    )
    startSync()
  }
  window.addEventListener('insert-snippet', handleInsertSnippet as (e: Event) => void)

  foldingDisposable = registerSqlFoldingProvider()
})

onBeforeUnmount(() => {
  if (autoSaveTimer) clearTimeout(autoSaveTimer)
  window.removeEventListener('insert-snippet', handleInsertSnippet as (e: Event) => void)
  if (!isScratchpadMode.value) {
    stopSync()
  }
  disposeMonacoDisposables()
  if (foldingDisposable) {
    foldingDisposable.dispose()
    foldingDisposable = null
  }
  disposeEditor()
})

watch(
  () => sqlExecutionStore.refreshRequests.get(panelId.value),
  timestamp => {
    if (timestamp) {
      handleExecute()
      sqlExecutionStore.refreshRequests.delete(panelId.value)
      sqlExecutionStore.refreshRequests = new Map(sqlExecutionStore.refreshRequests)
    }
  }
)
</script>

<style scoped>
.sql-editor-panel {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: var(--bg-primary, #1e1f22);
}

.sql-editor-panel.toolbar-left,
.sql-editor-panel.toolbar-right {
  flex-direction: row;
}

.editor-result-split {
  display: flex;
  flex: 1;
  min-height: 0;
  overflow: hidden;
}

.editor-and-result {
  display: flex;
  flex: 1;
  min-height: 0;
  overflow: hidden;
}

.editor-container {
  position: relative;
  min-height: 100px;
  overflow: hidden;
}

.monaco-container {
  width: 100%;
  height: 100%;
}

.split-handle {
  height: 4px;
  background: var(--border-color, #3e3e42);
  cursor: ns-resize;
  flex-shrink: 0;
}

.split-handle:hover {
  background: var(--accent-color, #e17055);
}

.result-container {
  min-height: 60px;
  overflow: auto;
}
</style>
