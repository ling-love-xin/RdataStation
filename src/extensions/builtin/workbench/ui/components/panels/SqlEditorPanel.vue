<template>
  <div :class="['sql-editor-panel', `toolbar-${toolbarPosition}`]">
    <!-- 扁平工具栏（纯图标，无中文分组） -->
    <div class="editor-toolbar">
      <!-- 执行按钮 -->
      <NTooltip trigger="hover" placement="bottom">
        <template #trigger>
          <NButton size="small" quaternary type="primary" @click="handleExecute">
            <Play :size="14" />
          </NButton>
        </template>
        <span>执行 SQL (Ctrl+Enter)</span>
      </NTooltip>

      <NTooltip trigger="hover" placement="bottom">
        <template #trigger>
          <NButton size="small" quaternary @click="handleExecuteNew">
            <Plus :size="14" />
          </NButton>
        </template>
        <span>新标签执行 (Ctrl+Shift+Enter)</span>
      </NTooltip>

      <NTooltip v-if="isDuckDbConnection" trigger="hover" placement="bottom">
        <template #trigger>
          <NButton size="small" quaternary type="success" @click="handleDuckDbExecute">
            <Zap :size="14" />
          </NButton>
        </template>
        <span>DuckDB 加速执行</span>
      </NTooltip>

      <NTooltip trigger="hover" placement="bottom">
        <template #trigger>
          <NButton size="small" quaternary @click="handleExplain">
            <Search :size="14" />
          </NButton>
        </template>
        <span>执行计划</span>
      </NTooltip>

      <NDivider vertical class="toolbar-divider" />

      <!-- 格式化/验证/转换 -->
      <NTooltip trigger="hover" placement="bottom">
        <template #trigger>
          <NButton size="small" quaternary @click="handleFormat">
            <AlignLeft :size="14" />
          </NButton>
        </template>
        <span>格式化 (Ctrl+Shift+F)</span>
      </NTooltip>

      <NTooltip trigger="hover" placement="bottom">
        <template #trigger>
          <NButton size="small" quaternary @click="handleValidate">
            <Sparkles :size="14" />
          </NButton>
        </template>
        <span>验证 SQL</span>
      </NTooltip>

      <NTooltip trigger="hover" placement="bottom">
        <template #trigger>
          <NButton size="small" quaternary @click="showTranspileMenu = true">
            <ArrowLeftRight :size="14" />
          </NButton>
        </template>
        <span>方言转换</span>
      </NTooltip>

      <NDivider vertical class="toolbar-divider" />

      <!-- 历史/设置 -->
      <NTooltip trigger="hover" placement="bottom">
        <template #trigger>
          <NButton size="small" quaternary @click="showHistory = true">
            <History :size="14" />
          </NButton>
        </template>
        <span>执行历史</span>
      </NTooltip>

      <NTooltip trigger="hover" placement="bottom">
        <template #trigger>
          <NButton size="small" quaternary @click="handleSettings">
            <Settings :size="14" />
          </NButton>
        </template>
        <span>设置</span>
      </NTooltip>

      <div class="toolbar-spacer" />

      <!-- 工具栏位置切换 -->
      <NTooltip trigger="hover" placement="bottom">
        <template #trigger>
          <NButton size="small" quaternary @click="toggleToolbarPosition">
            <PanelLeftClose v-if="toolbarPosition === 'left'" :size="14" />
            <PanelRightClose v-else-if="toolbarPosition === 'right'" :size="14" />
            <PanelTopClose v-else :size="14" />
          </NButton>
        </template>
        <span>工具栏位置</span>
      </NTooltip>
    </div>

    <!-- 方言转换弹窗 -->
    <NModal v-model:show="showTranspileMenu">
      <div class="transpile-modal">
        <div class="transpile-header">
          <h3>SQL 方言转换</h3>
          <NButton quaternary circle size="small" @click="showTranspileMenu = false">
            <X :size="16" />
          </NButton>
        </div>
        <p class="transpile-hint">将当前 SQL 转换为目标方言：</p>
        <div class="transpile-options">
          <NButton
            v-for="opt in dialectOptions"
            :key="opt.key"
            size="large"
            quaternary
            block
            @click="handleTranspileAndClose(opt.key)"
          >
            {{ opt.label }}
          </NButton>
        </div>
      </div>
    </NModal>

    <!-- 编辑器与结果垂直分割区域 -->
    <div class="editor-result-split">
      <!-- 编辑器区域 -->
      <div ref="editorContainer" class="editor-container" />

      <!-- 分割线（拖拽调整大小） -->
      <div
        v-if="hasResults"
        class="split-handle"
        @mousedown.prevent="startSplitDrag"
      />

      <!-- 内嵌结果面板 -->
      <div v-show="hasResults" ref="resultContainerRef" class="result-container">
        <QueryResultPanel
          v-if="currentResultData"
          :result-data="currentResultData"
          :compact="true"
        />
      </div>
    </div>

    <!-- 编辑器水印（空编辑器时显示为透明背景提示） -->
    <div v-if="showWelcome" class="editor-watermark">
      <div class="watermark-text">
        <div class="watermark-title">SQL 编辑器</div>
        <div class="watermark-shortcuts">
          <span class="shortcut-hint"><kbd>Ctrl</kbd>+<kbd>Enter</kbd> 执行</span>
          <span class="shortcut-hint"><kbd>Ctrl</kbd>+<kbd>Shift</kbd>+<kbd>F</kbd> 格式化</span>
          <span class="shortcut-hint"><kbd>Ctrl</kbd>+<kbd>/</kbd> 注释</span>
          <span class="shortcut-hint"><kbd>F5</kbd> 执行全部</span>
        </div>
      </div>
    </div>

    <!-- 状态栏 -->
    <div class="editor-statusbar">
      <!-- 左侧：光标位置、选中文本、编辑器模式 -->
      <div class="status-left">
        <span class="status-item">{{ cursorPosition }}</span>
        <span class="status-item">{{ selectedTextInfo }}</span>
        <span class="status-item">{{ editorMode }}</span>
        
        <!-- 执行状态指示器 -->
        <span v-if="executing" class="status-item executing">
          <span class="loading-dot"></span>
          执行中...
        </span>
        <span v-else-if="lastExecutionTime" class="status-item success">
          ✓ {{ lastExecutionTime }}ms
        </span>
      </div>

      <!-- 中间：连接信息（点击切换连接） -->
      <div class="status-center">
        <NPopselect
          v-model:value="selectedConnection"
          :options="popselectOptions"
          trigger="click"
          size="small"
          scrollable
          @update:value="onConnectionSelected"
        >
          <div class="connection-info-tag">
            <Database :size="12" />
            <span>{{ connectionInfoText }}</span>
          </div>
        </NPopselect>
      </div>

      <!-- 右侧：预留扩展 -->
      <div class="status-right" />
    </div>
  </div>
</template>

<script setup lang="ts">
import { invoke } from '@tauri-apps/api/core'
import {
  Play, Plus, AlignLeft, Settings, History, Sparkles, Search,
  Zap, ArrowLeftRight, Database, X,
  PanelLeftClose, PanelRightClose, PanelTopClose
} from 'lucide-vue-next'
import * as monaco from 'monaco-editor'
import { createDiscreteApi, darkTheme, lightTheme, NButton, NPopselect, NTooltip, NDivider, NModal } from 'naive-ui'
import { computed, onMounted, onUnmounted, ref, watch } from 'vue'

// 导入 SQL 语言支持
import 'monaco-editor/esm/vs/basic-languages/sql/sql.contribution'

import { useConnectionStore } from '@/extensions/builtin/connection/ui/stores/connection-store'
import { useRuntimeConnectionStore } from '@/extensions/builtin/connection/ui/stores/runtime-connection-store'
import { registerDialectHighlight } from '@/extensions/builtin/workbench/services/sql-dialect-highlight'
import {
  formatSql,
  parseSql,
  registerDatabaseCompletionProvider,
  transpileSql,
  type SqlDialect,
  unregisterCompletionProvider,
  validateSql
} from '@/extensions/builtin/workbench/services/sql-editor-service'
import { addHistory } from '@/extensions/builtin/workbench/services/sql-history-service'
import { getAllSnippets } from '@/extensions/builtin/workbench/services/sql-snippets'
import { useSqlExecutionStore } from '@/extensions/builtin/workbench/ui/stores/sql-execution-store'
import { useUiStore } from '@/shared/stores/ui'

import QueryResultPanel from './QueryResultPanel.vue'

// 使用 createDiscreteApi 创建独立于 NMessageProvider 的 message/dialog 实例
const uiStore = useUiStore()
const configProviderPropsRef = ref({
  theme: uiStore.isDark ? darkTheme : lightTheme
})
const { message, dialog } = createDiscreteApi(
  ['message', 'dialog'],
  {
    configProviderProps: configProviderPropsRef
  }
)

// 获取当前连接的数据库类型
const getCurrentDialect = (): SqlDialect => {
  const conn = connectionStore.connections.find(c => c.connId === selectedConnection.value)
  if (!conn) return 'generic'
  
  const typeMap: Record<string, SqlDialect> = {
    'mysql': 'mysql',
    'postgres': 'postgres',
    'sqlite': 'sqlite',
    'duckdb': 'duckdb',
    'mssql': 'mssql',
    'oracle': 'oracle',
    'snowflake': 'snowflake',
    'bigquery': 'bigquery',
    'redshift': 'redshift'
  }
  
  const dbType = (conn as any).dbType || (conn as any).type || ''
  return typeMap[dbType.toLowerCase()] || 'generic'
}

// Stores
const sqlExecutionStore = useSqlExecutionStore()
const connectionStore = useConnectionStore()
const runtimeConnectionStore = useRuntimeConnectionStore()

// 面板 ID（用于绑定执行结果）
const panelId = computed(() => props.params?.panelId || (props.params as any)?.params?.panelId || `sql-editor-${Date.now()}`)

// 1:n 内嵌结果面板状态
const hasResults = ref(false)
const resultContainerRef = ref<HTMLElement>()
const splitRatio = ref(0.55)
let splitDragging = false

interface ResultData {
  columns: string[]
  rows: unknown[][]
  originalSql?: string
  connectionId?: string
  elapsedMs?: number
}
const currentResultData = ref<ResultData | null>(null)

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
  const onUp = () => { splitDragging = false; document.removeEventListener('mousemove', onMove); document.removeEventListener('mouseup', onUp) }
  document.addEventListener('mousemove', onMove)
  document.addEventListener('mouseup', onUp)
}



// NPopselect 选项（从 runtimeConnectionIds + connectionStore 合并）
const popselectOptions = computed(() => {
  const options: Array<{ label: string; value: string }> = []
  const seen = new Set<string>()

  // 先从 connectionStore 取已连接的
  for (const conn of connectionStore.connections) {
    if (conn.status === 'connected' || conn.connId === selectedConnection.value) {
      options.push({
        label: `${conn.name || conn.connId} (${(conn as any).dbType || (conn as any).type || 'unknown'})`,
        value: conn.connId
      })
      seen.add(conn.connId)
    }
  }

  // 再从 runtimeConnectionIds 补充（可能 connectionStore 尚未同步）
  for (const [connId] of runtimeConnectionStore.runtimeConnectionIds) {
    if (!seen.has(connId)) {
      options.push({
        label: `${connId} (已连接)`,
        value: connId
      })
      seen.add(connId)
    }
  }

  return options
})

// 连接选择回调
const onConnectionSelected = (connId: string) => {
  selectedConnection.value = connId
  updateDialectHighlight()
}

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
  // Dockview params
  params?: {
    connectionId?: string
    databaseName?: string
    initialSql?: string
    panelId?: string
    schema?: string
  }
}

const props = withDefaults(defineProps<Props>(), {
  modelValue: '',
  language: 'sql',
  theme: 'vs-dark',
  connections: () => [],
  params: undefined
})

const emit = defineEmits<{
  'update:modelValue': [value: string]
  'execute': [sql: string, connectionId: string]
  'format': [sql: string]
}>()

// 编辑器实例
let editor: monaco.editor.IStandaloneCodeEditor | null = null
const editorContainer = ref<HTMLElement>()

// 状态
const executing = ref(false)
const lastExecutionTime = ref<number | null>(null)
const selectedConnection = ref(props.connectionId || props.params?.connectionId || (props.params as any)?.params?.connectionId || '')
const cursorPosition = ref('Ln 1, Col 1')
const selectedTextInfo = ref('')
const editorMode = ref('SQL')
const showHistory = ref(false)
const validationErrors = ref<string[]>([])
const currentDialect = ref<SqlDialect>('generic')
const showWelcome = ref(true)
const showTranspileMenu = ref(false)
const toolbarPosition = ref<'top' | 'left' | 'right'>('top')
let currentHighlightDisposable: monaco.IDisposable | null = null

// 当前选中连接的详细信息
const selectedConnectionInfo = computed(() => {
  if (!selectedConnection.value) return null
  return connectionStore.connections.find(c => c.connId === selectedConnection.value) || null
})

// 运行时连接 ID（后端实际使用的 ID），从配置ID翻译得来
const runtimeConnId = computed(() => {
  if (!selectedConnection.value) return ''
  return runtimeConnectionStore.runtimeConnectionIds.get(selectedConnection.value) || ''
})

// 是否为 DuckDB 连接（或已启用 DuckDB 本地加速）
const isDuckDbConnection = computed(() => {
  if (!selectedConnection.value) return false
  return runtimeConnectionStore.isDuckDbEnabled(selectedConnection.value)
})

// 状态栏连接信息文本
const connectionInfoText = computed(() => {
  // 兼容 Dockview 嵌套 params
  const paramsDbName = props.params?.databaseName || (props.params as any)?.params?.databaseName || ''
  
  // 方式一：从 connectionStore 获取详细信息
  const conn = selectedConnectionInfo.value
  if (conn) {
    const name = conn.name || conn.connId
    const db = paramsDbName || (conn as any).database || ''
    const schema = props.params?.schema || (props.params as any)?.params?.schema || 'public'
    const dbType = (conn as any).dbType || (conn as any).type || ''
    if (db) return `${name} → ${db} → ${schema}`
    return `${name} → ${dbType}`
  }
  
  // 方式二：运行时连接已建立但 connectionStore 未同步
  if (selectedConnection.value && runtimeConnectionStore.runtimeConnectionIds.has(selectedConnection.value)) {
    return `${selectedConnection.value} (已连接)`
  }
  
  // 方式三：有 ID 但无运行时连接
  if (selectedConnection.value) {
    return selectedConnection.value
  }
  
  return '未连接'
})

// 方言转换选项
const dialectOptions = computed(() => {
  const currentDialect = getCurrentDialect()
  const allDialects: Array<{ label: string; key: SqlDialect }> = [
    { label: 'MySQL', key: 'mysql' },
    { label: 'PostgreSQL', key: 'postgres' },
    { label: 'SQLite', key: 'sqlite' },
    { label: 'DuckDB', key: 'duckdb' },
    { label: 'SQL Server', key: 'mssql' },
    { label: 'Oracle', key: 'oracle' },
    { label: 'Snowflake', key: 'snowflake' },
    { label: 'BigQuery', key: 'bigquery' },
    { label: 'Redshift', key: 'redshift' }
  ]
  
  // 过滤掉当前方言
  return allDialects.filter(d => d.key !== currentDialect)
})

// 初始化编辑器
const initEditor = () => {
  if (!editorContainer.value) return

  // 获取初始 SQL（优先使用 params.initialSql）
  let initialValue = props.params?.initialSql || props.modelValue
  
  // 尝试恢复草稿
  const draftKey = `sql-draft-${panelId.value}`
  const draft = localStorage.getItem(draftKey)
  if (draft && !initialValue) {
    initialValue = draft
    message.info('已恢复上次未保存的内容')
  }

  // 配置 Monaco Editor
  editor = monaco.editor.create(editorContainer.value, {
    value: initialValue,
    language: props.language,
    theme: props.theme,
    automaticLayout: true,
    minimap: {
      enabled: true,
      scale: 1,
      showSlider: 'mouseover'
    },
    fontSize: 14,
    fontFamily: 'JetBrains Mono, Fira Code, Consolas, monospace',
    lineNumbers: 'on',
    roundedSelection: false,
    scrollBeyondLastLine: false,
    readOnly: false,
    cursorStyle: 'line',
    wordWrap: 'on',
    folding: true,
    foldingStrategy: 'indentation',
    showFoldingControls: 'always',
    matchBrackets: 'always',
    autoIndent: 'full',
    formatOnPaste: true,
    formatOnType: true,
    suggestOnTriggerCharacters: true,
    quickSuggestions: {
      other: true,
      comments: false,
      strings: false
    },
    snippetSuggestions: 'top',
    wordBasedSuggestions: 'currentDocument',
    parameterHints: {
      enabled: true,
      cycle: true
    },
    hover: {
      enabled: true,
      delay: 300
    },
    // 禁用颜色检测（避免 Web Worker 问题）
    colorDecorators: false,
    links: false,
    // 禁用所有可能触发 worker 的功能
    renderValidationDecorations: 'off'
  })

  // 监听内容变化
  let validationTimer: ReturnType<typeof setTimeout> | null = null
  let saveTimer: ReturnType<typeof setTimeout> | null = null
  
  // 初始化欢迎页显示状态
  showWelcome.value = !initialValue || initialValue.trim().length === 0
  
  editor.onDidChangeModelContent(() => {
    const value = editor?.getValue() || ''
    emit('update:modelValue', value)
    
    // 控制欢迎页显示
    showWelcome.value = value.trim().length === 0
    
    // 实时验证（500ms 防抖）
    if (validationTimer) {
      clearTimeout(validationTimer)
    }
    validationTimer = setTimeout(async () => {
      if (value.trim().length > 0) {
        const dialect = getCurrentDialect()
        const markers = await validateSql(value, dialect)
        
        const model = editor!.getModel()
        if (model) {
          monaco.editor.setModelMarkers(model, 'sql-validation', markers)
        }
        
        // 更新错误列表
        validationErrors.value = markers.map(m => `第 ${m.startLineNumber} 行: ${m.message}`)
      }
    }, 500)
    
    // 自动保存草稿（1000ms 防抖）
    if (saveTimer) {
      clearTimeout(saveTimer)
    }
    saveTimer = setTimeout(() => {
      const draftKey = `sql-draft-${panelId.value}`
      localStorage.setItem(draftKey, value)
    }, 1000)
  })

  // 监听光标位置变化
  editor.onDidChangeCursorPosition((e) => {
    cursorPosition.value = `Ln ${e.position.lineNumber}, Col ${e.position.column}`
  })

  // 监听选择变化
  editor.onDidChangeCursorSelection(() => {
    const selection = editor?.getSelection()
    if (selection && !selection.isEmpty()) {
      const selectedText = editor?.getModel()?.getValueInRange(selection) || ''
      const lines = selectedText.split('\n').length
      const chars = selectedText.length
      selectedTextInfo.value = `已选择 ${lines} 行, ${chars} 字符`
    } else {
      selectedTextInfo.value = ''
    }
  })

  // 添加快捷键
  // Ctrl+Enter: 执行 SQL
  editor.addCommand(monaco.KeyMod.CtrlCmd | monaco.KeyCode.Enter, () => {
    handleExecute()
  })

  // Ctrl+Shift+F: 格式化 SQL
  editor.addCommand(monaco.KeyMod.CtrlCmd | monaco.KeyMod.Shift | monaco.KeyCode.KeyF, () => {
    handleFormat()
  })

  // Ctrl+R: 执行选中 SQL
  editor.addCommand(monaco.KeyMod.CtrlCmd | monaco.KeyCode.KeyR, () => {
    handleExecute()
  })

  // Ctrl+/: 注释/取消注释
  editor.addCommand(monaco.KeyMod.CtrlCmd | monaco.KeyCode.Slash, () => {
    editor!.trigger('keyboard', 'editor.action.commentLine', {})
  })

  // Ctrl+Shift+R: 刷新结果
  editor.addCommand(monaco.KeyMod.CtrlCmd | monaco.KeyMod.Shift | monaco.KeyCode.KeyR, () => {
    window.dispatchEvent(new CustomEvent('refresh-query-result', {
      detail: { panelId: panelId.value }
    }))
  })

  // Ctrl+L: 清空编辑器
  editor.addCommand(monaco.KeyMod.CtrlCmd | monaco.KeyCode.KeyL, () => {
    if (editor) {
      editor.setValue('')
      message.success('已清空编辑器')
    }
  })

  // Ctrl+S: 保存 SQL 文件
  editor.addCommand(monaco.KeyMod.CtrlCmd | monaco.KeyCode.KeyS, () => {
    handleSaveSql()
  })

  // F5: 执行全部
  editor.addCommand(monaco.KeyCode.F5, () => {
    handleExecute()
  })

  // Ctrl+Shift+Enter: 执行并打开新结果标签
  editor.addCommand(monaco.KeyMod.CtrlCmd | monaco.KeyMod.Shift | monaco.KeyCode.Enter, () => {
    handleExecuteNew()
  })

  // 注册 SQL 代码补全提供器
  registerSqlCompletionProvider()
  
  // 如果有连接和数据库信息，注册数据库相关的代码补全（兼容 Dockview 嵌套 params）
  const initConnId = props.params?.connectionId || (props.params as any)?.params?.connectionId || ''
  const initDbName = props.params?.databaseName || (props.params as any)?.params?.databaseName || ''
  if (initConnId && initDbName) {
    const conn = connectionStore.connections.find(c => c.connId === initConnId)
    registerDatabaseCompletionProvider(
      initConnId,
      initDbName,
      'public',
      (conn as any)?.dbType
    )
  }

  // 注册方言高亮规则
  updateDialectHighlight()
}

// 更新方言高亮规则
const updateDialectHighlight = () => {
  const dialect = getCurrentDialect()
  
  // 如果方言没有变化，不重新注册
  if (dialect === currentDialect.value) return
  
  // 注销旧的高亮规则
  if (currentHighlightDisposable) {
    currentHighlightDisposable.dispose()
    currentHighlightDisposable = null
  }
  
  // 注册新的高亮规则
  currentHighlightDisposable = registerDialectHighlight(dialect)
  currentDialect.value = dialect
  
  console.log(`[SqlEditor] 已切换到 ${dialect} 方言高亮`)
}

// 注册 SQL 代码补全提供器
const registerSqlCompletionProvider = () => {
  monaco.languages.registerCompletionItemProvider('sql', {
    triggerCharacters: ['.', ' '],
    provideCompletionItems: (model, position) => {
      const word = model.getWordUntilPosition(position)
      const range = {
        startLineNumber: position.lineNumber,
        endLineNumber: position.lineNumber,
        startColumn: word.startColumn,
        endColumn: word.endColumn
      }

      // SQL 关键字
      const keywords = [
        'SELECT', 'FROM', 'WHERE', 'INSERT', 'UPDATE', 'DELETE',
        'CREATE', 'DROP', 'ALTER', 'TABLE', 'INDEX', 'VIEW',
        'JOIN', 'LEFT', 'RIGHT', 'INNER', 'OUTER', 'ON',
        'GROUP', 'BY', 'ORDER', 'HAVING', 'LIMIT', 'OFFSET',
        'AND', 'OR', 'NOT', 'NULL', 'IS', 'IN', 'EXISTS',
        'COUNT', 'SUM', 'AVG', 'MAX', 'MIN', 'AS', 'DISTINCT'
      ]

      const suggestions = keywords.map((keyword, index) => ({
        label: keyword,
        kind: monaco.languages.CompletionItemKind.Keyword,
        insertText: keyword,
        range: range,
        sortText: String(index).padStart(3, '0')
      }))

      // 使用 SQL 模板库
      const snippets = getAllSnippets()

      snippets.forEach((snippet, index) => {
        suggestions.push({
          label: snippet.label,
          kind: monaco.languages.CompletionItemKind.Snippet,
          insertText: snippet.insertText,
          range: range,
          sortText: 'z' + String(index).padStart(3, '0'),
          insertTextRules: monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet
        } as any)
      })

      return { suggestions }
    }
  })
}

/**
 * 确保运行时连接已建立，未建立时自动连接
 */
async function ensureConnection(connectionId: string): Promise<boolean> {
  console.log(`[ensureConnection] connectionId="${connectionId}"`)
  console.log(`[ensureConnection]   runtimeConnectionIds has? ${runtimeConnectionStore.runtimeConnectionIds.has(connectionId)}`)
  
  // 运行时连接已存在
  if (runtimeConnectionStore.runtimeConnectionIds.has(connectionId)) {
    const runtimeId = runtimeConnectionStore.runtimeConnectionIds.get(connectionId)
    console.log(`[ensureConnection] ✅ runtime connection exists, runtimeId="${runtimeId}"`)
    return true
  }

  // 尝试从 connectionStore 获取连接配置
  const conn = connectionStore.connections.find(c => c.connId === connectionId)
  console.log(`[ensureConnection]   connectionStore.find result:`, conn ? { connId: conn.connId, name: conn.name, dbType: conn.dbType, url: conn.url } : null)
  
  if (conn) {
    console.log(`[ensureConnection]   establishing from connectionStore...`)
    const result = await runtimeConnectionStore.establishFromConnection(conn)
    console.log(`[ensureConnection]   establishFromConnection result:`, result)
    return result !== null
  }

  console.log(`[ensureConnection] ❌ connection not found anywhere`)
  return false
}

// 执行 SQL
const handleExecute = async () => {
  if (!editor || executing.value) return

  const sql = editor.getValue().trim()
  if (!sql) return

  // 如果有选中的文本，执行选中的部分
  const selection = editor.getSelection()
  let executeSql = sql
  if (selection && !selection.isEmpty()) {
    executeSql = editor.getModel()?.getValueInRange(selection) || sql
  }

  if (!selectedConnection.value) {
    message.warning('请先选择数据库连接')
    return
  }

  console.log(`[handleExecute] selectedConnection="${selectedConnection.value}"`)
  console.log(`[handleExecute] runtimeConnId="${runtimeConnId.value}"`)
  console.log(`[handleExecute] connectionStore.connections=`, JSON.parse(JSON.stringify(connectionStore.connections.map(c => ({ connId: c.connId, name: c.name, status: c.status })))))
  console.log(`[handleExecute] runtimeConnectionIds=`, Object.fromEntries(runtimeConnectionStore.runtimeConnectionIds))

  // 自动建立运行时连接（如果尚未建立）
  const connOk = await ensureConnection(selectedConnection.value)
  if (!connOk) {
    message.error('无法连接到数据库，请检查连接配置')
    return
  }

  // 检测多语句
  const dialect = getCurrentDialect()
  const parseResult = await parseSql(executeSql, dialect)
  
  let statementsCount = 0
  if (parseResult.success) {
    statementsCount = parseResult.statementsCount
  } else {
    // parseSql 可能对 DDL 语句（如 CREATE TABLE）解析失败
    // 此时按单语句执行，由后端 SQL 引擎去处理语法
    statementsCount = 1
  }

  executing.value = true
  try {
    if (statementsCount > 1) {
      // 多语句执行
      await executeMultipleStatements(executeSql, dialect)
    } else {
      // 单语句执行
      await executeSingleStatement(executeSql)
    }
  } catch (error) {
    const errorMsg = error instanceof Error ? error.message : String(error) || '执行失败'
    message.error(errorMsg)
    console.error('[handleExecute] 执行失败:', error)
  } finally {
    executing.value = false
  }
}

// 执行并打开新结果标签（Execute+）
const handleExecuteNew = async () => {
  if (!editor || executing.value) return
  const sql = editor.getValue().trim()
  if (!sql) return

  const selection = editor.getSelection()
  let executeSql = sql
  if (selection && !selection.isEmpty()) {
    executeSql = editor.getModel()?.getValueInRange(selection) || sql
  }

  if (!selectedConnection.value) {
    message.warning('请先选择数据库连接')
    return
  }

  const connOk = await ensureConnection(selectedConnection.value)
  if (!connOk) {
    message.error('无法连接到数据库')
    return
  }

  executing.value = true
  const startTime = Date.now()
  try {
    const result = await sqlExecutionStore.executeSql(panelId.value, executeSql, runtimeConnId.value)
    const executionTime = Date.now() - startTime
    lastExecutionTime.value = executionTime

    // 写入内嵌结果面板（Execute+ 打开新标签）
    if (result.result && result.result.columns && result.result.rows) {
      currentResultData.value = {
        columns: result.result.columns,
        rows: result.result.rows,
        originalSql: executeSql,
        connectionId: selectedConnection.value,
        elapsedMs: executionTime
      }
      hasResults.value = true
    }

    // 使用 query-result-new 事件通知面板打开新标签
    window.dispatchEvent(new CustomEvent('query-result-new', {
      detail: {
        result: result.result,
        error: result.error,
        originalSql: executeSql,
        connectionId: selectedConnection.value,
        elapsedMs: executionTime
      }
    }))

    if (result.error) {
      message.error(result.error)
    } else {
      message.success(`执行成功，${result.result?.rowCount || 0} 行`)
    }
  } catch (error) {
    message.error(error instanceof Error ? error.message : String(error))
  } finally {
    executing.value = false
  }
}

// 执行单条 SQL
const executeSingleStatement = async (sql: string) => {
  const startTime = Date.now()
  let result: any
  try {
    result = await sqlExecutionStore.executeSql(
      panelId.value,
      sql,
      runtimeConnId.value
    )
  } catch (e) {
    console.error('[executeSingleStatement] Tauri invoke 失败:', e)
    message.error(e instanceof Error ? e.message : String(e))
    return
  }
  
  const executionTime = Date.now() - startTime
  lastExecutionTime.value = executionTime

  // 写入内嵌结果面板
  if (result.result && result.result.columns && result.result.rows) {
    currentResultData.value = {
      columns: result.result.columns,
      rows: result.result.rows,
      originalSql: sql,
      connectionId: selectedConnection.value,
      elapsedMs: executionTime
    }
    hasResults.value = true
  }
  
  // 发送结果到结果面板
  window.dispatchEvent(new CustomEvent('sql-execution-result', {
    detail: {
      panelId: panelId.value,
      result: result.result,
      error: result.error,
      statementIndex: 0,
      totalStatements: 1,
      originalSql: sql,
      connectionId: selectedConnection.value,
      elapsedMs: executionTime
    }
  }))
  
  // 记录执行历史
  const conn = connectionStore.connections.find(c => c.connId === selectedConnection.value)
  const dbType = (conn as any)?.dbType || (conn as any)?.type || 'unknown'
  addHistory({
    sql,
    connectionId: selectedConnection.value!,
    connectionName: conn?.name || selectedConnection.value!,
    databaseType: dbType,
    executionTime,
    rowCount: result.result?.rowCount || 0,
    success: !result.error,
    error: result.error || undefined,
    isFavorite: false
  })
  
  // 显示执行结果提示
  if (result.error) {
    message.error(result.error)
  } else {
    message.success(`执行成功，${result.result?.rowCount || 0} 行，耗时 ${result.result?.executionTime || 0}ms`)
  }
}

// 执行多条 SQL
const executeMultipleStatements = async (sql: string, dialect: SqlDialect) => {
  // 解析并分割语句
  const statements = await splitSqlStatements(sql, dialect)
  
  // 显示执行模式选择
  const mode = await showExecutionModeDialog(statements.length)
  
  if (mode === 'cancel') return
  
  const results: Array<{ index: number; result: any; error: string | null }> = []
  
  if (mode === 'batch') {
    // 批量执行（全部执行完再显示结果）
    for (let i = 0; i < statements.length; i++) {
      try {
        const result = await sqlExecutionStore.executeSql(
          panelId.value,
          statements[i],
          runtimeConnId.value
        )
        results.push({ index: i, result: result.result, error: null })
      } catch (error) {
        const errorMsg = error instanceof Error ? error.message : '执行失败'
        results.push({ index: i, result: null, error: errorMsg })
        // 批量模式下遇到错误继续执行
      }
    }
    
    // 显示所有结果
    window.dispatchEvent(new CustomEvent('sql-execution-result', {
      detail: {
        panelId: panelId.value,
        results,
        error: null,
        statementIndex: -1,
        totalStatements: statements.length,
        isBatch: true
      }
    }))
    
    const successCount = results.filter(r => !r.error).length
    message.success(`批量执行完成，${successCount}/${statements.length} 成功`)
  } else {
    // 逐条执行（立即显示每条结果）
    for (let i = 0; i < statements.length; i++) {
      try {
        const result = await sqlExecutionStore.executeSql(
          panelId.value,
          statements[i],
          runtimeConnId.value
        )
        results.push({ index: i, result: result.result, error: null })
        
        // 立即发送当前结果
        window.dispatchEvent(new CustomEvent('sql-execution-result', {
          detail: {
            panelId: panelId.value,
            result: result.result,
            error: null,
            statementIndex: i,
            totalStatements: statements.length,
            isBatch: false
          }
        }))
        
        message.success(`语句 ${i + 1}/${statements.length} 执行成功`)
      } catch (error) {
        const errorMsg = error instanceof Error ? error.message : '执行失败'
        results.push({ index: i, result: null, error: errorMsg })
        
        window.dispatchEvent(new CustomEvent('sql-execution-result', {
          detail: {
            panelId: panelId.value,
            result: null,
            error: errorMsg,
            statementIndex: i,
            totalStatements: statements.length,
            isBatch: false
          }
        }))
        
        message.error(`语句 ${i + 1}/${statements.length} 执行失败: ${errorMsg}`)
        // 逐条模式下遇到错误停止执行
        break
      }
    }
  }
}

// 分割 SQL 语句
const splitSqlStatements = async (sql: string, dialect: SqlDialect): Promise<string[]> => {
  try {
    // 使用 sqlglot-rust 解析语句
    const result = await invoke<any>('split_sql', {
      sql,
      dialect: dialect || 'generic'
    })
    
    if (result.success && result.statements) {
      return result.statements
    }
  } catch (error) {
    console.warn('SQL 分割失败，使用简单分割:', error)
  }
  
  // 降级方案：简单按分号分割
  return sql
    .split(';')
    .map(s => s.trim())
    .filter(s => s.length > 0)
}

// 显示执行模式对话框
const showExecutionModeDialog = (statementCount: number): Promise<'batch' | 'sequential' | 'cancel'> => {
  return new Promise((resolve) => {
    // 使用 naive-ui 对话框
    const d = dialog.info({
      title: '检测到多条 SQL 语句',
      content: `共 ${statementCount} 条语句，请选择执行模式：`,
      positiveText: '批量执行',
      negativeText: '逐条执行',
      closable: true,
      onPositiveClick: () => resolve('batch'),
      onNegativeClick: () => resolve('sequential'),
      onClose: () => resolve('cancel')
    })
    
    // 5 秒后自动选择批量执行
    setTimeout(() => {
      if (d) {
        d.destroy()
        resolve('batch')
      }
    }, 5000)
  })
}

// 格式化 SQL
const handleFormat = async () => {
  if (!editor) return

  const sql = editor.getValue()
  const dialect = getCurrentDialect()
  const formatted = await formatSql(sql, dialect)
  
  editor.setValue(formatted)
  emit('format', formatted)
}

// 验证 SQL
const handleValidate = async () => {
  if (!editor) return

  const sql = editor.getValue()
  const dialect = getCurrentDialect()
  const markers = await validateSql(sql, dialect)
  
  // 在编辑器中显示错误标记
  const model = editor!.getModel()
  if (model) {
    monaco.editor.setModelMarkers(model, 'sql-validation', markers)
  }
  
  // 更新错误列表
  validationErrors.value = markers.map(m => `第 ${m.startLineNumber} 行: ${m.message}`)
  
  if (markers.length === 0) {
    // 显示成功提示
    const model = editor.getModel()
    if (model) {
      monaco.editor.setModelMarkers(model, 'sql-validation', [{
        severity: monaco.MarkerSeverity.Info,
        message: '✓ SQL 语法检查通过',
        startLineNumber: 1,
        startColumn: 1,
        endLineNumber: 1,
        endColumn: 1
      }])
    }
  }
}

// 设置
const handleSettings = () => {
  // TODO: 打开编辑器设置面板
  console.log('打开编辑器设置')
}

// 保存 SQL 文件
const handleSaveSql = () => {
  if (!editor) return
  
  const sql = editor.getValue()
  if (!sql.trim()) {
    message.warning('没有可保存的内容')
    return
  }
  
  // 触发保存对话框
  window.dispatchEvent(new CustomEvent('save-sql-file', {
    detail: {
      sql,
      panelId: panelId.value
    }
  }))
  
  message.success('SQL 已保存')
}

// 执行计划
const handleExplain = async () => {
  if (!editor || executing.value) return

  const sql = editor.getValue().trim()
  if (!sql) return

  if (!selectedConnection.value) {
    message.warning('请先选择数据库连接')
    return
  }

  // 自动建立运行时连接
  const connOk = await ensureConnection(selectedConnection.value)
  if (!connOk) {
    message.error('无法连接到数据库，请检查连接配置')
    return
  }

  executing.value = true
  const startTime = Date.now()
  try {
    // 在 SQL 前添加 EXPLAIN
    const explainSql = `EXPLAIN ${sql}`
    
    // 使用 Pinia Store 执行
    const result = await sqlExecutionStore.executeSql(
      panelId.value,
      explainSql,
      runtimeConnId.value
    )
    
    // 发送结果到结果面板
    window.dispatchEvent(new CustomEvent('sql-execution-result', {
      detail: {
        panelId: panelId.value,
        result: result.result,
        error: result.error,
        isExplain: true,
        originalSql: explainSql,
        connectionId: selectedConnection.value,
        elapsedMs: Date.now() - startTime
      }
    }))
    
    // 显示执行结果提示
    if (result.error) {
      message.error(result.error)
    } else {
      message.success('执行计划已生成')
    }
  } catch (error) {
    const errorMsg = error instanceof Error ? error.message : '执行失败'
    message.error(errorMsg)
  } finally {
    executing.value = false
  }
}

// DuckDB 本地加速执行
const handleDuckDbExecute = async () => {
  if (!editor || executing.value) return

  const sql = editor.getValue().trim()
  if (!sql) return

  if (!selectedConnection.value) {
    message.warning('请先选择数据库连接')
    return
  }

  // 自动建立运行时连接
  const connOk = await ensureConnection(selectedConnection.value)
  if (!connOk) {
    message.error('无法连接到数据库，请检查连接配置')
    return
  }

  executing.value = true
  const startTime = Date.now()
  try {
    const result = await sqlExecutionStore.executeSql(
      panelId.value,
      sql,
      runtimeConnId.value
    )

    window.dispatchEvent(new CustomEvent('sql-execution-result', {
      detail: {
        panelId: panelId.value,
        result: result.result,
        error: result.error,
        isDuckDb: true,
        originalSql: sql,
        connectionId: selectedConnection.value,
        elapsedMs: Date.now() - startTime
      }
    }))

    if (result.error) {
      message.error(result.error)
    } else {
      const rowCount = result.result?.rowCount || 0
      const execTime = result.result?.executionTime || 0
      message.success(`DuckDB 加速执行成功，${rowCount} 行，耗时 ${execTime}ms`)
    }
  } catch (error) {
    const errorMsg = error instanceof Error ? error.message : '执行失败'
    message.error(errorMsg)
  } finally {
    executing.value = false
  }
}

// SQL 方言转换
const handleTranspileSql = async (targetDialect: SqlDialect) => {
  if (!editor) return

  const sql = editor.getValue().trim()
  if (!sql) {
    message.warning('编辑器中没有 SQL')
    return
  }

  try {
    const sourceDialect = getCurrentDialect()
    const transpiled = await transpileSql(sql, sourceDialect, targetDialect)
    
    if (transpiled && transpiled !== sql) {
      editor.setValue(transpiled)
      message.success(`已转换为 ${targetDialect} 方言`)
    } else if (transpiled === sql) {
      message.info('转换结果与原 SQL 相同')
    } else {
      message.warning('转换失败，请检查 SQL 语法')
    }
  } catch (error) {
    const errorMsg = error instanceof Error ? error.message : '转换失败'
    message.error(errorMsg)
  }
}

// 转换并关闭弹窗
const handleTranspileAndClose = async (targetDialect: SqlDialect) => {
  showTranspileMenu.value = false
  await handleTranspileSql(targetDialect)
}

// 切换工具栏位置
const toggleToolbarPosition = () => {
  const positions: Array<'top' | 'left' | 'right'> = ['top', 'left', 'right']
  const currentIndex = positions.indexOf(toolbarPosition.value)
  toolbarPosition.value = positions[(currentIndex + 1) % positions.length]
}

// 监听外部值变化
watch(() => props.modelValue, (newVal) => {
  if (editor && editor.getValue() !== newVal) {
    editor.setValue(newVal)
  }
})

// 监听主题变化
watch(() => props.theme, (newTheme) => {
  if (editor) {
    monaco.editor.setTheme(newTheme)
  }
})

// 监听连接变化
watch(() => props.connectionId, (newId) => {
  if (newId) {
    selectedConnection.value = newId
    updateDialectHighlight()
  }
})

// 监听 params.connectionId 变化（兼容 Dockview 嵌套 params）
const paramsConnectionId = computed(() => {
  return props.params?.connectionId || (props.params as any)?.params?.connectionId || ''
})
watch(paramsConnectionId, (newId) => {
  if (newId && newId !== selectedConnection.value) {
    selectedConnection.value = newId
    updateDialectHighlight()
  }
})

// 监听 selectedConnection 变化（用户手动选择连接）
watch(selectedConnection, () => {
  updateDialectHighlight()
})

// 暴露方法
defineExpose({
  getEditor: () => editor,
  getValue: () => editor?.getValue() || '',
  setValue: (value: string) => editor?.setValue(value),
  getSelectedText: () => {
    const selection = editor?.getSelection()
    if (selection && !selection.isEmpty()) {
      return editor?.getModel()?.getValueInRange(selection) || ''
    }
    return ''
  },
  insertText: (text: string) => {
    if (!editor) return
    const position = editor.getPosition()
    if (position) {
      editor.executeEdits('', [{
        range: new monaco.Range(position.lineNumber, position.column, position.lineNumber, position.column),
        text: text
      }])
    }
  },
  focus: () => editor?.focus(),
  handleFormat,
  selectedConnection: computed(() => selectedConnection.value)
})

// 聚焦编辑器
const focusEditor = () => {
  if (editor) {
    editor.focus()
  }
}

// 插入示例 SQL
const insertSampleSql = () => {
  if (!editor) return
  
  const sampleSql = `-- 示例 SQL 查询
-- 选择数据库连接后开始编写你的 SQL

SELECT 
    id,
    name,
    created_at
FROM users
WHERE status = 'active'
ORDER BY created_at DESC
LIMIT 100;

-- 提示：
-- Ctrl+Enter: 执行 SQL
-- Ctrl+Shift+F: 格式化 SQL
-- Ctrl+/: 注释/取消注释`
  
  editor.setValue(sampleSql)
  editor.focus()
}

onMounted(() => {
  console.log(`[SqlEditorPanel] onMounted panelId=${panelId.value}, params=`, JSON.parse(JSON.stringify(props.params)))
  initEditor()
  
  // 从导航栏打开时，等待连接列表加载后自动匹配
  // Dockview 会把 params 嵌套在 props.params.params 中
  const directId = props.params?.connectionId || props.connectionId
  const nestedId = (props.params as any)?.params?.connectionId
  const targetId = directId || nestedId || ''
  
  if (targetId) {
    console.log(`[SqlEditorPanel] waitForConnection starting, targetId="${targetId}"`)
    console.log(`[SqlEditorPanel]   runtimeConnectionIds has? ${runtimeConnectionStore.runtimeConnectionIds.has(targetId)}`)
    console.log(`[SqlEditorPanel]   connectionStore.connections=`, connectionStore.connections.map(c => c.connId))
    waitForConnection(targetId)
  }

  // 监听结果面板刷新请求（用户点击结果面板的"刷新"按钮）
  window.addEventListener('query-result-refresh', handleQueryResultRefresh as unknown as (e: Event) => void)
  // 监听结果面板导出 INSERT 请求
  window.addEventListener('query-result-export-insert', handleQueryResultExportInsert as unknown as (e: Event) => void)

  // 监听自己的执行结果，展开内嵌结果面板
  window.addEventListener('query-result-updated', handleEmbeddedResult as unknown as (e: Event) => void)
})

/**
 * 内嵌结果面板：收到匹配自己 panelId 的结果时展开
 */
const handleEmbeddedResult = (event: CustomEvent) => {
  if (event.detail?.panelId === panelId.value) {
    hasResults.value = true
  }
}

/**
 * 处理结果面板刷新请求 — 重新执行当前 SQL
 */
const handleQueryResultRefresh = async (event: CustomEvent) => {
  if (event.detail?.connectionId) {
    selectedConnection.value = event.detail.connectionId
  }
  if (event.detail?.sql) {
    if (editor) editor.setValue(event.detail.sql)
  }
  if (editor && editor.getValue()) {
    await handleExecute()
  }
}

/**
 * 处理结果面板导出 INSERT 请求
 */
const handleQueryResultExportInsert = (_event: CustomEvent) => {
  message.success('INSERT 语句已复制')
}

/**
 * 等待指定 connectionId 可用
 *
 * 分两步：
 * 1. 先查 runtimeConnectionIds（已有运行时连接的直接匹配）
 * 2. 如果不在运行时中，等待 connectionStore 加载后匹配
 *
 * 最多等待 10 秒（50 次 × 200ms）
 */
async function waitForConnection(targetId: string) {
  console.log(`[waitForConnection] targetId="${targetId}"`)
  console.log(`[waitForConnection]   runtimeConnectionIds keys:`, [...runtimeConnectionStore.runtimeConnectionIds.keys()])
  console.log(`[waitForConnection]   connectionStore connections:`, JSON.parse(JSON.stringify(connectionStore.connections.map(c => ({ connId: c.connId, name: c.name, status: c.status })))))
  console.log(`[waitForConnection]   selectedConnection.value="${selectedConnection.value}"`)

  // 第一步：立即检查 runtimeConnectionStore
  if (runtimeConnectionStore.runtimeConnectionIds.has(targetId)) {
    console.log(`[waitForConnection] ✅ found in runtimeConnectionIds, setting selectedConnection="${targetId}"`)
    selectedConnection.value = targetId
    updateDialectHighlight()
    console.log(`[waitForConnection] Done (path=1)`)
    return
  }

  // 第二步：轮询 connectionStore 加载
  for (let i = 0; i < 50; i++) {
    const found = connectionStore.connections.find(c => c.connId === targetId)
    if (found) {
      console.log(`[waitForConnection] ✅ found in connectionStore (attempt=${i}), setting selectedConnection="${targetId}"`)
      selectedConnection.value = targetId
      updateDialectHighlight()
      console.log(`[waitForConnection] Done (path=2)`)
      return
    }
    // 重新检查 runtime 连接（可能在等待期间被其他组件建立）
    if (runtimeConnectionStore.runtimeConnectionIds.has(targetId)) {
      console.log(`[waitForConnection] ✅ runtime appeared during polling (attempt=${i})`)
      selectedConnection.value = targetId
      updateDialectHighlight()
      console.log(`[waitForConnection] Done (path=3)`)
      return
    }
    // 1 秒后仍未找到，主动重新加载连接列表
    if (i === 5) {
      console.log(`[waitForConnection] attempt=${i}, reloading connections...`)
      await connectionStore.loadConnections()
      console.log(`[waitForConnection]   after reload: connections=`, JSON.parse(JSON.stringify(connectionStore.connections.map(c => ({ connId: c.connId, status: c.status })))))
      console.log(`[waitForConnection]   after reload: runtimeConnectionIds keys=`, [...runtimeConnectionStore.runtimeConnectionIds.keys()])
      // 加载完成后再次检查 runtime 连接
      if (runtimeConnectionStore.runtimeConnectionIds.has(targetId)) {
        console.log(`[waitForConnection] ✅ runtime appeared after reload`)
        selectedConnection.value = targetId
        updateDialectHighlight()
        console.log(`[waitForConnection] Done (path=4)`)
        return
      }
    }
    await new Promise(r => setTimeout(r, 200))
  }
  // 超时后，自动选中第一个可用的运行时连接
  console.log(`[waitForConnection] ⚠️ timeout, trying fallback to first runtime connection`)
  for (const [connId] of runtimeConnectionStore.runtimeConnectionIds) {
    console.log(`[waitForConnection] fallback: setting selectedConnection="${connId}"`)
    selectedConnection.value = connId
    updateDialectHighlight()
    console.log(`[waitForConnection] Done (path=5)`)
    return
  }
  // 最后退路：connectionStore 中第一个已连接的
  const firstConnected = connectionStore.connections.find(c => c.status === 'connected')
  if (firstConnected) {
    console.log(`[waitForConnection] fallback2: setting selectedConnection="${firstConnected.connId}"`)
    selectedConnection.value = firstConnected.connId
    updateDialectHighlight()
    console.log(`[waitForConnection] Done (path=6)`)
  } else {
    console.log(`[waitForConnection] ❌ all paths exhausted, selectedConnection remains empty`)
  }
}

onUnmounted(() => {
  if (editor) {
    try {
      // 先清理模型标记
      const model = editor.getModel()
      if (model) {
        monaco.editor.setModelMarkers(model, 'sql-validation', [])
        monaco.editor.setModelMarkers(model, 'sql-completion', [])
      }
      
      // 清理补全提供者
      if (selectedConnection.value) {
        unregisterCompletionProvider(selectedConnection.value)
      }
      
      // 清理方言高亮
      if (currentHighlightDisposable) {
        currentHighlightDisposable.dispose()
        currentHighlightDisposable = null
      }
      
      // 先设置空模型，再销毁编辑器，避免 Web Worker 清理错误
      const emptyModel = monaco.editor.createModel('', 'sql')
      editor.setModel(emptyModel)
      
      // 销毁编辑器（Monaco Web Worker 的 removeEventListener 错误是已知问题）
      editor.dispose()
      editor = null
      
      // 延迟清理空模型
      setTimeout(() => {
        emptyModel.dispose()
      }, 100)
    } catch (error) {
      // 忽略 Monaco Web Worker 清理错误，不影响功能
      console.warn('[SqlEditor] Editor cleanup warning:', error)
    }
  }
  window.removeEventListener('query-result-refresh', handleQueryResultRefresh as unknown as (e: Event) => void)
  window.removeEventListener('query-result-export-insert', handleQueryResultExportInsert as unknown as (e: Event) => void)
  window.removeEventListener('query-result-updated', handleEmbeddedResult as unknown as (e: Event) => void)
})
</script>

<style scoped>
.sql-editor-panel {
  display: flex;
  flex-direction: column;
  height: 100%;
  position: relative;
  background-color: var(--editor-bg, #1e1e1e);
  overflow: hidden;
}

/* 工具栏在左侧 */
.sql-editor-panel.toolbar-left {
  flex-direction: row;
}

.sql-editor-panel.toolbar-left .editor-toolbar {
  flex-direction: column;
  width: 48px;
  border-right: 1px solid var(--border-color, #333);
  border-bottom: none;
  padding: 8px 0;
  justify-content: flex-start;
  height: auto;
  flex-shrink: 0;
  gap: 4px;
}

.sql-editor-panel.toolbar-left .toolbar-section {
  flex-direction: column;
  align-items: center;
  padding: 2px 0;
}

.sql-editor-panel.toolbar-left .section-header {
  writing-mode: vertical-lr;
  padding: 4px 2px;
}

.sql-editor-panel.toolbar-left .section-content {
  flex-direction: column;
  margin-left: 0;
  margin-top: 2px;
}

.sql-editor-panel.toolbar-left .toolbar-divider {
  width: 32px;
  height: 1px;
  margin: 2px 0;
}

.sql-editor-panel.toolbar-left .toolbar-spacer {
  flex: 1;
  min-height: 8px;
}

.sql-editor-panel.toolbar-left .editor-container {
  flex: 1 1 0;
  padding-bottom: 24px;
}

.sql-editor-panel.toolbar-left .editor-statusbar {
  position: absolute;
  bottom: 0;
  left: 48px;
  right: 0;
  z-index: 10;
  background-color: var(--bg-tertiary, #2d2d30);
  border-top: 1px solid var(--border-color, #3e3e42);
}

/* 工具栏在右侧 */
.sql-editor-panel.toolbar-right {
  flex-direction: row-reverse;
}

.sql-editor-panel.toolbar-right .editor-toolbar {
  flex-direction: column;
  width: 48px;
  border-left: 1px solid var(--border-color, #333);
  border-bottom: none;
  padding: 8px 0;
  justify-content: flex-start;
  height: auto;
  flex-shrink: 0;
  gap: 4px;
}

.sql-editor-panel.toolbar-right .toolbar-section {
  flex-direction: column;
  align-items: center;
  padding: 2px 0;
}

.sql-editor-panel.toolbar-right .section-header {
  writing-mode: vertical-lr;
  padding: 4px 2px;
}

.sql-editor-panel.toolbar-right .section-content {
  flex-direction: column;
  margin-left: 0;
  margin-top: 2px;
}

.sql-editor-panel.toolbar-right .toolbar-divider {
  width: 32px;
  height: 1px;
  margin: 2px 0;
}

.sql-editor-panel.toolbar-right .toolbar-spacer {
  flex: 1;
  min-height: 8px;
}

.sql-editor-panel.toolbar-right .editor-container {
  flex: 1 1 0;
  padding-bottom: 24px;
}

.sql-editor-panel.toolbar-right .editor-statusbar {
  position: absolute;
  bottom: 0;
  left: 0;
  right: 48px;
  z-index: 10;
}

.editor-toolbar {
  display: flex;
  align-items: center;
  height: 40px;
  padding: 0 8px;
  background-color: var(--toolbar-bg, #252526);
  border-bottom: 1px solid var(--border-color, #333);
  gap: 0;
  flex-shrink: 0;
}

.toolbar-section {
  display: flex;
  align-items: center;
  gap: 2px;
  padding: 0 4px;
}

.section-header {
  display: flex;
  align-items: center;
  gap: 3px;
  padding: 2px 4px;
  cursor: pointer;
  border-radius: 3px;
  transition: background-color 0.15s;
  white-space: nowrap;
}

.section-header:hover {
  background-color: var(--bg-tertiary, #2d2d30);
}

.section-chevron {
  transition: transform 0.15s;
  color: var(--text-tertiary, #666);
  flex-shrink: 0;
}

.section-chevron.collapsed {
  transform: rotate(-90deg);
}

.section-label {
  font-size: 11px;
  font-weight: 500;
  color: var(--text-tertiary, #888);
  text-transform: uppercase;
  letter-spacing: 0.5px;
  user-select: none;
}

.section-content {
  display: flex;
  align-items: center;
  gap: 2px;
  margin-left: 4px;
}

.toolbar-divider {
  height: 20px;
  margin: 0 2px;
}

.toolbar-section.toolbar-end {
  margin-left: auto;
}

.toolbar-spacer {
  flex: 1;
}

.editor-container {
  flex: 1 1 0%;
  overflow: hidden;
  min-height: 0;
}

/* 1:n 结果分割布局 */
.editor-result-split {
  flex: 1;
  min-height: 0;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}
.editor-result-split .editor-container {
  flex: 1 1 0%;
  min-height: 0;
}
.split-handle {
  height: 4px;
  cursor: row-resize;
  background: var(--border-color, #333);
  flex-shrink: 0;
  position: relative;
  z-index: 10;
}
.split-handle:hover { background: var(--primary-color, #0078d4); }
.result-container {
  flex: 1;
  min-height: 0;
  overflow: hidden;
  border-top: 1px solid var(--border-color, #333);
  background: var(--bg-primary, #1e1e1e);
}

.editor-statusbar {
  display: flex;
  align-items: center;
  height: 24px;
  min-height: 24px;
  padding: 0 12px;
  background-color: var(--bg-tertiary, #2d2d30);
  color: var(--text-secondary, #858585);
  font-size: 12px;
  flex-shrink: 0;
  border-top: 1px solid var(--border-color, #3e3e42);
}

.status-left {
  display: flex;
  align-items: center;
  gap: 12px;
}

.status-center {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
}

.status-right {
  display: flex;
  align-items: center;
  gap: 12px;
}

.status-item {
  white-space: nowrap;
}

.status-item.executing {
  display: flex;
  align-items: center;
  gap: 6px;
  color: #ffd700;
  font-weight: 500;
}

.status-item.success {
  color: #4caf50;
}

/* 状态栏连接信息标签（点击切换连接） */
.connection-info-tag {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  color: var(--text-primary, #cccccc);
  font-weight: 500;
  padding: 0 10px;
  background: var(--statusbar-conn-bg, #094771);
  border-radius: 3px;
  height: 18px;
  cursor: pointer;
  transition: background-color 0.2s;
  border: 1px solid transparent;
}

.connection-info-tag:hover {
  background: var(--statusbar-conn-hover-bg, #106ebe);
  border-color: rgba(255, 255, 255, 0.2);
}

.loading-dot {
  display: inline-block;
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background-color: #ffd700;
  animation: pulse 1s ease-in-out infinite;
}

@keyframes pulse {
  0%, 100% {
    opacity: 1;
    transform: scale(1);
  }
  50% {
    opacity: 0.5;
    transform: scale(1.2);
  }
}

/* 编辑器水印样式（空编辑器时显示为透明提示） */
.editor-watermark {
  position: absolute;
  top: 40px;
  left: 0;
  right: 0;
  bottom: 24px;
  display: flex;
  align-items: center;
  justify-content: center;
  pointer-events: none;
  z-index: 1;
}

.watermark-text {
  text-align: center;
  opacity: 0.35;
  user-select: none;
}

.watermark-title {
  font-size: 20px;
  font-weight: 600;
  color: var(--text-primary, #cccccc);
  margin-bottom: 16px;
}

.watermark-shortcuts {
  display: flex;
  flex-direction: column;
  gap: 6px;
  align-items: center;
}

.shortcut-hint {
  font-size: 12px;
  color: var(--text-secondary, #858585);
  line-height: 1.6;
}

.shortcut-hint kbd {
  display: inline-block;
  padding: 1px 5px;
  margin: 0 2px;
  background: var(--bg-tertiary, #2d2d30);
  border: 1px solid var(--border-color, #3e3e42);
  border-radius: 3px;
  font-family: 'JetBrains Mono', monospace;
  font-size: 11px;
  color: var(--text-secondary, #858585);
}

/* 方言转换弹窗样式 */
.transpile-modal {
  position: fixed;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  width: 360px;
  padding: 24px;
  background: var(--bg-secondary, #252526);
  border: 1px solid var(--border-color, #3e3e42);
  border-radius: 8px;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.4);
}

.transpile-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 16px;
}

.transpile-header h3 {
  margin: 0;
  font-size: 16px;
  font-weight: 600;
  color: var(--text-primary, #cccccc);
}

.transpile-hint {
  margin: 0 0 16px;
  font-size: 13px;
  color: var(--text-secondary, #858585);
}

.transpile-options {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.quick-actions {
  display: flex;
  gap: 8px;
}
</style>
