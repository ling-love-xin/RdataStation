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
          <!-- Header: 3 rows -->
          <div class="right-header">
            <!-- Row 1: Name + Scope -->
            <div class="rh-row">
              <span class="rh-label">{{ $t('navigator.name') }}</span>
              <NInput
                v-model:value="formName"
                :placeholder="$t('navigator.dataSourceNamePlaceholder')"
                size="small"
                class="rh-name-input"
              />
              <NCheckbox v-model:checked="scopeGlobal" size="small">
                {{ $t('navigator.globalConnection') }}
              </NCheckbox>
              <NCheckbox v-model:checked="scopeProject" size="small">
                {{ $t('navigator.projectConnection') }}
              </NCheckbox>
            </div>
            <!-- Row 2: Description standalone -->
            <div class="rh-row">
              <span class="rh-label">{{ $t('navigator.description') }}</span>
              <NInput
                v-model:value="formDesc"
                type="textarea"
                :placeholder="$t('navigator.dataSourceDescPlaceholder')"
                size="small"
                :rows="2"
                class="rh-desc-input"
              />
            </div>
            <!-- Row 3: Driver + URI -->
            <div class="rh-row uri-row">
              <span class="rh-label">{{ $t('navigator.driver') }}</span>
              <NSelect
                v-model:value="selectedDriverId"
                :options="driverOptions"
                :placeholder="$t('navigator.selectDbType')"
                size="small"
                class="rh-driver-select"
                @update:value="onDriverChange"
              />
              <span class="uri-label">URI</span>
              <NInput
                v-if="uriEditing"
                v-model:value="manualUri"
                size="small"
                class="uri-edit-input"
                placeholder="jdbc:mysql://..."
              />
              <div v-else class="uri-display">{{ uriPreview || '—' }}</div>
              <NButton
                size="tiny"
                quaternary
                :type="uriEditing ? 'primary' : 'default'"
                @click="uriEditing = !uriEditing"
              >
                <template #icon><Edit :size="13" /></template>
              </NButton>
            </div>
          </div>

          <!-- Tabs -->
          <NTabs v-model:value="activeTab" type="line" size="small" class="dlg-tabs">
            <NTabPane name="general" :tab="$t('navigator.tabGeneral')">
              <GeneralTab :driver="selectedDriver" :form-data="formData" @update:form-data="onFormData" @auth-config-change="onAuthConfigChange" />
            </NTabPane>
            <NTabPane name="network" :tab="$t('navigator.tabNetwork')">
              <NetworkTab :driver="selectedDriver" @extra-config="onExtraConfig" />
            </NTabPane>
            <NTabPane name="capabilities" :tab="$t('navigator.tabCapabilities')">
              <CapabilitiesTab :driver="selectedDriver" />
            </NTabPane>
            <NTabPane name="properties" :tab="$t('navigator.tabDriverProps')">
              <DriverPropsTab :driver="selectedDriver" @extra-config="onExtraConfig" />
            </NTabPane>
            <NTabPane name="advanced" :tab="$t('navigator.tabAdvanced')">
              <AdvancedTab :driver="selectedDriver" :form-data="formData" @update:form-data="onFormData" @extra-config="onExtraConfig" />
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
import { Database, Edit } from 'lucide-vue-next'
import {
  NButton, NCheckbox, NInput, NModal, NSelect, NTabs, NTabPane, useMessage,
} from 'naive-ui'
import { ref, computed, watch, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'

import { useProjectStore } from '@/core/project/stores/project'

import AddDataSourceSidebar from './AddDataSourceSidebar.vue'
import AdvancedTab from './tabs/AdvancedTab.vue'
import CapabilitiesTab from './tabs/CapabilitiesTab.vue'
import DriverPropsTab from './tabs/DriverPropsTab.vue'
import GeneralTab from './tabs/GeneralTab.vue'
import NetworkTab from './tabs/NetworkTab.vue'
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

// Dialog state
const activeTab = ref('general')
const selectedTypeId = ref<string | null>(null)
const selectedDriverId = ref<string | null>(null)
const formName = ref('')
const formDesc = ref('')
const scopeGlobal = ref(true)
const scopeProject = ref(false)
const uriEditing = ref(false)
const manualUri = ref('')
const formData = ref<Record<string, unknown>>({})
const testResult = ref<{ success: boolean; message: string; latencyMs?: number } | null>(null)
const testing = ref(false)
const saving = ref(false)
const isEditing = ref(false)

// Staging list
interface StagingItem {
  name: string
  driver?: string
  driverId?: string
  formData?: Record<string, unknown>
  networkConfigId?: string | null
  driverProperties?: string | null
  advancedOptions?: string | null
}
const stagingItems = ref<StagingItem[]>([{ name: '' }])
const stagingIndex = ref(0)

// Extra config from child tabs
const networkConfigId = ref<string | null>(null)
const driverProperties = ref<string | null>(null)
const advancedOptions = ref<string | null>(null)
const authConfigId = ref<string | null>(null)
const authMethod = ref<string>('password')

// Computed
const selectedDriver = computed(() =>
  drivers.value.find(d => d.id === selectedDriverId.value) ?? null
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
function onDriverChange(_id: string) {
  formData.value = {}
  testResult.value = null
  authConfigId.value = null
  authMethod.value = 'password'
}

function onFormData(d: Record<string, unknown>) {
  formData.value = { ...formData.value, ...d }
  if (!formName.value && d.name) formName.value = String(d.name)
}

function onExtraConfig(config: Record<string, unknown>) {
  if (config.networkConfigId !== undefined) networkConfigId.value = config.networkConfigId as string | null
  if (config.driverProperties !== undefined) driverProperties.value = config.driverProperties as string | null
  if (config.advancedOptions !== undefined) advancedOptions.value = config.advancedOptions as string | null
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
  formName.value = s.name || ''
  formDesc.value = ''
  if (s.driver) {
    const d = drivers.value.find(x => x.name.toLowerCase() === s.driver?.toLowerCase())
    if (d) {
      selectedTypeId.value = d.type_id
      selectedDriverId.value = d.id
    }
  } else if (s.driverId) {
    selectedDriverId.value = s.driverId
    const d = drivers.value.find(x => x.id === s.driverId)
    if (d) selectedTypeId.value = d.type_id
  }
  formData.value = s.formData ? { ...s.formData } : {}
  networkConfigId.value = s.networkConfigId ?? null
  driverProperties.value = s.driverProperties ?? null
  advancedOptions.value = s.advancedOptions ?? null
  testResult.value = null
}

function buildUrl(): string {
  if (uriEditing.value && manualUri.value) return manualUri.value
  const d = selectedDriver.value
  if (!d) return ''
  const fd = formData.value
  if (d.is_file) return `${d.name.toLowerCase()}://${fd.file_path || fd.database || './data.db'}`
  const h = String(fd.host || 'localhost')
  const po = String(fd.port || d.default_port || '')
  const db = String(fd.database || '')
  const u = String(fd.username || '')
  const pw = String(fd.password || '')
  if (u && pw) return `${d.name.toLowerCase()}://${u}:${pw}@${h}${po ? ':' + po : ''}/${db}`
  return `${d.name.toLowerCase()}://${u}@${h}${po ? ':' + po : ''}/${db}`
}

async function handleTest() {
  if (!selectedDriver.value) { message.warning(t('navigator.selectDbType')); return }
  testing.value = true
  try {
    const url = buildUrl()
    const driverName = selectedDriver.value.name
    const { invoke } = await import('@tauri-apps/api/core')
    const params: Record<string, unknown> = {
      dbType: driverName.toLowerCase(),
      url,
    }
    if (networkConfigId.value) params.networkConfigId = networkConfigId.value
    const r = await invoke<{ success: boolean; message?: string; latency_ms?: number }>('test_connection', params)
    testResult.value = {
      success: r.success,
      message: r.success
        ? `✓ ${t('navigator.connectionSuccess', { name: driverName })} — ${driverName} — [本机] → ${r.message || 'DB'}`
        : (r.message || t('navigator.connectionFailedGeneric')),
      latencyMs: r.success ? (r.latency_ms ?? undefined) : undefined,
    }
  } catch (e) {
    testResult.value = { success: false, message: (e as Error).message }
  } finally { testing.value = false }
}

async function doSave(): Promise<void> {
  if (!selectedDriver.value) { message.warning(t('navigator.selectDbType')); return }
  if (!scopeGlobal.value && !scopeProject.value) { message.warning(t('navigator.selectSaveLocation')); return }

  saving.value = true
  try {
    const url = buildUrl()
    const name = formName.value || selectedDriver.value.name
    const d = selectedDriver.value
    const fd = formData.value

    stagingItems.value[stagingIndex.value] = {
      name,
      driver: d.name.toLowerCase(),
      driverId: selectedDriverId.value ?? undefined,
      formData: { ...formData.value },
      networkConfigId: networkConfigId.value,
      driverProperties: driverProperties.value,
      advancedOptions: advancedOptions.value,
    }

    const { invoke } = await import('@tauri-apps/api/core')

    if (scopeProject.value && projectStore.hasProject) {
      await projectConnectionStore.createConnection({
        name,
        driver: d.name.toLowerCase(),
        host: d.is_file ? String(fd.file_path || fd.database || '') : String(fd.host || ''),
        port: d.is_file ? 0 : Number(fd.port || d.default_port || 0),
        database: String(fd.database || ''),
        username: String(fd.username || ''),
        password: String(fd.password || ''),
        use_duckdb_fed: false,
      })
      await invoke('connect_database', {
        input: {
          db_type: d.name.toLowerCase(),
          url,
          name,
          connection_type: 'project',
          project_id: projectStore.currentProject?.id,
          driver_id: selectedDriverId.value,
          network_config_id: networkConfigId.value,
          driver_properties: driverProperties.value,
          advanced_options: advancedOptions.value,
          auth_config_id: authConfigId.value,
          auth_method: authMethod.value,
          description: formDesc.value || null,
        },
      })
    }
    if (scopeGlobal.value) {
      await invoke('connect_database', {
        input: {
          db_type: d.name.toLowerCase(),
          url,
          name,
          connection_type: 'global',
          driver_id: selectedDriverId.value,
          network_config_id: networkConfigId.value,
          driver_properties: driverProperties.value,
          advanced_options: advancedOptions.value,
          auth_config_id: authConfigId.value,
          auth_method: authMethod.value,
          description: formDesc.value || null,
        },
      })
    }

    message.success(t('navigator.connectionSavedTo', { name, locations: '' }))
    emit('save')
  } catch (e) {
    message.error(`${t('common.operationFailed')}: ${(e as Error).message}`)
    console.error('Save failed:', e)
    throw e
  } finally { saving.value = false }
}

async function handleSave() {
  try {
    await doSave()
    resetAndClose()
  } catch { /* error already shown in doSave */ }
}

async function handleApply() {
  try {
    await doSave()
    resetAndClose()
  } catch { /* error already shown in doSave */ }
}

function resetAndClose() {
  testResult.value = null
  formName.value = ''
  formDesc.value = ''
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
    driverProperties.value = null
    advancedOptions.value = null
    authConfigId.value = null
    authMethod.value = 'password'
    manualUri.value = ''
    uriEditing.value = false
    if (props.initialDriver) {
      selectedTypeId.value = props.initialDriver.type_id
      selectedDriverId.value = props.initialDriver.id
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

/* ===== Header: 3 rows ===== */
.right-header {
  padding: var(--spacing-md) var(--spacing-md) var(--spacing-sm);
  border-bottom: 1px solid var(--color-border-subtle);
  display: flex;
  flex-direction: column;
  gap: var(--spacing-sm);
  flex-shrink: 0;
}

.rh-row {
  display: flex;
  align-items: center;
  gap: var(--spacing-sm);
}

.rh-label {
  font-size: var(--font-size-sm);
  font-weight: 600;
  color: var(--color-text-muted);
  width: 48px;
  flex-shrink: 0;
  text-align: right;
}

.rh-name-input { flex: 1; max-width: 280px; }
.rh-desc-input { flex: 1; }

.rh-driver-select { flex: 0 0 200px; }

/* URI row */
.uri-row { gap: var(--spacing-xs); }
.uri-label {
  font-size: 11px;
  color: var(--color-text-muted);
  flex-shrink: 0;
  padding: 0 2px;
}

.uri-display {
  flex: 1;
  height: 28px;
  padding: 0 10px;
  font-size: 11px;
  font-family: 'JetBrains Mono', monospace;
  color: var(--brand-success);
  background: var(--color-bg-elevated);
  border: 1px solid var(--color-border-subtle);
  border-radius: var(--border-radius-sm);
  display: flex;
  align-items: center;
  overflow: hidden;
  white-space: nowrap;
  text-overflow: ellipsis;
  min-width: 0;
}

.uri-edit-input { flex: 1; min-width: 0; }
.uri-edit-input :deep(.n-input__input) {
  font-family: 'JetBrains Mono', monospace;
  font-size: 11px;
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