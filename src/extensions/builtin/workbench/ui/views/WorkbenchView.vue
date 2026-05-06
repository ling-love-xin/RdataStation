<template>
  <div :class="['workbench-view', uiStore.isDark ? 'dockview-theme-dark' : 'dockview-theme-light']">
    <!-- Dockview 布局 -->
    <!-- 面板组件通过 app.component() 全局注册，由 dockview-vue 的 findComponent 自动解析 -->
    <DockviewVue
      ref="dockviewRef"
      class="dockview"
      :style="dockviewStyle"
      :popout-url="'/popout.html'"
      :floating-group-bounds="'boundedWithinViewport'"
      :right-header-actions-component="'panelHeaderActions'"
      :get-tab-context-menu-items="getTabContextMenuItems"
      @ready="onReady"
    />

    <!-- 底部状态栏 -->
    <WorkbenchStatusBar />

    <!-- 新建连接对话框 -->
    <ConnectionModal
      v-model="showConnectionModal"
      @save="handleSaveConnection"
    />

    <!-- 自定义布局对话框 -->
    <CustomizeLayoutDialog
      v-if="showCustomizeLayoutDialog"
      @close="showCustomizeLayoutDialog = false"
    />
  </div>
</template>

<script setup lang="ts">
import { DockviewVue, type DockviewReadyEvent, type DockviewApi as DockviewVueApi, type IDockviewPanel, type GetTabContextMenuItemsParams, type ContextMenuItem } from 'dockview-vue'
import { useMessage } from 'naive-ui'
import { ref, computed, onMounted, onUnmounted } from 'vue'

import { panelRegistry } from '@/core/panel-registry'
import ConnectionModal from '@/extensions/builtin/connection/ui/components/ConnectionModal.vue'
import { useConnectionStore } from '@/extensions/builtin/connection/ui/stores/connection-store'
import type { ConnectionConfig } from '@/extensions/builtin/connection/ui/types/connection'
import CustomizeLayoutDialog from '@/extensions/builtin/workbench/ui/components/CustomizeLayoutDialog.vue'
import WorkbenchStatusBar from '@/extensions/builtin/workbench/ui/components/WorkbenchStatusBar.vue'
import { useLayoutStore } from '@/extensions/builtin/workbench/ui/stores/layout-store'
import { useUiStore } from '@/shared/stores/ui'

const uiStore = useUiStore()
const layoutStore = useLayoutStore()
const connectionStore = useConnectionStore()
const message = useMessage()

const dockviewRef = ref<InstanceType<typeof DockviewVue> | null>(null)
const showConnectionModal = ref(false)
const showCustomizeLayoutDialog = ref(false)

const dockviewStyle = computed(() => ({
  height: '100%',
  width: '100%',
}))

const activeSqlEditorPanel = ref<IDockviewPanel | null>(null)

let dockviewApi: DockviewVueApi | null = null
let sqlEditorCounter = 0

const pinnedPanelIds = new Set<string>()

const getTabContextMenuItems = (params: GetTabContextMenuItemsParams): ContextMenuItem[] => {
  const { panel, group, api } = params
  const maximized = !!group.api.isMaximized?.()
  const isPinned = pinnedPanelIds.has(panel.id)
  return [
    {
      label: isPinned ? '取消钉住' : '钉住',
      action: () => {
        if (isPinned) pinnedPanelIds.delete(panel.id)
        else pinnedPanelIds.add(panel.id)
      }
    },
    'separator',
    {
      label: 'Float Tab',
      action: () => { api.addFloatingGroup(panel) }
    },
    {
      label: 'Popout Tab',
      action: () => { api.addPopoutGroup(panel) }
    },
    'separator',
    {
      label: maximized ? '还原最大化' : '最大化组',
      action: () => { if (maximized) group.api.exitMaximized(); else group.api.maximize() }
    },
    {
      label: '浮动整组',
      action: () => { api.addFloatingGroup(group) }
    },
    {
      label: '弹出整组窗口',
      action: () => { api.addPopoutGroup(group) }
    },
    'separator',
    'close',
    'closeOthers',
    'closeAll',
  ]
}


const handleSaveConnection = async (data: Partial<ConnectionConfig>) => {
  console.log('保存连接:', data)
  try {
    const driver = String((data as Record<string, unknown>).db_type || data.driver)
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
      const emptyPanel = dockviewApi.getPanel('panel_emptyWorkbench')
      if (emptyPanel) {
        emptyPanel.api.close()
      }

      sqlEditorCounter++
      const panelId = `panel_sqlEditor_${sqlEditorCounter}`
      dockviewApi.addPanel({
        id: panelId,
        component: 'sqlEditor',
        title: `SQL ${sqlEditorCounter}`,
        position: { direction: 'center' },
        params: {
          connectionId: data.name,
          databaseName: data.database || '',
          initialSql: ''
        }
      })

      ensureResultPanel()
    }

    message.success(`连接 "${data.name}" 保存成功`)
  } catch (error) {
    const errorMsg = error instanceof Error ? error.message : '保存连接失败'
    message.error(`保存失败: ${errorMsg}`)
    console.error('保存连接失败:', error)
  }
}

const onReady = (event: DockviewReadyEvent) => {
  const api = event.api
  dockviewApi = api
  layoutStore.setDockviewApi(api)

  const panels = panelRegistry.getAll()
  console.log(`[Workbench] Creating ${panels.length} panels from registry`)

  const leftPanels = panels.filter(p => p.location === 'left').sort((a, b) => (a.order || 0) - (b.order || 0))
  const centerPanels = panels.filter(p => p.location === 'center').sort((a, b) => (a.order || 0) - (b.order || 0))
  const rightPanels = panels.filter(p => p.location === 'right').sort((a, b) => (a.order || 0) - (b.order || 0))

  const containerEl = dockviewRef.value?.$el as HTMLElement | undefined
  const totalWidth = containerEl?.clientWidth || 1200
  const ratioA = Math.round(totalWidth * 0.2)
  const ratioC = Math.round(totalWidth * 0.2)

  // ============================================
  // 使用 dockview 6.0 创建布局
  // B 区域初始只显示欢迎页（EmptyWorkbenchPanel），SQL 编辑器和结果面板在用户操作时动态创建
  // 侧边栏使用 Edge Group，中心/底部区域使用普通 Group
  // ============================================

  // ---- 第 1 步：B 区域 - 欢迎页面（EmptyWorkbenchPanel） ----
  // 注意：只创建 emptyWorkbench，不自动创建 sqlEditor 和底部面板
  // SQL 编辑器会在用户点击"新建查询"或创建连接时通过 handleOpenSqlEditor 动态创建
  const welcomePanel = centerPanels.find(p => p.id === 'emptyWorkbench')

  if (welcomePanel) {
    api.addPanel({
      id: `panel_${welcomePanel.id}`,
      component: welcomePanel.id,
      title: welcomePanel.name,
      position: { direction: 'right' },
    })
    console.log(`[Workbench] Created welcome panel: panel_${welcomePanel.id}`)
    layoutStore.updatePanelConfig(`panel_${welcomePanel.id}`, { location: 'center', isVisible: true, order: welcomePanel.order || 0 })
  }

  // ---- 第 3 步：左侧 Edge Group（侧边栏） ----
  // Edge Group 会在中心区域左侧自动插入，中心内容自适应
  if (leftPanels.length > 0) {
    api.addEdgeGroup('left', {
      id: 'left-edge',
      initialSize: ratioA,
      minimumSize: 200,
      maximumSize: 500,
    })

    const firstLeftPanel = leftPanels[0]
    const firstLeftPanelId = `panel_${firstLeftPanel.id}`

    api.addPanel({
      id: firstLeftPanelId,
      component: firstLeftPanel.id,
      title: firstLeftPanel.name,
      position: { referenceGroup: 'left-edge' },
    })
    console.log(`[Workbench] Created left edge panel: ${firstLeftPanelId}`)
    layoutStore.updatePanelConfig(firstLeftPanelId, { location: 'left', isVisible: true, order: firstLeftPanel.order || 0 })

    for (let i = 1; i < leftPanels.length; i++) {
      const panel = leftPanels[i]
      const panelId = `panel_${panel.id}`
      api.addPanel({
        id: panelId,
        component: panel.id,
        title: panel.name,
        position: { referencePanel: firstLeftPanelId, direction: 'within' },
      })
      console.log(`[Workbench] Added left edge panel: ${panelId}`)
      layoutStore.updatePanelConfig(panelId, { location: 'left', isVisible: true, order: panel.order || i })
    }
  }

  // ---- 第 4 步：右侧 Edge Group（侧边栏） ----
  if (rightPanels.length > 0) {
    api.addEdgeGroup('right', {
      id: 'right-edge',
      initialSize: ratioC,
      minimumSize: 200,
      maximumSize: 500,
    })

    const firstRightPanel = rightPanels[0]
    const firstRightPanelId = `panel_${firstRightPanel.id}`

    api.addPanel({
      id: firstRightPanelId,
      component: firstRightPanel.id,
      title: firstRightPanel.name,
      position: { referenceGroup: 'right-edge' },
    })
    console.log(`[Workbench] Created right edge panel: ${firstRightPanelId}`)
    layoutStore.updatePanelConfig(firstRightPanelId, { location: 'right', isVisible: true, order: firstRightPanel.order || 0 })

    for (let i = 1; i < rightPanels.length; i++) {
      const panel = rightPanels[i]
      const panelId = `panel_${panel.id}`
      api.addPanel({
        id: panelId,
        component: panel.id,
        title: panel.name,
        position: { referencePanel: firstRightPanelId, direction: 'within' },
      })
      console.log(`[Workbench] Added right edge panel: ${panelId}`)
      layoutStore.updatePanelConfig(panelId, { location: 'right', isVisible: true, order: panel.order || i })
    }
  }

  console.log(`[Workbench] Final layout - groups: ${api.groups.length}, panels: ${api.panels.length}`)
  console.log('[Workbench] All groups:', api.groups.map(g => ({
    id: g.id,
    panels: g.panels.map(p => p.id)
  })))

  layoutStore.setBottomPanelMode('editor')

  // ============================================
  // 事件监听
  // ============================================
  api.onDidActivePanelChange?.((panel) => {
    if (panel?.id?.startsWith('panel_sqlEditor_')) {
      activeSqlEditorPanel.value = panel
    }
  })

  window.addEventListener('open-object-properties', handleOpenObjectProperties as (e: Event) => void)
  window.addEventListener('open-sql-editor', handleOpenSqlEditor as (e: Event) => void)
  window.addEventListener('sql-execution-result', handleSqlExecutionResult as (e: Event) => void)
}

const handleLayoutSettingsUpdate = (_event: CustomEvent) => {
  const { leftWidth, rightWidth, minimumWidth, maximumWidth } = _event.detail || {}
  if (!dockviewApi) return

  try {
    const groups = dockviewApi.groups || []

    if (leftWidth !== undefined) {
      const leftGroup = groups.find((g: { model?: { panels?: { id: string }[] } }) =>
        g.model?.panels?.some((p: { id: string }) => {
          const panelId = p.id
          return panelId === 'panel_databaseNavigator' || panelId === 'panel_analytics-resource-manager' || panelId === 'panel_plugins'
        })
      )
      if (leftGroup) {
        const g = leftGroup as unknown as Record<string, (args: Record<string, number>) => void>
        g.setSize?.({ width: leftWidth })
      }
    }

    if (rightWidth !== undefined) {
      const rightGroup = groups.find((g: { model?: { panels?: { id: string }[] } }) =>
        g.model?.panels?.some((p: { id: string }) => {
          const panelId = p.id
          return panelId === 'panel_sqlHistory' || panelId === 'panel_columnInsights'
        })
      )
      if (rightGroup) {
        const g = rightGroup as unknown as Record<string, (args: Record<string, number>) => void>
        g.setSize?.({ width: rightWidth })
      }
    }

    if (minimumWidth !== undefined || maximumWidth !== undefined) {
      for (const group of groups) {
        const constraints: Record<string, number> = {}
        if (minimumWidth !== undefined) constraints.minimumWidth = minimumWidth
        if (maximumWidth !== undefined) constraints.maximumWidth = maximumWidth
        const g = group as unknown as Record<string, (args: Record<string, number>) => void>
        g.updateConstraints?.(constraints)
      }
    }
  } catch (error) {
    console.error('[Workbench] Failed to apply layout settings:', error)
  }
}

const handleResetLayout = () => {
  if (!dockviewApi) return
  try {
    const allGroups = dockviewApi.groups || []
    for (const group of allGroups) {
      const g = group as { model?: { panels?: { id: string }[] } }
      const panels = g.model?.panels || []
      for (const panelInfo of panels) {
        const panel = dockviewApi.getPanel?.(panelInfo.id)
        panel?.api?.close?.()
      }
    }
    setTimeout(() => {
      onReady({ api: dockviewApi } as unknown as DockviewReadyEvent)
    }, 100)
  } catch (error) {
    console.error('[Workbench] Failed to reset layout:', error)
  }
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
    const refPanelId = findActiveSqlEditorPanelId()
    if (!refPanelId) return
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
    const refPanelId = findActiveSqlEditorPanelId()
    if (!refPanelId) return
    dockviewApi.addPanel({
      id: 'panel_multiTabResult',
      component: 'multiTabResult',
      title: '查询结果',
      position: { referencePanel: refPanelId, direction: 'below' }
    })
    console.log(`[Workbench] 自动创建多 TAB 查询结果面板，位置参考: ${refPanelId}`)
  }
}

const handleOpenSqlEditor = (event: CustomEvent) => {
  if (!dockviewApi) return

  sqlEditorCounter++
  const panelId = `panel_sqlEditor_${sqlEditorCounter}`
  const { connectionId, databaseName, sql } = event.detail || {}

  let existingSqlPanel: IDockviewPanel | undefined
  let existingSqlPanelId: string | null = null

  for (const group of dockviewApi.groups || []) {
    for (const panelInfo of group.model?.panels || []) {
      const panel = dockviewApi.getPanel(panelInfo.id)
      if (panel?.id?.startsWith('panel_sqlEditor_')) {
        existingSqlPanel = panel
        existingSqlPanelId = panel.id
        break
      }
    }
    if (existingSqlPanel) break
  }

  try {
    dockviewApi.addPanel({
      id: panelId,
      component: 'sqlEditor',
      title: `SQL ${sqlEditorCounter}`,
      position: existingSqlPanelId
        ? { referencePanel: existingSqlPanelId, direction: 'within' }
        : { direction: 'center' },
      params: {
        connectionId: connectionId || '',
        databaseName: databaseName || '',
        initialSql: sql || ''
      }
    })
    console.log(`[Workbench] 创建 SQL 编辑器面板: ${panelId}`)
  } catch (error) {
    console.error('[Workbench] 创建 SQL 编辑器面板失败:', error)
  }
}

const handleOpenObjectProperties = (event: CustomEvent) => {
  if (!dockviewApi) return
  const { objectType, objectName, schema, database } = event.detail || {}

  const panelId = `panel_dynamicObjectProperties_${Date.now()}`
  dockviewApi.addPanel({
    id: panelId,
    component: 'dynamicObjectProperties',
    title: `${objectType}: ${objectName}`,
    position: { direction: 'center' },
    params: {
      objectType,
      objectName,
      schema,
      database
    }
  })
  console.log(`[Workbench] 创建对象属性面板: ${panelId}`)
}

onMounted(() => {
  window.addEventListener('layout-settings-update', handleLayoutSettingsUpdate as (e: Event) => void)
  window.addEventListener('reset-layout', handleResetLayout as (e: Event) => void)
})

onUnmounted(() => {
  window.removeEventListener('layout-settings-update', handleLayoutSettingsUpdate as (e: Event) => void)
  window.removeEventListener('reset-layout', handleResetLayout as (e: Event) => void)
  window.removeEventListener('open-object-properties', handleOpenObjectProperties as (e: Event) => void)
  window.removeEventListener('open-sql-editor', handleOpenSqlEditor as (e: Event) => void)
  window.removeEventListener('sql-execution-result', handleSqlExecutionResult as (e: Event) => void)
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
