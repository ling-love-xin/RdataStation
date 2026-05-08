<template>
  <div :class="['workbench-view', uiStore.isDark ? 'dockview-theme-dark' : 'dockview-theme-light']">
    <!-- Dockview 布局 -->
    <!-- 面板组件通过 app.component() 全局注册，由 dockview-vue 的 findComponent 自动解析 -->
    <DockviewVue
      ref="dockviewRef"
      class="dockview"
      :popout-url="'/popout.html'"
      :floating-group-bounds="'boundedWithinViewport'"
      :right-header-actions-component="'PanelHeaderActions'"
      :get-tab-context-menu-items="getTabContextMenuItems"
      @ready="onReady"
    />

    <!-- 底部状态栏 -->
    <WorkbenchStatusBar />

    <!-- 新建连接对话框 -->
    <ConnectionModal v-model="showConnectionModal" @save="handleSaveConnection" />

    <!-- 自定义布局对话框 -->
    <CustomizeLayoutDialog
      v-if="layoutStore.showCustomizeLayoutDialog"
      @close="layoutStore.closeCustomizeLayoutDialog()"
    />
  </div>
</template>

<script setup lang="ts">
import {
  DockviewVue,
  type DockviewReadyEvent,
  type DockviewApi as DockviewVueApi,
  type IDockviewPanel,
  type GetTabContextMenuItemsParams,
  type ContextMenuItem,
} from 'dockview-vue'
import { useMessage } from 'naive-ui'
import { ref, onMounted, onUnmounted } from 'vue'
import { useI18n } from 'vue-i18n'

import { panelRegistry } from '@/core/panel-registry'
import ConnectionModal from '@/extensions/builtin/connection/ui/components/ConnectionModal.vue'
import { useConnectionStore } from '@/extensions/builtin/connection/ui/stores/connection-store'
import type { ConnectionConfig } from '@/extensions/builtin/connection/ui/types/connection'
import CustomizeLayoutDialog from '@/extensions/builtin/workbench/ui/components/CustomizeLayoutDialog.vue'
import WorkbenchStatusBar from '@/extensions/builtin/workbench/ui/components/WorkbenchStatusBar.vue'
import { useLayoutStore } from '@/extensions/builtin/workbench/ui/stores/layout-store'
import { useUiStore } from '@/shared/stores/ui'
import { useAppStore } from '@/stores/useAppStore'

const { t } = useI18n()
const uiStore = useUiStore()
const layoutStore = useLayoutStore()
const appStore = useAppStore()
const connectionStore = useConnectionStore()
const message = useMessage()

const dockviewRef = ref<InstanceType<typeof DockviewVue> | null>(null)
const showConnectionModal = ref(false)

let activeSqlEditorPanelId: string | null = null

let dockviewApi: DockviewVueApi | null = null
let sqlEditorCounter = 0

const getTabContextMenuItems = (params: GetTabContextMenuItemsParams): ContextMenuItem[] => {
  const panelId = params.panel.id
  const maximized = !!params.group.api.isMaximized?.()
  const isPinned = layoutStore.isPanelPinned(panelId)

  const groupApi = params.group.api as any
  const panelApi = params.panel.api as any

  const menuItems: ContextMenuItem[] = [
    {
      label: isPinned ? t('workbench.unpin') : t('workbench.pin'),
      action: () => {
        layoutStore.togglePanelPinned(panelId)
      },
    },
    'separator',
    {
      label: 'Float Tab',
      action: () => {
        params.api.addFloatingGroup(params.panel)
      },
    },
    {
      label: 'Popout Tab',
      action: () => {
        params.api.addPopoutGroup(params.panel)
      },
    },
    'separator',
    {
      label: 'Add to new group',
      action: () => {
        const tabGroup = groupApi.createTabGroup?.({
          label: 'New Group',
          color: 'blue',
        })
        if (tabGroup) {
          panelApi.moveToTabGroup?.(tabGroup)
        }
      },
    },
    {
      label: 'Move to next group',
      action: () => {
        const groups = groupApi.getTabGroups?.() || []
        const currentGroup = groupApi.getTabGroupForPanel?.(params.panel)
        if (currentGroup && groups.length > 1) {
          const currentIndex = groups.indexOf(currentGroup)
          const nextIndex = (currentIndex + 1) % groups.length
          panelApi.moveToTabGroup?.(groups[nextIndex])
        }
      },
    },
    'separator',
    {
      label: maximized ? t('workbench.restoreMaximize') : t('workbench.maximizeGroup'),
      action: () => {
        if (maximized) params.group.api.exitMaximized()
        else params.group.api.maximize()
      },
    },
    {
      label: t('workbench.floatGroup'),
      action: () => {
        params.api.addFloatingGroup(params.group)
      },
    },
    {
      label: t('workbench.popoutGroup'),
      action: () => {
        params.api.addPopoutGroup(params.group)
      },
    },
    'separator',
  ]

  if (!isPinned) {
    menuItems.push('close', 'closeOthers', 'closeAll')
  }

  return menuItems
}

const handleSaveConnection = async (data: Partial<ConnectionConfig>) => {
  console.log('保存连接:', data)
  try {
    const driver = String((data as Record<string, unknown>).db_type || data.driver)
    if (!driver) {
      message.error(t('workbench.selectDbType'))
      return
    }

    let url = ''
    const isFileDb = driver === 'sqlite' || driver === 'duckdb'

    if (isFileDb) {
      const filePath = data.database
      if (!filePath) {
        message.error(t('workbench.selectDbFile'))
        return
      }
      url = `${driver}://${filePath}`
    } else {
      const host = data.host
      if (!host) {
        message.error(t('workbench.enterHost'))
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
          initialSql: '',
        },
      })

      ensureResultPanel()
    }

    message.success(t('workbench.connectionSaved', { name: data.name }))
  } catch (error) {
    const errorMsg = error instanceof Error ? error.message : t('workbench.saveConnectionFailed')
    message.error(t('workbench.saveFailed', { error: errorMsg }))
    console.error('保存连接失败:', error)
  }
}

const onReady = (event: DockviewReadyEvent) => {
  const api = event.api
  dockviewApi = api
  layoutStore.setDockviewApi(api)

  const panels = panelRegistry.getAll()
  console.log(`[Workbench] Creating ${panels.length} panels from registry`)

  const leftPanelsAll = panels
    .filter(p => p.location === 'left')
    .sort((a, b) => (a.order || 0) - (b.order || 0))
  const centerPanels = panels
    .filter(p => p.location === 'center')
    .sort((a, b) => (a.order || 0) - (b.order || 0))
  const rightPanels = panels
    .filter(p => p.location === 'right')
    .sort((a, b) => (a.order || 0) - (b.order || 0))

  // 分离数据库导航 + 草稿箱面板：放入 B 区独立 Normal Group
  const dbNavPanel = leftPanelsAll.find(p => p.id === 'databaseNavigator')
  const scratchpadPanel = leftPanelsAll.find(p => p.id === 'scratchpad')
  // 其余左侧面板仍放回左侧 Edge Group（分析资源管理、插件管理等）
  const leftEdgePanels = leftPanelsAll.filter(
    p => p.id !== 'databaseNavigator' && p.id !== 'scratchpad'
  )

  const containerEl = dockviewRef.value?.$el as HTMLElement | undefined
  const totalWidth = containerEl?.clientWidth || 1200
  const oneQuarter = Math.round(totalWidth * 0.25)

  // ============================================
  // 第 1 步：左侧 Edge Group
  // 收起态: 48px 窄条  展开态: 25%，使 A:B:C:D = 1:1:1:1
  // ============================================
  api.addEdgeGroup('left', {
    id: 'left-edge',
    initialSize: oneQuarter,
    minimumSize: 48,
    maximumSize: Math.round(totalWidth * 0.4),
  })

  if (leftEdgePanels.length > 0) {
    const firstLeftPanel = leftEdgePanels[0]
    const firstLeftPanelId = `panel_${firstLeftPanel.id}`
    api.addPanel({
      id: firstLeftPanelId,
      component: firstLeftPanel.id,
      title: firstLeftPanel.name,
      position: { referenceGroup: 'left-edge' },
    })
    layoutStore.updatePanelConfig(firstLeftPanelId, { location: 'left', isVisible: false, order: 0 })

    for (let i = 1; i < leftEdgePanels.length; i++) {
      const panel = leftEdgePanels[i]
      const panelId = `panel_${panel.id}`
      api.addPanel({
        id: panelId,
        component: panel.id,
        title: panel.name,
        position: { referencePanel: firstLeftPanelId, direction: 'within' },
      })
      layoutStore.updatePanelConfig(panelId, { location: 'left', isVisible: false, order: i })
    }
  }
  console.log(`[Workbench] Created left edge group with ${leftEdgePanels.length} panels`)

  // ============================================
  // 第 2 步：B 区 — 数据库导航 + 草稿箱（同一 Normal Group，tab 切换）
  // ============================================
  if (dbNavPanel) {
    api.addPanel({
      id: `panel_${dbNavPanel.id}`,
      component: dbNavPanel.id,
      title: dbNavPanel.name,
      position: { direction: 'left' },
    })
    console.log(`[Workbench] Created database navigator as normal group: panel_${dbNavPanel.id}`)
    layoutStore.updatePanelConfig(`panel_${dbNavPanel.id}`, {
      location: 'center',
      isVisible: true,
      order: 0,
    })
  }

  if (scratchpadPanel) {
    const scratchpadPanelId = `panel_${scratchpadPanel.id}`
    api.addPanel({
      id: scratchpadPanelId,
      component: scratchpadPanel.id,
      title: scratchpadPanel.name,
      position: dbNavPanel
        ? { referencePanel: `panel_${dbNavPanel.id}`, direction: 'within' }
        : { direction: 'left' },
    })
    console.log(`[Workbench] Created scratchpad panel: ${scratchpadPanelId}`)
    layoutStore.updatePanelConfig(scratchpadPanelId, {
      location: 'center',
      isVisible: true,
      order: 1,
    })
  }

  // ============================================
  // 第 3 步：欢迎页（EmptyWorkbenchPanel，中心区域）
  // ============================================
  const welcomePanel = centerPanels.find(p => p.id === 'emptyWorkbench')
  if (welcomePanel) {
    api.addPanel({
      id: `panel_${welcomePanel.id}`,
      component: welcomePanel.id,
      title: welcomePanel.name,
      position: { direction: 'right' },
    })
    console.log(`[Workbench] Created welcome panel: panel_${welcomePanel.id}`)
    layoutStore.updatePanelConfig(`panel_${welcomePanel.id}`, {
      location: 'center',
      isVisible: true,
      order: 0,
    })
  }

  // ============================================
  // 第 4 步：右侧 Edge Group（默认展开）
  // ============================================
  if (rightPanels.length > 0) {
    api.addEdgeGroup('right', {
      id: 'right-edge',
      initialSize: oneQuarter,
      minimumSize: 200,
      maximumSize: Math.round(totalWidth * 0.4),
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
    layoutStore.updatePanelConfig(firstRightPanelId, {
      location: 'right',
      isVisible: true,
      order: 0,
    })

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
      layoutStore.updatePanelConfig(panelId, { location: 'right', isVisible: true, order: i })
    }
  }

  console.log(
    `[Workbench] Final layout - groups: ${api.groups.length}, panels: ${api.panels.length}`
  )
  console.log(
    '[Workbench] All groups:',
    api.groups.map(g => ({
      id: g.id,
      panels: g.panels.map(p => p.id),
    }))
  )

  layoutStore.setBottomPanelMode('editor')

  layoutStore.collapseLeftEdgeGroup()

  // ============================================
  // 事件监听
  // ============================================
  api.onDidActivePanelChange?.(panel => {
    if (panel?.id?.startsWith('panel_sqlEditor_')) {
      activeSqlEditorPanelId = panel.id
    }
  })

  api.onDidRemovePanel?.(panel => {
    const panelId = panel.id
    if (layoutStore.isPanelPinned(panelId)) {
      console.log(`[Workbench] Attempted to close pinned panel: ${panelId}, restoring...`)
      setTimeout(() => {
        restorePinnedPanel(panelId)
      }, 100)
    }
  })

  window.addEventListener(
    'open-object-properties',
    handleOpenObjectProperties as (e: Event) => void
  )
  window.addEventListener('open-sql-editor', handleOpenSqlEditor as (e: Event) => void)

  api.onDidLayoutChange?.(() => {
    try {
      const serialized = api.toJSON()
      layoutStore.setLayoutData(serialized)

      const ids = api.panels.map((p: { id: string }) => p.id)
      layoutStore.setOpenPanelIds(ids)
    } catch {
      // dockview serialization may fail transiently
    }
  })

  restoreSavedPanels()
}

function restoreSavedPanels() {
  if (!dockviewApi) return

  const savedIds = layoutStore.openPanelIds
  if (!savedIds || savedIds.length === 0) return

  const currentIds = new Set(dockviewApi.panels.map((p: { id: string }) => p.id))

  for (const panelId of savedIds) {
    if (currentIds.has(panelId)) continue

    const componentKey = panelId.replace(/^panel_/, '').replace(/_\d+$/, '')
    const panelInfo = panelRegistry.get(componentKey)
    if (!panelInfo) continue

    try {
      dockviewApi.addPanel({
        id: panelId,
        component: panelInfo.id,
        title: panelInfo.name,
      })
    } catch {
      // restore of individual panel may fail
    }
  }
}

function restorePinnedPanel(panelId: string) {
  if (!dockviewApi) return

  const panelConfig = layoutStore.getPanelConfig(panelId)
  if (!panelConfig) return

  const panelInfo = panelRegistry.get(panelId.replace('panel_', ''))
  if (!panelInfo) return

  dockviewApi.addPanel({
    id: panelId,
    component: panelInfo.id,
    title: panelInfo.name,
    position: {
      direction:
        panelConfig.location === 'left'
          ? 'left'
          : panelConfig.location === 'right'
            ? 'right'
            : 'center',
    },
  })
  console.log(`[Workbench] Restored pinned panel: ${panelId}`)
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
          return (
            panelId === 'panel_databaseNavigator' ||
            panelId === 'panel_analytics-resource-manager' ||
            panelId === 'panel_plugins'
          )
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

function findActiveSqlEditorPanelId(): string | null {
  if (!dockviewApi) return null
  if (activeSqlEditorPanelId) {
    return activeSqlEditorPanelId
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
      title: t('workbench.queryResult'),
      position: { referencePanel: refPanelId, direction: 'below' },
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
      title: t('workbench.queryResult'),
      position: { referencePanel: refPanelId, direction: 'below' },
    })
    console.log(`[Workbench] 自动创建多 TAB 查询结果面板，位置参考: ${refPanelId}`)
  }
}

const handleOpenSqlEditor = (event: CustomEvent) => {
  if (!dockviewApi) return

  const { connectionId, databaseName, sql, scratchpadRelativePath, scratchpadFileName, language } =
    event.detail || {}

  if (scratchpadRelativePath) {
    for (const group of dockviewApi.groups || []) {
      for (const panelInfo of group.model?.panels || []) {
        const panel = dockviewApi.getPanel(panelInfo.id)
        const params = panel?.params?.params as Record<string, unknown> | undefined
        if (
          panel?.id?.startsWith('panel_sqlEditor_') &&
          params?.scratchpadRelativePath === scratchpadRelativePath
        ) {
          panel.focus()
          return
        }
      }
    }
  }

  sqlEditorCounter++
  const panelId = `panel_sqlEditor_${sqlEditorCounter}`

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
      title: scratchpadFileName ? `📜 ${scratchpadFileName}` : `SQL ${sqlEditorCounter}`,
      position: existingSqlPanelId
        ? { referencePanel: existingSqlPanelId, direction: 'within' }
        : { direction: 'center' },
      params: {
        connectionId: connectionId || '',
        databaseName: databaseName || '',
        initialSql: sql || '',
        scratchpadRelativePath: scratchpadRelativePath || '',
        scratchpadFileName: scratchpadFileName || '',
        language: language || 'sql',
      },
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
      database,
    },
  })
  console.log(`[Workbench] 创建对象属性面板: ${panelId}`)
}

const handleKeydown = (e: KeyboardEvent) => {
  if ((e.ctrlKey || e.metaKey) && e.shiftKey && e.code === 'KeyE') {
    e.preventDefault()
    const conns = connectionStore.connections
    const firstConn = conns.length > 0 ? conns[0] : null
    handleOpenSqlEditor(
      new CustomEvent('open-sql-editor', {
        detail: {
          connectionId: firstConn?.name || '',
          databaseName: firstConn?.url || '',
          sql: '',
        },
      })
    )
  }
}

onMounted(() => {
  layoutStore.loadLayoutConfig()

  window.addEventListener(
    'layout-settings-update',
    handleLayoutSettingsUpdate as (e: Event) => void
  )
  window.addEventListener('reset-layout', handleResetLayout as (e: Event) => void)
  window.addEventListener(
    'open-object-properties',
    handleOpenObjectProperties as (e: Event) => void
  )
  window.addEventListener('open-sql-editor', handleOpenSqlEditor as (e: Event) => void)
  window.addEventListener('keydown', handleKeydown)
})

onUnmounted(() => {
  appStore.closeProject().catch(() => {})

  window.removeEventListener(
    'layout-settings-update',
    handleLayoutSettingsUpdate as (e: Event) => void
  )
  window.removeEventListener('reset-layout', handleResetLayout as (e: Event) => void)
  window.removeEventListener(
    'open-object-properties',
    handleOpenObjectProperties as (e: Event) => void
  )
  window.removeEventListener('open-sql-editor', handleOpenSqlEditor as (e: Event) => void)
  window.removeEventListener('keydown', handleKeydown)
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
