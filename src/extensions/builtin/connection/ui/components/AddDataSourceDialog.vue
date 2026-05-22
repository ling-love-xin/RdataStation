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
            :name-label="$t('navigator.name')"
            :name-placeholder="$t('navigator.dataSourceNamePlaceholder')"
            :desc-label="$t('navigator.description')"
            :desc-placeholder="$t('navigator.dataSourceDescPlaceholder')"
            :global-label="$t('navigator.globalConnection')"
            :project-label="$t('navigator.projectConnection')"
            :driver-label="$t('navigator.driver')"
            :driver-placeholder="$t('navigator.selectDbType')"
            uri-label="URI"
            uri-placeholder="jdbc:mysql://..."
            @driver-change="onDriverChange"
          />

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
</template>

<script setup lang="ts">
import { Database } from 'lucide-vue-next'
import {
  NButton, NModal, NTabs, NTabPane, useMessage,
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
import { useAddDataSource } from '../composables/useAddDataSource'
import { useDriverRegistry } from '../composables/useDriverRegistry'
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
  formData?: Record<string, unknown>
  networkConfigId?: string | null
  driverProperties?: string | null
  advancedOptions?: string | null
  environmentId?: string | null
}
const stagingItems = ref<StagingItem[]>([{ name: '' }])
const stagingIndex = ref(0)

// Computed
const selectedDriver = computed(() =>
  drivers.value.find(d => d.id === headerData.selectedDriverId) ?? null
)

const driverOptions = computed(() => {
  if (!selectedTypeId.value) return []
  return drivers.value
    .filter(d => d.type_id === selectedTypeId.value && d.enabled)
    .map(d => ({ label: d.name, value: d.id }))
})

const uriPreview = computed(() => {
  const d = selectedDriver.value
  if (!d) return ''
  const fd = formData.value
  if (d.is_file) return `${d.name.toLowerCase()}://${fd.file_path || fd.database || './data.db'}`
  const usr = fd.username || 'user'
  const pw = fd.password ? '****' : ''
  const h = fd.host || 'localhost'
  const p = fd.port || d.default_port || ''
  const db = fd.database || ''
  if (pw) return `${d.name.toLowerCase()}://${usr}:${pw}@${h}${p ? ':' + p : ''}/${db}`
  return `${d.name.toLowerCase()}://${usr}@${h}${p ? ':' + p : ''}/${db}`
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
}

function buildUrl(): string {
  if (uriEditing.value && manualUri.value) return manualUri.value
  const d = selectedDriver.value
  if (!d) return ''
  const proto = d.type_id.toLowerCase()
  const fd = formData.value
  if (d.is_file) return `${proto}://${fd.file_path || fd.database || './data.db'}`
  const h = String(fd.host || 'localhost')
  const po = String(fd.port || d.default_port || '')
  const db = String(fd.database || '')
  const u = String(fd.username || '')
  const pw = String(fd.password || '')
  if (u && pw) return `${proto}://${u}:${pw}@${h}${po ? ':' + po : ''}/${db}`
  return `${proto}://${u}@${h}${po ? ':' + po : ''}/${db}`
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
  } catch (e) {
    const msg = e instanceof Error ? e.message : (typeof e === 'string' ? e : JSON.stringify(e))
    console.error('[test_connection] 失败:', msg)
    testResult.value = { success: false, message: msg }
  } finally { testing.value = false }
}

async function doSave(): Promise<void> {
  if (!selectedDriver.value) { message.warning(t('navigator.selectDbType')); return }

  // 使用 useAddDataSource 统一校验
  const validation = validate()
  if (!validation.valid) {
    const firstError = Object.values(validation.errors)[0]
    message.warning(firstError)
    return
  }

  if (!scope.global && !scope.project) { message.warning(t('navigator.selectSaveLocation')); return }

  saving.value = true
  try {
    const url = buildUrl()
    const name = headerData.name || selectedDriver.value.name
    const d = selectedDriver.value

    const { invoke } = await import('@tauri-apps/api/core')

    // 项目级连接：快照引用的全局认证/网络配置，防止全局修改影响已有项目
    if (scope.project && projectStore.hasProject) {
      const pp = projectStore.currentProject?.path
      try {
        if (authConfigId.value?.startsWith('G_') && !authConfigId.value.startsWith('GP_')) {
          const r = await invoke<{ snapshot_id: string }>('snapshot_global_auth', { globalAuthId: authConfigId.value, projectPath: pp })
          authConfigId.value = r.snapshot_id
        }
        if (networkConfigId.value?.startsWith('G_') && !networkConfigId.value.startsWith('GP_')) {
          const r = await invoke<{ snapshot_id: string }>('snapshot_global_network', { globalNetId: networkConfigId.value, projectPath: pp })
          networkConfigId.value = r.snapshot_id
        }
      } catch (snapErr) {
        console.error('[snapshot] 认证/网络快照失败:', snapErr)
      }
    }

    stagingItems.value[stagingIndex.value] = {
      name,
      driver: d.type_id,
      driverId: headerData.selectedDriverId ?? undefined,
      formData: { ...formData.value },
      networkConfigId: networkConfigId.value,
      driverProperties: driverPropertiesExtra.value,
      advancedOptions: advancedOptions.value,
      environmentId: selectedEnvId.value ?? null,
    }

    const connectOpts = {
      driverId: headerData.selectedDriverId,
      networkConfigId: networkConfigId.value,
      environmentId: selectedEnvId.value ?? undefined,
      authConfigId: authConfigId.value,
      driverProperties: driverPropertiesExtra.value ?? undefined,
      advancedOptions: advancedOptions.value ?? undefined,
      description: headerData.description || undefined,
    }

    let returnedConnId: string | null = null

    if (scope.project && projectStore.hasProject) {
      const fd = formData.value
      await projectConnectionStore.createConnection({
        name,
        driver: d.type_id,
        host: d.is_file ? String(fd.file_path || fd.database || '') : String(fd.host || ''),
        port: d.is_file ? 0 : Number(fd.port || d.default_port || 0),
        database: String(fd.database || ''),
        username: String(fd.username || ''),
        password: String(fd.password || ''),
        use_duckdb_fed: false,
      })
      const connResponse = await connectDatabaseService(
        d.type_id,
        url,
        name,
        'project',
        projectStore.currentProject?.id,
        connectOpts
      )
      returnedConnId = connResponse.conn_id
    }
    if (scope.global) {
      const connResponse = await connectDatabaseService(
        d.type_id,
        url,
        name,
        'global',
        undefined,
        connectOpts
      )
      returnedConnId = connResponse.conn_id
    }

    message.success(t('navigator.connectionSavedTo', { name, locations: '' }))
    emit('save')
  } catch (e) {
    message.error(`${t('common.operationFailed')}: ${(e as Error).message}`)
    throw e
  } finally { saving.value = false }
}

async function handleSave() {
  try {
    await doSave()
    resetAndClose()
  } catch (err) { console.warn('[handleSave] 保存失败:', err) /* error already shown in doSave */ }
}

async function handleApply() {
  try {
    await doSave()
    resetAndClose()
  } catch (err) { console.warn('[handleSave] 保存失败:', err) /* error already shown in doSave */ }
}

function resetAndClose() {
  testResult.value = null
  headerData.name = ''
  headerData.description = ''
  formData.value = {}
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