<template>
  <div class="general-tab">
    <!-- Driver info banner -->
    <NAlert v-if="driver" type="info" :bordered="false" class="drv-banner">
      <template #header>
        <span class="drv-tag" :class="`drv-${driver.driver_kind}`">{{ driver.driver_kind }}</span>
        {{ driver.name }}
      </template>
      <template #default>
        {{ driver.version ? `v${driver.version} · ` : '' }}{{ driver.is_file ? $t('navigator.fileDbHint') : $t('navigator.networkDbHint') }}
      </template>
    </NAlert>

    <div v-if="!driver" class="empty-hint">{{ $t('navigator.noDriver') }}</div>

    <template v-else>
      <!-- Network DB form -->
      <NSpace v-if="!driver.is_file" vertical :size="14">
        <div class="sec-title">{{ $t('navigator.connectionParams') }}</div>
        <div class="form-row">
          <div class="form-grp" style="flex:2">
            <span class="form-label">{{ $t('navigator.host') }}</span>
            <NInput v-model:value="local.host" size="small" placeholder="localhost" @update:value="emitUpdate" />
          </div>
          <div class="form-grp" style="flex:1">
            <span class="form-label">{{ $t('navigator.port') }}</span>
            <NInputNumber v-model:value="local.port" size="small" :min="1" :max="65535" @update:value="emitUpdate" />
          </div>
        </div>
        <div class="form-row">
          <div class="form-grp" style="flex:1">
            <span class="form-label">{{ $t('navigator.database') }}</span>
            <NInput v-model:value="local.database" size="small" placeholder="mydb" @update:value="emitUpdate" />
          </div>
        </div>

        <!-- Database Auth: Two-column layout -->
        <div class="sec-title" style="margin-top:4px">{{ $t('navigator.databaseAuth') || '数据库认证' }}</div>
        <div class="form-row auth-two-col">
          <div class="form-grp" style="flex:1">
            <span class="form-label">{{ $t('navigator.authMethod') || '认证方法' }}</span>
            <NSelect
              v-model:value="authMethod"
              size="small"
              :options="authMethodOpts"
              :placeholder="$t('navigator.selectAuthMethod') || '选择认证方法'"
              @update:value="onAuthMethodChange"
            />
          </div>
          <div class="form-grp" style="flex:1.5">
            <span class="form-label">{{ $t('navigator.savedAuthConfig') || '已保存的认证配置' }}</span>
            <div class="auth-cfg-row">
              <NSelect
                v-model:value="selectedAuthConfigId"
                size="small"
                :options="filteredAuthConfigOpts"
                :placeholder="$t('navigator.selectAuthConfig') || '使用已保存配置'"
                class="auth-cfg-select"
                filterable
                clearable
                @update:value="onAuthConfigSelect"
              />
              <NButton size="tiny" quaternary :title="$t('navigator.manageAuth') || '管理认证配置'" @click="showAuthManager = true">
                📋
              </NButton>
            </div>
          </div>
        </div>

        <!-- Dynamic auth fields -->
        <template v-if="!selectedAuthConfigId">
          <div v-if="authMethod === 'password'" class="form-row">
            <div class="form-grp" style="flex:1">
              <span class="form-label">{{ $t('navigator.username') }}</span>
              <NInput v-model:value="local.username" size="small" placeholder="root" @update:value="emitUpdate" />
            </div>
            <div class="form-grp" style="flex:1">
              <span class="form-label">{{ $t('navigator.password') }}</span>
              <NInput v-model:value="local.password" type="password" size="small" show-password-on="click" placeholder="****" @update:value="emitUpdate" />
            </div>
          </div>
          <div v-else-if="authMethod === 'pg_class'" class="form-row">
            <div class="form-grp" style="flex:1">
              <span class="form-label">{{ $t('navigator.clientCert') || '客户端证书 (.crt)' }}</span>
              <div class="file-input-row">
                <NInput v-model:value="local.certPath" size="small" placeholder="~/client.crt" @update:value="emitUpdate" />
                <NButton size="tiny" secondary @click="browseCert">
                  📂
                </NButton>
              </div>
            </div>
            <div class="form-grp" style="flex:1">
              <span class="form-label">{{ $t('navigator.clientKey') || '私钥 (.key)' }}</span>
              <div class="file-input-row">
                <NInput v-model:value="local.certKeyPath" size="small" placeholder="~/client.key" @update:value="emitUpdate" />
                <NButton size="tiny" secondary @click="browseCertKey">
                  📂
                </NButton>
              </div>
            </div>
          </div>
          <div v-else-if="authMethod === 'kerberos'" class="form-row">
            <div class="form-grp" style="flex:1">
              <span class="form-label">{{ $t('navigator.principal') || 'Principal' }}</span>
              <NInput v-model:value="local.principal" size="small" placeholder="user@REALM.COM" @update:value="emitUpdate" />
            </div>
            <div class="form-grp" style="flex:1">
              <span class="form-label">{{ $t('navigator.keytabPath') || 'Keytab 文件' }}</span>
              <div class="file-input-row">
                <NInput v-model:value="local.keytabPath" size="small" placeholder="/etc/krb5.keytab" @update:value="emitUpdate" />
                <NButton size="tiny" secondary @click="browseKeytab">
                  📂
                </NButton>
              </div>
            </div>
          </div>
          <div v-else-if="authMethod === 'oauth2'" class="form-row">
            <div class="form-grp" style="flex:1">
              <span class="form-label">{{ $t('navigator.tokenEndpoint') || 'Token 端点' }}</span>
              <NInput v-model:value="local.tokenEndpoint" size="small" placeholder="https://auth.example.com/token" @update:value="emitUpdate" />
            </div>
            <div class="form-grp" style="flex:1">
              <span class="form-label">{{ $t('navigator.clientId') || 'Client ID' }}</span>
              <NInput v-model:value="local.clientId" size="small" placeholder="your-client-id" @update:value="emitUpdate" />
            </div>
          </div>
          <div v-if="authMethod === 'oauth2'" class="form-row">
            <div class="form-grp" style="flex:1">
              <span class="form-label">{{ $t('navigator.clientSecret') || 'Client Secret' }}</span>
              <NInput v-model:value="local.clientSecret" type="password" size="small" show-password-on="click" placeholder="****" @update:value="emitUpdate" />
            </div>
          </div>
        </template>

        <!-- Pre-filled from auth config -->
        <div v-else class="form-row">
          <div class="form-grp" style="flex:1">
            <span class="form-label">{{ $t('navigator.username') }}</span>
            <NInput v-model:value="local.username" size="small" disabled />
          </div>
          <div v-if="authMethod === 'password'" class="form-grp" style="flex:1">
            <span class="form-label">{{ $t('navigator.password') }}</span>
            <NInput v-model:value="local.password" type="password" size="small" disabled show-password-on="click" />
          </div>
          <div class="form-config-badge">
            🔐 {{ $t('navigator.credentialsFromAuth') || '凭据来自认证配置' }}
          </div>
        </div>
        <div v-if="selectedAuthConfigId && authMethod === 'pg_class'" class="form-row">
          <div class="form-grp" style="flex:1">
            <span class="form-label">{{ $t('navigator.clientCert') || '客户端证书' }}</span>
            <NInput v-model:value="local.certPath" size="small" disabled />
          </div>
          <div class="form-grp" style="flex:1">
            <span class="form-label">{{ $t('navigator.clientKey') || '私钥' }}</span>
            <NInput v-model:value="local.certKeyPath" size="small" disabled />
          </div>
        </div>
        <div v-if="selectedAuthConfigId && authMethod === 'kerberos'" class="form-row">
          <div class="form-grp" style="flex:1">
            <span class="form-label">{{ $t('navigator.principal') || 'Principal' }}</span>
            <NInput v-model:value="local.principal" size="small" disabled />
          </div>
          <div class="form-grp" style="flex:1">
            <span class="form-label">{{ $t('navigator.keytabPath') || 'Keytab' }}</span>
            <NInput v-model:value="local.keytabPath" size="small" disabled />
          </div>
        </div>
        <div v-if="selectedAuthConfigId && authMethod === 'oauth2'" class="form-row">
          <div class="form-grp" style="flex:1">
            <span class="form-label">{{ $t('navigator.tokenEndpoint') || 'Token 端点' }}</span>
            <NInput v-model:value="local.tokenEndpoint" size="small" disabled />
          </div>
          <div class="form-grp" style="flex:1">
            <span class="form-label">{{ $t('navigator.clientId') || 'Client ID' }}</span>
            <NInput v-model:value="local.clientId" size="small" disabled />
          </div>
        </div>
      </NSpace>

      <!-- File DB form -->
      <NSpace v-else vertical :size="14">
        <div class="sec-title">{{ $t('navigator.databaseFile') }}</div>
        <div class="form-row">
          <div class="form-grp" style="flex:1">
            <span class="form-label">{{ $t('navigator.filePath') }}</span>
            <NSpace :size="8">
              <NInput v-model:value="local.file_path" size="small" :placeholder="filePathPlaceholder" style="flex:1" @update:value="emitUpdate" />
              <NButton size="small" secondary @click="browseFile">{{ $t('navigator.browse') }}</NButton>
              <NButton size="small" secondary class="btn-new-file" @click="createNewDbFile">{{ $t('navigator.newFile') || '新建' }}</NButton>
            </NSpace>
          </div>
        </div>
      </NSpace>
    </template>

    <!-- Auth Config Manager overlay -->
    <AuthConfigManager
      v-if="showAuthManager"
      @close="onAuthManagerClose"
      @select="onAuthConfigExternalSelect"
    />
  </div>
</template>

<script setup lang="ts">
import { NAlert, NButton, NInput, NInputNumber, NSelect, NSpace } from 'naive-ui'
import { reactive, ref, computed, watch, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'

import AuthConfigManager from '../AuthConfigManager.vue'

import type { Driver } from '../../../domain/types'

/** 认证配置数据模型 — 由后端 API (list_auth_configs / snapshot_global_auth) 提供 */
interface AuthConfig {
  id: string
  name: string
  authType: string
  scope: 'global' | 'project'
  username?: string
  password?: string
  certPath?: string
  certKeyPath?: string
  principal?: string
  keytabPath?: string
  tokenEndpoint?: string
  clientId?: string
  clientSecret?: string
}

/** Backend raw shape — snake_case + auth_data JSON */
interface BackendAuthConfig {
  id: string
  name: string | null
  auth_type: string
  auth_data: string
  origin: string | null
  created_at: string
  updated_at: string
}

function parseAuthConfig(raw: BackendAuthConfig): AuthConfig {
  let data: Record<string, unknown> = {}
  try { data = JSON.parse(raw.auth_data || '{}') } catch { /* ignore */ }
  return {
    id: raw.id,
    name: raw.name || '',
    authType: raw.auth_type,
    scope: (raw.origin === 'global' ? 'global' : 'project') as 'global' | 'project',
    username: data.username as string | undefined,
    password: data.password as string | undefined,
    certPath: data.certPath as string | undefined,
    certKeyPath: data.certKeyPath as string | undefined,
    principal: data.principal as string | undefined,
    keytabPath: data.keytabPath as string | undefined,
    tokenEndpoint: data.tokenEndpoint as string | undefined,
    clientId: data.clientId as string | undefined,
    clientSecret: data.clientSecret as string | undefined,
  }
}

interface Props {
  driver: Driver | null
  formData: Record<string, unknown>
}

const props = withDefaults(defineProps<Props>(), {
  driver: null,
  formData: () => ({}),
})

const emit = defineEmits<{
  'update:form-data': [data: Record<string, unknown>]
  'auth-config-change': [authConfigId: string | null, authMethod: string]
}>()

const { t } = useI18n()

interface LocalForm {
  host: string
  port: number
  database: string
  username: string
  password: string
  file_path: string
  certPath: string
  certKeyPath: string
  principal: string
  keytabPath: string
  tokenEndpoint: string
  clientId: string
  clientSecret: string
}

const local = reactive<LocalForm>({
  host: '',
  port: 0,
  database: '',
  username: '',
  password: '',
  file_path: '',
  certPath: '',
  certKeyPath: '',
  principal: '',
  keytabPath: '',
  tokenEndpoint: '',
  clientId: '',
  clientSecret: '',
})

// Auth state
const authMethod = ref('password')
const selectedAuthConfigId = ref<string | null>(null)
const showAuthManager = ref(false)

// Auth configs — loaded from backend API via loadAuthConfigs()
const authConfigs = ref<AuthConfig[]>([])

const filePathPlaceholder = computed(() => {
  if (props.driver?.name?.toLowerCase().includes('duckdb')) return '~/data.duckdb'
  return '~/data.db'
})

// Auth method options (database auth only - real PostgreSQL/MySQL auth method names)
const authMethodOpts = computed(() => [
  { label: '🔑 SCRAM-SHA-256 / mysql_native_password', value: 'password' },
  { label: '📜 SSL 客户端证书 (mTLS)', value: 'pg_class' },
  { label: '🎫 GSSAPI Kerberos', value: 'kerberos' },
  { label: '🔗 OAuth 2.0 Bearer Token', value: 'oauth2' },
])

// Filter saved auth configs by current auth method
const filteredAuthConfigOpts = computed(() => {
  const configs = authConfigs.value.filter(ac => ac.authType === authMethod.value)
  if (configs.length === 0) return []
  return [
    { label: t('navigator.noSavedConfig') || '— 手动填写 —', value: '' },
    ...configs.map(ac => ({
      label: `${ac.name} · ${ac.scope === 'global' ? '🌐' : '📝'}`,
      value: ac.id,
    })),
  ]
})

function emitUpdate() {
  emit('update:form-data', {
    ...local,
    authMethod: authMethod.value,
    selectedAuthConfigId: selectedAuthConfigId.value,
  })
}

function onAuthMethodChange() {
  selectedAuthConfigId.value = null
  emitUpdate()
}

function onAuthConfigSelect(configId: string | null) {
  if (!configId) {
    // User selected "不使用已保存配置"
    selectedAuthConfigId.value = null
    // 清空认证字段以便手动填写
    local.username = ''
    local.password = ''
    local.certPath = ''
    local.certKeyPath = ''
    local.principal = ''
    local.keytabPath = ''
    local.tokenEndpoint = ''
    local.clientId = ''
    local.clientSecret = ''
    emitUpdate()
    return
  }

  const config = authConfigs.value.find(ac => ac.id === configId)
  if (!config) return

  selectedAuthConfigId.value = configId
  // 同步认证方法类型
  authMethod.value = config.authType
  // 预填字段
  if (config.username) local.username = config.username
  if (config.password) local.password = config.password
  if (config.certPath) local.certPath = config.certPath
  if (config.certKeyPath) local.certKeyPath = config.certKeyPath
  if (config.principal) local.principal = config.principal
  if (config.keytabPath) local.keytabPath = config.keytabPath
  if (config.tokenEndpoint) local.tokenEndpoint = config.tokenEndpoint
  if (config.clientId) local.clientId = config.clientId
  if (config.clientSecret) local.clientSecret = config.clientSecret

  emit('auth-config-change', configId, config.authType)
  emitUpdate()
}

/** Called when AuthConfigManager fires a select event */
function onAuthConfigExternalSelect(configId: string) {
  showAuthManager.value = false
  onAuthConfigSelect(configId)
}

/** Refresh auth config list after AuthConfigManager operations */
async function onAuthManagerClose() {
  showAuthManager.value = false
  try {
    const { invoke } = await import('@tauri-apps/api/core')
    authConfigs.value = await invoke<AuthConfig[]>('list_auth_configs')
  } catch { /* 静默降级 */ }
}

async function browseFile() {
  try {
    const { open } = await import('@tauri-apps/plugin-dialog')
    const file = await open({
      filters: [{ name: 'Database', extensions: ['db', 'sqlite', 'sqlite3', 'duckdb'] }],
    })
    if (file) {
      local.file_path = file as string
      emitUpdate()
    }
  } catch {
    /* dialog not available in browser */
  }
}

async function browseCert() {
  try {
    const { open } = await import('@tauri-apps/plugin-dialog')
    const file = await open({
      filters: [{ name: 'Certificate', extensions: ['crt', 'pem', 'cert'] }],
    })
    if (file) {
      local.certPath = file as string
      emitUpdate()
    }
  } catch { /* browser fallback */ }
}

async function browseCertKey() {
  try {
    const { open } = await import('@tauri-apps/plugin-dialog')
    const file = await open({
      filters: [{ name: 'Private Key', extensions: ['key', 'pem'] }],
    })
    if (file) {
      local.certKeyPath = file as string
      emitUpdate()
    }
  } catch { /* browser fallback */ }
}

async function browseKeytab() {
  try {
    const { open } = await import('@tauri-apps/plugin-dialog')
    const file = await open({
      filters: [{ name: 'Keytab', extensions: ['keytab'] }],
    })
    if (file) {
      local.keytabPath = file as string
      emitUpdate()
    }
  } catch { /* browser fallback */ }
}

function createNewDbFile() {
  const ext = props.driver?.name?.toLowerCase().includes('duckdb') ? 'duckdb' : 'db'
  const defaultName = `new_database.${ext}`
  const newPath = prompt(
    `${t('navigator.newDbFilePrompt') || '新建数据库文件\n请输入文件路径（已存在则复用，不存在则自动创建）：'}`,
    local.file_path || defaultName,
  )
  if (newPath) {
    local.file_path = newPath
    emitUpdate()
  }
}

// Sync from props.formData on creation
onMounted(async () => {
  if (props.formData) {
    local.host = String(props.formData.host ?? '')
    local.port = Number(props.formData.port ?? props.driver?.default_port ?? 0)
    local.database = String(props.formData.database ?? '')
    local.username = String(props.formData.username ?? '')
    local.password = String(props.formData.password ?? '')
    local.file_path = String(props.formData.file_path ?? '')
    if (props.formData.authMethod) authMethod.value = String(props.formData.authMethod)
    if (props.formData.selectedAuthConfigId) selectedAuthConfigId.value = String(props.formData.selectedAuthConfigId)
  } else if (props.driver?.default_port) {
    local.port = props.driver.default_port
  }

  // 从后端加载已保存的认证配置列表
  try {
    const { invoke } = await import('@tauri-apps/api/core')
    const configs = await invoke<BackendAuthConfig[]>('list_auth_configs')
    authConfigs.value = configs.map(parseAuthConfig)
  } catch {
    // API 不可用时静默降级，authConfigs 保持空数组
  }
})

// Reset port when driver changes
watch(
  () => props.driver?.id,
  () => {
    local.port = props.driver?.default_port ?? 0
    emitUpdate()
  },
)
</script>

<style scoped>
.general-tab{display:flex;flex-direction:column;gap:16px;padding:4px 0}
.drv-banner{border-radius:6px}
.drv-tag{font-size:10px;padding:2px 8px;border-radius:4px;font-weight:600;margin-right:6px}
.drv-tag.drv-native{background:rgba(255,255,255,.06);color:var(--color-text-secondary)}
.drv-tag.drv-jdbc{background:rgba(244,102,35,.15);color:var(--driver-jdbc)}
.drv-tag.drv-python{background:rgba(55,118,171,.15);color:var(--driver-python)}
.drv-tag.drv-js{background:rgba(247,223,30,.15);color:var(--driver-js)}
.empty-hint{display:flex;align-items:center;justify-content:center;height:120px;font-size:13px;color:var(--color-text-muted)}
.sec-title{font-size:11px;font-weight:700;text-transform:uppercase;color:var(--color-text-muted);letter-spacing:.5px}
.form-row{display:flex;gap:12px}
.form-grp{display:flex;flex-direction:column;gap:4px}
.form-label{font-size:12px;color:var(--color-text-secondary);font-weight:500}

/* Auth two-column */
.auth-two-col{align-items:flex-start}
.auth-cfg-row{display:flex;gap:6px;align-items:center}
.auth-cfg-select{flex:1}

/* File input row */
.file-input-row{display:flex;gap:6px;align-items:center}
.file-input-row :first-child{flex:1}

/* Auth config badge */
.form-config-badge{
  display:flex;align-items:center;gap:4px;
  font-size:11px;color:var(--brand-accent);padding:4px 8px;
  background:var(--brand-accent-soft);border-radius:var(--border-radius-sm)
}

/* New file button */
.btn-new-file{white-space:nowrap}
</style>