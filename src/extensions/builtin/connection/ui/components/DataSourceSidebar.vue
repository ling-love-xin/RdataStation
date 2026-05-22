<template>
  <div class="datasource-sidebar">
    <!-- 搜索框 -->
    <div class="ds-search">
      <Search :size="14" class="ds-search-icon" />
      <input
        v-model="searchQuery"
        type="text"
        class="ds-search-input"
        :placeholder="$t('navigator.searchConnection')"
      />
    </div>

    <!-- 数据源类型区域 -->
    <div class="ds-section">
      <div class="ds-section-header">
        <span class="ds-section-title">{{ $t('navigator.dataSourceTypes') }}</span>
        <NButton size="tiny" quaternary @click="openAddDialog()">
          <template #icon><Plus :size="14" /></template>
        </NButton>
      </div>
      <div class="ds-type-list">
        <div
          v-for="type in filteredTypes"
          :key="type.id"
          :class="['ds-type-item', { active: selectedTypeId === type.id }]"
          @click="selectType(type.id)"
        >
          <component :is="typeIcon(type.category)" :size="16" class="ds-type-icon" />
          <span class="ds-type-name">{{ type.name }}</span>
          <span class="ds-type-count">{{ getDriverCount(type.id) }}</span>
        </div>
      </div>
    </div>

    <!-- 驱动列表 -->
    <div v-if="selectedTypeDrivers.length > 0" class="ds-section">
      <div class="ds-section-header">
        <span class="ds-section-title">{{ $t('navigator.drivers') }}</span>
      </div>
      <div class="ds-driver-list">
        <div
          v-for="driver in selectedTypeDrivers"
          :key="driver.id"
          :class="['ds-driver-item', { active: selectedDriver?.id === driver.id }]"
          @click="selectDriver(driver)"
        >
          <div class="ds-driver-badge" :style="{ background: driverColor(driver.name) }">
            {{ driverInitials(driver.name) }}
          </div>
          <span class="ds-driver-name">{{ driver.name }}</span>
        </div>
      </div>
    </div>

    <!-- 已有连接区域 -->
    <div class="ds-section ds-section-last">
      <div class="ds-section-header">
        <span class="ds-section-title">{{ $t('navigator.existingConnections') }}</span>
      </div>

      <div v-if="!projectStore.hasProject" class="ds-empty">
        {{ $t('navigator.noOpenProject') }}
      </div>
      <div v-else-if="connections.length === 0" class="ds-empty">
        {{ $t('navigator.noDatabaseConnections') }}
      </div>
      <div v-else class="ds-conn-list">
        <div
          v-for="conn in connections"
          :key="conn.id"
          class="ds-conn-item"
          @click="openSavedConnection(conn)"
        >
          <div class="ds-conn-indicator" :class="'status-' + (conn.status || 'disconnected')" />
          <div class="ds-conn-body">
            <span class="ds-conn-name">{{ conn.name }}</span>
            <span class="ds-conn-meta">{{ conn.driver }}</span>
          </div>
          <NButton
            size="tiny"
            text
            :loading="testingId === conn.id"
            title="测试连接"
            @click.stop="testSavedConnection(conn)"
          >
            <template #icon><RefreshCw :size="12" /></template>
          </NButton>
        </div>
      </div>
    </div>

    <!-- 驱动管理区域 -->
    <div v-if="driversWithStatus.length > 0" class="ds-section">
      <div class="ds-section-header">
        <span class="ds-section-title">驱动管理</span>
      </div>
      <div class="ds-driver-mgmt-list">
        <div
          v-for="d in driversWithStatus"
          :key="d.driver.id"
          class="ds-driver-mgmt-item"
        >
          <div class="ds-driver-mgmt-info">
            <span class="ds-driver-mgmt-name">{{ d.driver.name }}</span>
            <span class="ds-driver-mgmt-status" :class="'dms-' + d.status">
              {{ d.status === 'ready' ? '✓ 就绪' : d.status === 'not_installed' ? '⚠ 未安装' : '✗ 未启用' }}
            </span>
          </div>
          <NButton
            v-if="d.status === 'not_installed'"
            size="tiny"
            secondary
            type="warning"
            @click="handleInstallDriver(d.driver.id)"
          >
            安装
          </NButton>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { Plus, Search, Database, Server, Globe, HardDrive, Cloud, Radio, RefreshCw } from 'lucide-vue-next'
import { NButton } from 'naive-ui'
import { ref, computed, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'

import { useProjectStore } from '@/core/project/stores/project'
import {
  WorkbenchEvent,
  dispatchWorkbenchEvent,
} from '@/extensions/builtin/workbench/ui/constants/workbench-events'

import { useDriverRegistry } from '../composables/useDriverRegistry'
import { useProjectConnectionStore } from '../stores/project-connection-store'

import type { Driver } from '../../domain/types'
import type { ProjectConnection } from '../../types/connection'

const { t } = useI18n()
const projectStore = useProjectStore()
const projectConnectionStore = useProjectConnectionStore()
const { drivers, dataSourceTypes, loadAll, installDriver, getDriverDetail } = useDriverRegistry()

const driverDetailCache = ref<Map<string, string>>(new Map()) // driver_id → availability
const installingDriverId = ref<string | null>(null)

/** 带有安装状态的驱动列表 */
const driversWithStatus = computed(() => {
  // 只显示有驱动（非 necessarily native，可能是外部 JAR 等需安装的）
  // 但实际上对于 native 驱动也会显示（扭计为小）
  return drivers.value.map(d => ({
    driver: d,
    status: driverDetailCache.value.get(d.id) || (d.driver_kind === 'native' ? 'ready' : 'not_installed'),
  }))
})

/** 安装驱动 */
async function handleInstallDriver(driverId: string) {
  installingDriverId.value = driverId
  try {
    await installDriver(driverId)
    driverDetailCache.value.set(driverId, 'ready')
  } catch (e) {
    // eslint-disable-next-line no-console
    console.error(`[sidebar:driver] 安装 ${driverId} 失败:`, e)
  } finally {
    installingDriverId.value = null
  }
}

// State
const searchQuery = ref('')
const selectedTypeId = ref<string | null>(null)
const selectedDriver = ref<Driver | null>(null)
const testingId = ref<string | null>(null)

// Computed
/** 只显示已连接的数据库（排除 disconnected/connecting/error 状态） */
const connections = computed(() =>
  (projectConnectionStore.connections as ProjectConnection[]).filter(c => c.status === 'connected')
)

const filteredTypes = computed(() => {
  const q = searchQuery.value.toLowerCase()
  if (!q) return dataSourceTypes.value
  return dataSourceTypes.value.filter(
    t => t.enabled && t.name.toLowerCase().includes(q)
  )
})

const selectedTypeDrivers = computed(() => {
  if (!selectedTypeId.value) return []
  return drivers.value.filter(
    d => d.type_id === selectedTypeId.value && d.enabled
  )
})

function getDriverCount(typeId: string): number {
  return drivers.value.filter(d => d.type_id === typeId && d.enabled).length
}

// Icons
const categoryIcons: Record<string, typeof Database> = {
  relational: Database,
  'file-based': HardDrive,
  nosql: Radio,
  analytics: Server,
  cloud: Cloud,
  mq: Globe,
  http: Globe,
}

function typeIcon(category: string) {
  return categoryIcons[category] || Database
}

// Driver colors & initials
const driverColors: Record<string, string> = {
  mysql: '#00758f', postgresql: '#336791', sqlite: '#003b57',
  duckdb: '#f9a825', mariadb: '#c49a6c', oracle: '#f80000',
  mssql: '#0089b6', clickhouse: '#faff00', mongodb: '#47a248',
  redis: '#dc382d', cassandra: '#1287b1', cockroachdb: '#6933ff',
  snowflake: '#29bfff', bigquery: '#4285f4', redshift: '#8c4fff',
  elasticsearch: '#00bfb3', neo4j: '#018bff', couchbase: '#ea2328',
  influxdb: '#22adf6', timescaledb: '#fec514',
}

function driverColor(name: string): string {
  return driverColors[name.toLowerCase()] || '#555'
}

function driverInitials(name: string): string {
  return name.slice(0, 2).toUpperCase()
}

// Actions
function selectType(typeId: string) {
  selectedTypeId.value = typeId
  selectedDriver.value = null
}

function selectDriver(driver: Driver) {
  selectedDriver.value = driver
  dispatchWorkbenchEvent(WorkbenchEvent.NewConnection, { driver })
}

function openAddDialog(driver?: Driver) {
  dispatchWorkbenchEvent(WorkbenchEvent.NewConnection, { driver: driver || null })
}

/** 从侧边栏打开已有连接 → 切换活动连接并打开查询编辑器 */
async function openSavedConnection(conn: ProjectConnection) {
  try {
    const { invoke } = await import('@tauri-apps/api/core')
    const url = projectConnectionStore.getConnectionUrl(conn)
    const driverName = conn.driver || 'mysql'

    // 1. 建立（或复用）运行时连接
    const r = await invoke<{ conn_id: string; name: string; db_type: string; url: string }>(
      'connect_database',
      {
        input: {
          db_type: driverName,
          url,
          name: conn.name,
          connection_type: conn.connection_type || 'project',
          project_id: projectStore.currentProject?.id || null,
        },
      }
    )

    // 2. 切换为活动连接
    await invoke('switch_connection', { connId: r.conn_id })

    // 3. 派发事件打开查询编辑器
    dispatchWorkbenchEvent(WorkbenchEvent.NewQuery, {
      connectionId: r.conn_id,
      databaseName: conn.database || '',
      sql: '',
    })

    // 4. 刷新侧边栏连接列表（更新状态）
    await projectConnectionStore.loadConnections()
  } catch (e) {
    const msg = e instanceof Error ? e.message : String(e)
    // eslint-disable-next-line no-console
    console.error(`[sidebar:open] ${conn.name}:`, msg)
  }
}

/** 从侧边栏测试已保存连接 */
async function testSavedConnection(conn: ProjectConnection) {
  testingId.value = conn.id
  try {
    const { invoke } = await import('@tauri-apps/api/core')
    const url = projectConnectionStore.getConnectionUrl(conn)
    const driverName = conn.driver || 'mysql'
    const r = await invoke<{ success: boolean; message?: string; response_time_ms?: number }>(
      'test_connection',
      { dbType: driverName, url }
    )
    const msg = r.success
      ? `✓ 连接成功 (${r.response_time_ms ?? '?'}ms)`
      : `✗ ${r.message || '连接失败'}`
    // 更新本地连接状态
    await projectConnectionStore.updateConnectionStatus(
      conn.id,
      r.success ? 'connected' : 'error',
      r.success ? undefined : (r.message || '连接失败')
    )
    // eslint-disable-next-line no-console
    console.warn(`[sidebar:test] ${conn.name}: ${msg}`)
  } catch (e) {
    const msg = e instanceof Error ? e.message : String(e)
    // eslint-disable-next-line no-console
    console.error(`[sidebar:test] ${conn.name} 失败:`, msg)
    await projectConnectionStore.updateConnectionStatus(conn.id, 'error', msg)
  } finally {
    testingId.value = null
  }
}

// Init
onMounted(async () => {
  await loadAll(projectStore.currentProject?.path)
  if (projectStore.hasProject) {
    await projectConnectionStore.loadConnections()
  }
})
</script>

<style scoped>
.datasource-sidebar {
  display: flex;
  flex-direction: column;
  height: 100%;
  overflow: hidden;
  padding: 8px;
  gap: 2px;
}

.ds-search {
  position: relative;
  margin-bottom: 4px;
}

.ds-search-icon {
  position: absolute;
  left: 8px;
  top: 50%;
  transform: translateY(-50%);
  color: var(--color-text-muted, #6c7086);
}

.ds-search-input {
  width: 100%;
  height: 28px;
  padding: 0 8px 0 28px;
  border: 1px solid var(--color-border-subtle, #3c3f41);
  border-radius: 4px;
  font-size: 12px;
  background: var(--color-bg-secondary, #2b2d30);
  color: var(--color-text-primary, #e5e7eb);
  outline: none;
}

.ds-search-input:focus {
  border-color: var(--brand-accent, #e17055);
}

.ds-section {
  display: flex;
  flex-direction: column;
}

.ds-section-last {
  flex: 1;
  overflow: auto;
  min-height: 0;
}

.ds-section-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 6px 4px;
}

.ds-section-title {
  font-size: 11px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  color: var(--color-text-muted, #6c7086);
}

.ds-type-list {
  display: flex;
  flex-direction: column;
  gap: 1px;
}

.ds-type-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 6px 8px;
  border-radius: 4px;
  cursor: pointer;
  font-size: 12px;
  color: var(--color-text-secondary, #9ca3af);
  transition: background 0.12s;
}

.ds-type-item:hover {
  background: var(--color-hover, #454545);
}

.ds-type-item.active {
  background: var(--brand-accent-soft, rgba(225, 112, 85, 0.15));
  color: var(--brand-accent, #e17055);
}

.ds-type-icon {
  flex-shrink: 0;
  opacity: 0.7;
}

.ds-type-item.active .ds-type-icon {
  opacity: 1;
}

.ds-type-name {
  flex: 1;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.ds-type-count {
  font-size: 10px;
  color: var(--color-text-muted, #6c7086);
  background: var(--color-bg-elevated, rgba(255,255,255,0.04));
  padding: 1px 6px;
  border-radius: 8px;
}

.ds-driver-list {
  display: flex;
  flex-direction: column;
  gap: 1px;
}

.ds-driver-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 5px 8px;
  border-radius: 4px;
  cursor: pointer;
  transition: background 0.12s;
}

.ds-driver-item:hover {
  background: var(--color-hover, #454545);
}

.ds-driver-item.active {
  background: var(--brand-accent-soft, rgba(225, 112, 85, 0.15));
}

.ds-driver-badge {
  width: 22px;
  height: 22px;
  border-radius: 4px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 10px;
  font-weight: 700;
  color: #fff;
  flex-shrink: 0;
}

.ds-driver-name {
  font-size: 12px;
  color: var(--color-text-secondary, #9ca3af);
}

.ds-conn-list {
  display: flex;
  flex-direction: column;
  gap: 1px;
}

.ds-conn-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 5px 8px;
  border-radius: 4px;
  cursor: pointer;
  transition: background 0.12s;
}

.ds-conn-item:hover {
  background: var(--color-hover, #454545);
}

.ds-conn-indicator {
  width: 7px;
  height: 7px;
  border-radius: 50%;
  flex-shrink: 0;
}

.status-connected { background: var(--brand-success, #00b894); }
.status-disconnected { background: var(--color-text-muted, #6c7086); }
.status-connecting { background: var(--brand-accent, #e17055); animation: pulse 1s infinite; }
.status-error { background: var(--brand-danger, #d63031); }

@keyframes pulse {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.4; }
}

.ds-conn-body {
  display: flex;
  flex-direction: column;
  min-width: 0;
  flex: 1;
}

.ds-conn-name {
  font-size: 12px;
  color: var(--color-text-primary, #e5e7eb);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.ds-conn-meta {
  font-size: 10px;
  color: var(--color-text-muted, #6c7086);
}

.ds-empty {
  padding: 16px 8px;
  font-size: 12px;
  color: var(--color-text-muted, #6c7086);
  text-align: center;
}

/* ── 驱动管理 ── */
.ds-driver-mgmt-list {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.ds-driver-mgmt-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 4px 8px;
  border-radius: 4px;
  background: var(--color-bg-secondary, #2b2d30);
}

.ds-driver-mgmt-info {
  display: flex;
  align-items: center;
  gap: 8px;
  min-width: 0;
}

.ds-driver-mgmt-name {
  font-size: 12px;
  color: var(--color-text-primary, #e5e7eb);
  white-space: nowrap;
}

.ds-driver-mgmt-status {
  font-size: 11px;
  white-space: nowrap;
}

.ds-driver-mgmt-status.dms-ready {
  color: var(--color-success, #22c55e);
}

.ds-driver-mgmt-status.dms-not_installed {
  color: var(--color-warning, #f59e0b);
}

.ds-driver-mgmt-status.dms-not_enabled {
  color: var(--color-text-muted, #6c7086);
}
</style>