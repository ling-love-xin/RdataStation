<template>
  <div class="database-manager">
    <!-- 页面标题 -->
    <div class="page-header">
      <h1 class="page-title">{{ t('navigator.databaseConnectionManager') }}</h1>
      <div class="header-actions">
        <button class="btn btn-primary" @click="handleNewConnection">
          <Plus :size="16" class="btn-icon" />
          {{ t('navigator.newConnection') }}
        </button>
      </div>
    </div>

    <!-- 搜索和筛选 -->
    <div class="search-filter">
      <div class="search-input-wrapper">
        <Search :size="16" class="search-icon" />
        <input
          v-model="searchQuery"
          type="text"
          class="search-input"
          :placeholder="t('navigator.searchConnection')"
        />
        <button v-if="searchQuery" class="clear-btn" @click="searchQuery = ''">
          <X :size="14" />
        </button>
      </div>

      <div class="filter-options">
        <NDropdown :options="filterOptions" @select="handleFilterChange">
          <button class="filter-btn">
            <Filter :size="14" class="filter-icon" />
            {{ getFilterLabel(currentFilter) }}
            <ChevronDown :size="12" class="dropdown-icon" />
          </button>
        </NDropdown>
      </div>
    </div>

    <!-- 连接列表 -->
    <div class="connection-list-container">
      <!-- 加载状态 -->
      <div v-if="loading" class="loading-overlay">
        <Loader2 :size="24" class="loading-icon" />
        <span>{{ t('navigator.loadingConnections') }}</span>
      </div>

      <!-- 空状态 -->
      <div v-else-if="filteredConnections.length === 0" class="empty-state">
        <Database :size="48" class="empty-icon" />
        <h3 class="empty-title">{{ t('navigator.noDatabaseConnections') }}</h3>
        <p class="empty-desc">{{ t('navigator.clickNewConnection') }}</p>
        <button class="btn btn-primary" @click="handleNewConnection">
          {{ t('navigator.newConnection') }}
        </button>
      </div>

      <!-- 连接列表表格 -->
      <table v-else class="connection-table">
        <thead>
          <tr>
            <th class="col-name">{{ t('navigator.connectionName') }}</th>
            <th class="col-type">{{ t('navigator.databaseType') }}</th>
            <th class="col-host">{{ t('navigator.host') }}</th>
            <th class="col-db">{{ t('navigator.database') }}</th>
            <th class="col-status">{{ t('navigator.status') }}</th>
            <th class="col-actions">{{ t('navigator.operation') }}</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="connection in filteredConnections" :key="connection.id" class="connection-row">
            <td class="col-name">
              <div class="connection-info">
                <DbIcon :type="connection.driver" :size="18" class="db-icon" />
                <span class="connection-name">{{ connection.name }}</span>
              </div>
            </td>
            <td class="col-type">
              <span class="db-type-tag">{{ connection.driver }}</span>
            </td>
            <td class="col-host">
              {{ connection.host || '-' }}
            </td>
            <td class="col-db">
              {{ connection.database || '-' }}
            </td>
            <td class="col-status">
              <div
                class="status-indicator"
                :class="`status-${connection.status || 'disconnected'}`"
              >
                <div
                  :class="['status-dot', `status-dot-${connection.status || 'disconnected'}`]"
                ></div>
                <span class="status-text">{{ getStatusLabel(connection.status) }}</span>
              </div>
            </td>
            <td class="col-actions">
              <div class="action-buttons">
                <button
                  v-if="connection.status === 'disconnected' || connection.status === 'error'"
                  class="action-btn action-btn-success"
                  :title="t('navigator.connectDatabase')"
                  :disabled="connectingId === connection.id"
                  @click="handleConnectDatabase(connection)"
                >
                  <Power :size="14" />
                </button>
                <button
                  v-else-if="connection.status === 'connected'"
                  class="action-btn action-btn-warning"
                  :title="t('navigator.disconnect')"
                  :disabled="connectingId === connection.id"
                  @click="handleDisconnectDatabase(connection)"
                >
                  <PowerOff :size="14" />
                </button>
                <button
                  class="action-btn"
                  :title="t('navigator.editConnection')"
                  @click="handleEditConnection(connection)"
                >
                  <Pencil :size="14" />
                </button>
                <button
                  class="action-btn action-btn-danger"
                  :title="t('navigator.deleteConnection')"
                  @click="handleDeleteConnection(connection.id)"
                >
                  <Trash2 :size="14" />
                </button>
              </div>
            </td>
          </tr>
        </tbody>
      </table>
    </div>

    <!-- 添加数据源对话框 -->
    <AddDataSourceDialog
      v-model="showConnectionModal"
      :is-editing="!!editingConnection"
      :edit-data="editingConnectionAsConfig"
      :project-path="projectStore.currentProject?.path || null"
      @save="handleSaveConnection"
    />
  </div>
</template>

<script setup lang="ts">
import {
  Plus,
  Search,
  X,
  Filter,
  ChevronDown,
  Loader2,
  Database,
  Pencil,
  Trash2,
  Power,
  PowerOff,
} from 'lucide-vue-next'
import { NDropdown, useMessage } from 'naive-ui'
import { ref, computed, onMounted, watch } from 'vue'
import { useI18n } from 'vue-i18n'

import { useProjectStore } from '@/core/project/stores/project'
import DbIcon from '@/shared/components/common/DbIcon.vue'

import AddDataSourceDialog from './AddDataSourceDialog.vue'
import * as connectionService from '../services/connection'
import { useProjectConnectionStore } from '../stores/project-connection-store'
import { useRuntimeConnectionStore } from '../stores/runtime-connection-store'

import type { ProjectConnection, ConnectionConfiguration } from '../types/connection'

// 状态管理
const { t } = useI18n()
const projectConnectionStore = useProjectConnectionStore()
const runtimeConnectionStore = useRuntimeConnectionStore()
const projectStore = useProjectStore()
const message = useMessage()

// 状态
const searchQuery = ref('')
const currentFilter = ref('all')
const loading = ref(false)
const showConnectionModal = ref(false)
const editingConnection = ref<ProjectConnection | null>(null)
const connectingId = ref<string | null>(null)

// 获取状态标签
const getStatusLabel = (status?: string): string => {
  switch (status) {
    case 'connected':
      return t('navigator.connected')
    case 'connecting':
      return t('navigator.connecting')
    case 'error':
      return t('navigator.connectionError')
    default:
      return t('navigator.disconnected')
  }
}

// 处理连接数据库
const handleConnectDatabase = async (connection: ProjectConnection) => {
  if (connectingId.value) return

  connectingId.value = connection.id

  try {
    // 更新状态为连接中
    await projectConnectionStore.updateConnectionStatus(connection.id, 'connecting')

    // 建立运行时连接
    const runtimeConnId = await runtimeConnectionStore.establishRuntimeConnection(connection)

    if (runtimeConnId) {
      // 更新状态为已连接
      await projectConnectionStore.updateConnectionStatus(connection.id, 'connected')
      message.success(t('navigator.connectionSuccess', { name: connection.name }))
    } else {
      // 更新状态为错误
      await projectConnectionStore.updateConnectionStatus(connection.id, 'error', '连接失败')
      message.error(t('navigator.connectionFailed', { name: connection.name }))
    }
  } catch (error) {
    const errorMsg = error instanceof Error ? error.message : t('navigator.connectionFailedGeneric')
    await projectConnectionStore.updateConnectionStatus(connection.id, 'error', errorMsg)
    message.error(`连接失败: ${errorMsg}`)
  } finally {
    connectingId.value = null
  }
}

// 处理断开连接
const handleDisconnectDatabase = async (connection: ProjectConnection) => {
  if (connectingId.value) return

  connectingId.value = connection.id

  try {
    // 关闭运行时连接
    await runtimeConnectionStore.closeRuntimeConnection(connection.id)

    // 更新状态为未连接
    await projectConnectionStore.updateConnectionStatus(connection.id, 'disconnected')
    message.success(t('navigator.disconnectedSuccess', { name: connection.name }))
  } catch (error) {
    const errorMsg = error instanceof Error ? error.message : t('navigator.disconnectFailed')
    message.error(`${t('navigator.disconnectFailed')}: ${errorMsg}`)
  } finally {
    connectingId.value = null
  }
}

// 转换 ProjectConnection 为 ConnectionConfiguration
const editingConnectionAsConfig = computed<ConnectionConfiguration | null>(() => {
  if (!editingConnection.value) return null
  const conn = editingConnection.value
  return {
    id: conn.id,
    name: conn.name,
    driver: conn.driver,
    host: conn.host || '',
    port: conn.port,
    database: conn.database,
    username: conn.username,
    password: conn.password,
    properties: conn.properties,
  }
})

// 过滤选项
const filterOptions = [
  { key: 'all', label: t('navigator.allConnections') },
  { key: 'mysql', label: 'MySQL' },
  { key: 'postgres', label: 'PostgreSQL' },
  { key: 'sqlite', label: 'SQLite' },
  { key: 'duckdb', label: 'DuckDB' },
]

// 计算属性：过滤后的连接列表
const filteredConnections = computed(() => {
  let connections = [...projectConnectionStore.connections]

  // 按搜索词过滤
  if (searchQuery.value) {
    const query = searchQuery.value.toLowerCase()
    connections = connections.filter(
      conn =>
        conn.name.toLowerCase().includes(query) ||
        conn.host?.toLowerCase().includes(query) ||
        conn.database?.toLowerCase().includes(query)
    )
  }

  // 按类型过滤
  if (currentFilter.value !== 'all') {
    connections = connections.filter(conn => conn.driver === currentFilter.value)
  }

  return connections
})

// 辅助函数：获取过滤标签
const getFilterLabel = (key: string): string => {
  const option = filterOptions.find(opt => opt.key === key)
  return option?.label || t('navigator.allConnections')
}

// 处理新建连接
const handleNewConnection = () => {
  editingConnection.value = null
  showConnectionModal.value = true
}

// 处理编辑连接
const handleEditConnection = (connection: ProjectConnection) => {
  editingConnection.value = connection
  showConnectionModal.value = true
}

// 处理保存连接
const handleSaveConnection = async (
  data: Partial<ProjectConnection> & {
    saveToGlobal?: boolean
    saveToProject?: boolean
    useDuckdbFed?: boolean
    url?: string
    networkConfigId?: string | null
    driverId?: string
    environmentId?: string
    authConfigId?: string
    advanced?: Record<string, unknown>
    driverProps?: Record<string, unknown>
  }
) => {
  try {
    const driver = data.driver
    if (!driver) {
      message.error(t('navigator.selectDbType'))
      // eslint-disable-next-line no-console
      console.warn('db_type 为空，完整数据:', data)
      return
    }

    // 构建连接 URL
    const url = data.url || buildConnectionUrlFromData(data)
    if (!url) {
      message.error(t('navigator.buildUrlFailed'))
      return
    }

    // 检查是否至少选择了一个保存位置
    if (!data.saveToGlobal && !data.saveToProject) {
      message.error(t('navigator.selectSaveLocation'))
      return
    }

    const isFileDatabase = driver === 'sqlite' || driver === 'duckdb'
    const connectionData = {
      name: data.name || '',
      driver: driver,
      host: isFileDatabase ? data.database || data.host || '' : data.host || '',
      port: isFileDatabase ? 0 : data.port || 0,
      database: isFileDatabase ? data.database || '' : data.database || '',
      username: data.username || '',
      password: data.password || '',
      properties: data.properties || {},
      use_duckdb_fed: data.useDuckdbFed || false,
    }

    // 提取新增字段
    const connectOpts = {
      driverId: data.driverId,
      networkConfigId: data.networkConfigId,
      environmentId: data.environmentId,
      authConfigId: data.authConfigId,
      driverProperties: data.driverProps ? JSON.stringify(data.driverProps) : undefined,
      advancedOptions: data.advanced ? JSON.stringify(data.advanced) : undefined,
    }

    if (editingConnection.value) {
      // 更新连接：先保存到存储，不重新连接
      const updatedConnection: ProjectConnection = {
        ...editingConnection.value,
        ...data,
        updated_at: new Date().toISOString(),
      }
      await projectConnectionStore.updateConnection(updatedConnection)
      message.success(t('navigator.connectionUpdated', { name: data.name }))
    } else {
      // 新建连接：根据选择保存到全局和/或项目
      const savedLocations: string[] = []

      // 1. 保存到全局
      if (data.saveToGlobal) {
        try {
          // 建立全局运行时连接
          await connectionService.connectDatabase(driver, url, data.name, 'global', undefined, connectOpts)
          savedLocations.push(t('navigator.global'))
        } catch (error) {
          // eslint-disable-next-line no-console
          console.error('创建全局连接失败:', error)
          // 不阻断后续项目连接创建
        }
      }

      // 2. 保存到项目
      if (data.saveToProject) {
        if (!projectStore.currentProject?.path) {
          message.warning(t('navigator.noOpenProject'))
        } else {
          try {
            // 建立项目运行时连接
            await connectionService.connectDatabase(
              driver,
              url,
              data.name,
              'project',
              projectStore.currentProject.id,
              connectOpts
            )

            // 保存配置到项目存储
            await projectConnectionStore.createConnection(connectionData)
            savedLocations.push(t('navigator.project'))
            } catch (error) {
            // eslint-disable-next-line no-console
            console.error('创建项目连接失败:', error)
          }
        }
      }

      if (savedLocations.length > 0) {
        message.success(
          t('navigator.connectionSavedTo', {
            name: data.name,
            locations: savedLocations.join(', '),
          })
        )
      } else {
        message.error(t('navigator.connectionSaveFailed'))
        return
      }
    }

    // 刷新连接列表
    await projectConnectionStore.loadConnections()

    // 触发导航树刷新事件
    window.dispatchEvent(new CustomEvent('navigator-refresh'))

    // 触发打开工作台事件，传入新连接信息
    window.dispatchEvent(
      new CustomEvent('open-sql-editor', {
        detail: {
          connectionId: data.name,
          databaseName: data.database || '',
          sql: '',
        },
      })
    )

    handleCloseConnectionModal()
  } catch (error) {
    const errorMsg = error instanceof Error ? error.message : t('navigator.connectionSaveFailed')
    message.error(`${t('common.operationFailed')}: ${errorMsg}`)
    // eslint-disable-next-line no-console
    console.error('保存连接失败:', error)
  }
}

// 辅助函数：从数据构建连接 URL
function buildConnectionUrlFromData(data: Partial<ProjectConnection>): string {
  const driver = data.driver
  if (!driver) return ''

  if (driver === 'sqlite' || driver === 'duckdb') {
    // 文件型数据库：优先使用 database 字段，其次是 host 字段
    const filePath = data.database || data.host || ''
    return `${driver}://${filePath}`
  }

  const host = data.host || 'localhost'
  const port = data.port || (driver === 'mysql' ? 3306 : 5432)
  const database = data.database || ''
  const username = data.username || ''
  const password = data.password || ''

  if (username && password) {
    return `${driver}://${username}:${password}@${host}:${port}/${database}`
  }
  return `${driver}://${host}:${port}/${database}`
}

// 处理关闭连接对话框
const handleCloseConnectionModal = () => {
  showConnectionModal.value = false
  editingConnection.value = null
}

// 处理删除连接
const handleDeleteConnection = async (connectionId: string) => {
  try {
    await projectConnectionStore.deleteConnection(connectionId)
    message.success(t('navigator.connectionDeleted'))
    await projectConnectionStore.loadConnections()
  } catch (error) {
    const errorMsg = error instanceof Error ? error.message : t('navigator.deleteFailed')
    message.error(`${t('common.deleteFailed')}: ${errorMsg}`)
  }
}

// 处理过滤变化
const handleFilterChange = (key: string) => {
  currentFilter.value = key
}

// 监听项目变化
watch(
  () => projectStore.currentProject,
  async newProject => {
    if (newProject?.path) {
      loading.value = true
      await projectConnectionStore.loadConnections()
      loading.value = false
    } else {
      // 没有项目打开时，清空连接列表
      projectConnectionStore.reset()
    }
  },
  { immediate: true }
)

// 组件挂载
onMounted(async () => {
  if (projectStore.currentProject?.path) {
    loading.value = true
    await projectConnectionStore.loadConnections()
    loading.value = false
  }
})
</script>

<style scoped>
.database-manager {
  padding: 24px;
  max-width: 1200px;
  margin: 0 auto;
  height: 100%;
  display: flex;
  flex-direction: column;
}

/* 页面标题 */
.page-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 24px;
}

.page-title {
  margin: 0;
  font-size: 24px;
  font-weight: 600;
  color: var(--text-primary);
}

.header-actions {
  display: flex;
  gap: 12px;
}

/* 按钮样式 */
.btn {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  padding: 8px 16px;
  border-radius: 6px;
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s ease;
  border: 1px solid transparent;
}

.btn-primary {
  background-color: var(--primary-color);
  color: white;
}

.btn-primary:hover {
  background-color: var(--primary-hover);
}

.btn-icon {
  width: 16px;
  height: 16px;
}

/* 搜索和筛选 */
.search-filter {
  display: flex;
  gap: 16px;
  margin-bottom: 24px;
  align-items: center;
  flex-wrap: wrap;
}

.search-input-wrapper {
  flex: 1;
  min-width: 300px;
  position: relative;
}

.search-input {
  width: 100%;
  height: 36px;
  padding: 0 12px 0 36px;
  border: 1px solid var(--border-color);
  border-radius: 6px;
  font-size: 14px;
  background-color: var(--bg-primary);
  color: var(--text-primary);
  transition: all 0.2s ease;
}

.search-input:focus {
  outline: none;
  border-color: var(--primary-color);
  box-shadow: 0 0 0 3px rgba(22, 93, 255, 0.1);
}

.search-icon {
  position: absolute;
  left: 12px;
  top: 50%;
  transform: translateY(-50%);
  color: var(--text-tertiary);
}

.clear-btn {
  position: absolute;
  right: 12px;
  top: 50%;
  transform: translateY(-50%);
  width: 20px;
  height: 20px;
  border: none;
  background: transparent;
  color: var(--text-tertiary);
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 3px;
  transition: all 0.2s ease;
}

.clear-btn:hover {
  background-color: var(--bg-hover);
  color: var(--text-secondary);
}

.filter-options {
  display: flex;
  gap: 8px;
}

.filter-btn {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 8px 12px;
  border: 1px solid var(--border-color);
  border-radius: 6px;
  font-size: 14px;
  background-color: var(--bg-primary);
  color: var(--text-secondary);
  cursor: pointer;
  transition: all 0.2s ease;
}

.filter-btn:hover {
  border-color: var(--primary-color);
  color: var(--text-primary);
}

.filter-icon {
  width: 14px;
  height: 14px;
}

.dropdown-icon {
  width: 12px;
  height: 12px;
  color: var(--text-tertiary);
}

/* 连接列表容器 */
.connection-list-container {
  flex: 1;
  overflow: auto;
  background-color: var(--bg-primary);
  border-radius: 8px;
  border: 1px solid var(--border-color);
  position: relative;
}

/* 加载状态 */
.loading-overlay {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 8px;
  background-color: rgba(var(--bg-primary), 0.8);
  color: var(--text-secondary);
  font-size: 14px;
}

.loading-icon {
  animation: spin 1s linear infinite;
}

@keyframes spin {
  from {
    transform: rotate(0deg);
  }
  to {
    transform: rotate(360deg);
  }
}

/* 连接表格 */
.connection-table {
  width: 100%;
  border-collapse: collapse;
}

.connection-table th,
.connection-table td {
  padding: 12px 16px;
  text-align: left;
  border-bottom: 1px solid var(--border-color);
}

.connection-table th {
  background-color: var(--bg-secondary);
  font-weight: 600;
  font-size: 13px;
  color: var(--text-secondary);
  white-space: nowrap;
}

.connection-table td {
  font-size: 14px;
  color: var(--text-primary);
}

.connection-row {
  transition: background-color 0.2s ease;
}

.connection-row:hover {
  background-color: var(--bg-hover);
}

.connection-row.connected {
  background-color: rgba(16, 185, 129, 0.05);
}

/* 列样式 */
.col-name {
  width: 25%;
}

.col-type {
  width: 15%;
}

.col-host {
  width: 20%;
}

.col-db {
  width: 20%;
}

.col-status {
  width: 12%;
}

.col-actions {
  width: 8%;
  text-align: right;
}

/* 连接信息 */
.connection-info {
  display: flex;
  align-items: center;
  gap: 10px;
}

.db-icon {
  color: var(--primary-color);
}

.connection-name {
  font-weight: 500;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

/* 数据库类型标签 */
.db-type-tag {
  display: inline-block;
  padding: 4px 8px;
  border-radius: 4px;
  font-size: 12px;
  font-weight: 500;
  background-color: var(--primary-light);
  color: var(--primary-color);
}

/* 状态指示器 */
.status-indicator {
  display: flex;
  align-items: center;
  gap: 6px;
}

.status-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
}

.status-dot-connected {
  background-color: var(--success-color);
  animation: pulse 2s infinite;
}

.status-dot-disconnected {
  background-color: var(--text-tertiary);
}

.status-dot-connecting {
  background-color: var(--primary-color);
  animation: pulse 1s infinite;
}

.status-dot-error {
  background-color: var(--danger-color);
}

@keyframes pulse {
  0%,
  100% {
    opacity: 1;
  }
  50% {
    opacity: 0.5;
  }
}

.status-text {
  font-size: 13px;
  color: var(--text-secondary);
}

/* 操作按钮 */
.action-buttons {
  display: flex;
  gap: 8px;
  justify-content: flex-end;
}

.action-btn {
  width: 28px;
  height: 28px;
  border: none;
  border-radius: 4px;
  background-color: transparent;
  color: var(--text-secondary);
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.2s ease;
}

.action-btn:hover {
  background-color: var(--bg-hover);
  color: var(--text-primary);
}

.action-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.action-btn-success {
  color: var(--success-color);
}

.action-btn-success:hover {
  background-color: rgba(16, 185, 129, 0.1);
  color: var(--success-color);
}

.action-btn-warning {
  color: var(--warning-color);
}

.action-btn-warning:hover {
  background-color: rgba(245, 158, 11, 0.1);
  color: var(--warning-color);
}

.action-btn-danger:hover {
  background-color: var(--danger-light);
  color: var(--danger-color);
}

/* 空状态 */
.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 64px 24px;
  text-align: center;
}

.empty-icon {
  margin-bottom: 16px;
  color: var(--text-tertiary);
}

.empty-title {
  margin: 0 0 8px 0;
  font-size: 18px;
  font-weight: 600;
  color: var(--text-primary);
}

.empty-desc {
  margin: 0 0 24px 0;
  font-size: 14px;
  color: var(--text-secondary);
}

/* 测试结果对话框 */
.test-result {
  display: flex;
  gap: 16px;
  align-items: center;
  padding: 16px;
}

.result-icon {
  flex-shrink: 0;
}

.result-icon.success {
  color: var(--success-color);
}

.result-icon.error {
  color: var(--danger-color);
}

.result-content {
  flex: 1;
}

.result-title {
  margin: 0 0 4px 0;
  font-size: 16px;
  font-weight: 600;
  color: var(--text-primary);
}

.result-message {
  margin: 0;
  font-size: 14px;
  color: var(--text-secondary);
}
</style>
