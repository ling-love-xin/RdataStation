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
          @add-staging="addStaging"
          @remove-staging="removeStaging"
          @select-staging="selectStaging"
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
              <GeneralTab :driver="selectedDriver" :form-data="formData" :scope="scope" @update:form-data="onFormData" @auth-config-change="onAuthConfigChange" />
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
    @close="showTestModal = false"
  />
</template>

<script setup lang="ts">
import { Database } from 'lucide-vue-next'
import {
  NButton, NModal, NTabs, NTabPane, NAlert, useMessage,
} from 'naive-ui'
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
import { connectDatabase as connectDatabaseService } from '../services/connection'
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
const projectStore = useProjectStore()
const projectConnectionStore = useProjectConnectionStore()
const { drivers, loadAll } = useDriverRegistry()
const {
  headerData, scope, selectedEnvId,
  setFileDb,
  buildSubmitPayload, validate,
} = useAddDataSource()

// Dialog state — composable provides: headerData, scope, selectedEnvId, protocolChain, etc.
const activeTab = ref('general')
const selectedTypeId = ref<string | null>(null)
const uriEditing = ref(false)
const manualUri = ref('')
const formData = ref<Record<string, unknown>>({})
const testResult = ref<{ success: boolean; message: string; latencyMs?: number } | null>(null)
const testing = ref(false)
const saving = ref(false)
const isEditing = ref(false)
const scopeChangedWarning = ref(false)

// Auth config (not yet in composable)
const authConfigId = ref<string | null>(null)
const authMethod = ref<string>('password')

// Extra config from child tabs
const networkConfigId = ref<string | null>(null)
const driverPropertiesExtra = ref<string | null>(null)
const advancedOptions = ref<string | null>(null)

// Staging list
interface StagingItem {
  name: string
  driver?: string
  driverId?: string
  url?: string
  formData?: Record<string, unknown>
  authConfigId?: string | null
  authMethod?: string
  networkConfigId?: string | null
  driverProperties?: string | null
  advancedOptions?: string | null
  environmentId?: string | null
  scope?: 'global' | 'project'
  description?: string
}
const stagingItems = ref<StagingItem[]>([{ name: '' }])
const stagingIndex = ref(0)
const isResetting = ref(false) // 守卫：防止重置时 watch 覆盖暂存列表名字

const { sshProfiles, proxyProfiles, loadAll: loadNetworkProfiles } = useNetworkProfiles()

// Test connection modal state
const showTestModal = ref(false)
const lastTestResult = ref<{ success: boolean; message: string; serverVersion?: string; responseTimeMs?: number }>({ success: false, message: '' })

const testNetworkInfo = computed(() => {
  if (!networkConfigId.value) return t('navigator.directConnect')
  // 从已加载的网络配置中查找
  const allProfiles = [...sshProfiles.value, ...proxyProfiles.value]
  const profile = allProfiles.find(p => p.id === networkConfigId.value)
  if (!profile) return `${t('navigator.configured')} (${networkConfigId.value})`

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

function addStaging() {
  stagingItems.value.push({ name: '' })
  stagingIndex.value = stagingItems.value.length - 1
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

function removeStaging(i: number) {
  if (stagingItems.value.length <= 1) return
  stagingItems.value.splice(i, 1)
  if (stagingIndex.value >= stagingItems.value.length) stagingIndex.value = stagingItems.value.length - 1
}

function selectStaging(i: number) {
  stagingIndex.value = i
  const s = stagingItems.value[i]
  if (!s) return
  isResetting.value = true
  headerData.name = s.name || ''
  headerData.description = ''
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
  networkConfigId.value = s.networkConfigId ?? null
  driverPropertiesExtra.value = s.driverProperties ?? null
  advancedOptions.value = s.advancedOptions ?? null
  selectedEnvId.value = s.environmentId ?? null
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

    // 测试成功后自动保存认证信息到认证表
    if (r.success) {
      const fd = formData.value
      const hasAuth = fd.username || fd.password
      if (hasAuth) {
        try {
          // eslint-disable-next-line @typescript-eslint/no-explicit-any
          const authData: Record<string, any> = {}
          if (fd.username) authData.username = String(fd.username)
          if (fd.password) authData.password = String(fd.password)
          if (fd.host) authData.host = String(fd.host)
          if (fd.port) authData.port = String(fd.port)
          if (fd.database) authData.database = String(fd.database)

          const authName = headerData.name
            ? `${headerData.name} — 认证`
            : `${driverName} — 认证`

          await invoke('create_auth_config', {
            ac: {
              id: '',
              name: authName,
              auth_type: authMethod.value,
              auth_data: JSON.stringify(authData),
              origin: scope.global ? 'global' : 'project',
              source_id: null,
              snapshot_at: null,
              created_at: '',
              updated_at: '',
            },
          })
          // eslint-disable-next-line no-console
          console.log(`[auth] 测试成功，认证配置已保存: ${authName}`)
        } catch (authErr) {
          console.warn('[auth] 保存认证配置失败:', authErr)
        }
      }
    }
  } catch (e) {
    const msg = e instanceof Error ? e.message : (typeof e === 'string' ? e : JSON.stringify(e))
    console.error('[test_connection] 失败:', msg)
    testResult.value = { success: false, message: msg }
  } finally { testing.value = false }
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

  // 写入当前 staging 位置
  stagingItems.value[stagingIndex.value] = {
    name,
    driver: d.type_id,
    driverId: headerData.selectedDriverId ?? undefined,
    url,
    formData: { ...formData.value },
    authConfigId: authConfigId.value,
    authMethod: authMethod.value,
    networkConfigId: networkConfigId.value,
    driverProperties: driverPropertiesExtra.value,
    advancedOptions: advancedOptions.value,
    environmentId: selectedEnvId.value ?? null,
    scope: scope.global ? 'global' : 'project',
    description: headerData.description || undefined,
  }

  message.success(t('navigator.savedToStaging', { name }))
}

async function handleSave() {
  saveToStaging()
}

/** 批量应用所有暂存连接 → 写入数据库 */
async function handleApply() {
  if (stagingItems.value.length === 0 || !stagingItems.value[0].name) {
    message.warning(t('navigator.noStagingToApply'))
    return
  }

  saving.value = true
  let successCount = 0
  const errors: string[] = []

  try {
    const { invoke } = await import('@tauri-apps/api/core')

    for (const item of stagingItems.value) {
      if (!item.name) continue
      try {
        const driverName = item.driver || 'mysql'
        const url = item.url || ''
        const name = item.name
        const itemScope = item.scope || 'global'
        const projectId = itemScope === 'project' ? projectStore.currentProject?.id ?? null : null

        // 项目级连接快照全局配置
        let snapshotNetId = item.networkConfigId ?? null
        let snapshotAuthId = item.authConfigId ?? null
        if (itemScope === 'project' && projectStore.hasProject) {
          const pp = projectStore.currentProject?.path
          try {
            if (snapshotAuthId?.startsWith('G_') && !snapshotAuthId.startsWith('GP_')) {
              const r = await invoke<{ snapshot_id: string }>('snapshot_global_auth', { globalAuthId: snapshotAuthId, projectPath: pp })
              snapshotAuthId = r.snapshot_id
            }
            if (snapshotNetId?.startsWith('G_') && !snapshotNetId.startsWith('GP_')) {
              const r = await invoke<{ snapshot_id: string }>('snapshot_global_network', { globalNetId: snapshotNetId, projectPath: pp })
              snapshotNetId = r.snapshot_id
            }
          } catch (snapErr) {
            console.error('[snapshot] 认证/网络快照失败:', snapErr)
          }
        }

        const connectOpts = {
          driverId: item.driverId,
          networkConfigId: snapshotNetId,
          environmentId: item.environmentId ?? undefined,
          authConfigId: snapshotAuthId,
          driverProperties: item.driverProperties ?? undefined,
          advancedOptions: item.advancedOptions ?? undefined,
          description: item.description || undefined,
        }

        if (itemScope === 'project' && projectStore.hasProject) {
          const fd = item.formData || {}
          await projectConnectionStore.createConnection({
            name,
            driver: driverName,
            host: String(fd.host || ''),
            port: Number(fd.port || 0),
            database: String(fd.database || ''),
            username: String(fd.username || ''),
            password: String(fd.password || ''),
            use_duckdb_fed: false,
          })
        }

        await connectDatabaseService(
          driverName,
          url,
          name,
          itemScope,
          itemScope === 'project' ? projectStore.currentProject?.id : undefined,
          connectOpts
        )
        successCount++
      } catch (e) {
        const msg = e instanceof Error ? e.message : String(e)
        errors.push(`${item.name}: ${msg}`)
        console.error(`[apply] ${item.name} 应用失败:`, msg)
      }
    }

    if (errors.length === 0) {
      message.success(t('navigator.applySuccess', { count: successCount }))
    } else if (successCount > 0) {
      message.warning(t('navigator.applyPartial', { success: successCount, fail: errors.length }))
    } else {
      message.error(`${t('common.operationFailed')}: ${errors.join('; ')}`)
    }

    if (successCount > 0) {
      emit('save')
      resetAndClose()
    }
  } catch (e) {
    message.error(`${t('common.operationFailed')}: ${(e as Error).message}`)
  } finally { saving.value = false }
}

function resetAndClose() {
  testResult.value = null
  headerData.name = ''
  headerData.description = ''
  formData.value = {}
  stagingItems.value = [{ name: '' }]
  stagingIndex.value = 0
  emit('update:modelValue', false)
}

function handleClose() { resetAndClose() }

// Init
onMounted(async () => { await loadAll(projectStore.currentProject?.path); await loadNetworkProfiles() })

watch(() => props.modelValue, (open) => {
  if (open) {
    loadAll(projectStore.currentProject?.path)
    loadNetworkProfiles()
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