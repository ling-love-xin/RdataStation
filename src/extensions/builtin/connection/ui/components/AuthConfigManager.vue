<template>
  <div class="auth-manager-overlay" @click.self="$emit('close')">
    <div class="auth-manager-dialog">
      <!-- Header -->
      <div class="am-header">
        <h3 class="am-title">
          <Shield :size="16" />
          {{ t('navigator.authConfigManager') || '认证配置管理器' }}
        </h3>
        <NButton text size="small" @click="$emit('close')">
          <template #icon><X :size="16" /></template>
        </NButton>
      </div>

      <!-- Tabs -->
      <div class="am-tabs">
        <button class="am-tab" :class="{ active: amTab === 'database' }" @click="amTab = 'database'">
          📊 {{ t('navigator.databaseAuth') || '数据库认证' }}
          <span class="am-badge">{{ dbAuthConfigs.length }}</span>
        </button>
        <button class="am-tab" :class="{ active: amTab === 'ssh' }" @click="amTab = 'ssh'">
          🖥 {{ t('navigator.sshAuth') || 'SSH 认证' }}
          <span class="am-badge">{{ sshAuthConfigs.length }}</span>
        </button>
      </div>

      <!-- Content -->
      <div class="am-content">
        <!-- Add new form toggle -->
        <div v-if="!showAddForm" class="am-toolbar">
          <NButton size="small" type="primary" ghost @click="openAddForm">
            + {{ t('navigator.newAuthConfig') || '新建认证配置' }}
          </NButton>
        </div>

        <!-- Add / Edit form -->
        <div v-if="showAddForm" class="am-form-box">
          <div class="am-form-row">
            <div class="am-fg" style="flex:1">
              <span class="am-fl">{{ t('navigator.name') }}</span>
              <NInput v-model:value="newCfg.name" size="small" :placeholder="t('navigator.dataSourceNamePlaceholder')" />
            </div>
            <div class="am-fg" style="flex:1">
              <span class="am-fl">{{ t('navigator.scope') || '范围' }}</span>
              <NSelect v-model:value="newCfg.scope" size="small" :options="scopeOpts" />
            </div>
          </div>
          <div class="am-form-row">
            <div class="am-fg" style="flex:1">
              <span class="am-fl">{{ t('navigator.authType') || '认证类型' }}</span>
              <NSelect
                v-model:value="newCfg.authType"
                size="small"
                :options="amTab === 'database' ? dbAuthTypeOpts : sshAuthTypeOpts"
                @update:value="onNewCfgTypeChange"
              />
            </div>
            <div v-if="needsUsername(newCfg.authType)" class="am-fg" style="flex:1">
              <span class="am-fl">{{ t('navigator.username') }}</span>
              <NInput v-model:value="newCfg.username" size="small" placeholder="root" />
            </div>
          </div>
          <!-- Dynamic extra fields -->
          <div v-if="newCfg.authType === 'password'" class="am-form-row">
            <div class="am-fg" style="flex:1">
              <span class="am-fl">{{ t('navigator.password') }}</span>
              <NInput v-model:value="newCfg.password" type="password" size="small" show-password-on="click" placeholder="****" />
            </div>
          </div>
          <div v-if="newCfg.authType === 'pg_class'" class="am-form-row">
            <div class="am-fg" style="flex:1">
              <span class="am-fl">{{ t('navigator.clientCert') || '客户端证书 (.crt)' }}</span>
              <NInput v-model:value="newCfg.certPath" size="small" placeholder="~/client.crt" />
            </div>
            <div class="am-fg" style="flex:1">
              <span class="am-fl">{{ t('navigator.clientKey') || '私钥 (.key)' }}</span>
              <NInput v-model:value="newCfg.certKeyPath" size="small" placeholder="~/client.key" />
            </div>
          </div>
          <div v-if="newCfg.authType === 'kerberos'" class="am-form-row">
            <div class="am-fg" style="flex:1">
              <span class="am-fl">{{ t('navigator.principal') || 'Principal' }}</span>
              <NInput v-model:value="newCfg.principal" size="small" placeholder="user@REALM.COM" />
            </div>
            <div class="am-fg" style="flex:1">
              <span class="am-fl">{{ t('navigator.keytabPath') || 'Keytab 文件' }}</span>
              <NInput v-model:value="newCfg.keytabPath" size="small" placeholder="/etc/krb5.keytab" />
            </div>
          </div>
          <div v-if="newCfg.authType === 'ssh_private_key'" class="am-form-row">
            <div class="am-fg" style="flex:1">
              <span class="am-fl">{{ t('navigator.privateKeyPath') || '私钥路径' }}</span>
              <NInput v-model:value="newCfg.keytabPath" size="small" placeholder="~/.ssh/id_rsa" />
            </div>
            <div class="am-fg" style="flex:1">
              <span class="am-fl">Passphrase</span>
              <NInput v-model:value="newCfg.passphrase" type="password" size="small" placeholder="可选" />
            </div>
          </div>
          <div class="am-form-row" style="margin-top:4px">
            <NButton size="tiny" type="primary" @click="saveNewCfg">{{ editingId ? (t('navigator.update') || '更新') : (t('navigator.save')) }}</NButton>
            <NButton size="tiny" @click="cancelEdit">{{ t('navigator.cancel') }}</NButton>
          </div>
        </div>

        <!-- Config list -->
        <div v-if="currentConfigs.length === 0 && !showAddForm" class="am-empty">
          {{ t('navigator.noAuthConfigs') || '暂无认证配置，点击上方按钮新建' }}
        </div>
        <div
          v-for="cfg in currentConfigs"
          :key="cfg.id"
          class="am-card"
        >
          <div class="am-card-info">
            <span class="am-card-type">{{ authTypeDef(cfg.authType)?.icon || '🔒' }} {{ authTypeDef(cfg.authType)?.label || cfg.authType }}</span>
            <span class="am-card-name">{{ cfg.name }}</span>
            <span class="am-card-detail">
              <template v-if="cfg.username">{{ cfg.username }} · </template>
              <template v-if="cfg.authType === 'pg_class'">{{ (cfg.certPath || '').split('/').pop() }} </template>
              <template v-if="cfg.authType === 'kerberos'">{{ cfg.principal }} </template>
            </span>
            <span class="am-card-scope">{{ cfg.scope === 'global' ? '🌐' : '📝' }}</span>
          </div>
          <div class="am-card-actions">
            <NButton text size="tiny" title="编辑" @click="editCfg(cfg)">
              <template #icon><Edit2 :size="14" /></template>
            </NButton>
            <NButton text size="tiny" title="删除" type="error" @click="deleteCfg(cfg.id)">
              <template #icon><Trash2 :size="14" /></template>
            </NButton>
            <NButton size="tiny" secondary title="应用到数据源" @click="$emit('select', cfg.id)">
              {{ t('navigator.apply') || '应用' }}
            </NButton>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { Shield, X, Edit2, Trash2 } from 'lucide-vue-next'
import { NButton, NInput, NSelect } from 'naive-ui'
import { ref, computed } from 'vue'
import { useI18n } from 'vue-i18n'

defineEmits<{
  close: []
  select: [configId: string]
}>()

const { t } = useI18n()

// Auth type definitions (matches prototype AUTH_TYPE_DEFS)
interface AuthTypeDef {
  category: 'database' | 'ssh'
  icon: string
  label: string
  fields: string[]
}

const AUTH_TYPE_DEFS: Record<string, AuthTypeDef> = {
  password:         { category: 'database', icon: '🔑', label: 'SCRAM-SHA-256 / mysql_native_password', fields: ['username', 'password'] },
  pg_class:         { category: 'database', icon: '📜', label: 'SSL 客户端证书 (mTLS)', fields: ['certPath', 'certKeyPath'] },
  kerberos:         { category: 'database', icon: '🎫', label: 'GSSAPI Kerberos', fields: ['principal', 'keytabPath'] },
  oauth2:           { category: 'database', icon: '🔗', label: 'OAuth 2.0 Bearer Token', fields: ['tokenEndpoint', 'clientId', 'clientSecret'] },
  ssh_password:     { category: 'ssh', icon: '🔑', label: 'SSH 密码认证', fields: ['username', 'password'] },
  ssh_private_key:  { category: 'ssh', icon: '🔐', label: 'SSH 公钥认证 (RSA/ED25519/ECDSA)', fields: ['username', 'keyPath', 'passphrase'] },
}

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
  passphrase?: string
  createdAt: string
}

/** Shared demo configs - will be replaced by API */
const allConfigs = ref<AuthConfig[]>([
  { id: 'auth-001', name: '生产 MySQL 认证', authType: 'password', scope: 'global', username: 'prod_admin', password: 'enc_pwd1', createdAt: '2026-05-01' },
  { id: 'auth-002', name: '开发 PG 认证', authType: 'password', scope: 'global', username: 'dev_user', password: 'enc_pwd2', createdAt: '2026-05-10' },
  { id: 'auth-003', name: 'SA 账户', authType: 'password', scope: 'project', username: 'sa', password: 'enc_sa', createdAt: '2026-05-12' },
  { id: 'auth-004', name: 'PG mTLS 证书认证', authType: 'pg_class', scope: 'global', certPath: '/certs/pg_client.crt', certKeyPath: '/certs/pg_client.key', createdAt: '2026-05-15' },
  { id: 'auth-005', name: 'GSSAPI Kerberos', authType: 'kerberos', scope: 'global', principal: 'pgadmin@REALM.COM', keytabPath: '/etc/krb5.keytab', createdAt: '2026-05-16' },
  { id: 'auth-007', name: '跳板机 SSH 密码', authType: 'ssh_password', scope: 'global', username: 'bastion_admin', password: 'ssh_enc', createdAt: '2026-05-18' },
  { id: 'auth-008', name: '跳板机 RSA 密钥', authType: 'ssh_private_key', scope: 'global', username: 'deployer', keytabPath: '~/.ssh/id_rsa', createdAt: '2026-05-18' },
  { id: 'auth-009', name: '开发机 ED25519', authType: 'ssh_private_key', scope: 'global', username: 'devops', keytabPath: '~/.ssh/id_ed25519', createdAt: '2026-05-19' },
])

// UI State
const amTab = ref<'database' | 'ssh'>('database')
const showAddForm = ref(false)
const editingId = ref<string | null>(null)
const newCfg = ref({
  name: '',
  authType: 'password',
  scope: 'global' as 'global' | 'project',
  username: '',
  password: '',
  certPath: '',
  certKeyPath: '',
  principal: '',
  keytabPath: '',
  tokenEndpoint: '',
  clientId: '',
  clientSecret: '',
  passphrase: '',
})

const scopeOpts = [
  { label: '🌐 全局', value: 'global' },
  { label: '📝 项目', value: 'project' },
]

const dbAuthTypeOpts = [
  { label: '🔑 SCRAM-SHA-256 / mysql_native_password', value: 'password' },
  { label: '📜 SSL 客户端证书 (mTLS)', value: 'pg_class' },
  { label: '🎫 GSSAPI Kerberos', value: 'kerberos' },
  { label: '🔗 OAuth 2.0 Bearer Token', value: 'oauth2' },
]

const sshAuthTypeOpts = [
  { label: '🔑 密码认证', value: 'ssh_password' },
  { label: '🔐 公钥认证 (RSA/ED25519/ECDSA)', value: 'ssh_private_key' },
]

// Computed
const dbAuthConfigs = computed(() =>
  allConfigs.value.filter(c => AUTH_TYPE_DEFS[c.authType]?.category === 'database'),
)

const sshAuthConfigs = computed(() =>
  allConfigs.value.filter(c => AUTH_TYPE_DEFS[c.authType]?.category === 'ssh'),
)

const currentConfigs = computed(() =>
  amTab.value === 'database' ? dbAuthConfigs.value : sshAuthConfigs.value,
)

function authTypeDef(type: string): AuthTypeDef | undefined {
  return AUTH_TYPE_DEFS[type]
}

function needsUsername(type: string): boolean {
  const def = AUTH_TYPE_DEFS[type]
  return !!def?.fields?.includes('username')
}

// Actions
function onNewCfgTypeChange() {
  // Reset type-specific fields
  newCfg.value.username = ''
  newCfg.value.password = ''
  newCfg.value.certPath = ''
  newCfg.value.certKeyPath = ''
  newCfg.value.principal = ''
  newCfg.value.keytabPath = ''
  newCfg.value.tokenEndpoint = ''
  newCfg.value.clientId = ''
  newCfg.value.clientSecret = ''
  newCfg.value.passphrase = ''
}

function openAddForm() {
  showAddForm.value = true
  editingId.value = null
  newCfg.value = {
    name: '',
    authType: amTab.value === 'database' ? 'password' : 'ssh_password',
    scope: 'global',
    username: '',
    password: '',
    certPath: '',
    certKeyPath: '',
    principal: '',
    keytabPath: '',
    tokenEndpoint: '',
    clientId: '',
    clientSecret: '',
    passphrase: '',
  }
}

function cancelEdit() {
  showAddForm.value = false
  editingId.value = null
}

function saveNewCfg() {
  if (!newCfg.value.name.trim()) return

  if (editingId.value) {
    // Update existing
    const idx = allConfigs.value.findIndex(c => c.id === editingId.value)
    if (idx >= 0) {
      allConfigs.value[idx] = {
        ...allConfigs.value[idx],
        name: newCfg.value.name,
        authType: newCfg.value.authType,
        scope: newCfg.value.scope,
        username: newCfg.value.username || undefined,
        password: newCfg.value.password || undefined,
        certPath: newCfg.value.certPath || undefined,
        certKeyPath: newCfg.value.certKeyPath || undefined,
        principal: newCfg.value.principal || undefined,
        keytabPath: newCfg.value.keytabPath || undefined,
        tokenEndpoint: newCfg.value.tokenEndpoint || undefined,
        clientId: newCfg.value.clientId || undefined,
        clientSecret: newCfg.value.clientSecret || undefined,
        passphrase: newCfg.value.passphrase || undefined,
      }
    }
  } else {
    // Create new
    const newId = `auth-${Date.now()}`
    allConfigs.value.push({
      id: newId,
      name: newCfg.value.name,
      authType: newCfg.value.authType,
      scope: newCfg.value.scope,
      username: newCfg.value.username || undefined,
      password: newCfg.value.password || undefined,
      certPath: newCfg.value.certPath || undefined,
      certKeyPath: newCfg.value.certKeyPath || undefined,
      principal: newCfg.value.principal || undefined,
      keytabPath: newCfg.value.keytabPath || undefined,
      tokenEndpoint: newCfg.value.tokenEndpoint || undefined,
      clientId: newCfg.value.clientId || undefined,
      clientSecret: newCfg.value.clientSecret || undefined,
      passphrase: newCfg.value.passphrase || undefined,
      createdAt: new Date().toISOString(),
    })
  }

  showAddForm.value = false
  editingId.value = null
}

function editCfg(cfg: AuthConfig) {
  editingId.value = cfg.id
  showAddForm.value = true
  amTab.value = AUTH_TYPE_DEFS[cfg.authType]?.category === 'ssh' ? 'ssh' : 'database'
  newCfg.value = {
    name: cfg.name,
    authType: cfg.authType,
    scope: cfg.scope,
    username: cfg.username || '',
    password: cfg.password || '',
    certPath: cfg.certPath || '',
    certKeyPath: cfg.certKeyPath || '',
    principal: cfg.principal || '',
    keytabPath: cfg.keytabPath || '',
    tokenEndpoint: cfg.tokenEndpoint || '',
    clientId: cfg.clientId || '',
    clientSecret: cfg.clientSecret || '',
    passphrase: cfg.passphrase || '',
  }
}

function deleteCfg(id: string) {
  const idx = allConfigs.value.findIndex(c => c.id === id)
  if (idx >= 0) allConfigs.value.splice(idx, 1)
}
</script>

<style scoped>
.auth-manager-overlay{
  position:fixed;top:0;left:0;width:100%;height:100%;
  background:rgba(0,0,0,.55);z-index:9999;
  display:flex;align-items:center;justify-content:center
}
.auth-manager-dialog{
  width:560px;max-height:72vh;
  background:var(--color-bg-primary);
  border:1px solid var(--color-border);
  border-radius:var(--border-radius-md);
  display:flex;flex-direction:column;
  overflow:hidden;
  box-shadow:0 8px 32px rgba(0,0,0,.4)
}
.am-header{
  display:flex;align-items:center;justify-content:space-between;
  padding:10px var(--spacing-md);
  border-bottom:1px solid var(--color-border);
  flex-shrink:0
}
.am-title{
  font-size:var(--font-size-md);font-weight:600;color:var(--color-text-primary);
  display:flex;align-items:center;gap:8px;margin:0
}
.am-tabs{
  display:flex;border-bottom:1px solid var(--color-border-subtle);flex-shrink:0
}
.am-tab{
  flex:1;display:flex;align-items:center;justify-content:center;gap:6px;
  padding:8px 12px;font-size:12px;font-weight:600;border:none;
  background:transparent;color:var(--color-text-secondary);
  cursor:pointer;border-bottom:2px solid transparent;
  transition:all .15s
}
.am-tab.active{
  color:var(--brand-accent);border-bottom-color:var(--brand-accent)
}
.am-tab:hover{background:var(--color-hover)}
.am-badge{
  font-size:10px;padding:1px 6px;border-radius:8px;
  background:var(--color-bg-elevated);color:var(--color-text-muted)
}
.am-content{
  flex:1;overflow-y:auto;padding:var(--spacing-md);
  display:flex;flex-direction:column;gap:8px
}
.am-toolbar{margin-bottom:4px}
.am-form-box{
  padding:12px;background:var(--color-bg-secondary);
  border:1px solid var(--color-border-subtle);
  border-radius:var(--border-radius-sm);
  display:flex;flex-direction:column;gap:8px
}
.am-form-row{display:flex;gap:12px}
.am-fg{display:flex;flex-direction:column;gap:4px}
.am-fl{font-size:11px;font-weight:600;color:var(--color-text-muted)}
.am-empty{
  text-align:center;padding:32px 0;font-size:12px;
  color:var(--color-text-muted)
}
.am-card{
  display:flex;align-items:center;justify-content:space-between;
  padding:10px 12px;background:var(--color-bg-secondary);
  border:1px solid var(--color-border-subtle);
  border-radius:var(--border-radius-sm)
}
.am-card-info{display:flex;flex-direction:column;gap:2px}
.am-card-type{font-size:11px;font-weight:600;color:var(--color-text-secondary)}
.am-card-name{font-size:13px;font-weight:600;color:var(--color-text-primary)}
.am-card-detail{font-size:11px;color:var(--color-text-muted)}
.am-card-scope{font-size:10px}
.am-card-actions{display:flex;align-items:center;gap:6px;flex-shrink:0}
</style>