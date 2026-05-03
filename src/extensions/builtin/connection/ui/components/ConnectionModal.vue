<template>
  <Teleport to="body">
    <Transition name="modal">
      <div v-if="modelValue" class="modal-overlay" @click.self="close">
        <div class="modal-container">
          <!-- 头部 - 优化布局 -->
          <header class="modal-header">
            <div class="header-left">
              <h2 class="header-title">{{ isEditing ? '编辑连接' : '新建数据库连接' }}</h2>
              <DbIcon v-if="selectedDriver" :type="selectedDriver.id" class="header-db-icon" />
            </div>
            
            <!-- 连接名称输入框 -->
            <div v-if="selectedDriver" class="header-connection-name">
              <input
                v-model="formData.name"
                type="text"
                class="connection-name-input"
                placeholder="输入连接名称"
                :class="{ error: errors.name }"
              />
              <span v-if="errors.name" class="error-text">{{ errors.name }}</span>
            </div>
            
            <!-- 全局/项目多选 -->
            <div v-if="selectedDriver" class="header-scope-toggle">
              <label class="scope-checkbox">
                <input v-model="saveToGlobal" type="checkbox" />
                <span class="checkmark"></span>
                <span class="scope-label">全局</span>
              </label>
              <label class="scope-checkbox" :class="{ disabled: !hasProject }">
                <input v-model="saveToProject" type="checkbox" :disabled="!hasProject" />
                <span class="checkmark"></span>
                <span class="scope-label">项目</span>
              </label>
              <NTooltip v-if="!hasProject" trigger="hover">
                <template #trigger>
                  <HelpCircle :size="14" class="help-icon" />
                </template>
                当前未打开项目
              </NTooltip>
            </div>
            
            <button class="btn-close" @click="close">
              <X :size="18" />
            </button>
          </header>

          <!-- 内容区 - 左右分栏 -->
          <div class="modal-body">
            <!-- 左侧：数据库类型树 -->
            <ConnectionSidebar
              v-if="!selectedDriver || showDriverTree"
              :drivers="drivers"
              :selected-driver="selectedDriver"
              :recent-driver-ids="recentDriverIds"
              @select="onDriverSelect"
            />

            <!-- 右侧：配置区域 -->
            <div class="modal-content">
              <template v-if="selectedDriver">
                <!-- 标签页导航 -->
                <div class="tabs-nav">
                  <button
                    v-for="tab in visibleTabs"
                    :key="tab.key"
                    class="tab-button"
                    :class="{ active: activeTab === tab.key, disabled: tab.disabled }"
                    :disabled="tab.disabled"
                    @click="!tab.disabled && (activeTab = tab.key)"
                  >
                    <component :is="tab.icon" :size="14" />
                    <span>{{ tab.label }}</span>
                    <span v-if="tab.disabled" class="disabled-badge">不可用</span>
                  </button>
                </div>

                <!-- 标签页内容 -->
                <div class="tabs-content">
                  <!-- 常规标签页 -->
                  <GeneralTab
                    v-show="activeTab === 'general'"
                    :form-data="formData"
                    :selected-driver="selectedDriver"
                    :has-project="hasProject"
                    :errors="errors"
                    :form-sections="dynamicFormSections"
                    @update:form-data="updateFormData"
                    @select-file="selectDatabaseFile"
                    @create-file="createNewDatabaseFile"
                  />

                  <!-- DuckDB 本地加速标签页 -->
                  <DuckdbAccelerationTab
                    v-show="activeTab === 'duckdb'"
                    :form-data="formData"
                    @update:form-data="updateFormData"
                  />

                  <!-- 驱动选项标签页 -->
                  <div v-show="activeTab === 'driver'" class="driver-tab-content">
                    <div class="driver-info-section">
                      <h4 class="section-title">驱动信息</h4>
                      <div class="driver-details">
                        <div class="detail-item">
                          <span class="detail-label">驱动名称</span>
                          <span class="detail-value">{{ selectedDriver.name }}</span>
                        </div>
                        <div class="detail-item">
                          <span class="detail-label">版本</span>
                          <span class="detail-value">{{ selectedDriver.version || '最新' }}</span>
                        </div>
                        <div class="detail-item">
                          <span class="detail-label">特性</span>
                          <div class="feature-tags">
                            <span
                              v-for="feature in selectedDriver.features"
                              :key="feature"
                              class="feature-tag"
                            >
                              {{ feature }}
                            </span>
                          </div>
                        </div>
                      </div>
                    </div>

                    <!-- 驱动自定义选项 -->
                    <div v-if="selectedDriver.extraOptions?.length" class="driver-options-section">
                      <h4 class="section-title">驱动选项</h4>
                      <div class="options-list">
                        <div
                          v-for="option in selectedDriver.extraOptions"
                          :key="option.name"
                          class="option-item"
                        >
                          <label class="option-label">{{ option.label }}</label>
                          <input
                            v-if="option.type === 'string' || option.type === 'number'"
                            v-model="(formData.options as Record<string, unknown>)[option.name]"
                            :type="option.type"
                            class="form-input"
                            :placeholder="option.description"
                          />
                          <select
                            v-else-if="option.type === 'select'"
                            v-model="(formData.options as Record<string, unknown>)[option.name]"
                            class="form-select"
                          >
                            <option
                              v-for="opt in option.options"
                              :key="opt.value"
                              :value="opt.value"
                            >
                              {{ opt.label }}
                            </option>
                          </select>
                        </div>
                      </div>
                    </div>
                  </div>
                </div>
              </template>

              <!-- 未选择驱动时的提示 -->
              <div v-else class="empty-state">
                <Database :size="48" class="empty-icon" />
                <h3 class="empty-title">选择数据库类型</h3>
                <p class="empty-desc">从左侧列表中选择您要连接的数据库类型</p>
              </div>
            </div>
          </div>

          <!-- 底部按钮 -->
          <footer class="modal-footer">
            <div class="footer-left">
              <button
                v-if="selectedDriver"
                type="button"
                class="btn-secondary"
                @click="showDriverTree = !showDriverTree"
              >
                <Grid :size="14" />
                {{ showDriverTree ? '隐藏' : '显示' }}数据库列表
              </button>
            </div>
            <div class="footer-right">
              <!-- 测试结果展示 -->
              <div v-if="testResult" class="test-result" :class="testResult.success ? 'success' : 'error'">
                <CheckCircle v-if="testResult.success" :size="14" />
                <XCircle v-else :size="14" />
                <span class="test-result-message">{{ testResult.message }}</span>
                <span v-if="testResult.response_time_ms" class="test-result-time">{{ testResult.response_time_ms }}ms</span>
              </div>
              
              <button type="button" class="btn-secondary" @click="close">
                取消
              </button>
              <button
                v-if="selectedDriver"
                type="button"
                class="btn-test"
                :disabled="testing"
                @click="testConnection"
              >
                <Loader2 v-if="testing" :size="14" class="animate-spin" />
                <TestTube v-else :size="14" />
                {{ testing ? '测试中...' : '测试连接' }}
              </button>
              <button
                type="button"
                class="btn-primary"
                :disabled="saving"
                @click="saveConnection"
              >
                <Check :size="14" />
                {{ saving ? '保存中...' : (isEditing ? '保存' : '保存连接') }}
              </button>
            </div>
          </footer>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<script setup lang="ts">
import { invoke } from '@tauri-apps/api/core'
import { open, save } from '@tauri-apps/plugin-dialog'
import {
  X,
  Database,
  Settings,
  Grid,
  TestTube,
  Check,
  HelpCircle,
  Zap,
  Loader2,
  CheckCircle,
  XCircle
} from 'lucide-vue-next'
import { NTooltip, useMessage } from 'naive-ui'
import { ref, computed, watch, onMounted } from 'vue'

import { useProjectStore } from '@/core/project/stores/project'
import DbIcon from '@/shared/components/common/DbIcon.vue'

import ConnectionSidebar from './ConnectionSidebar.vue'
import * as connectionService from '../services/connection'
import DuckdbAccelerationTab from './tabs/DuckdbAccelerationTab.vue'
import GeneralTab from './tabs/GeneralTab.vue'
import { loadDriverSchema, generateDefaultFormData } from '../utils/schema-loader'

import type { DriverDescriptor, ConnectionConfig } from '../types/connection'
import type { DriverFormSchema, FormSectionConfig } from '../types/form-schema'

interface Props {
  modelValue: boolean
  editingConnection?: ConnectionConfig | null
}

const props = withDefaults(defineProps<Props>(), {
  editingConnection: null
})

const emit = defineEmits<{
  'update:modelValue': [value: boolean]
  save: [connection: Record<string, unknown>]
}>()

const projectStore = useProjectStore()
const message = useMessage()

const isEditing = computed(() => !!props.editingConnection)
const hasProject = computed(() => projectStore.hasProject)

// 状态
const selectedDriver = ref<DriverDescriptor | null>(null)
const showDriverTree = ref(true)
const activeTab = ref('general')
const testing = ref(false)
const saving = ref(false)
const errors = ref<Record<string, string>>({})

// 动态表单相关
const currentSchema = ref<DriverFormSchema | null>(null)
const dynamicFormData = ref<Record<string, unknown>>({})
const dynamicFormSections = ref<FormSectionConfig[]>([])

// 最近使用的驱动
const recentDriverIds = ref<string[]>([])

// 表单数据类型定义
interface ConnectionFormData {
  name: string
  driver: string
  host: string
  port?: number
  database: string
  username?: string
  password?: string
  connectionType: 'global' | 'project'
  useDuckdbFed: boolean
  options: Record<string, unknown>
  enableSsh: boolean
  sshHost: string
  sshPort: number
  sshUsername: string
  sshPassword: string
  enableSsl: boolean
  sslCa: string
  sslCert: string
  sslKey: string
}

// 测试结果类型
interface TestResult {
  success: boolean
  message: string
  server_version?: string
  response_time_ms?: number
}

// 驱动默认值配置
const DRIVER_DEFAULTS: Record<string, Partial<ConnectionFormData>> = {
  mysql: { host: 'localhost', port: 3306, username: 'root' },
  postgres: { host: 'localhost', port: 5432, username: 'postgres' },
  sqlite: { database: './database.db' },
  duckdb: { database: './database.duckdb' }
}

// 表单数据
const formData = ref<ConnectionFormData>({
  name: '',
  driver: '',
  host: '',
  port: undefined,
  database: '',
  username: '',
  password: '',
  connectionType: 'global',
  useDuckdbFed: false,
  options: {},
  enableSsh: false,
  sshHost: '',
  sshPort: 22,
  sshUsername: '',
  sshPassword: '',
  enableSsl: false,
  sslCa: '',
  sslCert: '',
  sslKey: ''
})

// 驱动列表缓存
const driversCache = ref<DriverDescriptor[] | null>(null)
const drivers = ref<DriverDescriptor[]>([])

// 加载驱动列表（带缓存）
async function loadDrivers(force = false) {
  if (!force && driversCache.value) {
    drivers.value = driversCache.value
    return
  }
  
  try {
    drivers.value = await invoke<DriverDescriptor[]>('get_drivers')
    driversCache.value = drivers.value
  } catch (e) {
    console.error('加载驱动列表失败:', e)
  }
}

// 连接 URL 计算属性
const connectionUrl = computed(() => {
  const driver = formData.value.driver
  if (!driver) return ''
  
  if (requiresFile.value) {
    return `${driver}://${formData.value.database}`
  }
  return `${driver}://${formData.value.host}:${formData.value.port}/${formData.value.database}`
})

// 自动生成的连接名称
const autoGenerateName = computed(() => {
  const driver = formData.value.driver
  if (!driver) return ''
  
  if (requiresFile.value) {
    const path = formData.value.database
    if (!path) return driver
    const parts = path.split(/[\\/]/)
    const fileName = parts[parts.length - 1]
    return fileName.split('.')[0] || driver
  }
  
  const host = formData.value.host || 'localhost'
  const database = formData.value.database
  return `${driver}@${host}${database ? '/' + database : ''}`
})

// 跟踪是否手动修改过名称
const nameManuallyEdited = ref(false)
let previousAutoName = ''

// 测试结果状态
const testResult = ref<TestResult | null>(null)

// 全局/项目多选
const saveToGlobal = ref(true)
const saveToProject = ref(false)

// 可见的标签页
const visibleTabs = computed(() => {
  if (!selectedDriver.value) return []

  const tabs: Array<{ key: string; label: string; icon: typeof Settings; disabled?: boolean }> = [
    { key: 'general', label: '常规', icon: Settings }
  ]

  // DuckDB 加速标签页：DuckDB 本身不需要加速，其他数据库都支持
  const isDuckdbDriver = selectedDriver.value.id === 'duckdb'
  tabs.push({ key: 'duckdb', label: '本地加速', icon: Zap, disabled: isDuckdbDriver })

  tabs.push({ key: 'driver', label: '驱动', icon: Database })

  return tabs
})

const requiresFile = computed(() => {
  return selectedDriver.value?.requireFile === true || selectedDriver.value?.require_file === true
})

// 监听编辑连接
watch(
  () => props.editingConnection,
  (conn) => {
    if (conn) {
      formData.value = {
        name: conn.name || '',
        driver: conn.driver || '',
        host: conn.host || '',
        port: conn.port,
        database: conn.database || '',
        username: conn.username || '',
        password: conn.password || '',
        connectionType: 'global',
        useDuckdbFed: false,
        options: {},
        enableSsh: false,
        sshHost: '',
        sshPort: 22,
        sshUsername: '',
        sshPassword: '',
        enableSsl: false,
        sslCa: '',
        sslCert: '',
        sslKey: ''
      }
      selectedDriver.value = drivers.value.find((d: DriverDescriptor) => d.id === conn.driver) || null
      showDriverTree.value = false
    }
  },
  { immediate: true }
)

// 加载最近使用的驱动和驱动列表
onMounted(async () => {
  try {
    const recent = localStorage.getItem('recent-drivers')
    if (recent) {
      recentDriverIds.value = JSON.parse(recent)
    }
  } catch (e) {
    console.error('加载最近使用的驱动失败:', e)
  }
  
  // 加载驱动列表
  await loadDrivers()
})

async function onDriverSelect(driver: DriverDescriptor) {
  console.log('=== onDriverSelect 被调用 ===')
  console.log('选择的驱动:', driver.id)
  
  selectedDriver.value = driver
  showDriverTree.value = true
  formData.value.driver = driver.id
  
  console.log('设置后 formData.value.driver:', formData.value.driver)
  console.log('设置后 selectedDriver.value:', selectedDriver.value?.id)
  formData.value.port = driver.defaultPort
  
  // 应用智能默认值
  const defaults = DRIVER_DEFAULTS[driver.id]
  if (defaults) {
    if (defaults.host && !formData.value.host) {
      formData.value.host = defaults.host
    }
    if (defaults.port && !formData.value.port) {
      formData.value.port = defaults.port
    }
    if (defaults.username && !formData.value.username) {
      formData.value.username = defaults.username
    }
    if (defaults.database && !formData.value.database) {
      formData.value.database = defaults.database
    }
  }
  
  // 重置名称编辑状态
  nameManuallyEdited.value = false
  formData.value.name = autoGenerateName.value
  previousAutoName = autoGenerateName.value
  
  activeTab.value = 'general'

  // 加载动态表单配置
  await loadDynamicForm(driver.id)

  // 更新最近使用
  updateRecentDrivers(driver.id)
}

// 监听表单数据变化，自动生成名称
watch(
  [() => formData.value.driver, () => formData.value.host, () => formData.value.database],
  () => {
    if (!nameManuallyEdited.value) {
      const newName = autoGenerateName.value
      formData.value.name = newName
      previousAutoName = newName
    }
  },
  { deep: true }
)

// 监听名称手动修改
watch(
  () => formData.value.name,
  (newName) => {
    if (newName && newName !== previousAutoName && !nameManuallyEdited.value) {
      nameManuallyEdited.value = true
    }
  }
)

async function loadDynamicForm(driverId: string) {
  const schema = await loadDriverSchema(driverId)
  
  if (schema) {
    currentSchema.value = schema
    dynamicFormSections.value = schema.sections
    dynamicFormData.value = generateDefaultFormData(schema)
    
    // 合并到 formData
    formData.value = {
      ...formData.value,
      ...dynamicFormData.value,
      port: schema.metadata?.defaultPort || formData.value.port
    }
  } else {
    // 如果没有找到 schema，使用默认配置
    currentSchema.value = null
    dynamicFormSections.value = []
    dynamicFormData.value = {}
  }
}

function updateRecentDrivers(driverId: string) {
  const recent = recentDriverIds.value.filter(id => id !== driverId)
  recent.unshift(driverId)
  recentDriverIds.value = recent.slice(0, 5)

  try {
    localStorage.setItem('recent-drivers', JSON.stringify(recentDriverIds.value))
  } catch (e) {
    console.error('保存最近使用的驱动失败:', e)
  }
}

function updateFormData(data: Partial<ConnectionConfig>) {
  formData.value = {
    ...formData.value,
    ...data
  }
}

async function testConnection() {
  if (!validateForm()) {
    message.warning('请填写必填字段')
    return
  }

  testing.value = true
  testResult.value = null
  
  try {
    const driver = formData.value.driver
    if (!driver) return

    const url = connectionUrl.value
    const result = await connectionService.testConnection(driver, url)
    
    testResult.value = {
      success: result.success,
      message: result.message,
      server_version: result.server_version,
      response_time_ms: result.response_time_ms
    }
    
    if (result.success) {
      message.success(`连接成功！服务器版本: ${result.server_version}，响应时间: ${result.response_time_ms}ms`)
    } else {
      message.error(`连接失败：${result.message}`)
    }
  } catch (error) {
    const errorMsg = error instanceof Error ? error.message : String(error)
    testResult.value = {
      success: false,
      message: errorMsg
    }
    message.error(`连接测试失败: ${errorMsg}`)
  } finally {
    testing.value = false
  }
}

async function saveConnection() {
  console.log('=== saveConnection 开始 ===')
  console.log('formData.value.driver:', formData.value.driver)
  console.log('selectedDriver.value:', selectedDriver.value)
  console.log('selectedDriver.value?.id:', selectedDriver.value?.id)
  
  if (!validateForm()) {
    message.warning('请填写必填字段')
    return
  }

  saving.value = true
  try {
    // 优先使用 formData.value.driver，如果为空则从 selectedDriver 获取
    const driver = formData.value.driver || selectedDriver.value?.id || ''
    
    console.log('最终获取到的 driver:', driver)
    
    if (!driver) {
      message.error('请选择数据库类型')
      console.error('driver 为空，formData.driver:', formData.value.driver, 'selectedDriver:', selectedDriver.value?.id)
      saving.value = false
      return
    }

    // 构建连接 URL
    const url = connectionUrl.value

    // 发出连接配置数据，由父组件处理实际的连接和保存逻辑
    const connectionConfig = {
      id: '', // 由父组件在连接成功后填充
      name: formData.value.name || '',
      db_type: driver,
      host: requiresFile.value ? null : formData.value.host,
      port: requiresFile.value ? null : formData.value.port,
      database: formData.value.database,
      username: formData.value.username || null,
      password: formData.value.password || undefined,
      url: url,
      properties: {
        useDuckdbFed: formData.value.useDuckdbFed,
        enableSsh: formData.value.enableSsh,
        enableSsl: formData.value.enableSsl,
        sshConfig: formData.value.enableSsh ? {
          host: formData.value.sshHost,
          port: formData.value.sshPort,
          username: formData.value.sshUsername,
          password: formData.value.sshPassword
        } : null,
        sslConfig: formData.value.enableSsl ? {
          caCert: formData.value.sslCa,
          clientCert: formData.value.sslCert,
          clientKey: formData.value.sslKey
        } : null,
        ...formData.value.options
      },
      // 传递多选标志
      saveToGlobal: saveToGlobal.value,
      saveToProject: saveToProject.value && projectStore.hasProject,
      useDuckdbFed: formData.value.useDuckdbFed
    }

    console.log('ConnectionModal emit save:', connectionConfig)
    emit('save', connectionConfig)
  } catch (error) {
    const errorMsg = error instanceof Error ? error.message : String(error)
    message.error(`保存连接失败: ${errorMsg}`)
    console.error('保存连接失败详情:', error)
  } finally {
    saving.value = false
  }
}

function validateForm(): boolean {
  errors.value = {}

  if (!formData.value.name?.trim()) {
    errors.value.name = '请输入连接名称'
  }

  if (!requiresFile.value) {
    if (!formData.value.host?.trim()) {
      errors.value.host = '请输入主机地址'
    }
    if (!formData.value.database?.trim()) {
      errors.value.database = '请输入数据库名称'
    }
  } else {
    if (!formData.value.database?.trim()) {
      errors.value.database = '请选择或输入数据库文件路径'
    }
  }

  return Object.keys(errors.value).length === 0
}

async function selectDatabaseFile() {
  const selected = await open({
    multiple: false,
    filters: [
      {
        name: 'Database Files',
        extensions: selectedDriver.value?.id === 'sqlite' ? ['db', 'sqlite', 'sqlite3'] : ['duckdb', 'db']
      }
    ]
  })
  if (selected) {
    formData.value.database = selected as string
  }
}

async function createNewDatabaseFile() {
  const ext = selectedDriver.value?.id === 'sqlite' ? 'db' : 'duckdb'
  const defaultName = formData.value.name || 'new_database'
  
  const selected = await save({
    filters: [
      {
        name: 'Database Files',
        extensions: [ext]
      }
    ],
    defaultPath: `${defaultName}.${ext}`,
    title: '保存数据库文件'
  })
  
  if (selected) {
    try {
      // 调用后端创建文件
      const driverId = selectedDriver.value?.id
      if (!driverId) return
      
      const result = await connectionService.createDatabaseFile(
        driverId,
        selected
      )
      if (result.success) {
        formData.value.database = selected
      } else {
        errors.value.database = result.message
      }
    } catch {
      errors.value.database = '创建数据库文件失败'
    }
  }
}

function close() {
  emit('update:modelValue', false)
  resetForm()
}

function resetForm() {
  selectedDriver.value = null
  showDriverTree.value = true
  activeTab.value = 'general'
  nameManuallyEdited.value = false
  testResult.value = null
  formData.value = {
    name: '',
    driver: '',
    host: '',
    port: undefined,
    database: '',
    username: '',
    password: '',
    connectionType: 'global',
    useDuckdbFed: false,
    options: {},
    enableSsh: false,
    sshHost: '',
    sshPort: 22,
    sshUsername: '',
    sshPassword: '',
    enableSsl: false,
    sslCa: '',
    sslCert: '',
    sslKey: ''
  }
  errors.value = {}
}
</script>

<style scoped>
.modal-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.6);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
  backdrop-filter: blur(4px);
}

.modal-container {
  width: 90vw;
  max-width: 1200px;
  height: 80vh;
  max-height: 800px;
  background: var(--bg-primary);
  border-radius: var(--radius-lg);
  box-shadow: 0 20px 60px rgba(0, 0, 0, 0.3);
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

/* 头部 - 优化布局 */
.modal-header {
  display: flex;
  align-items: center;
  gap: 16px;
  padding: 16px 24px;
  border-bottom: 1px solid var(--border-color);
  background: var(--bg-secondary);
}

.header-left {
  display: flex;
  align-items: center;
  gap: 12px;
  flex-shrink: 0;
}

.header-title {
  font-size: 18px;
  font-weight: 600;
  color: var(--text-primary);
  margin: 0;
  white-space: nowrap;
}

.header-db-icon {
  width: 20px;
  height: 20px;
  flex-shrink: 0;
}

/* 连接名称输入框 */
.header-connection-name {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 2px;
  min-width: 0;
}

.connection-name-input {
  height: 36px;
  padding: 0 12px;
  background: var(--bg-tertiary);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-md);
  color: var(--text-primary);
  font-size: 14px;
  font-weight: 500;
  outline: none;
  transition: all 0.2s;
  width: 100%;
}

.connection-name-input:focus {
  border-color: var(--primary-color);
  background: var(--bg-primary);
}

.connection-name-input.error {
  border-color: var(--danger-color);
}

.error-text {
  font-size: 11px;
  color: var(--danger-color);
}

/* 全局/项目切换 */
.header-scope-toggle {
  display: flex;
  align-items: center;
  gap: 12px;
  flex-shrink: 0;
}

.scope-checkbox {
  display: flex;
  align-items: center;
  gap: 6px;
  cursor: pointer;
  user-select: none;
}

.scope-checkbox.disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.scope-checkbox input[type="checkbox"] {
  display: none;
}

.scope-checkbox .checkmark {
  width: 16px;
  height: 16px;
  border: 2px solid var(--border-color);
  border-radius: 3px;
  position: relative;
  transition: all 0.2s;
}

.scope-checkbox input[type="checkbox"]:checked + .checkmark {
  background: var(--primary-color);
  border-color: var(--primary-color);
}

.scope-checkbox input[type="checkbox"]:checked + .checkmark::after {
  content: '';
  position: absolute;
  left: 4px;
  top: 1px;
  width: 4px;
  height: 8px;
  border: solid white;
  border-width: 0 2px 2px 0;
  transform: rotate(45deg);
}

.scope-label {
  font-size: 13px;
  color: var(--text-secondary);
  font-weight: 500;
}

.help-icon {
  color: var(--text-tertiary);
  cursor: help;
}

.btn-close {
  width: 32px;
  height: 32px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: transparent;
  border: none;
  border-radius: var(--radius-md);
  color: var(--text-secondary);
  cursor: pointer;
  transition: all 0.2s;
  flex-shrink: 0;
}

.btn-close:hover {
  background: var(--bg-hover);
  color: var(--text-primary);
}

/* 内容区 */
.modal-body {
  flex: 1;
  display: flex;
  overflow: hidden;
}

.modal-content {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

/* 标签页导航 */
.tabs-nav {
  display: flex;
  gap: 4px;
  padding: 12px 24px;
  border-bottom: 1px solid var(--border-color);
  background: var(--bg-secondary);
}

.tab-button {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 8px 16px;
  background: transparent;
  border: none;
  border-radius: var(--radius-md);
  color: var(--text-secondary);
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s;
}

.tab-button:hover {
  background: var(--bg-hover);
  color: var(--text-primary);
}

.tab-button.active {
  background: var(--primary-light);
  color: var(--primary-color);
}

.tab-button.disabled {
  opacity: 0.5;
  cursor: not-allowed;
  pointer-events: none;
}

.tab-button.disabled:hover {
  background: transparent;
  color: var(--text-secondary);
}

.disabled-badge {
  font-size: 10px;
  padding: 1px 4px;
  background: var(--bg-tertiary);
  border-radius: 2px;
  color: var(--text-tertiary);
  margin-left: 4px;
}

/* 标签页内容 */
.tabs-content {
  flex: 1;
  overflow-y: auto;
  padding: 24px;
}

.driver-tab-content {
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.driver-info-section,
.driver-options-section {
  padding: 16px;
  background: var(--bg-secondary);
  border-radius: var(--radius-md);
  border: 1px solid var(--border-color);
}

.section-title {
  font-size: 14px;
  font-weight: 600;
  color: var(--text-primary);
  margin: 0 0 12px 0;
}

.driver-details {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.detail-item {
  display: flex;
  align-items: flex-start;
  gap: 12px;
}

.detail-label {
  min-width: 80px;
  font-size: 13px;
  color: var(--text-secondary);
}

.detail-value {
  flex: 1;
  font-size: 13px;
  color: var(--text-primary);
}

.feature-tags {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}

.feature-tag {
  padding: 4px 8px;
  background: var(--bg-tertiary);
  border-radius: var(--radius-sm);
  font-size: 12px;
  color: var(--text-secondary);
}

.options-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.option-item {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.option-label {
  font-size: 13px;
  font-weight: 500;
  color: var(--text-primary);
}

.form-input,
.form-select {
  height: 32px;
  padding: 0 12px;
  background: var(--bg-tertiary);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-md);
  color: var(--text-primary);
  font-size: 13px;
  outline: none;
  transition: all 0.2s;
}

.form-input:focus,
.form-select:focus {
  border-color: var(--primary-color);
  background: var(--bg-primary);
}

.form-select {
  cursor: pointer;
}

/* 空状态 */
.empty-state {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 12px;
  color: var(--text-tertiary);
}

.empty-icon {
  opacity: 0.3;
}

.empty-title {
  font-size: 18px;
  font-weight: 600;
  color: var(--text-secondary);
  margin: 0;
}

.empty-desc {
  font-size: 14px;
  margin: 0;
}

/* 底部 */
.modal-footer {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px 24px;
  border-top: 1px solid var(--border-color);
  background: var(--bg-secondary);
}

.footer-left,
.footer-right {
  display: flex;
  gap: 8px;
  align-items: center;
}

.test-result {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 6px 12px;
  border-radius: var(--radius-md);
  font-size: 12px;
  font-weight: 500;
  margin-right: 8px;
  transition: all 0.2s;
}

.test-result.success {
  background: rgba(34, 197, 94, 0.1);
  color: #22c55e;
  border: 1px solid rgba(34, 197, 94, 0.3);
}

.test-result.error {
  background: rgba(239, 68, 68, 0.1);
  color: #ef4444;
  border: 1px solid rgba(239, 68, 68, 0.3);
}

.test-result-message {
  max-width: 200px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.test-result-time {
  opacity: 0.7;
  font-size: 11px;
}

.btn-secondary,
.btn-primary,
.btn-test {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 8px 16px;
  border-radius: var(--radius-md);
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s;
  border: none;
}

.btn-secondary {
  background: var(--bg-tertiary);
  color: var(--text-primary);
  border: 1px solid var(--border-color);
}

.btn-secondary:hover {
  background: var(--bg-hover);
}

.btn-test {
  background: var(--bg-tertiary);
  color: var(--warning-color);
  border: 1px solid var(--warning-color);
}

.btn-test:hover:not(:disabled) {
  background: var(--warning-color);
  color: white;
}

.btn-test:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.btn-primary {
  background: var(--primary-color);
  color: white;
}

.btn-primary:hover:not(:disabled) {
  background: var(--primary-dark);
}

.btn-primary:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

/* 过渡动画 */
.modal-enter-active,
.modal-leave-active {
  transition: opacity 0.3s ease;
}

.modal-enter-from,
.modal-leave-to {
  opacity: 0;
}
</style>
