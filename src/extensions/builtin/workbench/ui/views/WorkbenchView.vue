<template>
  <div :class="['workbench-view', uiStore.isDark ? 'dockview-theme-dark' : 'dockview-theme-light']">
    <!-- Dockview 布局（包含 ActivityBar + Sidebar + Center + Panel） -->
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
import ActivityBarPanel from '@/extensions/builtin/workbench/ui/components/ActivityBarPanel.vue'
import CustomizeLayout from '@/extensions/builtin/workbench/ui/components/CustomizeLayout.vue'
import DynamicObjectPropertiesPanel from '@/extensions/builtin/workbench/ui/components/panels/DynamicObjectPropertiesPanel.vue'
import TableStructurePanel from '@/extensions/builtin/workbench/ui/components/panels/TableStructurePanel.vue'
import WorkbenchStatusBar from '@/extensions/builtin/workbench/ui/components/WorkbenchStatusBar.vue'
import SettingsPanel from '@/extensions/builtin/settings/ui/components/SettingsPanel.vue'
import { useLayoutStore } from '@/extensions/builtin/workbench/ui/stores/layout-store'
import { useUiStore } from '@/shared/stores/ui'

const uiStore = useUiStore()
const layoutStore = useLayoutStore()
const projectStore = useProjectStore()
const connectionStore = useConnectionStore()
const _queryStore = useQueryStore()
const _router = useRouter()
const message = useMessage()

const dockviewRef = ref<InstanceType<typeof DockviewVue> | null>(null)
const showConnectionModal = ref(false)

const instance = getCurrentInstance()
if (instance) {
  const components: Record<string, Component> = {}

  components['leftActivityBar'] = ActivityBarPanel as unknown as Component
  components['rightActivityBar'] = ActivityBarPanel as unknown as Component
  components['tableStructure'] = TableStructurePanel as unknown as Component
  components['dynamicObjectProperties'] = DynamicObjectPropertiesPanel as unknown as Component
  components['customizeLayout'] = CustomizeLayout as unknown as Component
  components['settings'] = SettingsPanel as unknown as Component

  const panels = panelRegistry.getAll()
  panels.forEach(panel => {
    const kebabName = panel.id.replace(/([A-Z])/g, '-$1').toLowerCase()
    components[panel.id] = panel.component as Component
    if (panel.id !== kebabName) {
      components[kebabName] = panel.component as Component
    }
    console.log(`[Workbench] Registered component: ${panel.id}`)
  })

  instance.appContext.components = {
    ...instance.appContext.components,
    ...components,
  }
}

const dockviewStyle = computed(() => ({
  height: '100%',
  width: '100%',
}))

const activeSqlEditorPanel = ref<unknown>(null)

const handleSaveConnection = async (data: Partial<ConnectionConfig>) => {
  console.log('保存连接:', data)
  try {
    const driver = (data as Record<string, unknown>).db_type || data.driver
    if (!driver) {
      message.error('请选择数据库类型')
      return
    }

    let url = ''
    const isFileDb = driver === 'sqlite' || driver === 'duckdb'

    if (isFileDb) {
      const filePath = data.database
      if (!filePath) {
        message.error('请选择数据库文件')
        return
      }
      url = `${driver}://${filePath}`
    } else {
      const host = data.host
      if (!host) {
        message.error('请输入主机地址')
        return
      }
      const port = data.port || (driver === 'postgres' ? 5432 : 3306)
      const database = data.database || ''
      const username = data.username || ''
      const password = data.password || ''
      const auth = username
        ? password
          ? `${encodeURIComponent(username)}:${encodeURIComponent(password)}@`
          : `${encodeURIComponent(username)}@`
        : ''
      url = `${driver}://${auth}${host}:${port}/${encodeURIComponent(database)}`
    }

    await connectionStore.connect(driver, url, data.name)
    await connectionStore.loadConnections()
    window.dispatchEvent(new CustomEvent('navigator-refresh'))

    if (dockviewApi) {
      const emptyPanel = dockviewApi.getPanel?.('panel_emptyWorkbench')
      if (emptyPanel) {
        emptyPanel.api?.close?.()
      }

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
    console.error('保存连接失败:', error)
  }
}

interface DockviewPanelApi {
  close?: () => void
  setActive?: () => void
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

const ACTIVITY_BAR_WIDTH = 48
const SIDEBAR_INITIAL_WIDTH = 280

const onReady = (event: DockviewReadyEvent) => {
  const api = event.api as unknown as DockviewApi
  dockviewApi = api
  layoutStore.setDockviewApi(api as unknown as import('dockview-vue').IDockviewApi)

  const panels = panelRegistry.getAll()
  console.log(`[Workbench] Creating ${panels.length} panels from registry`)

  const leftPanels = panels.filter(p => p.location === 'left').sort((a, b) => (a.order || 0) - (b.order || 0))
  const centerPanels = panels.filter(p => p.location === 'center' && p.id !== 'sqlEditor').sort((a, b) => (a.order || 0) - (b.order || 0))
  const rightPanels = panels.filter(p => p.location === 'right').sort((a, b) => (a.order || 0) - (b.order || 0))
  const bottomPanels = panels.filter(p => p.location === 'bottom' && p.id !== 'queryResult' && p.id !== 'multiTabResult').sort((a, b) => (a.order || 0) - (b.order || 0))

  console.log('[Workbench] Left panels:', leftPanels.map(p => p.id))
  console.log('[Workbench] Center panels:', centerPanels.map(p => p.id))
  console.log('[Workbench] Right panels:', rightPanels.map(p => p.id))
  console.log('[Workbench] Bottom panels:', bottomPanels.map(p => p.id))

  // 1. 左侧活动栏（固定 48px）
  api.addPanel({
    id: 'panel_leftActivityBar',
    component: 'leftActivityBar',
    title: '',
    minimumWidth: ACTIVITY_BAR_WIDTH,
    maximumWidth: ACTIVITY_BAR_WIDTH,
    initialWidth: ACTIVITY_BAR_WIDTH,
    params: {
      items: layoutStore.leftActivityItems,
      position: 'left',
      showToggle: true,
      isHidden: !layoutStore.primarySideBarVisible
    }
  })
  console.log('[Workbench] Created left activity bar')

  // 2. 左侧面板组（数据库导航、分析资源、插件、设置、自定义布局）
  let leftRefId: string | null = null
  leftPanels.forEach((panel, index) => {
    const panelId = `panel_${panel.id}`
    const config: Record<string, unknown> = {
      id: panelId,
      component: panel.id,
      title: panel.name,
      initialWidth: SIDEBAR_INITIAL_WIDTH,
    }
    if (index === 0) {
      config.position = { referencePanel: 'panel_leftActivityBar', direction: 'right' }
    } else if (leftRefId) {
      config.position = { referencePanel: leftRefId, direction: 'within' }
    }
    api.addPanel(config)
    if (index === 0) leftRefId = panelId
    console.log(`[Workbench] Created left panel: ${panelId}`)
  })

  // 添加设置面板到左侧组
  if (leftRefId) {
    api.addPanel({
      id: 'panel_settings',
      component: 'settings',
      title: '设置',
      position: { referencePanel: leftRefId, direction: 'within' }
    })
    console.log('[Workbench] Created left panel: panel_settings')
  }

  // 添加自定义布局面板到左侧组
  if (leftRefId) {
    api.addPanel({
      id: 'panel_customizeLayout',
      component: 'customizeLayout',
      title: '自定义布局',
      position: { referencePanel: leftRefId, direction: 'within' }
    })
    console.log('[Workbench] Created left panel: panel_customizeLayout')
  }

  // 3. 中心面板（空工作台）
  let centerRefId: string | null = null
  centerPanels.forEach((panel, index) => {
    const panelId = `panel_${panel.id}`
    const config: Record<string, unknown> = {
      id: panelId,
      component: panel.id,
      title: panel.name,
    }
    if (index === 0 && leftRefId) {
      config.position = { referencePanel: leftRefId, direction: 'right' }
    } else if (index > 0 && centerRefId) {
      config.position = { referencePanel: centerRefId, direction: 'within' }
    }
    api.addPanel(config)
    if (index === 0) centerRefId = panelId
    console.log(`[Workbench] Created center panel: ${panelId}`)
  })

  // 4. 右侧面板组（列洞察、SQL历史）
  let rightRefId: string | null = null
  rightPanels.forEach((panel, index) => {
    const panelId = `panel_${panel.id}`
    const config: Record<string, unknown> = {
      id: panelId,
      component: panel.id,
      title: panel.name,
      initialWidth: SIDEBAR_INITIAL_WIDTH,
    }
    if (index === 0 && centerRefId) {
      config.position = { referencePanel: centerRefId, direction: 'right' }
    } else if (index > 0 && rightRefId) {
      config.position = { referencePanel: rightRefId, direction: 'within' }
    }
    api.addPanel(config)
    if (index === 0) rightRefId = panelId
    console.log(`[Workbench] Created right panel: ${panelId}`)
  })

  // 5. 右侧活动栏（固定 48px）
  api.addPanel({
    id: 'panel_rightActivityBar',
    component: 'rightActivityBar',
    title: '',
    minimumWidth: ACTIVITY_BAR_WIDTH,
    maximumWidth: ACTIVITY_BAR_WIDTH,
    initialWidth: ACTIVITY_BAR_WIDTH,
    position: { referencePanel: rightRefId || centerRefId || '', direction: 'right' },
    params: {
      items: layoutStore.rightActivityItems,
      position: 'right',
      showToggle: true,
      isHidden: !layoutStore.secondarySideBarVisible
    }
  })
  console.log('[Workbench] Created right activity bar')

  // 6. 底部面板（输出）
  let bottomRefId: string | null = null
  bottomPanels.forEach((panel, index) => {
    const panelId = `panel_${panel.id}`
    const config: Record<string, unknown> = {
      id: panelId,
      component: panel.id,
      title: panel.name,
      initialHeight: 200,
    }
    if (index === 0 && centerRefId) {
      config.position = { referencePanel: centerRefId, direction: 'below' }
    } else if (index > 0 && bottomRefId) {
      config.position = { referencePanel: bottomRefId, direction: 'within' }
    }
    api.addPanel(config)
    if (index === 0) bottomRefId = panelId
    console.log(`[Workbench] Created bottom panel: ${panelId}`)
  })

  api.onDidActivePanelChange?.((panel: DockviewPanel | undefined) => {
    if (panel?.id?.startsWith('panel_sqlEditor_')) {
      activeSqlEditorPanel.value = panel
    }
  })

  window.addEventListener('open-object-properties', handleOpenObjectProperties as (e: Event) => void)
  window.addEventListener('open-sql-editor', handleOpenSqlEditor as (e: Event) => void)
  window.addEventListener('sql-execution-result', handleSqlExecutionResult as (e: Event) => void)
}

const handleSqlExecutionResult = (event: CustomEvent) => {
  const { panelId, result, error, results, originalSql, connectionId, elapsedMs } = event.detail || {}
  if (!panelId) return

  if (results && Array.isArray(results) && results.length > 0) {
    ensureMultiTabResultPanel()
    window.dispatchEvent(new CustomEvent('multi-tab-result-updated', {
      detail: { panelId, results }
    }))
  } else {
    window.dispatchEvent(new CustomEvent('query-result-updated', {
      detail: { panelId, result, error, originalSql, connectionId, elapsedMs }
    }))
  }
}

function findActiveSqlEditorPanelId(): string | null {
  if (!dockviewApi) return null
  if (activeSqlEditorPanel.value && typeof activeSqlEditorPanel.value === 'object' && (activeSqlEditorPanel.value as any)?.id) {
    return (activeSqlEditorPanel.value as any).id
  }
  for (const group of dockviewApi.groups || []) {
    for (const panelInfo of group.model?.panels || []) {
      if (panelInfo.id?.startsWith('panel_sqlEditor_')) {
        return panelInfo.id
      }
    }
  }
  return null
}

const ensureResultPanel = () => {
  if (!dockviewApi) return
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
    const refPanelId = findActiveSqlEditorPanelId() || 'panel_databaseNavigator'
    dockviewApi.addPanel({
      id: 'panel_queryResult',
      component: 'queryResult',
      title: '查询结果',
      position: { referencePanel: refPanelId, direction: 'below' }
    })
    console.log(`[Workbench] 自动创建查询结果面板，位置参考: ${refPanelId}`)
  }
}

const ensureMultiTabResultPanel = () => {
  if (!dockviewApi) return
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
    const refPanelId = findActiveSqlEditorPanelId() || 'panel_databaseNavigator'
    dockviewApi.addPanel({
      id: 'panel_multiTabResult',
      component: 'multiTabResult',
      title: '查询结果',
      position: { referencePanel: refPanelId, direction: 'below' }
    })
    console.log(`[Workbench] 自动创建多 Tab 查询结果面板，位置参考: ${refPanelId}`)
  }
}

const handleOpenSqlEditor = (event: CustomEvent) => {
  if (!dockviewApi) return

  sqlEditorCounter++
  const panelId = `panel_sqlEditor_${sqlEditorCounter}`
  const { connectionId, databaseName, sql } = event.detail || {}

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

  try {
    const emptyPanel = dockviewApi.getPanel?.('panel_emptyWorkbench')
    if (emptyPanel) {
      emptyPanel.api?.close?.()
    }
  } catch (e) {
    console.warn('[SQL Editor] 关闭空工作台面板失败:', e)
  }

  if (existingSqlPanel && existingSqlPanelId) {
    const editorPanel = dockviewApi.addPanel({
      id: panelId,
      component: 'sqlEditor',
      title: `SQL ${sqlEditorCounter}`,
      position: { referencePanel: existingSqlPanelId, direction: 'within' },
      params: { connectionId, databaseName, initialSql: sql }
    })
    activeSqlEditorPanel.value = editorPanel
    console.log('[SQL Editor] 添加到已有组（tab 模式）:', editorPanel)
    return
  }

  const editorPanel = dockviewApi.addPanel({
    id: panelId,
    component: 'sqlEditor',
    title: `SQL ${sqlEditorCounter}`,
    position: { referencePanel: 'panel_databaseNavigator', direction: 'right' },
    params: { connectionId, databaseName, initialSql: sql }
  })
  activeSqlEditorPanel.value = editorPanel
  console.log('[SQL Editor] 首次创建（单栏模式）:', editorPanel)
}

const handleOpenObjectProperties = (event: CustomEvent) => {
  if (!dockviewApi) return

  const { objectType, objectName, connectionName, databaseName, connectionId, dbType, schemaName } = event.detail
  const panelId = `panel_props_${connectionId}_${objectName}`

  const existingPanel = dockviewApi.getPanel?.(panelId)
  if (existingPanel) {
    dockviewApi.setActivePanel?.(existingPanel)
    return
  }

  if (objectType === 'table') {
    dockviewApi.addPanel({
      id: panelId,
      component: 'tableStructure',
      title: `${objectName}`,
      params: { connectionId, databaseName, schemaName: schemaName || 'public', tableName: objectName },
      position: { referencePanel: 'panel_navigator', direction: 'right' },
    })
    return
  }

  dockviewApi.addPanel({
    id: panelId,
    component: 'dynamicObjectProperties',
    title: `${objectName}`,
    params: { objectType, objectName, connectionName, databaseName, connectionId, dbType: dbType || 'mysql' },
    position: { referencePanel: 'panel_navigator', direction: 'right' },
  })
}

const handleProjectSwitched = async () => {
  connectionStore.reset()
  await connectionStore.loadConnections()
  message.info('已切换到新项目')
}

const handleOpenConnectionModal = () => {
  showConnectionModal.value = true
}

onMounted(async () => {
  await projectStore.loadRecentProjects()
  if (!projectStore.currentProject) {
    await projectStore.loadLastProject()
  }
  await connectionStore.loadConnections()
  window.addEventListener('project-switched', handleProjectSwitched as (e: Event) => void)
  window.addEventListener('open-connection-modal', handleOpenConnectionModal)
})

onUnmounted(() => {
  window.removeEventListener('open-object-properties', handleOpenObjectProperties as (e: Event) => void)
  window.removeEventListener('open-sql-editor', handleOpenSqlEditor as (e: Event) => void)
  window.removeEventListener('project-switched', handleProjectSwitched as (e: Event) => void)
  window.removeEventListener('open-connection-modal', handleOpenConnectionModal)
})
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
  min-width: 0;
  min-height: 0;
}
</style>
