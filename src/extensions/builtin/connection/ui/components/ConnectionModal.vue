<template>
  <Teleport to="body">
    <Transition name="modal">
      <div v-if="modelValue" class="modal-overlay" @click.self="close">
        <div class="modal-container">
          <ConnectionModalHeader
            :title="
              isEditing ? t('navigator.editConnection') : t('navigator.newDatabaseConnection')
            "
            :driver-icon="driver.selectedDriver.value?.id || null"
            :show-name-input="!!driver.selectedDriver.value"
            :show-scope-toggle="!!driver.selectedDriver.value"
            :name="form.formData.value.name"
            :name-error="form.errors.value.name"
            :save-to-global="form.saveToGlobal.value"
            :save-to-project="form.saveToProject.value"
            :has-project="form.hasProject.value"
            :global-label="String(t('navigator.global'))"
            :project-label="String(t('navigator.project'))"
            :no-project-tip="String(t('navigator.noProjectOpen'))"
            @close="close"
            @update:name="(v: string) => (form.formData.value.name = v)"
            @update:save-to-global="(v: boolean) => (form.saveToGlobal.value = v)"
            @update:save-to-project="(v: boolean) => (form.saveToProject.value = v)"
          />

          <div class="modal-body">
            <ConnectionSidebar
              v-if="!driver.selectedDriver.value || driver.showDriverTree.value"
              :drivers="driver.drivers.value"
              :selected-driver="driver.selectedDriver.value"
              @select="onDriverSelect"
            />

            <div class="modal-content">
              <template v-if="driver.selectedDriver.value">
                <div class="tabs-nav">
                  <button
                    v-for="tab in driver.visibleTabs.value"
                    :key="tab.key"
                    class="tab-button"
                    :class="{
                      active: driver.activeTab.value === tab.key,
                      disabled: tab.disabled,
                    }"
                    :disabled="tab.disabled"
                    @click="!tab.disabled && (driver.activeTab.value = tab.key)"
                  >
                    <component :is="tab.icon" :size="14" />
                    <span>{{ tab.label }}</span>
                    <span v-if="tab.disabled" class="disabled-badge">不可用</span>
                  </button>
                </div>

                <div class="tabs-content">
                  <GeneralTab
                    v-show="driver.activeTab.value === 'general'"
                    :form-data="form.formData.value"
                    :selected-driver="driver.selectedDriver.value"
                    :has-project="form.hasProject.value"
                    :errors="form.errors.value"
                    :form-sections="driver.dynamicFormSections.value"
                    @update:form-data="form.updateFormData"
                    @select-file="selectDatabaseFile"
                    @create-file="createNewDatabaseFile"
                  />
                  <SshTunnelTab
                    v-show="driver.activeTab.value === 'ssh'"
                    :config="form.sshConfig"
                  />
                  <DuckdbAccelerationTab
                    v-show="driver.activeTab.value === 'duckdb'"
                    :form-data="form.formData.value"
                    @update:form-data="form.updateFormData"
                  />
                  <ProxyTab
                    v-show="driver.activeTab.value === 'proxy'"
                    :config="form.proxyConfig"
                  />
                  <DriverInfoPanel
                    v-show="driver.activeTab.value === 'driver'"
                    :driver="driver.selectedDriver.value"
                    :options="form.formData.value.options"
                  />
                </div>
              </template>

              <div v-else class="empty-state">
                <Database :size="48" class="empty-icon" />
                <h3 class="empty-title">选择数据库类型</h3>
                <p class="empty-desc">从左侧列表中选择您要连接的数据库类型</p>
              </div>
            </div>
          </div>

          <ConnectionModalFooter
            :show-sidebar-toggle="!!driver.selectedDriver.value"
            :show-test-button="!!driver.selectedDriver.value"
            :sidebar-visible="driver.showDriverTree.value"
            :testing="testing"
            :saving="saving"
            :save-label="isEditing ? '保存' : '保存连接'"
            :test-result="testResultDisplay"
            @toggle-sidebar="driver.showDriverTree.value = !driver.showDriverTree.value"
            @cancel="close"
            @test="testConnection"
            @save="saveConnection"
          />
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<script setup lang="ts">
import { open, save } from '@tauri-apps/plugin-dialog'
import { Database } from 'lucide-vue-next'
import { ref, computed, watch, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'

import ConnectionModalFooter from './ConnectionModalFooter.vue'
import ConnectionModalHeader from './ConnectionModalHeader.vue'
import ConnectionSidebar from './ConnectionSidebar.vue'
import DriverInfoPanel from './DriverInfoPanel.vue'
import DuckdbAccelerationTab from './tabs/DuckdbAccelerationTab.vue'
import GeneralTab from './tabs/GeneralTab.vue'
import ProxyTab from './tabs/ProxyTab.vue'
import SshTunnelTab from './tabs/SshTunnelTab.vue'
import { useConnectionForm } from '../composables/useConnectionForm'
import { useDriverSelector } from '../composables/useDriverSelector'
import * as connectionService from '../services/connection'

import type { DriverDescriptor, ConnectionConfig } from '../types/connection'

interface Props {
  modelValue: boolean
  editingConnection?: ConnectionConfig | null
}

const props = withDefaults(defineProps<Props>(), {
  editingConnection: null,
})

const emit = defineEmits<{
  'update:modelValue': [value: boolean]
  save: [connection: Record<string, unknown>]
}>()

const { t } = useI18n()

const isEditing = computed(() => !!props.editingConnection)
const testing = ref(false)
const saving = ref(false)

const driver = useDriverSelector()
const form = useConnectionForm(
  () => props.editingConnection,
  () => driver.selectedDriver.value
)

const testResultDisplay = computed(() => {
  const tr = form.testResult.value
  if (!tr) return null
  return {
    success: tr.success,
    message: tr.message,
    responseTimeMs: tr.response_time_ms,
  }
})

watch(
  () => props.editingConnection,
  conn => {
    if (conn) {
      form.loadEditingData(conn)
      driver.selectedDriver.value = driver.findDriverById(conn.driver || '')
      driver.showDriverTree.value = false
    }
  },
  { immediate: true }
)

onMounted(async () => {
  await driver.loadDrivers()
})

async function onDriverSelect(selected: DriverDescriptor) {
  driver.selectedDriver.value = selected
  driver.showDriverTree.value = true
  form.formData.value.driver = selected.id
  form.formData.value.port = selected.defaultPort
  form.applyDriverDefaults(selected.id)
  form.resetNameEditState()
  driver.activeTab.value = 'general'

  const schemaResult = await driver.loadDynamicForm(selected.id)
  if (schemaResult) {
    form.formData.value = {
      ...form.formData.value,
      ...schemaResult.dynamicData,
      port: schemaResult.defaultPort || form.formData.value.port,
    }
  }
}

async function testConnection() {
  if (!form.validateForm()) return

  testing.value = true
  form.testResult.value = null

  try {
    const driverId = form.formData.value.driver
    if (!driverId) return

    const url = form.connectionUrl.value
    const result = await connectionService.testConnection(driverId, url)

    form.testResult.value = {
      success: result.success,
      message: result.message,
      server_version: result.server_version,
      response_time_ms: result.response_time_ms,
    }
  } catch (error) {
    const errorMsg = error instanceof Error ? error.message : String(error)
    form.testResult.value = { success: false, message: errorMsg }
  } finally {
    testing.value = false
  }
}

async function saveConnection() {
  if (!form.validateForm()) return

  const driverId =
    form.formData.value.driver || driver.selectedDriver.value?.id || ''
  if (!driverId) return

  saving.value = true
  try {
    const connectionConfig = form.buildConnectionConfig()
    emit('save', connectionConfig)
  } catch (error) {
    console.error('保存连接失败:', error)
  } finally {
    saving.value = false
  }
}

async function selectDatabaseFile() {
  const selected = await open({
    multiple: false,
    filters: [
      {
        name: 'Database Files',
        extensions:
          driver.selectedDriver.value?.id === 'sqlite'
            ? ['db', 'sqlite', 'sqlite3']
            : ['duckdb', 'db'],
      },
    ],
  })
  if (selected) {
    form.formData.value.database = selected as string
  }
}

async function createNewDatabaseFile() {
  const ext = driver.selectedDriver.value?.id === 'sqlite' ? 'db' : 'duckdb'
  const defaultName = form.formData.value.name || 'new_database'

  const selected = await save({
    filters: [{ name: 'Database Files', extensions: [ext] }],
    defaultPath: `${defaultName}.${ext}`,
    title: '保存数据库文件',
  })

  if (selected) {
    try {
      const driverId = driver.selectedDriver.value?.id
      if (!driverId) return

      const result = await connectionService.createDatabaseFile(driverId, selected)
      if (result.success) {
        form.formData.value.database = selected
      } else {
        form.errors.value.database = result.message
      }
    } catch {
      form.errors.value.database = '创建数据库文件失败'
    }
  }
}

function close() {
  emit('update:modelValue', false)
  form.resetForm()
  driver.selectedDriver.value = null
  driver.showDriverTree.value = true
  driver.activeTab.value = 'general'
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

.tabs-nav {
  display: flex;
  gap: 4px;
  padding: 12px 24px;
  border-bottom: 1px solid var(--border-color);
  background: var(--bg-secondary);
  flex-wrap: nowrap;
  overflow-x: auto;
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
  white-space: nowrap;
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

.disabled-badge {
  font-size: 10px;
  padding: 1px 4px;
  background: var(--bg-tertiary);
  border-radius: 2px;
  color: var(--text-tertiary);
  margin-left: 4px;
}

.tabs-content {
  flex: 1;
  overflow-y: auto;
  padding: 24px;
}

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

.modal-enter-active,
.modal-leave-active {
  transition: opacity 0.3s ease;
}

.modal-enter-from,
.modal-leave-to {
  opacity: 0;
}
</style>