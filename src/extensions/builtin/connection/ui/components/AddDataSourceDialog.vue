<template>
  <Teleport to="body">
    <Transition name="modal">
      <div v-if="modelValue" class="modal-overlay" @click.self="close">
        <div
          ref="dialogRef"
          class="modal-container"
          :style="{ width: dialogWidth + 'px', height: dialogHeight + 'px' }"
        >
          <div class="dialog-titlebar">
            <span class="title-label">{{ dialogTitle }}</span>
            <span v-if="formData.name" class="title-name">&mdash; {{ formData.name }}</span>
            <button class="close-btn" @click="close">&times;</button>
          </div>

          <div class="dialog-body">
            <DataSourceSidebar
              :drivers="allDrivers"
              :selected-db-type="selectedDbType"
              :selected-driver-id="selectedDriverId"
              @select-db-type="onSelectDbType"
              @select-staging="onSelectStaging"
            />

            <div class="dialog-main">
              <DataSourceHeader
                :name="formData.name ?? ''"
                :description="description"
                :uri="editUriMode ? editUriCustom : connectionUrl"
                :save-to-global="saveToGlobal"
                :save-to-project="saveToProject"
                :has-project="hasProject"
                :name-error="errors.name"
                :edit-uri-mode="editUriMode"
                :available-drivers="availableDriversForSelectedDb"
                :selected-driver-id="selectedDriverId"
                @update:name="name => { formData.name = name; clearError('name') }"
                @update:description="desc => { description = desc }"
                @update:uri="uri => { editUriCustom = uri }"
                @update:save-to-global="val => { saveToGlobal = val }"
                @update:save-to-project="val => { saveToProject = val }"
                @select-driver="onSelectDriver"
                @edit-uri="() => { editUriMode = !editUriMode; if (editUriMode) { editUriCustom = connectionUrl } }"
              />

              <div v-if="selectedDriver" class="tabs-nav">
                <button
                  v-for="tab in visibleTabs"
                  :key="tab.key"
                  class="tab-button"
                  :class="{ active: activeTab === tab.key }"
                  @click="activeTab = tab.key"
                >
                  <component :is="tab.icon" :size="14" />
                  <span>{{ tab.label }}</span>
                </button>
              </div>

              <div class="tabs-content">
                <template v-if="selectedDriver">
                  <GeneralTab
                    v-show="activeTab === 'general'"
                    :form-data="formData"
                    :form-sections="formSections"
                    :selected-driver="selectedDriver"
                    :has-project="hasProject"
                    :errors="errors"
                    @update:form-data="updateFormData"
                    @select-file="onSelectFile"
                    @create-file="onCreateFile"
                  />
                  <NetworkTab
                    v-show="activeTab === 'network'"
                    ref="networkTabRef"
                    :db-type="selectedDbType"
                    :scope="connectionScope"
                    @update:config="updateNetworkConfig"
                  />
                  <CapabilitiesTab
                    v-show="activeTab === 'capabilities'"
                    :selected-driver="selectedDriver"
                  />
                  <DriverPropsTab
                    v-show="activeTab === 'properties'"
                    :selected-driver="selectedDriver"
                    @update:values="updateDriverProps"
                  />
                  <AdvancedTab
                    v-show="activeTab === 'advanced'"
                    :db-type="selectedDbType"
                    @update:config="updateAdvancedConfig"
                  />
                </template>

                <div v-else class="no-driver-hint">
                  <Database :size="48" class="hint-icon" />
                  <h3>{{ t('navigator.selectDbTypeFirst') }}</h3>
                  <p>{{ t('navigator.selectDbTypeFirstHint') }}</p>
                </div>
              </div>
            </div>
          </div>

          <div class="dialog-footer">
            <div v-if="testResult" class="test-result" :class="testResult.success ? 'success' : 'error'">
              <template v-if="testResult.success">
                ✓ {{ t('navigator.connectionSuccess', { name: formData.name }) }}
                <span v-if="testResult.response_time_ms" class="test-time">
                  &mdash; {{ testResult.response_time_ms }}ms
                </span>
              </template>
              <template v-else>
                ✗ {{ testResult.message || t('navigator.connectionFailedGeneric') }}
              </template>
            </div>
            <div class="footer-actions">
              <button class="btn btn-test" :disabled="testing" @click="onTestConnection">
                <Wifi :size="14" />
                {{ t('navigator.testConnection') }}
              </button>
              <button class="btn btn-cancel" @click="close">
                {{ t('navigator.cancel') }}
              </button>
              <button class="btn btn-primary" @click="onSave">
                {{ isEditing ? t('navigator.updateDataSource') : t('navigator.saveDataSource') }}
              </button>
            </div>
          </div>

          <div
            class="resize-handle"
            @mousedown.prevent="onResizeStart"
          />
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<script setup lang="ts">
import { invoke } from '@tauri-apps/api/core'
import { Database, Wifi, Server, Shield, Settings, Zap } from 'lucide-vue-next'
import { ref, computed, watch, onUnmounted } from 'vue'
import { useI18n } from 'vue-i18n'

import DataSourceHeader from './DataSourceHeader.vue'
import DataSourceSidebar from './DataSourceSidebar.vue'
import AdvancedTab from './tabs/AdvancedTab.vue'
import CapabilitiesTab from './tabs/CapabilitiesTab.vue'
import DriverPropsTab from './tabs/DriverPropsTab.vue'
import GeneralTab from './tabs/GeneralTab.vue'
import NetworkTab from './tabs/NetworkTab.vue'
import {
  backendDriverToDescriptor,
  configSchemaToFormSchema,
  type BackendDriver,
} from '../adapters/driver-adapter'
import { useStagingList } from '../composables/useStagingList'

import type { DriverDescriptor, ConnectionConfig } from '../types/connection'
import type { FormSectionConfig } from '../types/form-schema'

const { t } = useI18n()

interface TestResult {
  success: boolean
  message?: string
  server_version?: string
  response_time_ms?: number
}

interface Props {
  modelValue: boolean
  isEditing?: boolean
  editData?: ConnectionConfig | null
  projectPath?: string | null
}

const props = withDefaults(defineProps<Props>(), {
  isEditing: false,
  editData: null,
  projectPath: null,
})

const emit = defineEmits<{
  (e: 'update:modelValue', value: boolean): void
  (e: 'save', data: Record<string, unknown>): void
}>()

const TAB_CONFIG = [
  { key: 'general', label: '常规', icon: Database },
  { key: 'network', label: '网络', icon: Server },
  { key: 'capabilities', label: '能力', icon: Shield },
  { key: 'properties', label: '驱动属性', icon: Settings },
  { key: 'advanced', label: '高级', icon: Zap },
] as const

type TabKey = typeof TAB_CONFIG[number]['key']

const dialogRef = ref<HTMLElement | null>(null)

const dialogWidth = ref(960)
const dialogHeight = ref(640)
const MIN_WIDTH = 680
const MIN_HEIGHT = 480

let resizing = false
let resizeStartX = 0
let resizeStartY = 0
let resizeStartW = 0
let resizeStartH = 0

function onResizeStart(e: MouseEvent) {
  resizing = true
  resizeStartX = e.clientX
  resizeStartY = e.clientY
  resizeStartW = dialogWidth.value
  resizeStartH = dialogHeight.value
  document.addEventListener('mousemove', onResizeMove)
  document.addEventListener('mouseup', onResizeEnd)
  document.body.style.userSelect = 'none'
}

function onResizeMove(e: MouseEvent) {
  if (!resizing) return
  const dx = e.clientX - resizeStartX
  const dy = e.clientY - resizeStartY
  dialogWidth.value = Math.max(MIN_WIDTH, resizeStartW + dx)
  dialogHeight.value = Math.max(MIN_HEIGHT, resizeStartH + dy)
}

function onResizeEnd() {
  resizing = false
  document.removeEventListener('mousemove', onResizeMove)
  document.removeEventListener('mouseup', onResizeEnd)
  document.body.style.userSelect = ''
}

onUnmounted(() => {
  if (resizing) {
    document.removeEventListener('mousemove', onResizeMove)
    document.removeEventListener('mouseup', onResizeEnd)
    document.body.style.userSelect = ''
  }
})

const allDrivers = ref<DriverDescriptor[]>([])
const selectedDbType = ref('')
const selectedDriverId = ref('')
const selectedDriver = ref<DriverDescriptor | null>(null)
const activeTab = ref<TabKey>('general')
const saveToGlobal = ref(true)
const saveToProject = ref(false)
const hasProject = ref(false)
const testing = ref(false)
const testResult = ref<TestResult | null>(null)

const formData = ref<Partial<ConnectionConfig>>({
  name: '',
  driver: '',
  host: '',
  port: undefined,
  database: '',
  username: '',
  password: '',
  options: {},
})
const description = ref('')
const formSections = ref<FormSectionConfig[]>([])
const editUriMode = ref(false)
const editUriCustom = ref('')
const networkConfig = ref<Record<string, unknown>>({})
const advancedConfig = ref<Record<string, unknown>>({})
const driverProps = ref<Record<string, string>>({})

const staging = useStagingList()
const stagingEntryId = ref<number | null>(null)

const networkTabRef = ref<InstanceType<typeof NetworkTab> | null>(null)

/** 统一 Scope 枚举 */
const connectionScope = computed<'global' | 'project'>(() => {
  if (saveToProject.value) return 'project'
  return 'global'
})

const errors = ref<Record<string, string>>({})

const visibleTabs = computed(() => TAB_CONFIG)

const dialogTitle = computed(() => {
  if (formData.value.name?.trim()) return formData.value.name
  return String(props.isEditing ? t('navigator.editDataSource') : t('navigator.addDataSource'))
})

const availableDriversForSelectedDb = computed<DriverDescriptor[]>(() => {
  if (!selectedDbType.value) return allDrivers.value
  return allDrivers.value.filter((d) => {
    const dbType = d.id.replace(/-.*$/, '')
    return dbType === selectedDbType.value
  })
})

const connectionUrl = computed(() => {
  if (!selectedDriver.value || !selectedDbType.value) return ''
  const d = selectedDriver.value
  const host = formData.value.host || 'localhost'
  const port = formData.value.port ?? d.defaultPort ?? d.default_port ?? ''
  const db = formData.value.database || 'mydb'
  const user = formData.value.username || 'root'
  const pwd = formData.value.password ? '****' : ''
  if (selectedDbType.value === 'sqlite' || selectedDbType.value === 'duckdb') {
    return `${selectedDbType.value}://${db || './database.db'}`
  }
  return `${selectedDbType.value}://${user}${pwd ? ':' + pwd : ''}@${host}${port ? ':' + port : ''}/${db}`
})

function buildRealUrl(): string {
  if (editUriMode.value && editUriCustom.value) {
    return editUriCustom.value
  }
  if (!selectedDriver.value || !selectedDbType.value) return ''
  const d = selectedDriver.value
  const host = formData.value.host || 'localhost'
  const port = formData.value.port ?? d.defaultPort ?? d.default_port ?? ''
  const db = formData.value.database || 'mydb'
  if (selectedDbType.value === 'sqlite' || selectedDbType.value === 'duckdb') {
    return `${selectedDbType.value}://${db || './database.db'}`
  }
  const user = formData.value.username || 'root'
  const pwd = formData.value.password || ''
  if (user && pwd) {
    return `${selectedDbType.value}://${user}:${pwd}@${host}${port ? ':' + port : ''}/${db}`
  }
  return `${selectedDbType.value}://${host}${port ? ':' + port : ''}/${db}`
}

async function loadDrivers() {
  try {
    const result = await invoke<{ drivers: BackendDriver[]; missing: unknown[] }>(
      'get_available_drivers',
      { projectPath: props.projectPath }
    )
    const backendDrivers = result?.drivers || []
    allDrivers.value = backendDrivers.map(backendDriverToDescriptor)
  } catch (err) {
    console.error('加载驱动列表失败，回退到旧路径:', err)
    try {
      const result = await invoke<DriverDescriptor[]>('get_drivers')
      allDrivers.value = result || []
    } catch {
      allDrivers.value = []
    }
  }
}

function onSelectDbType(dbType: string) {
  selectedDbType.value = dbType
  selectedDriverId.value = ''
  selectedDriver.value = null
  const drivers = availableDriversForSelectedDb.value
  if (drivers.length > 0) {
    onSelectDriver(drivers[0].id)
  }
}

async function onSelectDriver(driverId: string) {
  selectedDriverId.value = driverId
  const driver = allDrivers.value.find((d) => d.id === driverId)
  selectedDriver.value = driver || null
  if (driver) {
    formData.value.driver = driver.id
    const isFileDb = selectedDbType.value === 'sqlite' || selectedDbType.value === 'duckdb'
    formData.value.host = isFileDb ? '' : (formData.value.host || 'localhost')
    formData.value.port = driver.defaultPort ?? driver.default_port ?? undefined
    formData.value.database = formData.value.database || ''
    formData.value.username = isFileDb ? '' : (formData.value.username || 'root')
    if (!formData.value.name) {
      formData.value.name = `${driver.name}@localhost`
    }

    // 优先从 DB config_schema 加载（新路径）
    const schema = await loadDriverConfigSchema(driverId, driver)
    if (schema?.sections) {
      formSections.value = schema.sections
      const defaults = schema.sections.flatMap(s => s.fields.filter(f => f.default !== undefined))
      for (const field of defaults) {
        if (!(field.key in formData.value) && field.default !== undefined) {
          ;(formData.value as Record<string, unknown>)[field.key] = field.default
        }
      }
    } else {
      formSections.value = []
    }
  }
  activeTab.value = 'general'
}

/** 从 DB get_driver_detail 加载 config_schema（新路径），失败时回退到本地 JSON 文件 */
async function loadDriverConfigSchema(
  driverId: string,
  _driver: DriverDescriptor
): Promise<{ sections: FormSectionConfig[]; driverId: string; driverName: string } | null> {
  try {
    const detail = await invoke<{
      driver: BackendDriver
      availability: string
    }>('get_driver_detail', { driverId, projectPath: null })
    const formSchema = configSchemaToFormSchema(detail.driver)
    if (formSchema.sections.length > 0) {
      return formSchema
    }
  } catch (err) {
    console.warn('get_driver_detail 失败，回退到本地 JSON Schema:', err)
  }
  // 回退到旧路径（本地 JSON 文件）
  try {
    const { loadDriverSchema } = await import('../utils/schema-loader')
    const fallbackSchema = await loadDriverSchema(driverId)
    return fallbackSchema
  } catch {
    return null
  }
}

function updateFormData(data: Partial<ConnectionConfig>) {
  Object.assign(formData.value, data)
}

function onSelectFile() {
  import('@tauri-apps/plugin-dialog').then(({ open }) => {
    open({
      title: String(t('navigator.selectDatabaseFile')),
      filters: selectedDbType.value === 'duckdb'
        ? [{ name: 'DuckDB', extensions: ['duckdb', 'db'] }]
        : [{ name: 'SQLite', extensions: ['sqlite', 'db', 'sqlite3'] }],
      multiple: false,
    }).then((filePath) => {
      if (filePath && typeof filePath === 'string') {
        formData.value.database = filePath
        formData.value.host = ''
        formData.value.port = 0
        if (!formData.value.name) {
          formData.value.name = filePath.split(/[/\\]/).pop()?.replace(/\.[^.]+$/, '') || ''
        }
      }
    })
  })
}

function onCreateFile() {
  import('@tauri-apps/plugin-dialog').then(({ save }) => {
    save({
      title: String(t('navigator.createDatabaseFile')),
      defaultPath: selectedDbType.value === 'duckdb' ? 'database.duckdb' : 'database.db',
      filters: selectedDbType.value === 'duckdb'
        ? [{ name: 'DuckDB', extensions: ['duckdb', 'db'] }]
        : [{ name: 'SQLite', extensions: ['sqlite', 'db', 'sqlite3'] }],
    }).then(async (filePath) => {
      if (filePath && typeof filePath === 'string') {
        try {
          await invoke('create_database_file', {
            dbType: selectedDbType.value,
            filePath,
          })
          formData.value.database = filePath
          formData.value.host = ''
          formData.value.port = 0
          if (!formData.value.name) {
            formData.value.name = filePath.split(/[/\\]/).pop()?.replace(/\.[^.]+$/, '') || ''
          }
        } catch (err: unknown) {
          const msg = err instanceof Error ? err.message : String(err)
          errors.value.database = msg
        }
      }
    })
  })
}

function clearError(field: keyof typeof errors.value) {
  errors.value[field] = ''
}

function onSelectStaging(id: number) {
  const entry = staging.selectEntry(id)
  if (entry) {
    stagingEntryId.value = entry.id
    formData.value.name = entry.name
    selectedDbType.value = entry.dbType
    if (entry.driverId) {
      const drivers = availableDriversForSelectedDb.value
      if (drivers.length > 0) {
        onSelectDriver(entry.driverId)
      }
    }
  }
}

function updateNetworkConfig(config: Record<string, unknown>) {
  networkConfig.value = { ...networkConfig.value, ...config }
}

function updateAdvancedConfig(config: Record<string, unknown>) {
  advancedConfig.value = { ...advancedConfig.value, ...config }
}

function updateDriverProps(values: Record<string, string>) {
  driverProps.value = { ...driverProps.value, ...values }
}

interface BackendTestResult {
  success: boolean
  message: string
  server_version?: string
  response_time_ms?: number
}

async function onTestConnection() {
  if (!selectedDbType.value) return
  testing.value = true
  testResult.value = null
  try {
    const url = buildRealUrl()

    // 如果有协议链，先持久化以获取 networkConfigId
    let networkConfigId: string | null = null
    if (networkTabRef.value) {
      networkConfigId = await networkTabRef.value.saveNetworkChain(connectionScope.value)
    }

    const result = await invoke<BackendTestResult>('test_connection', {
      dbType: selectedDbType.value,
      url,
      ...(networkConfigId ? { network_config_id: networkConfigId } : {}),
    })
    testResult.value = {
      success: result.success,
      message: result.message,
      server_version: result.server_version,
      response_time_ms: result.response_time_ms,
    }
  } catch (err: unknown) {
    const msg = err instanceof Error ? err.message : String(t('navigator.connectionFailedGeneric'))
    testResult.value = { success: false, message: msg }
  } finally {
    testing.value = false
  }
}

async function onSave() {
  if (!formData.value.name?.trim()) {
    errors.value.name = String(t('navigator.validation.nameRequired'))
    return
  }
  if (!selectedDbType.value) {
    return
  }

  const isFileDb = selectedDbType.value === 'sqlite' || selectedDbType.value === 'duckdb'

  // 保存协议链到后端 DB，获取 networkConfigId
  let networkConfigId: string | null = null
  if (!isFileDb && networkTabRef.value) {
    networkConfigId = await networkTabRef.value.saveNetworkChain(connectionScope.value)
  }

  emit('save', {
    driver: selectedDbType.value,
    driverId: selectedDriverId.value,
    name: formData.value.name,
    host: isFileDb
      ? (formData.value.database || formData.value.host || '')
      : (formData.value.host || ''),
    port: isFileDb ? 0 : (formData.value.port ?? 0),
    database: formData.value.database || '',
    username: formData.value.username || '',
    password: formData.value.password || '',
    properties: formData.value.options || {},
    description: description.value,
    connectionType: connectionScope.value,
    connectionScope: connectionScope.value,
    saveToGlobal: saveToGlobal.value,
    saveToProject: saveToProject.value,
    url: buildRealUrl(),
    network: networkConfig.value,
    networkConfigId,
    advanced: advancedConfig.value,
    driverProps: driverProps.value,
  })
  close()
}

function close() {
  emit('update:modelValue', false)
  testResult.value = null
  errors.value = {}
}

watch(
  () => props.modelValue,
  async (val) => {
    if (val) {
      await loadDrivers()
      dialogWidth.value = 960
      dialogHeight.value = 640
      if (props.editData) {
        const editData = props.editData
        formData.value = { ...editData }
        selectedDriverId.value = editData.driver
        selectedDbType.value = editData.driver.replace(/-.*$/, '')
        const driver = allDrivers.value.find((d) => d.id === editData.driver)
        selectedDriver.value = driver || null
      } else {
        formData.value = {
          name: '', driver: '', host: '', port: undefined,
        database: '', username: '', password: '', options: {},
      }
      description.value = ''
        selectedDbType.value = 'mysql'
        const drivers = allDrivers.value.filter((d) => d.id.startsWith('mysql'))
        if (drivers.length > 0) {
          onSelectDriver(drivers[0].id)
        }
      }
    }
  }
)
</script>

<style scoped>
.modal-overlay {
  position: fixed;
  inset: 0;
  z-index: 9999;
  background: rgba(0, 0, 0, 0.55);
  display: flex;
  align-items: center;
  justify-content: center;
}

.modal-container {
  background: var(--color-bg-surface, #1a1b26);
  border-radius: 12px;
  box-shadow: 0 0 0 1px rgba(255,255,255,0.05), 0 24px 72px rgba(0,0,0,0.5);
  display: flex;
  flex-direction: column;
  overflow: hidden;
  position: relative;
  min-width: 680px;
  min-height: 480px;
}

.dialog-titlebar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 20px;
  border-bottom: 1px solid var(--color-border, rgba(255,255,255,0.07));
  font-size: 14px;
  font-weight: 600;
  color: var(--color-text-secondary, #a6adc8);
  flex-shrink: 0;
  user-select: none;
  gap: 6px;
}
.title-label {
  white-space: nowrap;
}
.title-name {
  font-size: 13px;
  font-weight: 400;
  color: var(--color-accent, #89b4fa);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.close-btn {
  background: none;
  border: none;
  font-size: 20px;
  color: var(--color-text-muted, #6c7086);
  cursor: pointer;
  padding: 0 4px;
  line-height: 1;
  transition: color 0.12s;
}
.close-btn:hover {
  color: var(--color-text-primary, #cdd6f4);
}

.dialog-body {
  display: flex;
  flex: 1;
  overflow: hidden;
  min-height: 0;
}

.dialog-main {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  min-width: 0;
}

.tabs-nav {
  display: flex;
  gap: 0;
  padding: 0 24px;
  border-bottom: 1px solid var(--color-border, rgba(255,255,255,0.07));
  background: #181825;
  flex-shrink: 0;
}
.tab-button {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 9px 16px;
  border: none;
  background: transparent;
  color: var(--color-text-muted, #6c7086);
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.15s;
  border-bottom: 2px solid transparent;
  position: relative;
  top: 1px;
}
.tab-button:hover {
  color: var(--color-text-primary, #cdd6f4);
}
.tab-button.active {
  color: var(--color-accent, #89b4fa);
  border-bottom-color: var(--color-accent, #89b4fa);
}

.tabs-content {
  flex: 1;
  overflow-y: auto;
  overflow-x: hidden;
  padding: 20px 24px;
  min-height: 0;
}

.no-driver-hint {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100%;
  gap: 12px;
  color: var(--color-text-muted, #6c7086);
}
.hint-icon {
  opacity: 0.3;
}
.no-driver-hint h3 {
  font-size: 15px;
  font-weight: 600;
  color: var(--color-text-secondary, #a6adc8);
}
.no-driver-hint p {
  font-size: 12px;
}

.dialog-footer {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px 24px;
  border-top: 1px solid var(--color-border, rgba(255,255,255,0.07));
  background: #181825;
  flex-shrink: 0;
}
.test-result {
  flex: 1;
  font-size: 12px;
  display: flex;
  align-items: center;
  gap: 6px;
}
.test-result.success {
  color: var(--color-success, #a6e3a1);
}
.test-result.error {
  color: var(--color-danger, #f38ba8);
}
.test-time {
  opacity: 0.5;
  font-size: 11px;
}
.footer-actions {
  display: flex;
  gap: 8px;
}

.btn {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  padding: 7px 16px;
  border-radius: 6px;
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.15s;
  border: none;
}
.btn-test {
  background: var(--color-bg-raised, #11111b);
  color: var(--color-text-secondary, #a6adc8);
  border: 1px solid var(--color-border, rgba(255,255,255,0.07));
}
.btn-test:hover {
  background: var(--color-bg-active, #2a2a3c);
  color: var(--color-text-primary, #cdd6f4);
}
.btn-test:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
.btn-cancel {
  background: transparent;
  color: var(--color-text-muted, #6c7086);
}
.btn-cancel:hover {
  background: var(--color-bg-hover, rgba(255,255,255,0.05));
  color: var(--color-text-primary, #cdd6f4);
}
.btn-primary {
  background: var(--color-accent, #89b4fa);
  color: #111;
  font-weight: 600;
}
.btn-primary:hover {
  filter: brightness(1.1);
}

.resize-handle {
  position: absolute;
  bottom: 0;
  right: 0;
  width: 20px;
  height: 20px;
  cursor: nwse-resize;
  background: linear-gradient(135deg,
    transparent 50%,
    rgba(255,255,255,0.08) 50%,
    rgba(255,255,255,0.08) 62%,
    transparent 62%,
    transparent 80%,
    rgba(255,255,255,0.08) 80%
  );
}

.modal-enter-active,
.modal-leave-active {
  transition: opacity 0.2s ease;
}
.modal-enter-active .modal-container,
.modal-leave-active .modal-container {
  transition: transform 0.2s ease, opacity 0.2s ease;
}
.modal-enter-from,
.modal-leave-to {
  opacity: 0;
}
.modal-enter-from .modal-container,
.modal-leave-to .modal-container {
  transform: scale(0.96);
  opacity: 0;
}
</style>