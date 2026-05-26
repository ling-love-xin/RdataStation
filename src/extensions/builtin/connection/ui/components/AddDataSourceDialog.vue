<template>
  <NModal :show="modelValue" :mask-closable="false" @update:show="handleClose">
    <div class="datasource-card">
      <!-- Card header -->
      <div class="card-header">
        <Database :size="16" class="card-icon" />
        <span class="card-title">{{ isEditing ? $t('navigator.editDataSource') : $t('navigator.addDataSource') }}</span>
      </div>

      <div class="card-body">
        <!-- Sidebar -->
        <AddDataSourceSidebar
          v-model:selected-type-id="selectedTypeId"
          :staging-items="stagingItems"
          :staging-index="stagingIndex"
          @add-staging="handleAddStaging"
          @remove-staging="handleRemoveStaging"
          @select-staging="handleSelectStaging"
        />

        <!-- Right panel -->
        <div class="dlg-right">
          <DataSourceHeader
            v-model:name="headerData.name"
            v-model:description="headerData.description"
            v-model:scope-global="scope.global"
            v-model:scope-project="scope.project"
            v-model:selected-driver-id="headerData.selectedDriverId"
            v-model:uri-editing="uriEditing"
            v-model:manual-uri="manualUri"
            :driver-options="driverOptions"
            :uri-preview="uriPreview"
            :name-label="'* ' + $t('navigator.name')"
            name-required
            :name-placeholder="$t('navigator.dataSourceNamePlaceholder')"
            :desc-label="$t('navigator.description')"
            :desc-placeholder="$t('navigator.dataSourceDescPlaceholder')"
            :global-label="$t('navigator.globalConnection')"
            :project-label="$t('navigator.projectConnection')"
            :driver-label="'* ' + $t('navigator.driver')"
            :driver-placeholder="$t('navigator.selectDbType')"
            uri-label="URI"
            uri-placeholder="jdbc:mysql://..."
            @driver-change="onDriverChange"
          />

          <NAlert
            v-if="scopeChangedWarning"
            type="warning"
            class="scope-warning"
            closable
            @close="scopeChangedWarning = false"
          >
            {{ $t('navigator.scopeChangeWarning') }}
          </NAlert>

          <!-- Tabs -->
          <NTabs v-model:value="activeTab" type="line" size="small" class="dlg-tabs">
            <NTabPane name="general" :tab="$t('navigator.tabGeneral')">
              <GeneralTab :driver="selectedDriver" :form-data="formData" :scope="scope" :project-path="projectStore.currentProject?.path" @update:form-data="onFormData" @auth-config-change="onAuthConfigChange" />
            </NTabPane>
            <NTabPane name="network" :tab="$t('navigator.tabNetwork')">
              <NetworkTab :driver="selectedDriver" :scope="scope" @extra-config="onExtraConfig" />
            </NTabPane>
            <NTabPane name="capabilities" :tab="$t('navigator.tabCapabilities')">
              <CapabilitiesTab :driver="selectedDriver" />
            </NTabPane>
            <NTabPane name="properties" :tab="$t('navigator.tabDriverProps')">
              <DriverPropsTab :driver="selectedDriver" @extra-config="onExtraConfig" />
            </NTabPane>
            <NTabPane name="advanced" :tab="$t('navigator.tabAdvanced')">
              <AdvancedTab :driver="selectedDriver" :form-data="formData" :scope="scope" @update:form-data="onFormData" @extra-config="onExtraConfig" />
            </NTabPane>
          </NTabs>

          <!-- Footer -->
          <div class="card-footer">
            <div v-if="testResult" class="test-result">
              <span :class="['test-icon', testResult.success ? 'ok' : 'fail']">
                {{ testResult.success ? '✓' : '✗' }}
              </span>
              <span class="test-msg">{{ testResult.message }}</span>
              <span v-if="testResult.latencyMs != null" class="test-latency">· {{ testResult.latencyMs }}ms</span>
            </div>
            <div class="footer-spacer" />
            <NButton @click="handleClose">{{ $t('navigator.cancel') }}</NButton>
            <NButton :loading="testing" @click="handleTest">{{ $t('navigator.testConnection') }}</NButton>
            <NButton type="primary" :loading="saving" @click="handleSave">{{ $t('navigator.save') }}</NButton>
            <NButton type="primary" secondary :loading="saving" @click="handleApply">
              {{ $t('navigator.apply') }}
            </NButton>
          </div>
        </div>
      </div>
    </div>
  </NModal>
  <!-- Test connection feedback modal -->
  <TestResultModal
    :show="showTestModal"
    :result="lastTestResult"
    :host="String(formData.host || '')"
    :port="String(formData.port || '')"
    :database="String(formData.database || '')"
    :user="String(formData.username || '')"
    :url="uriPreview"
    :driver-name="selectedDriver?.name ?? ''"
    :network-info="testNetworkInfo"
    @close="onTestModalClose"
  />
</template>

<script setup lang="ts">
import { Database } from 'lucide-vue-next'
import { NButton, NModal, NTabs, NTabPane, NAlert, useMessage, useDialog } from 'naive-ui'
import { ref, computed, watch, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'

import { useProjectStore } from '@/core/project/stores/project'

import AddDataSourceSidebar from './AddDataSourceSidebar.vue'
import DataSourceHeader from './DataSourceHeader.vue'
import AdvancedTab from './tabs/AdvancedTab.vue'
import CapabilitiesTab from './tabs/CapabilitiesTab.vue'
import DriverPropsTab from './tabs/DriverPropsTab.vue'
import GeneralTab from './tabs/GeneralTab.vue'
import NetworkTab from './tabs/NetworkTab.vue'
import TestResultModal from './TestResultModal.vue'
import { useAddDataSource } from '../composables/useAddDataSource'
import { useDriverRegistry } from '../composables/useDriverRegistry'
import { useNetworkProfiles } from '../composables/useNetworkProfiles'
import { useUrlBuilder } from '../composables/useUrlBuilder'
import { connectDatabase as connectDatabaseService, closeConnection } from '../services/connection'
import { useProjectConnectionStore } from '../stores/project-connection-store'

import type { Driver } from '../../domain/types'

interface Props {
  modelValue: boolean
  initialDriver?: Driver | null
  initialName?: string
}

const props = withDefaults(defineProps<Props>(), { modelValue: false, initialDriver: null, initialName: '' })

const emit = defineEmits<{
  (e: 'update:modelValue', v: boolean): void
  (e: 'save'): void
}>()

const { t } = useI18n()
const message = useMessage()
const dialog = useDialog()
const projectStore = useProjectStore()
const projectConnectionStore = useProjectConnectionStore()
const { drivers, loadAll } = useDriverRegistry()
const {
  headerData, scope, selectedEnvId,
  setFileDb,
  buildSubmitPayload, validate,
  stagingItems,
  stagingIndex,
  isResetting,
  buildStagingItem,
  applyStagingItem,
  addStaging,
  removeStaging,
  selectStaging,
  clearStagingItems,
} = useAddDataSource()

// Dialog state
const activeTab = ref('general')
const selectedTypeId = ref<string | null>(null)
const uriEditing = ref(false)
const manualUri = ref('')
const formData = ref<Record<string, unknown>>({})
const testResult = ref<{ success: boolean; message: string; latencyMs?: number } | null>(null)
const testing = ref(false)
const saving = ref(false)
const savingAuth = ref(false) // 防重复调用 create_auth_config
const isEditing = ref(false)
const scopeChangedWarning = ref(false)

// Auth config (not yet in composable)
const authConfigId = ref<string | null>(null)
const authMethod = ref<string>('password')

// Extra config from child tabs
const networkConfigId = ref<string | null>(null)
const driverPropertiesExtra = ref<string | null>(null)
const advancedOptions = ref<string | null>(null)

const { sshProfiles, proxyProfiles } = useNetworkProfiles()

// Test connection modal state
const showTestModal = ref(false)
const lastTestResult = ref<{ success: boolean; message: string; serverVersion?: string; responseTimeMs?: number }>({ success: false, message: '' })

const testNetworkInfo = computed(() => {
  if (!networkConfigId.value) return t('navigator.directConnect')
  // 从已加载的网络配置中查找
  const allProfiles = [...sshProfiles.value, ...proxyProfiles.value]
  const profile = allProfiles.find(p => p.id === networkConfigId.value)
  if (!profile) return t('navigator.none')

  const detail = profile.detail || ''
  const typeLabel = profile.type === 'ssh'
    ? 'SSH'
    : profile.type === 'proxy'
      ? t('navigator.proxy')
      : profile.type.toUpperCase()

  // 构建路径: RdataStation → SSH(user@host:22) → target
  const dbTarget = String(formData.value.host || formData.value.file_path || '—')
  return `RdataStation → ${typeLabel}(${detail}) → ${dbTarget}`
})

// Computed
const selectedDriver = computed(() =>
  drivers.value.find(d => d.id === headerData.selectedDriverId) ?? null
)

const { uriPreview, buildUrl } = useUrlBuilder({ selectedDriver, formData, uriEditing, manualUri })

const driverOptions = computed(() => {
  if (!selectedTypeId.value) return []
  return drivers.value
    .filter(d => d.type_id === selectedTypeId.value && d.enabled)
    .map(d => ({ label: d.name, value: d.id }))
})

// Actions
function onDriverChange(driverId: string) {
  formData.value = {}
  testResult.value = null
  authConfigId.value = null
  authMethod.value = 'password'
  selectedEnvId.value = null
  // Set file DB flag for URI preview
  const d = drivers.value.find(x => x.id === driverId)
  setFileDb(d?.is_file ?? false)
}

function onFormData(d: Record<string, unknown>) {
  formData.value = { ...formData.value, ...d }
  if (!headerData.name && d.name) headerData.name = String(d.name)
}

function onExtraConfig(config: Record<string, unknown>) {
  if (config.networkConfigId !== undefined) networkConfigId.value = config.networkConfigId as string | null
  if (config.driverProperties !== undefined) driverPropertiesExtra.value = config.driverProperties as string | null
  if (config.advancedOptions !== undefined) advancedOptions.value = config.advancedOptions as string | null
  if (config.environmentId !== undefined) selectedEnvId.value = config.environmentId as string | null
}

function onAuthConfigChange(authCfgId: string | null, method: string) {
  authConfigId.value = authCfgId
  authMethod.value = method
}

function handleAddStaging() {
  addStaging()
  // 新增时清空右侧表单
  isResetting.value = true
  headerData.name = ''
  headerData.description = ''
  formData.value = {}
  testResult.value = null
  authConfigId.value = null
  authMethod.value = 'password'
  networkConfigId.value = null
  driverPropertiesExtra.value = null
  advancedOptions.value = null
  selectedEnvId.value = null
  activeTab.value = 'general'
  isResetting.value = false
}

function handleRemoveStaging(i: number) {
  removeStaging(i)
}

function handleSelectStaging(i: number) {
  selectStaging(i)
  const s = stagingItems.value[i]
  if (!s) return
  isResetting.value = true
  headerData.name = s.name || ''
  headerData.description = s.description || ''
  if (s.driver) {
    const d = drivers.value.find(x => x.name.toLowerCase() === s.driver?.toLowerCase())
    if (d) {
      selectedTypeId.value = d.type_id
      headerData.selectedDriverId = d.id
    }
  } else if (s.driverId) {
    headerData.selectedDriverId = s.driverId
    const d = drivers.value.find(x => x.id === s.driverId)
    if (d) selectedTypeId.value = d.type_id
  }
  formData.value = s.formData ? { ...s.formData } : {}
  if (s.scope) {
    scope.global = s.scope === 'global'
    scope.project = s.scope === 'project'
  }
  networkConfigId.value = s.networkConfigId ?? null
  driverPropertiesExtra.value = s.driverProperties ?? null
  advancedOptions.value = s.advancedOptions ?? null
  selectedEnvId.value = s.environmentId ?? null
  authConfigId.value = s.authConfigId ?? null
  authMethod.value = s.authMethod ?? 'password'
  testResult.value = null
  isResetting.value = false
}

async function handleTest() {
  if (!selectedDriver.value) { message.warning(t('navigator.selectDbType')); return }
  testing.value = true
  try {
    const url = buildUrl()
    const driverName = selectedDriver.value.name
    const dbType = selectedDriver.value.type_id // 使用 type_id 而非 name，避免 "mysql (native)" 含空格无法匹配后端注册表
    const { invoke } = await import('@tauri-apps/api/core')
    const params: Record<string, unknown> = {
      dbType: dbType,
      url,
    }
    if (networkConfigId.value) params.networkConfigId = networkConfigId.value
    if (authConfigId.value) params.authConfigId = authConfigId.value
    if (authMethod.value) params.authMethod = authMethod.value
    const r = await invoke<{ success: boolean; message?: string; server_version?: string; response_time_ms?: number }>('test_connection', params)
    testResult.value = {
      success: r.success,
      message: r.success
        ? `✓ ${t('navigator.connectionSuccess', { name: driverName })} — ${driverName} — [本机] → ${r.message || 'DB'}`
        : (r.message || t('navigator.connectionFailedGeneric')),
      latencyMs: r.success ? (r.response_time_ms ?? undefined) : undefined,
    }

    // 存储结构化结果并弹出详细反馈弹窗
    lastTestResult.value = {
      success: r.success,
      message: r.message || '',
      serverVersion: r.server_version,
      responseTimeMs: r.response_time_ms,
    }
    showTestModal.value = true
  } catch (e) {
    const msg = e instanceof Error ? e.message : (typeof e === 'string' ? e : JSON.stringify(e))
    console.error('[test_connection] 失败:', msg)
    testResult.value = { success: false, message: msg }
    lastTestResult.value = { success: false, message: msg }
    showTestModal.value = true
  } finally { testing.value = false }
}

/** 测试连接弹窗关闭回调 — 成功时弹出确认对话框再决定是否保存认证 */
async function onTestModalClose() {
  showTestModal.value = false

  if (savingAuth.value) return
  if (!lastTestResult.value?.success) return

  const fd = formData.value
  const authType = authMethod.value
  const hasAuth = isAuthRequired(authType) && (
    fd.username || fd.password || fd.certPath || fd.principal || fd.tokenEndpoint
  )
  if (!hasAuth) return

  const d = dialog.info({
    title: t('navigator.testSuccess'),
    content: t('navigator.saveAuthConfirm'),
    positiveText: t('navigator.confirm'),
    negativeText: t('navigator.cancel'),
    onPositiveClick: async () => {
      d.loading = true
      await doSaveAuth(authType, fd)
      d.loading = false
    },
    onNegativeClick: () => {
      d.destroy()
    },
  })
}

/** 判断认证方式是否需要凭据 */
function isAuthRequired(authMethod: string): boolean {
  return ['password', 'ldap', 'pg_class', 'kerberos', 'oauth2'].includes(authMethod)
}

/** 实际执行认证保存 */
async function doSaveAuth(authType: string, fd: Record<string, unknown>) {
  if (savingAuth.value) return
  savingAuth.value = true
  try {
    const { invoke: invokeTauri } = await import('@tauri-apps/api/core')
    const authData = buildAuthData(authType, fd)

    const driverName = selectedDriver.value?.name || ''
    const authName = headerData.name
      ? `${headerData.name} — 认证`
      : `${driverName} — 认证`

    const authDataStr = JSON.stringify(authData)

    const shouldSaveGlobal = scope.global
    const shouldSaveProject = scope.project && projectStore.hasProject

    if (shouldSaveGlobal && shouldSaveProject) {
      const globalId = await invokeTauri<{ id: string }>('create_auth_config', {
        ac: {
          id: '',
          name: `${authName} (全局)`,
          auth_type: authType,
          auth_data: authDataStr,
          created_at: '',
          updated_at: '',
        },
      })

      const pp = projectStore.currentProject!.path
      const projectId = await invokeTauri<{ id: string }>('project_create_auth_config', {
        name: `${authName} (项目)`,
        authType,
        authData: authDataStr,
        projectPath: pp,
      })

      authConfigId.value = projectId.id
      authMethod.value = authType
      message.info(t('navigator.authSavedHint', { global: true, project: true }))
    } else if (shouldSaveGlobal) {
      const created = await invokeTauri<{ id: string }>('create_auth_config', {
        ac: {
          id: '',
          name: authName,
          auth_type: authType,
          auth_data: authDataStr,
          created_at: '',
          updated_at: '',
        },
      })
      authConfigId.value = created.id
      authMethod.value = authType
      message.info(t('navigator.authSavedHint', { global: true, project: false }))
    } else if (shouldSaveProject) {
      const pp = projectStore.currentProject!.path
      const created = await invokeTauri<{ id: string }>('project_create_auth_config', {
        name: authName,
        authType,
        authData: authDataStr,
        projectPath: pp,
      })
      authConfigId.value = created.id
      authMethod.value = authType
      message.info(t('navigator.authSavedHint', { global: false, project: true }))
    }
  } catch (authErr) {
    const msg = authErr instanceof Error ? authErr.message : JSON.stringify(authErr)
    console.error('[auth] 保存认证配置失败:', msg)
    message.error(`${t('navigator.authSaveFailed')}: ${msg}`)
  } finally {
    savingAuth.value = false
  }
}

/** 构建认证数据对象 */
function buildAuthData(authType: string, fd: Record<string, unknown>): Record<string, unknown> {
  const authData: Record<string, unknown> = {}

  if (authType === 'password' || authType === 'ldap') {
    if (fd.username) authData.username = String(fd.username)
    if (fd.password) authData.password = String(fd.password)
  } else if (authType === 'pg_class') {
    if (fd.certPath) authData.certPath = String(fd.certPath)
    if (fd.certKeyPath) authData.certKeyPath = String(fd.certKeyPath)
  } else if (authType === 'kerberos') {
    if (fd.principal) authData.principal = String(fd.principal)
    if (fd.keytabPath) authData.keytabPath = String(fd.keytabPath)
  } else if (authType === 'oauth2') {
    if (fd.tokenEndpoint) authData.tokenEndpoint = String(fd.tokenEndpoint)
    if (fd.clientId) authData.clientId = String(fd.clientId)
    if (fd.clientSecret) authData.clientSecret = String(fd.clientSecret)
  }

  return authData
}

/** 保存到暂存列表（不连库，不关对话框） */
function saveToStaging() {
  if (!selectedDriver.value) { message.warning(t('navigator.selectDbType')); return }

  const validation = validate()
  if (!validation.valid) {
    const firstError = Object.values(validation.errors)[0]
    message.warning(firstError)
    return
  }

  if (!scope.global && !scope.project) { message.warning(t('navigator.selectSaveLocation')); return }

  const url = buildUrl()
  const name = headerData.name || selectedDriver.value.name
  const d = selectedDriver.value

  stagingItems.value[stagingIndex.value] = buildStagingItem(
    name,
    d.type_id,
    headerData.selectedDriverId ?? undefined,
    url,
    { ...formData.value },
    authConfigId.value,
    authMethod.value,
    networkConfigId.value,
    driverPropertiesExtra.value,
    advancedOptions.value,
    selectedEnvId.value ?? null,
    headerData.description || undefined,
    (formData.value.schema_name as string) || undefined,
    (formData.value.options as string) || undefined,
    (formData.value.metadata_path as string) || undefined,
    (formData.value.tags as string) || undefined,
    (formData.value.use_duckdb_fed as boolean) ?? false
  )

  message.success(t('navigator.savedToStaging', { name }))
}

async function handleSave() {
  saveToStaging()
}

function syncCurrentToStaging() {
  const idx = stagingIndex.value
  const name = headerData.name || stagingItems.value[idx]?.name || selectedDriver.value?.name || ''
  
  stagingItems.value[idx] = buildStagingItem(
    name,
    selectedDriver.value?.type_id,
    headerData.selectedDriverId ?? undefined,
    buildUrl(),
    { ...formData.value },
    authConfigId.value,
    authMethod.value,
    networkConfigId.value,
    driverPropertiesExtra.value,
    advancedOptions.value,
    selectedEnvId.value ?? null,
    headerData.description || undefined,
    (formData.value.schema_name as string) || undefined,
    (formData.value.options as string) || undefined,
    (formData.value.metadata_path as string) || undefined,
    (formData.value.tags as string) || undefined,
    (formData.value.use_duckdb_fed as boolean) ?? false
  )
}



/** 构建连接选项 */
function buildConnectOpts(
  item: StagingItem,
  networkConfigId: string | null,
  authConfigId: string | null
): {
  driverId?: string
  authConfigId?: string
  authMethod?: string
  networkConfigId?: string
  driverProperties?: string
  advancedOptions?: string
  environmentId?: string
  description?: string
} {
  return {
    driverId: item.driverId,
    authConfigId: authConfigId || undefined,
    authMethod: item.authMethod,
    networkConfigId: networkConfigId || undefined,
    driverProperties: item.driverProperties,
    advancedOptions: item.advancedOptions,
    environmentId: item.environmentId || undefined,
    description: item.description
  }
}

/** 批量应用所有暂存连接 → 写入数据库 */
async function handleApply() {
  syncCurrentToStaging()

  const validItems = stagingItems.value.filter(item => item.name)
  if (validItems.length === 0) {
    message.warning(t('navigator.noStagingToApply'))
    return
  }

  saving.value = true
  let successCount = 0
  const errors: string[] = []
  const appliedIndices: number[] = []

  try {
    const { invoke } = await import('@tauri-apps/api/core')

    for (let idx = 0; idx < validItems.length; idx++) {
      const item = validItems[idx]
      const originalIndex = stagingItems.value.findIndex(
        (i, j) => i.id === item.id || (i.name === item.name && j === idx)
      )
      
      try {
        const driverName = item.driver || 'mysql'
        const url = item.url || ''
        const name = item.name
        
        const shouldSaveGlobal = scope.global || (item.scope === 'global')
        const shouldSaveProject = scope.project || (item.scope === 'project')

        let itemSuccess = false

        if (shouldSaveProject && !shouldSaveGlobal) {
          await saveToProjectOnly(item, driverName, url, name, errors, invoke)
          itemSuccess = true
        } else {
          let snapshotNetId = item.networkConfigId ?? null
          let snapshotAuthId = item.authConfigId ?? null
          
          if (shouldSaveProject && projectStore.hasProject) {
            const pp = projectStore.currentProject?.path
            snapshotAuthId = await snapshotIfNeeded(snapshotAuthId, 'auth', pp, name, errors, invoke)
            snapshotNetId = await snapshotIfNeeded(snapshotNetId, 'network', pp, name, errors, invoke)
            
            if (snapshotAuthId === 'failed' || snapshotNetId === 'failed') {
              continue
            }
          }

          const connectOpts = buildConnectOpts(item, snapshotNetId, snapshotAuthId)

          let globalConnId: string | null = null
          if (shouldSaveGlobal) {
            try {
              const result = await connectDatabaseService(
                driverName, url, name, 'global', undefined, connectOpts
              )
              globalConnId = result.conn_id
            } catch (e) {
              errors.push(`${name} (全局): ${String(e)}`)
              if (!shouldSaveProject) continue
            }
          }

          if (shouldSaveProject && projectStore.hasProject) {
            try {
              await saveToProject(item, driverName, url, name, item.networkConfigId ?? null, item.authConfigId ?? null, invoke)
            } catch (e) {
              errors.push(`${name} (项目): ${String(e)}`)
              if (globalConnId) {
                try {
                  await closeConnection(globalConnId)
                } catch {}
              }
            }
          }

          if (!errors.some(err => err.startsWith(name))) {
            itemSuccess = true
          }
        }

        if (itemSuccess) {
          successCount++
          if (originalIndex !== -1) {
            appliedIndices.push(originalIndex)
          }
        }
      } catch (e) {
        errors.push(`${name}: ${String(e)}`)
      }
    }

    // 标记成功的项为已应用
    appliedIndices.forEach(index => markStagingApplied(index))

    if (errors.length === 0) {
      message.success(t('navigator.applySuccess', { count: successCount }))
    } else if (successCount > 0) {
      message.warning(t('navigator.applyPartial', { success: successCount, fail: errors.length }))
    } else {
      message.error(`${t('common.operationFailed')}: ${errors.join('; ')}`)
    }

    if (successCount > 0) {
      emit('save')
      emit('connectionsChanged')
      resetAndClose()
    }
  } catch (e) {
    message.error(`${t('common.operationFailed')}: ${(e as Error).message}`)
  } finally { saving.value = false }
}

async function snapshotIfNeeded(
  configId: string | null,
  type: 'auth' | 'network',
  projectPath: string | undefined,
  name: string,
  errors: string[],
  invoke: typeof import('@tauri-apps/api/core').invoke
): Promise<string | null> {
  if (!configId?.startsWith('G_') || configId.startsWith('GP_')) {
    return configId
  }

  const invokeFn = type === 'auth' ? 'snapshot_global_auth' : 'snapshot_global_network'
  const paramName = type === 'auth' ? 'globalAuthId' : 'globalNetId'

  try {
    const r = await invoke<{ snapshot_id: string }>(invokeFn, { [paramName]: configId, projectPath })
    return r.snapshot_id
  } catch (e) {
    errors.push(`${name}: ${type === 'auth' ? '认证' : '网络'}配置快照失败`)
    return 'failed'
  }
}

function buildConnectOpts(
  item: StagingItem,
  networkConfigId: string | null,
  authConfigId: string | null
) {
  return {
    driverId: item.driverId,
    networkConfigId,
    environmentId: item.environmentId ?? undefined,
    authConfigId,
    authMethod: item.authMethod ?? undefined,
    driverProperties: item.driverProperties ?? undefined,
    advancedOptions: item.advancedOptions ?? undefined,
    description: item.description || undefined,
    options: item.options || undefined,
    tags: item.tags || undefined,
    metadataPath: item.metadataPath || undefined,
    schemaName: item.schemaName || undefined,
    useDuckdbFed: item.useDuckdbFed ?? false,
  }
}

async function saveToProjectOnly(
  item: StagingItem,
  driverName: string,
  url: string,
  name: string,
  errors: string[],
  invoke: typeof import('@tauri-apps/api/core').invoke
) {
  if (!projectStore.hasProject) {
    errors.push(`${name}: 没有打开的项目`)
    return
  }

  const pp = projectStore.currentProject?.path
  let snapshotNetId = await snapshotIfNeeded(item.networkConfigId ?? null, 'network', pp, name, errors, invoke)
  let snapshotAuthId = await snapshotIfNeeded(item.authConfigId ?? null, 'auth', pp, name, errors, invoke)

  if (snapshotAuthId === 'failed' || snapshotNetId === 'failed') {
    return
  }

  const fd = item.formData || {}
  
  // 如果没有使用已保存的认证配置，且用户填写了认证信息，则保存新的认证配置
  let finalAuthConfigId = snapshotAuthId
  let finalAuthMethod = item.authMethod || authMethod.value
  const hasAuthData = (fd.username && fd.password) || fd.certPath || fd.principal
  if (!finalAuthConfigId && hasAuthData) {
    try {
      const authName = `${name} (认证)`
      const authData = buildAuthData(finalAuthMethod, fd as Record<string, unknown>)
      
      const r = await invoke<{ id: string }>('project_create_auth_config', {
        name: authName,
        authType: finalAuthMethod,
        authData: JSON.stringify(authData),
        projectPath: pp
      })
      finalAuthConfigId = r.id
    } catch (e) {
      console.warn('[AddDataSource] 保存认证配置失败:', e)
    }
  }

  const conn = await projectConnectionStore.createConnection({
    name,
    driver: driverName,
    host: String(fd.host || ''),
    port: Number(fd.port || 0),
    database: String(fd.database || ''),
    schema_name: item.schemaName || undefined,
    username: String(fd.username || ''),
    password: String(fd.password || ''),
    options: item.options || undefined,
    tags: item.tags || undefined,
    use_duckdb_fed: item.useDuckdbFed ?? false,
    metadata_path: item.metadataPath || undefined,
    description: item.description || undefined,
    driver_id: item.driverId,
    environment_id: item.environmentId ?? undefined,
    auth_config_id: finalAuthConfigId ?? undefined,
    auth_method: finalAuthMethod ?? undefined,
    network_config_id: snapshotNetId ?? undefined,
    driver_properties: item.driverProperties ?? undefined,
    advanced_options: item.advancedOptions ?? undefined,
  })

  await projectConnectionStore.loadConnections()
  
  // 自动建立项目连接
  if (pp) {
    try {
      await connectDatabaseService(
        driverName,
        url,
        name,
        'project',
        pp,
        buildConnectOpts(item, snapshotNetId, finalAuthConfigId)
      )
    } catch (e) {
      console.warn('[AddDataSource] 自动建立项目连接失败:', e)
    }
  }
  
  return conn
}

async function saveToProject(
  item: StagingItem,
  driverName: string,
  url: string,
  name: string,
  networkConfigId: string | null,
  authConfigId: string | null,
  invoke: typeof import('@tauri-apps/api/core').invoke
) {
  const pp = projectStore.currentProject?.path
  
  // 先对全局配置做快照
  const snapshotNetId = await snapshotIfNeeded(networkConfigId, 'network', pp, name, [], invoke)
  const snapshotAuthId = await snapshotIfNeeded(authConfigId, 'auth', pp, name, [], invoke)
  
  const fd = item.formData || {}
  const conn = await projectConnectionStore.createConnection({
    name,
    driver: driverName,
    host: String(fd.host || ''),
    port: Number(fd.port || 0),
    database: String(fd.database || ''),
    schema_name: item.schemaName || undefined,
    username: String(fd.username || ''),
    password: String(fd.password || ''),
    options: item.options || undefined,
    tags: item.tags || undefined,
    use_duckdb_fed: item.useDuckdbFed ?? false,
    metadata_path: item.metadataPath || undefined,
    description: item.description || undefined,
    driver_id: item.driverId,
    environment_id: item.environmentId ?? undefined,
    auth_config_id: (snapshotAuthId !== 'failed' ? snapshotAuthId : authConfigId) ?? undefined,
    auth_method: item.authMethod ?? undefined,
    network_config_id: (snapshotNetId !== 'failed' ? snapshotNetId : networkConfigId) ?? undefined,
    driver_properties: item.driverProperties ?? undefined,
    advanced_options: item.advancedOptions ?? undefined,
  })

  await projectConnectionStore.loadConnections()
  
  // 自动建立项目连接
  if (pp) {
    try {
      await connectDatabaseService(
        driverName,
        url,
        name,
        'project',
        pp,
        buildConnectOpts(item, snapshotNetId !== 'failed' ? snapshotNetId : networkConfigId, snapshotAuthId !== 'failed' ? snapshotAuthId : authConfigId)
      )
    } catch (e) {
      console.warn('[AddDataSource] 自动建立项目连接失败:', e)
    }
  }
  
  return conn
}

function resetAndClose() {
  testResult.value = null
  headerData.name = ''
  headerData.description = ''
  formData.value = {}
  clearStagingItems()
  emit('update:modelValue', false)
}

function handleClose() { resetAndClose() }

// Init
onMounted(async () => { await loadAll(projectStore.currentProject?.path) })

watch(() => props.modelValue, (open) => {
  if (open) {
    loadAll(projectStore.currentProject?.path)
    activeTab.value = 'general'
    testResult.value = null
    networkConfigId.value = null
    driverPropertiesExtra.value = null
    advancedOptions.value = null
    authConfigId.value = null
    authMethod.value = 'password'
    selectedEnvId.value = null
    manualUri.value = ''
    uriEditing.value = false
    if (props.initialDriver) {
      selectedTypeId.value = props.initialDriver.type_id
      headerData.selectedDriverId = props.initialDriver.id
    }
  }
}, { immediate: true })

watch(uriEditing, (editing) => {
  if (editing) {
    manualUri.value = uriPreview.value
  }
})

// T1: 暂存列表名字实时跟随右侧名称（表单重置时跳过，flush:sync 确保表单重置时同步拦截）
watch(() => headerData.name, (name) => {
  if (isResetting.value) return
  if (stagingItems.value[stagingIndex.value]) {
    stagingItems.value[stagingIndex.value].name = name
  }
}, { flush: 'sync' })

// T6: 选择数据源类型时自动选中第一个驱动
watch(selectedTypeId, (typeId) => {
  if (!typeId) return
  // 仅在未选择驱动或当前驱动不属于该类型时自动选中
  const currentDriver = drivers.value.find(d => d.id === headerData.selectedDriverId)
  if (currentDriver && currentDriver.type_id === typeId) return
  const firstDriver = drivers.value.find(d => d.type_id === typeId && d.enabled)
  if (firstDriver) {
    headerData.selectedDriverId = firstDriver.id
    formData.value = {}
    testResult.value = null
  }
})

watch(
  () => ({ global: scope.global, project: scope.project }),
  (_newVal, oldVal) => {
    // 仅在 scope 实际发生变化且用户已填写了网络/环境/存储配置时显示警告
    if (!oldVal || (_newVal.global === oldVal.global && _newVal.project === oldVal.project)) return
    const hasExtra =
      networkConfigId.value ||
      selectedEnvId.value ||
      driverPropertiesExtra.value ||
      advancedOptions.value
    if (hasExtra) {
      scopeChangedWarning.value = true
    } else {
      scopeChangedWarning.value = false
    }
  }
)
</script>

<style scoped>
/* ===== Card shell (matches settings-card pattern) ===== */
.datasource-card {
  display: flex;
  flex-direction: column;
  width: 980px;
  max-height: 88vh;
  background: var(--color-bg-primary);
  border: 1px solid var(--color-border);
  border-radius: var(--border-radius-md);
  overflow: hidden;
}

.card-header {
  display: flex;
  align-items: center;
  gap: var(--spacing-sm);
  padding: 6px var(--spacing-lg);
  border-bottom: 1px solid var(--color-border);
  flex-shrink: 0;
}

.card-icon {
  color: var(--brand-accent);
  flex-shrink: 0;
}

.card-title {
  font-size: var(--font-size-md);
  font-weight: 600;
  color: var(--color-text-primary);
  white-space: nowrap;
}

.card-body {
  display: flex;
  height: 520px;
  min-height: 420px;
  overflow: hidden;
}

/* ===== Right panel ===== */
.dlg-right {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  min-width: 0;
}

.scope-warning {
  margin: var(--spacing-sm) var(--spacing-sm) 0;
  flex-shrink: 0;
}

/* ===== Tabs ===== */
.dlg-tabs {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  min-height: 0;
}

.dlg-tabs :deep(.n-tabs-nav) { flex-shrink: 0; padding-left: var(--spacing-sm); }
.dlg-tabs :deep(.n-tabs-content) { flex: 1; min-height: 0; }
.dlg-tabs :deep(.n-tab-pane) {
  height: 100%;
  overflow-y: auto;
  padding: var(--spacing-md);
  box-sizing: border-box;
}

/* ===== Footer ===== */
.card-footer {
  display: flex;
  align-items: center;
  justify-content: flex-end;
  padding: var(--spacing-sm) var(--spacing-md);
  border-top: 1px solid var(--color-border);
  flex-shrink: 0;
  gap: var(--spacing-sm);
}

.test-result {
  font-size: var(--font-size-sm);
  display: flex;
  align-items: center;
  gap: 6px;
  flex-shrink: 1;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  margin-right: auto;
}

.test-icon.ok { color: var(--brand-success); }
.test-icon.fail { color: var(--brand-danger); }
.test-msg { color: var(--color-text-secondary); }
.test-latency { color: var(--brand-accent); font-family: var(--font-mono); font-size: var(--font-size-xs); }

.footer-spacer { flex: 1; }
</style>