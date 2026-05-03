<template>
  <div :class="['workbench-view', uiStore.isDark ? 'dockview-theme-dark' : 'dockview-theme-light']">
    <!-- Dockview 布局 -->
    <DockviewVue
      ref="dockviewRef"
      class="dockview"
      :style="dockviewStyle"
      @ready="onReady"
    />

    <!-- 底部状态栏 -->
    <WorkbenchStatusBar />

    <!-- 新建连接对话框 -->
    <ConnectionModal
      v-model="showConnectionModal"
      @save="handleSaveConnection"
    />
  </div>
</template>

<script setup lang="ts">
import { DockviewVue, type DockviewReadyEvent } from 'dockview-vue'
import { useMessage } from 'naive-ui'
import { ref, computed, onMounted, onUnmounted, getCurrentInstance, type Component } from 'vue'
import { useRouter } from 'vue-router'

import { panelRegistry } from '@/core/panel-registry'
import { useProjectStore } from '@/core/project/stores/project'
import ConnectionModal from '@/extensions/builtin/connection/ui/components/ConnectionModal.vue'
import { useConnectionStore } from '@/extensions/builtin/connection/ui/stores/connection-store'
import type { ConnectionConfig } from '@/extensions/builtin/connection/ui/types/connection'
import { useQueryStore } from '@/extensions/builtin/query/ui/stores/query-store'
import WorkbenchStatusBar from '@/extensions/builtin/workbench/ui/components/WorkbenchStatusBar.vue'
import { useUiStore } from '@/shared/stores/ui'

// Stores
const uiStore = useUiStore()
const projectStore = useProjectStore()
const connectionStore = useConnectionStore()
const _queryStore = useQueryStore()

// Router
const _router = useRouter()

// Message
const message = useMessage()

// Refs
const dockviewRef = ref<InstanceType<typeof DockviewVue> | null>(null)

// Modal states
const showConnectionModal = ref(false)

// 注册面板组件到 appContext，供 dockview-vue 查找
const instance = getCurrentInstance()
if (instance) {
  const components: Record<string, Component> = {}
  
  // 从面板注册表动态读取并注册组件
  const panels = panelRegistry.getAll()
  panels.forEach(panel => {
    // 注册原始 ID 和 kebab-case 两种形式，确保都能找到
    const kebabName = panel.id.replace(/([A-Z])/g, '-$1').toLowerCase()
    components[panel.id] = panel.component as Component
    if (panel.id !== kebabName) {
      components[kebabName] = panel.component as Component
    }
    console.log(`[Workbench] Registered component: ${panel.id} (also as ${kebabName})`)
  })
  
  instance.appContext.components = {
    ...instance.appContext.components,
    ...components,
  }
}

// Computed
const dockviewStyle = computed(() => {
  const isDark = uiStore.isDark
  return {
    height: '100%',
    width: '100%',
    '--dv-group-view-background-color': isDark ? '#1e1e1e' : '#ffffff',
    '--dv-tabs-and-actions-container-background-color': isDark ? '#252526' : '#f5f5f5',
    '--dv-activegroup-visiblepanel-tab-background-color': isDark ? '#1e1e1e' : '#ffffff',
    '--dv-activegroup-hiddenpanel-tab-background-color': isDark ? '#2d2d30' : '#e8e8e8',
    '--dv-inactivegroup-visiblepanel-tab-background-color': isDark ? '#2d2d30' : '#e8e8e8',
    '--dv-inactivegroup-hiddenpanel-tab-background-color': isDark ? '#2d2d30' : '#e8e8e8',
    '--dv-tab-divider-color': isDark ? '#3e3e42' : '#d4d4d4',
    '--dv-border-color': isDark ? '#3e3e42' : '#d4d4d4',
    '--dv-activegroup-visiblepanel-tab-color': isDark ? '#ffffff' : '#333333',
    '--dv-activegroup-hiddenpanel-tab-color': isDark ? '#999999' : '#666666',
    '--dv-inactivegroup-visiblepanel-tab-color': isDark ? '#999999' : '#666666',
    '--dv-inactivegroup-hiddenpanel-tab-color': isDark ? '#666666' : '#999999',
    '--dv-tabs-and-actions-container-font-size': '12px',
    '--dv-tabs-and-actions-container-height': '35px',
    '--dv-activegroup-visiblepanel-tab-height': '35px',
    '--dv-activegroup-hiddenpanel-tab-height': '35px',
    '--dv-inactivegroup-visiblepanel-tab-height': '35px',
    '--dv-inactivegroup-hiddenpanel-tab-height': '35px',
  }
})


// 当前活动的 SQL 编辑器面板引用
const activeSqlEditorPanel = ref<unknown>(null)
const _isExecuting = ref(false)

// 保存连接
const handleSaveConnection = async (data: Partial<ConnectionConfig>) => {
  console.log('保存连接:', data)
  try {
    // 兼容两种字段名：db_type (ConnectionModal emit) 或 driver (旧版)
    const driver = (data as Record<string, unknown>).db_type || data.driver
    if (!driver) {
      message.error('请选择数据库类型')
      return
    }

    let url = ''
    
    // 检查是否是文件型数据库
    const isFileDb = driver === 'sqlite' || driver === 'duckdb'
    
    if (isFileDb) {
      // 文件型数据库
      const filePath = data.database
      if (!filePath) {
        message.error('请选择数据库文件')
        return
      }
      // Windows 路径不需要额外的斜杠，直接使用 sqlite://path 格式
      url = `${driver}://${filePath}`
    } else {
      // 网络型数据库
      const host = data.host
      if (!host) {
        message.error('请输入主机地址')
        return
      }
      
      const port = data.port || (driver === 'postgres' ? 5432 : 3306)
      const database = data.database || ''
      const username = data.username || ''
      const password = data.password || ''
      
      // 构建 URL
      const auth = username 
        ? password 
          ? `${encodeURIComponent(username)}:${encodeURIComponent(password)}@`
          : `${encodeURIComponent(username)}@`
        : ''
      url = `${driver}://${auth}${host}:${port}/${encodeURIComponent(database)}`
    }
    
    // 调用连接 store 创建连接
    await connectionStore.connect(driver, url, data.name)
    
    // 刷新连接列表
    await connectionStore.loadConnections()
    
    // 触发导航树刷新事件
    window.dispatchEvent(new CustomEvent('navigator-refresh'))
    
    // 打开 SQL 编辑器，跳转到工作台
    if (dockviewApi) {
      const emptyPanel = dockviewApi.getPanel?.('panel_emptyWorkbench')
      if (emptyPanel) {
        // 移除空状态面板
        emptyPanel.api?.close?.()
      }
      
      // 创建 SQL 编辑器面板
      sqlEditorCounter++
      const panelId = `panel_sqlEditor_${sqlEditorCounter}`
      dockviewApi.addPanel({
        id: panelId,
        component: 'sqlEditor',
        title: `SQL ${sqlEditorCounter}`,
        position: { referencePanel: 'panel_databaseNavigator', direction: 'right' },
        params: {
          connectionId: data.name,
          databaseName: data.database || '',
          initialSql: ''
        }
      })
    }
    
    message.success(`连接 "${data.name}" 保存成功`)
  } catch (error) {
    const errorMsg = error instanceof Error ? error.message : '保存连接失败'
    message.error(`保存失败: ${errorMsg}`)
    // eslint-disable-next-line no-console
    console.error('保存连接失败:', error)
  }
}

// Dockview API 引用
interface DockviewPanelApi {
  close?: () => void
}

interface DockviewPanel {
  id: string
  api: DockviewPanelApi
  group?: {
    id: string
    addPanel: (config: Record<string, unknown>) => void
    model: {
      panels: Array<{ id: string }>
    }
  }
}

interface DockviewApi {
  addPanel: (config: Record<string, unknown>) => DockviewPanel
  getPanel: (id: string) => DockviewPanel | undefined
  closePanel: (id: string) => void
  setActivePanel: (panel: DockviewPanel) => void
  onDidActivePanelChange: (callback: (panel: DockviewPanel | undefined) => void) => void
  onDidRemovePanel: (callback: (panel: DockviewPanel) => void) => void
  onDidAddPanel: (callback: (panel: DockviewPanel) => void) => void
  moveToGroup: (fromPanel: DockviewPanel, toGroup: string) => void
  groups: Array<{ id: string; model: { panels: Array<{ id: string }> } }>
}

let dockviewApi: DockviewApi | null = null
let sqlEditorCounter = 0

// Dockview ready
const onReady = (event: DockviewReadyEvent) => {
  const api = event.api as unknown as DockviewApi
  dockviewApi = api

  // 从面板注册表动态创建面板
  const panels = panelRegistry.getAll()
  console.log(`[Workbench] Creating ${panels.length} panels from registry`)
  
  // 按位置分组
  const leftPanels = panels.filter(p => p.location === 'left')
  const bottomPanels = panels.filter(p => p.location === 'bottom')
  const rightPanels = panels.filter(p => p.location === 'right')

  // 1. 先创建左侧面板（数据库导航）
  let leftPanelId: string | null = null
  leftPanels.forEach(panel => {
    const panelConfig: Record<string, unknown> = {
      id: `panel_${panel.id}`,
      component: panel.id,
      title: panel.name,
    }
    
    api.addPanel(panelConfig)
    leftPanelId = `panel_${panel.id}`
    console.log(`[Workbench] Created left panel: ${panelConfig.id}`)
  })

  // 2. 底部面板（查询结果）初始时不创建，等有查询结果时才动态创建
  // 这样用户打开应用时不会看到空的结果面板
  const bottomPanelId: string | null = null
  // 跳过初始创建，改为在 handleSqlExecutionResult 中按需创建
  // bottomPanels.forEach((panel, index) => { ... })

  // 3. 创建右侧面板（如果有）
  rightPanels.forEach((panel, index) => {
    const panelConfig: Record<string, unknown> = {
      id: `panel_${panel.id}`,
      component: panel.id,
      title: panel.name,
    }
    
    const referenceId = leftPanelId
    if (index === 0 && referenceId) {
      panelConfig.position = {
        referencePanel: referenceId,
        direction: 'right'
      }
    } else if (index > 0) {
      panelConfig.position = {
        referencePanel: `panel_${rightPanels[index - 1].id}`,
        direction: 'within'
      }
    }
    
    api.addPanel(panelConfig)
    console.log(`[Workbench] Created right panel: ${panelConfig.id}`)
  })

  // 监听面板激活事件
  api.onDidActivePanelChange?.((panel: DockviewPanel | undefined) => {
    if (panel?.id?.startsWith('panel_sqlEditor_')) {
      activeSqlEditorPanel.value = panel
    }
  })

  // 监听打开对象属性事件
  window.addEventListener('open-object-properties', handleOpenObjectProperties as (e: Event) => void)

  // 监听打开 SQL 编辑器事件
  window.addEventListener('open-sql-editor', handleOpenSqlEditor as (e: Event) => void)

  // 监听 SQL 执行结果事件（从 SqlEditorPanel 发送）
  window.addEventListener('sql-execution-result', handleSqlExecutionResult as (e: Event) => void)
}

/**
 * 处理 SQL 执行结果
 */
const handleSqlExecutionResult = (event: CustomEvent) => {
  const { panelId, result, error, results, isBatch, originalSql, connectionId, elapsedMs } = event.detail || {}
  if (!panelId) return

  // 判断是多语句还是单语句
  if (results && Array.isArray(results) && results.length > 0) {
    // 多语句执行结果
    ensureMultiTabResultPanel()
    
    // 发送多语句结果到 MultiTabResults 面板
    window.dispatchEvent(new CustomEvent('multi-tab-result-updated', {
      detail: {
        panelId,
        results
      }
    }))
  } else {
    // 单语句执行结果：现在不创建独立结果面板，结果嵌入 SQL 编辑器中
    // 直接透传事件到 query-result-updated，由 SqlEditorPanel 内嵌面板接收
    window.dispatchEvent(new CustomEvent('query-result-updated', {
      detail: {
        panelId,
        result,
        error,
        originalSql,
        connectionId,
        elapsedMs
      }
    }))
  }
}

/**
 * 查找第一个 SQL 编辑器面板 ID（用于结果面板定位）
 */
function findActiveSqlEditorPanelId(): string | null {
  if (!dockviewApi) return null
  
  // 优先使用当前激活的 SQL 编辑器
  if (activeSqlEditorPanel.value && typeof activeSqlEditorPanel.value === 'object' && (activeSqlEditorPanel.value as any)?.id) {
    return (activeSqlEditorPanel.value as any).id
  }
  
  // 遍历查找第一个 SQL 编辑器
  for (const group of dockviewApi.groups || []) {
    for (const panelInfo of group.model?.panels || []) {
      if (panelInfo.id?.startsWith('panel_sqlEditor_')) {
        return panelInfo.id
      }
    }
  }
  return null
}

/**
 * 确保结果面板存在，如果不存在则创建
 * 结果面板出现在 SQL 编辑器下方（同一组 vertical split），而非导航栏下方
 */
const ensureResultPanel = () => {
  if (!dockviewApi) return

  // 查找是否已有结果面板
  let hasResultPanel = false
  for (const group of dockviewApi.groups || []) {
    for (const panelInfo of group.model?.panels || []) {
      if (panelInfo.id?.startsWith('panel_queryResult')) {
        hasResultPanel = true
        break
      }
    }
    if (hasResultPanel) break
  }

  if (!hasResultPanel) {
    // 结果面板放在 SQL 编辑器下方
    const refPanelId = findActiveSqlEditorPanelId() || 'panel_databaseNavigator'
    
    dockviewApi.addPanel({
      id: 'panel_queryResult',
      component: 'queryResult',
      title: '查询结果',
      position: {
        referencePanel: refPanelId,
        direction: 'below'
      }
    })
    console.log(`[Workbench] 自动创建查询结果面板，位置参考: ${refPanelId}`)
  }
}

/**
 * 确保多 Tab 结果面板存在，如果不存在则创建
 */
const ensureMultiTabResultPanel = () => {
  if (!dockviewApi) return

  // 查找是否已有多 Tab 结果面板
  let hasMultiTabPanel = false
  for (const group of dockviewApi.groups || []) {
    for (const panelInfo of group.model?.panels || []) {
      if (panelInfo.id?.startsWith('panel_multiTabResult')) {
        hasMultiTabPanel = true
        break
      }
    }
    if (hasMultiTabPanel) break
  }

  if (!hasMultiTabPanel) {
    // 多 Tab 结果面板放在 SQL 编辑器下方
    const refPanelId = findActiveSqlEditorPanelId() || 'panel_databaseNavigator'
    
    dockviewApi.addPanel({
      id: 'panel_multiTabResult',
      component: 'multiTabResult',
      title: '查询结果',
      position: {
        referencePanel: refPanelId,
        direction: 'below'
      }
    })
    console.log(`[Workbench] 自动创建多 Tab 查询结果面板，位置参考: ${refPanelId}`)
  }
}

/**
 * 打开 SQL 编辑器面板
 * 默认单栏多标签模式，分栏为高级功能
 */
const handleOpenSqlEditor = (event: CustomEvent) => {
  if (!dockviewApi) return

  sqlEditorCounter++
  const panelId = `panel_sqlEditor_${sqlEditorCounter}`
  const { connectionId, databaseName, sql } = event.detail || {}

  // 查找已有的 SQL 编辑器面板
  let existingSqlPanel: DockviewPanel | undefined
  let existingSqlPanelId: string | null = null
  
  for (const group of dockviewApi.groups || []) {
    for (const panelInfo of group.model?.panels || []) {
      const panel = dockviewApi.getPanel?.(panelInfo.id)
      if (panel?.id?.startsWith('panel_sqlEditor_')) {
        existingSqlPanel = panel
        existingSqlPanelId = panel.id
        break
      }
    }
    if (existingSqlPanel) break
  }

  // 关闭空工作台面板（如果存在）
  try {
    const emptyPanel = dockviewApi.getPanel?.('panel_emptyWorkbench')
    if (emptyPanel) {
      emptyPanel.api?.close?.()
    }
  } catch (e) {
    console.warn('[SQL Editor] 关闭空工作台面板失败:', e)
  }

  if (existingSqlPanel && existingSqlPanelId) {
    // 已有 SQL 编辑器，添加到同一组（tab 模式）
    const editorPanel = dockviewApi.addPanel({
      id: panelId,
      component: 'sqlEditor',
      title: `SQL ${sqlEditorCounter}`,
      position: {
        referencePanel: existingSqlPanelId,
        direction: 'within'
      },
      params: {
        connectionId,
        databaseName,
        initialSql: sql
      }
    })

    activeSqlEditorPanel.value = editorPanel
    console.log('[SQL Editor] 添加到已有组（tab 模式）:', editorPanel)
    return
  }

  // 首次创建 SQL 编辑器 - 在中心区域创建新组
  const editorPanel = dockviewApi.addPanel({
    id: panelId,
    component: 'sqlEditor',
    title: `SQL ${sqlEditorCounter}`,
    position: {
      referencePanel: 'panel_databaseNavigator',
      direction: 'right'
    },
    params: {
      connectionId,
      databaseName,
      initialSql: sql
    }
  })

  activeSqlEditorPanel.value = editorPanel
  console.log('[SQL Editor] 首次创建（单栏模式）:', editorPanel)
}

// 打开对象属性面板
const handleOpenObjectProperties = (event: CustomEvent) => {
  if (!dockviewApi) return

  const { objectType, objectName, connectionName, databaseName, connectionId, dbType, schemaName } = event.detail

  // 生成唯一面板ID
  const panelId = `panel_props_${connectionId}_${objectName}`

  // 检查面板是否已存在
  const existingPanel = dockviewApi.getPanel?.(panelId)
  if (existingPanel) {
    // 激活已存在的面板
    dockviewApi.setActivePanel?.(existingPanel)
    return
  }

  // 如果是表类型，使用 TableStructurePanel
  if (objectType === 'table') {
    dockviewApi.addPanel({
      id: panelId,
      component: 'tableStructure',
      title: `${objectName}`,
      params: {
        connectionId,
        databaseName,
        schemaName: schemaName || 'public',
        tableName: objectName
      },
      position: { referencePanel: 'panel_navigator', direction: 'right' },
    })
    return
  }

  // 创建新的属性面板（使用动态属性面板）
  dockviewApi.addPanel({
    id: panelId,
    component: 'dynamicObjectProperties',
    title: `${objectName}`,
    params: {
      objectType,
      objectName,
      connectionName,
      databaseName,
      connectionId,
      dbType: dbType || 'mysql'
    },
    position: { referencePanel: 'panel_navigator', direction: 'right' },
  })
}

// 项目切换处理
const handleProjectSwitched = async () => {
  // 清空当前连接数据
  connectionStore.reset()
  
  // 重新加载连接数据
  await connectionStore.loadConnections()
  
  message.info('已切换到新项目')
}

// 打开连接模态框处理
const handleOpenConnectionModal = () => {
  showConnectionModal.value = true
}

// Lifecycle
onMounted(async () => {
  // 加载项目数据
  await projectStore.loadRecentProjects()
  if (!projectStore.currentProject) {
    await projectStore.loadLastProject()
  }

  // 加载连接数据
  await connectionStore.loadConnections()
  
  // 监听项目切换事件
  window.addEventListener('project-switched', handleProjectSwitched as (e: Event) => void)
  
  // 监听打开连接模态框事件
  window.addEventListener('open-connection-modal', handleOpenConnectionModal)
})

onUnmounted(() => {
  // 移除事件监听
  window.removeEventListener('open-object-properties', handleOpenObjectProperties as (e: Event) => void)
  window.removeEventListener('open-sql-editor', handleOpenSqlEditor as (e: Event) => void)
  window.removeEventListener('project-switched', handleProjectSwitched as (e: Event) => void)
  window.removeEventListener('open-connection-modal', handleOpenConnectionModal)
})
</script>

<script lang="ts">
// 导出组件定义，供 dockview 使用
export default {
  components: {},
}
</script>

<style scoped>
.workbench-view {
  width: 100%;
  height: 100%;
  background: var(--bg-primary);
  color: var(--text-primary);
  font-family: var(--font-sans);
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.dockview {
  flex: 1 1 0%;
  overflow: hidden;
  min-height: 0;
}
</style>
