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
      :default-tab-component="'IconTab'"
      :get-tab-context-menu-items="getTabContextMenuItems"
      @ready="onReady"
    />

    <!-- 新建连接对话框 -->
    <ConnectionModal v-model="showConnectionModal" @save="handleSaveConnection" />

    <!-- 自定义布局对话框 -->
    <CustomizeLayoutDialog />
  </div>
</template>

<script setup lang="ts">
import {
  DockviewVue,
  type DockviewReadyEvent,
  type DockviewApi as DockviewVueApi,
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
import IconTabComponent from '@/extensions/builtin/workbench/ui/components/IconTab.vue'
import { useDockviewKeyboard } from '@/extensions/builtin/workbench/ui/composables/useDockviewKeyboard'
import { WorkbenchEvent, listenWorkbenchEvent } from '@/extensions/builtin/workbench/ui/constants/workbench-events'
import { useLayoutStore } from '@/extensions/builtin/workbench/ui/stores/layout-store'
import { useUiStore } from '@/shared/stores/ui'
import { useAppStore } from '@/stores/useAppStore'
import { useScratchpadEditorStore } from '@/stores/useScratchpadEditorStore'

defineOptions({
  components: {
    IconTab: IconTabComponent,
  },
})

const { t } = useI18n()
const uiStore = useUiStore()
const layoutStore = useLayoutStore()
const appStore = useAppStore()
const connectionStore = useConnectionStore()
const editorStore = useScratchpadEditorStore()
const message = useMessage()

const dockviewRef = ref<InstanceType<typeof DockviewVue> | null>(null)
const showConnectionModal = ref(false)

let activeSqlEditorPanelId: string | null = null

let dockviewApi: DockviewVueApi | null = null
let sqlEditorCounter = 0

useDockviewKeyboard({ layoutStore })

const getTabContextMenuItems = (params: GetTabContextMenuItemsParams): ContextMenuItem[] => {
  const panelId = params.panel.id
  const maximized = params.group.api.isMaximized()
  const isPinned = layoutStore.isPanelPinned(panelId)

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
        const tabGroup = params.api.createTabGroup({
          groupId: params.group.id,
          label: 'New Group',
          color: 'blue',
        })
        params.api.addPanelToTabGroup({
          groupId: params.group.id,
          tabGroupId: tabGroup.id,
          panelId: params.panel.id,
        })
      },
    },
    {
      label: 'Move to next group',
      action: () => {
        const groups = params.api.getTabGroups({ groupId: params.group.id })
        const currentGroup = params.api.getTabGroupForPanel({
          groupId: params.group.id,
          panelId: params.panel.id,
        })
        if (currentGroup && groups.length > 1) {
          const currentIndex = groups.indexOf(currentGroup)
          const nextIndex = (currentIndex + 1) % groups.length
          params.api.addPanelToTabGroup({
            groupId: params.group.id,
            tabGroupId: groups[nextIndex].id,
            panelId: params.panel.id,
          })
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

    const url = data.url
    if (!url) {
      message.error('连接 URL 不能为空')
      return
    }

    await connectionStore.connect(driver, url, data.name)
    await connectionStore.loadConnections()
    window.dispatchEvent(new CustomEvent('navigator-refresh'))

    if (dockviewApi) {
      const centerRef = findCenterGroupReference()

      sqlEditorCounter++
      const panelId = `panel_sqlEditor_${sqlEditorCounter}`
      dockviewApi.addPanel({
        id: panelId,
        component: 'sqlEditor',
        title: `SQL ${sqlEditorCounter}`,
        position: centerRef,
        params: {
          connectionId: data.name,
          databaseName: data.database || '',
          initialSql: '',
        },
      })

      const emptyPanel = dockviewApi.getPanel('panel_emptyWorkbench')
      if (emptyPanel) {
        emptyPanel.api.close()
      }

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

  const containerEl = dockviewRef.value?.$el as HTMLElement | undefined
  const totalWidth = containerEl?.clientWidth || 1200
  const oneQuarter = Math.round(totalWidth * 0.25)

  // ============================================
  // 第 1 步：A 栏 — 左侧 Edge Group（面板直接作为 tab 列在标题区）
  //   [草稿箱, 数据库导航, 分析资源, 插件]
  //   初始展开，A:B:C = 1:2:1
  // ============================================
  api.addEdgeGroup('left', {
    id: 'left-edge',
    initialSize: oneQuarter,
    minimumSize: 48,
    maximumSize: Math.round(totalWidth * 0.4),
  })

  if (leftPanelsAll.length > 0) {
    const firstLeftPanel = leftPanelsAll[0]
    const firstLeftPanelId = `panel_${firstLeftPanel.id}`
    api.addPanel({
      id: firstLeftPanelId,
      component: firstLeftPanel.id,
      title: firstLeftPanel.name,
      position: { referenceGroup: 'left-edge' },
      renderer: 'onlyWhenVisible',
    })
    layoutStore.updatePanelConfig(firstLeftPanelId, { location: 'left', isVisible: true, order: 0 })

    for (let i = 1; i < leftPanelsAll.length; i++) {
      const panel = leftPanelsAll[i]
      const panelId = `panel_${panel.id}`
      api.addPanel({
        id: panelId,
        component: panel.id,
        title: panel.name,
        position: { referencePanel: firstLeftPanelId, direction: 'within' },
        renderer: 'onlyWhenVisible',
      })
      layoutStore.updatePanelConfig(panelId, { location: 'left', isVisible: true, order: i })
    }
  }
  console.log(`[Workbench] Created left edge group with ${leftPanelsAll.length} panels`)

  // ============================================
  // 第 2 步：B 栏 — 中心 Normal Group（主工作区）
  //   初始: Welcome Page，动态: SQL Editor / Query Result
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
  // 第 3 步：C 栏 — 右侧 Edge Group（面板直接作为 tab 列在标题区）
  //   [列洞察, Mock数据, SQL历史]
  //   初始展开，A:B:C = 1:2:1
  // ============================================
  if (rightPanels.length > 0) {
    api.addEdgeGroup('right', {
      id: 'right-edge',
      initialSize: oneQuarter,
      minimumSize: 48,
      maximumSize: Math.round(totalWidth * 0.4),
    })

    const firstRightPanel = rightPanels[0]
    const firstRightPanelId = `panel_${firstRightPanel.id}`
    api.addPanel({
      id: firstRightPanelId,
      component: firstRightPanel.id,
      title: firstRightPanel.name,
      position: { referenceGroup: 'right-edge' },
      renderer: 'onlyWhenVisible',
    })
    layoutStore.updatePanelConfig(firstRightPanelId, { location: 'right', isVisible: true, order: 0 })

    for (let i = 1; i < rightPanels.length; i++) {
      const panel = rightPanels[i]
      const panelId = `panel_${panel.id}`
      api.addPanel({
        id: panelId,
        component: panel.id,
        title: panel.name,
        position: { referencePanel: firstRightPanelId, direction: 'within' },
        renderer: 'onlyWhenVisible',
      })
      layoutStore.updatePanelConfig(panelId, { location: 'right', isVisible: true, order: i })
    }
  }
  console.log(`[Workbench] Created right edge group with ${rightPanels.length} panels`)

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

  // A 和 C 初始均展开，不调用 collapse

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
    editorStore.removeByPanelId(panelId)
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

function findCenterGroupReference(): { referencePanel?: string; direction?: 'within' | 'right' } {
  if (!dockviewApi) return { direction: 'right' }

  const sqlPanelId = findActiveSqlEditorPanelId()
  if (sqlPanelId) {
    return { referencePanel: sqlPanelId, direction: 'within' }
  }

  const welcomePanel = dockviewApi.getPanel('panel_emptyWorkbench')
  if (welcomePanel) {
    return { referencePanel: 'panel_emptyWorkbench', direction: 'within' }
  }

  for (const group of dockviewApi.groups || []) {
    if (group.id !== 'left-edge' && group.id !== 'right-edge') {
      const firstPanel = group.model?.panels?.[0]
      if (firstPanel) {
        return { referencePanel: firstPanel.id, direction: 'within' }
      }
    }
  }

  return { direction: 'right' }
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

  const { connectionId, databaseName, sql, scratchpadRelativePath, scratchpadFileName, language, initialLine } =
    event.detail || {}

  const isSqlContext =
    !!connectionId ||
    !!databaseName ||
    language === 'sql' ||
    (scratchpadFileName && String(scratchpadFileName).endsWith('.sql'))

  const editorComponent = isSqlContext ? 'sqlEditor' : 'codeEditor'

  if (scratchpadRelativePath) {
    const existingPanelId = editorStore.getPanelId(scratchpadRelativePath)
    if (existingPanelId) {
      const existingPanel = dockviewApi.getPanel(existingPanelId)
      if (existingPanel) {
        existingPanel.focus()
        return
      }
    }
  }

  sqlEditorCounter++
  const panelId = isSqlContext
    ? `panel_sqlEditor_${sqlEditorCounter}`
    : `panel_codeEditor_${sqlEditorCounter}`

  const panelTitle = scratchpadFileName
    ? scratchpadFileName
    : isSqlContext
      ? `SQL ${sqlEditorCounter}`
      : scratchpadRelativePath
        ? scratchpadRelativePath.split('/').pop() || 'Untitled'
        : 'Untitled'

  try {
    const centerRef = findCenterGroupReference()

    dockviewApi.addPanel({
      id: panelId,
      component: editorComponent,
      title: panelTitle,
      position: centerRef,
      params: {
        connectionId: connectionId || '',
        databaseName: databaseName || '',
        initialSql: sql || '',
        initialValue: sql || '',
        scratchpadRelativePath: scratchpadRelativePath || '',
        scratchpadFileName: scratchpadFileName || '',
        fileName: scratchpadFileName || '',
        filePath: scratchpadRelativePath || '',
        language: language || (isSqlContext ? 'sql' : 'plaintext'),
        initialLine: initialLine || 0,
        panelId,
      },
    })

    const welcomePanel = dockviewApi.getPanel('panel_emptyWorkbench')
    if (welcomePanel) {
      welcomePanel.api.close()
    }

    if (scratchpadRelativePath) {
      editorStore.setOpen(scratchpadRelativePath, panelId, panelTitle as string)
    }

    console.log(`[Workbench] 创建${isSqlContext ? 'SQL' : '代码'}编辑器面板: ${panelId}`)
  } catch (error) {
    console.error('[Workbench] 创建编辑器面板失败:', error)
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
    position: findCenterGroupReference(),
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

// 标题栏菜单/工具栏/命令面板事件处理
const handleWorkbenchNewQuery = () => {
  const conns = connectionStore.connections
  const firstConn = conns.length > 0 ? conns[0] : null
  handleOpenSqlEditor(
    new CustomEvent('open-sql-editor', {
      detail: {
        connectionId: firstConn?.name || '',
        databaseName: '',
        sql: '',
      },
    })
  )
}

const handleWorkbenchNewConnection = () => {
  showConnectionModal.value = true
}

const handleWorkbenchSave = () => {
  window.dispatchEvent(new CustomEvent('sql-editor-save'))
}

const handleWorkbenchExecuteSql = () => {
  window.dispatchEvent(new CustomEvent('sql-editor-execute'))
}

const handleWorkbenchOpenDocs = () => {
  // 打开文档链接
  window.open('https://docs.rdatastation.dev', '_blank')
}

const handleWorkbenchKeyboardShortcuts = () => {
  message.info(t('workbench.keyboardShortcuts') + ' - ' + t('workbench.comingSoon'))
}

const handleWorkbenchOpenTerminal = () => {
  message.info(t('workbench.terminal') + ' - ' + t('workbench.comingSoon'))
}

const handleWorkbenchOpenHistory = () => {
  if (!dockviewApi) return
  const panelId = 'panel_sqlHistory'
  const existing = dockviewApi.getPanel(panelId)
  if (existing) {
    existing.focus()
    return
  }
  dockviewApi.addPanel({
    id: panelId,
    component: 'sqlHistory',
    title: t('workbench.sqlHistory'),
    position: { direction: 'right' },
  })
}

const handleWorkbenchToggleSidebar = () => {
  layoutStore.toggleLeftEdgeGroup()
}

const handleWorkbenchOpenCustomizeLayout = () => {
  layoutStore.openCustomizeLayoutDialog()
}

const handleWorkbenchTogglePanel = () => {
  layoutStore.toggleBottomPanelMode()
}

// 标题栏事件监听器清理函数数组
const cleanupListeners: (() => void)[] = []

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

  // 标题栏事件监听 - 使用常量枚举 + listenWorkbenchEvent
  cleanupListeners.push(listenWorkbenchEvent(WorkbenchEvent.NewQuery, handleWorkbenchNewQuery))
  cleanupListeners.push(listenWorkbenchEvent(WorkbenchEvent.NewConnection, handleWorkbenchNewConnection))
  cleanupListeners.push(listenWorkbenchEvent(WorkbenchEvent.Save, handleWorkbenchSave))
  cleanupListeners.push(listenWorkbenchEvent(WorkbenchEvent.ExecuteSql, handleWorkbenchExecuteSql))
  cleanupListeners.push(listenWorkbenchEvent(WorkbenchEvent.OpenDocs, handleWorkbenchOpenDocs))
  cleanupListeners.push(listenWorkbenchEvent(WorkbenchEvent.KeyboardShortcuts, handleWorkbenchKeyboardShortcuts))
  cleanupListeners.push(listenWorkbenchEvent(WorkbenchEvent.OpenTerminal, handleWorkbenchOpenTerminal))
  cleanupListeners.push(listenWorkbenchEvent(WorkbenchEvent.OpenHistory, handleWorkbenchOpenHistory))
  cleanupListeners.push(listenWorkbenchEvent(WorkbenchEvent.ToggleSidebar, handleWorkbenchToggleSidebar))
  cleanupListeners.push(listenWorkbenchEvent(WorkbenchEvent.TogglePanel, handleWorkbenchTogglePanel))
  cleanupListeners.push(listenWorkbenchEvent(WorkbenchEvent.OpenCustomizeLayout, handleWorkbenchOpenCustomizeLayout))
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

  // 清理标题栏事件监听
  cleanupListeners.forEach(cleanup => cleanup())
  cleanupListeners.length = 0
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
