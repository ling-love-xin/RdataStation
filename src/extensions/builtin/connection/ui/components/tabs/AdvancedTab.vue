<template>
  <div class="adv-tab">
    <div v-if="!driver" class="empty-hint">{{ $t('navigator.noDriver') }}</div>

    <template v-else>
      <!-- Environment selector -->
      <div class="adv-sec env-sec">
        <div class="env-row">
          <span class="sec-title">🏷 {{ $t('navigator.advancedEnv') }}</span>
          <NSelect
            v-model:value="envId"
            :options="envSelectOpts"
            size="small"
            class="env-select"
            @update:value="onEnvChange"
          />
          <NButton size="tiny" text @click="showEnvMgr = true">{{ $t('navigator.manage') }}</NButton>
        </div>
        <div v-if="envPolicyTags.length" class="env-tags">
          <span v-for="tag in envPolicyTags" :key="tag.key" :class="['epi-tag', tag.kind]">{{ tag.label }}</span>
        </div>
        <div v-if="!isPolicyOverridden" class="env-preset-indicator">
          ← 🟢 {{ currentEnvDef.name }} 预设
        </div>
        <div v-if="isPolicyOverridden" class="env-override-hint">⚠ {{ $t('navigator.envOverrideHint', { name: currentEnvDef.name }) }}</div>
      </div>

      <!-- DuckDB acceleration -->
      <div class="adv-sec">
        <div class="sec-title" style="color:var(--brand-warning)">
          ⚡ {{ $t('connection.advancedTab.localAccel') }} <span class="sec-sub">DuckDB</span>
        </div>
        <div :class="['accel-card', { on: duckdbEnabled }]">
          <div class="accel-row">
            <span class="accel-icon">🦆</span>
            <div class="accel-body">
              <span class="accel-name">{{ $t('connection.advancedTab.enableDuckdbAccel') }}</span>
              <span class="accel-desc">{{ $t('connection.advancedTab.accelDesc', { dbType: driver?.name || 'DB' }) }}</span>
            </div>
            <NSwitch v-model:value="duckdbEnabled" size="small" />
          </div>
          <div v-if="duckdbEnabled" class="accel-expand">
            <div class="accel-benefits">
              <span class="benefit-tag">🚀 大表分析</span>
              <span class="benefit-tag">🔗 跨库联邦</span>
              <span class="benefit-tag">📊 重复报表</span>
              <span class="benefit-tag">🗜 列式压缩</span>
            </div>
            <div class="accel-storage-info">
              <span class="storage-icon">📁</span>
              <span class="storage-path">存储路径: .rdata/duckdb/accel.duckdb</span>
              <span class="storage-hint">联邦查询中间结果缓存目录</span>
            </div>
            <div class="adv-grid" style="margin-top:10px">
              <div class="adv-cell">
                <span class="adv-lbl">{{ $t('connection.advancedTab.syncStrategy') }}</span>
                <NSelect v-model:value="duckdbSync" size="small" :options="syncOpts" />
              </div>
              <div class="adv-cell">
                <span class="adv-lbl">{{ $t('connection.advancedTab.syncInterval') }}</span>
                <NInputNumber v-model:value="duckdbInterval" size="small" :min="1" :max="1440" />
              </div>
              <div class="adv-cell">
                <span class="adv-lbl">{{ $t('connection.advancedTab.memoryLimit') }}</span>
                <NInputNumber v-model:value="duckdbMemory" size="small" :min="64" :max="8192" />
              </div>
              <div class="adv-cell">
                <span class="adv-lbl">{{ $t('connection.advancedTab.threads') }}</span>
                <NInputNumber v-model:value="duckdbThreads" size="small" :min="1" :max="16" />
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- Security policies -->
      <div class="adv-sec">
        <NCollapse>
          <NCollapseItem name="sec">
            <template #header>
              <span class="sec-title" style="margin-bottom:0">🔐 {{ $t('navigator.advancedSecurity') }}</span>
              <span class="collapse-summary">{{ $t('navigator.policySummary', { summary: securitySummary }) }}</span>
            </template>
            <div class="policy-grid">
              <div class="policy-row">
                <span class="pol-item">{{ $t('navigator.advancedReadOnly') }} <NSwitch v-model:value="polReadonly" size="small" @update:value="checkPolicyOverride" /></span>
                <span class="pol-item">{{ $t('navigator.writeConfirm') }} <NSwitch v-model:value="polWriteConfirm" size="small" @update:value="checkPolicyOverride" /></span>
                <span class="pol-item">{{ $t('navigator.advancedDdlConfirm') || 'DDL确认' }} <NSwitch v-model:value="polDdlConfirm" size="small" @update:value="checkPolicyOverride" /></span>
              </div>
              <div class="policy-row">
                <span class="pol-item">{{ $t('navigator.advancedDropOp') || 'DROP操作' }}
                  <NSelect v-model:value="polDrop" size="small" :options="dropOpts" style="width:80px" @update:value="checkPolicyOverride" />
                </span>
                <span class="pol-item">{{ $t('navigator.autoCommit') || '自动提交' }} <NSwitch v-model:value="polAutocommit" size="small" @update:value="checkPolicyOverride" /></span>
                <span class="pol-item">{{ $t('navigator.rowLimit') }} <NInputNumber v-model:value="polRowLimit" size="small" :min="0" style="width:80px" @update:value="checkPolicyOverride" /></span>
                <span class="pol-item">{{ $t('navigator.sizeLimit') }} <NInputNumber v-model:value="polSizeLimit" size="small" :min="0" :max="10240" style="width:80px" @update:value="checkPolicyOverride" /></span>
              </div>
            </div>
          </NCollapseItem>
        </NCollapse>
      </div>

      <!-- Connection params -->
      <div class="adv-sec">
        <div class="sec-title">{{ $t('navigator.connectionParams') }}</div>
        <div class="adv-grid">
          <div class="adv-cell">
            <span class="adv-lbl">{{ $t('navigator.advancedTimeout') }}</span>
            <NInputNumber v-model:value="advConnectTimeout" size="small" :min="1" :max="300" />
          </div>
          <div class="adv-cell">
            <span class="adv-lbl">{{ $t('navigator.advancedQueryTimeout') || '查询超时 (秒)' }}</span>
            <NInputNumber v-model:value="advQueryTimeout" size="small" :min="0" :max="3600" />
          </div>
          <div class="adv-cell">
            <span class="adv-lbl">{{ $t('navigator.keepAliveInterval') }}</span>
            <NInputNumber v-model:value="advHeartbeat" size="small" :min="10" :max="600" />
          </div>
          <div class="adv-cell">
            <span class="adv-lbl">{{ $t('navigator.advancedMaxReconnect') || '最大重连' }}</span>
            <NInputNumber v-model:value="advMaxReconnect" size="small" :min="0" :max="20" />
          </div>
        </div>
      </div>

      <!-- Schema + Encoding -->
      <div class="adv-sec">
        <div class="adv-inline">
          <div class="adv-cell" style="flex:2">
            <span class="adv-lbl">{{ $t('navigator.advancedSchema') }}</span>
            <NSelect v-model:value="schemaStrategy" size="small" :options="schemaOpts" />
          </div>
          <div class="adv-cell" style="flex:1;max-width:160px">
            <span class="adv-lbl">{{ $t('navigator.advancedEncoding') }}</span>
            <NSelect v-model:value="encoding" size="small" :options="encOpts" />
          </div>
        </div>
      </div>

      <!-- Env Manager Modal -->
      <NModal v-model:show="showEnvMgr" preset="card" :style="{ maxWidth: '560px' }" :title="$t('navigator.envManager') || '环境管理器'">
        <div class="env-mgr-list">
          <div v-if="envListLoading">{{ $t('dataPreview.loading') }}</div>
          <div
            v-for="env in loadedEnvs"
            :key="env.id"
            class="env-mgr-card"
            :style="{ borderLeft: `3px solid ${env.color}` }"
          >
            <div class="env-mgr-color-dot" :style="{ background: env.color }" />
            <div class="env-mgr-content">
              <div class="env-mgr-header">
                <span class="env-mgr-icon">{{ env.icon }}</span>
                <span class="env-mgr-name">{{ env.name }}</span>
                <span v-if="env.builtin" class="env-mgr-badge">{{ $t('navigator.builtinBadge') || '内置' }}</span>
              </div>
              <div class="env-mgr-desc">{{ env.desc }}</div>
              <div v-if="env.ui" class="env-mgr-meta">
                <span class="env-mgr-meta-lbl">UI:</span>
                <span class="env-mgr-meta-dot" :style="{ background: env.ui.summaryUI }"></span>
                <span class="env-mgr-meta-val">{{ env.ui.summaryUI }}</span>
              </div>
              <div class="env-policy-tags">
                <span class="policy-tag security">{{ env.summarySecurity }}</span>
                <span class="policy-tag schema">{{ env.summarySchema }}</span>
                <span class="policy-tag performance">{{ env.summaryPerf }}</span>
                <span class="policy-tag audit">{{ env.summaryAudit }}</span>
              </div>
            </div>
            <div class="env-mgr-actions">
              <NButton v-if="!env.builtin" size="tiny" type="error" text @click="handleDeleteEnv(env.id)">✕</NButton>
            </div>
          </div>
        </div>
        <!-- Create new environment form -->
        <div class="env-mgr-create-section">
          <NButton v-if="!showEnvCreateForm" size="small" dashed @click="showEnvCreateForm = true">+ {{ $t('navigator.createEnv') }}</NButton>
          <div v-else class="env-create-form">
            <div class="env-create-row">
              <label class="env-create-lbl">{{ $t('navigator.envName') }}</label>
              <NInput v-model:value="newEnvName" size="small" :placeholder="$t('navigator.envNamePlaceholder') || '输入环境名称'" />
            </div>
            <div class="env-create-row">
              <label class="env-create-lbl">{{ $t('navigator.envIcon') }}</label>
              <NInput v-model:value="newEnvIcon" size="small" placeholder="🟢" style="max-width:80px" />
              <label class="env-create-lbl" style="margin-left:12px">{{ $t('navigator.envColor') }}</label>
              <input v-model="newEnvColor" type="color" class="color-input" />
            </div>
            <div class="env-create-row">
              <label class="env-create-lbl">{{ $t('navigator.envDesc') }}</label>
              <NInput v-model:value="newEnvDesc" size="small" :placeholder="$t('navigator.envDescPlaceholder') || '输入环境描述'" />
            </div>
            <div class="env-create-row">
              <label class="env-create-lbl">{{ $t('navigator.envTemplate') }}</label>
              <NSelect v-model:value="newEnvTemplate" size="small" :options="envTemplateOpts" style="flex:1" />
            </div>
            <div class="env-create-actions">
              <NButton size="tiny" type="primary" @click="handleCreateEnv">{{ $t('common.save') }}</NButton>
              <NButton size="tiny" @click="showEnvCreateForm = false">{{ $t('common.cancel') }}</NButton>
            </div>
          </div>
        </div>
      </NModal>
    </template>
  </div>
</template>

<script setup lang="ts">
import {
  NCollapse, NCollapseItem, NSelect, NSwitch, NInputNumber, NButton, NModal, NInput,
} from 'naive-ui'
import { ref, computed, watch, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'

import type { Driver } from '../../../domain/types'
import type { SelectOption } from 'naive-ui'

const { t } = useI18n()
defineProps<{ driver?: Driver | null; formData?: Record<string, unknown> }>()

const emit = defineEmits<{
  'update:formData': [data: Record<string, unknown>]
  'extra-config': [config: Record<string, unknown>]
}>()

// ========== Environment ==========
const envId = ref('env-dev')
const showEnvMgr = ref(false)

const envSelectOpts = computed<SelectOption[]>(() =>
  envDefs.map(e => ({
    label: `${e.icon} ${e.name}`,
    value: e.id,
  }))
)

const currentEnvDef = computed<EnvDefItem>(() =>
  envDefs.find(e => e.id === envId.value) || envDefs[0]
)

interface EnvPolicyTag { key: string; label: string; kind: string }
interface EnvDefItem {
  id: string; name: string; color: string; icon: string; desc: string; builtin: boolean
  summarySecurity: string; summarySchema: string; summaryPerf: string; summaryAudit: string
  ui: { summaryUI: string }
  policy: { ro: boolean; wc: boolean; ddl: boolean; drop: string; ac: boolean; rl: number; sl: number }
}

const envDefs: EnvDefItem[] = [
  { id: 'env-dev', name: '开发环境', color: '#a6e3a1', icon: '🟢', desc: '本地开发、调试数据库', builtin: true,
    summarySecurity: '读写·自动提交', summarySchema: '自动Schema+系统表', summaryPerf: '池10·超时0s·重连3', summaryAudit: '无审计',
    ui: { summaryUI: '#a6e3a1' },
    policy: { ro: false, wc: false, ddl: false, drop: 'false', ac: true, rl: 0, sl: 0 } },
  { id: 'env-test', name: '测试环境', color: '#f9e2af', icon: '🟡', desc: '集成测试、QA 验证', builtin: true,
    summarySecurity: '读写·DDL确认·行限1w', summarySchema: '自动Schema+系统表', summaryPerf: '池10·超时120s·重连3', summaryAudit: '基础审计',
    ui: { summaryUI: '#f9e2af' },
    policy: { ro: false, wc: false, ddl: true, drop: 'true', ac: true, rl: 10000, sl: 100 } },
  { id: 'env-staging', name: '预发布', color: '#89b4fa', icon: '🔵', desc: '灰度验证、预发布环境', builtin: true,
    summarySecurity: '写确认·DDL确认·行限5k', summarySchema: '自动Schema', summaryPerf: '池15·超时180s·重连5', summaryAudit: '完整审计',
    ui: { summaryUI: '#89b4fa' },
    policy: { ro: false, wc: true, ddl: true, drop: 'true', ac: false, rl: 5000, sl: 50 } },
  { id: 'env-prod', name: '生产环境', color: '#f38ba8', icon: '🔴', desc: '线上生产数据库，谨慎操作', builtin: true,
    summarySecurity: '默认只读·写确认·DROP禁用', summarySchema: '按需Schema', summaryPerf: '池20·超时60s·重连3', summaryAudit: '全面审计',
    ui: { summaryUI: '#f38ba8' },
    policy: { ro: true, wc: true, ddl: true, drop: 'disable', ac: false, rl: 1000, sl: 20 } },
  { id: 'env-sandbox', name: '沙箱环境', color: '#cba6f7', icon: '🟣', desc: '安全隔离的沙箱数据库', builtin: true,
    summarySecurity: '读写·行限1k', summarySchema: '自动Schema', summaryPerf: '池5·超时60s·重连2', summaryAudit: '无审计',
    ui: { summaryUI: '#cba6f7' },
    policy: { ro: false, wc: false, ddl: false, drop: 'false', ac: true, rl: 1000, sl: 50 } },
]

const envPolicyTags = computed<EnvPolicyTag[]>(() => {
  const envMap: Record<string, EnvPolicyTag[]> = {
    'env-dev': [{ key: 'rw', label: '读写', kind: '' }],
    'env-test': [
      { key: 'rw', label: '读写', kind: '' },
      { key: 'ddl', label: 'DDL确认', kind: '' },
      { key: 'row', label: '行限10000', kind: '' },
    ],
    'env-staging': [
      { key: 'wc', label: '写确认', kind: 'locked' },
      { key: 'ddl', label: 'DDL确认', kind: 'locked' },
      { key: 'schema', label: '手动Schema', kind: '' },
      { key: 'row', label: '行限5000', kind: 'locked' },
      { key: 'audit', label: '审计', kind: 'audit' },
    ],
    'env-prod': [
      { key: 'ro', label: '默认只读', kind: 'danger' },
      { key: 'wc', label: '写确认', kind: 'locked' },
      { key: 'drop', label: 'DROP禁用', kind: 'danger' },
      { key: 'row', label: '行限1000', kind: 'locked' },
      { key: 'audit', label: '审计', kind: 'audit' },
    ],
    'env-sandbox': [
      { key: 'rw', label: '读写', kind: '' },
      { key: 'row', label: '行限1000', kind: '' },
    ],
  }
  return envMap[envId.value] || []
})

// ========== Security policy summary ==========
const securitySummary = computed(() => {
  const parts: string[] = []
  if (polReadonly.value) parts.push('只读')
  else parts.push('读写')
  if (polWriteConfirm.value) parts.push('写确认')
  if (polDdlConfirm.value) parts.push('DDL确认')
  if (polDrop.value === 'disable') parts.push('DROP禁用')
  else if (polDrop.value === 'true') parts.push('DROP确认')
  if (polRowLimit.value > 0) parts.push(`行限${polRowLimit.value}`)
  if (polSizeLimit.value > 0) parts.push(`限${polSizeLimit.value}M`)
  return parts.join('·') || '默认'
})

const isPolicyOverridden = computed(() => {
  const p = envDefs.find(e => e.id === envId.value)?.policy || null
  if (!p) return false
  if (polReadonly.value !== p.ro) return true
  if (polWriteConfirm.value !== p.wc) return true
  if (polDdlConfirm.value !== p.ddl) return true
  if (polDrop.value !== p.drop) return true
  if (polAutocommit.value !== p.ac) return true
  if (polRowLimit.value !== p.rl) return true
  if (polSizeLimit.value !== p.sl) return true
  return false
})

function checkPolicyOverride() {
  // isPolicyOverridden is computed, auto-updates
}

function onEnvChange(id: string) {
  const defaults: Record<string, Record<string, unknown>> = {
    'env-dev': { ro: false, wc: false, ddl: false, drop: 'false', ac: true, rl: 0, sl: 0, ct: 30, qt: 0, hb: 60, mr: 3 },
    'env-test': { ro: false, wc: false, ddl: true, drop: 'true', ac: true, rl: 10000, sl: 100, ct: 30, qt: 120, hb: 60, mr: 3 },
    'env-staging': { ro: false, wc: true, ddl: true, drop: 'true', ac: false, rl: 5000, sl: 50, ct: 30, qt: 180, hb: 60, mr: 5 },
    'env-prod': { ro: true, wc: true, ddl: true, drop: 'disable', ac: false, rl: 1000, sl: 20, ct: 15, qt: 60, hb: 30, mr: 3 },
    'env-sandbox': { ro: false, wc: false, ddl: false, drop: 'false', ac: true, rl: 1000, sl: 50, ct: 30, qt: 60, hb: 60, mr: 2 },
  }
  const d = defaults[id] || defaults['env-dev']
  tempDefaultLocked.value = true
  polReadonly.value = d.ro as boolean
  polWriteConfirm.value = d.wc as boolean
  polDdlConfirm.value = d.ddl as boolean
  polAutocommit.value = d.ac as boolean
  polDrop.value = d.drop as string
  polRowLimit.value = d.rl as number
  polSizeLimit.value = d.sl as number
  advConnectTimeout.value = d.ct as number
  advQueryTimeout.value = d.qt as number
  advHeartbeat.value = d.hb as number
  advMaxReconnect.value = d.mr as number
  schemaStrategy.value = id === 'env-prod' ? 'manual' : 'auto'
  setTimeout(() => { tempDefaultLocked.value = false }, 0)
}

// ========== DuckDB ==========
const duckdbEnabled = ref(false)
const duckdbSync = ref('auto')
const duckdbInterval = ref(15)
const duckdbMemory = ref(512)
const duckdbThreads = ref(4)
const syncOpts = [
  { label: t('connection.advancedTab.syncAuto'), value: 'auto' },
  { label: t('connection.advancedTab.syncScheduled'), value: 'scheduled' },
  { label: t('connection.advancedTab.syncManual'), value: 'manual' },
]

// ========== Security policies ==========
const polReadonly = ref(false)
const polWriteConfirm = ref(false)
const polDdlConfirm = ref(false)
const polAutocommit = ref(true)
const polDrop = ref('false')
const polRowLimit = ref(0)
const polSizeLimit = ref(0)
const tempDefaultLocked = ref(false)
const dropOpts = [
  { label: t('navigator.advancedDropAllow') || '允许', value: 'false' },
  { label: t('navigator.advancedDropConfirm') || '确认', value: 'true' },
  { label: t('navigator.advancedDropDisable') || '禁用', value: 'disable' },
]

// ========== Connection params ==========
const advConnectTimeout = ref(30)
const advQueryTimeout = ref(0)
const advHeartbeat = ref(60)
const advMaxReconnect = ref(3)

// ========== Schema + Encoding ==========
const schemaStrategy = ref('auto')
const encoding = ref('UTF-8')
const schemaOpts = [
  { label: t('connection.advancedTab.schemaAuto'), value: 'auto' },
  { label: t('connection.advancedTab.schemaManual'), value: 'manual' },
]
const encOpts = [
  { label: 'UTF-8', value: 'UTF-8' },
  { label: 'GBK', value: 'GBK' },
  { label: 'Latin-1', value: 'Latin-1' },
]

// ========== Load real environments from backend ==========
interface EnvInfo {
  id: string; name: string; color: string; icon: string; desc: string
  builtin: boolean
  summarySecurity: string; summarySchema: string; summaryPerf: string; summaryAudit: string
  ui: { summaryUI: string }
}

const loadedEnvs = ref<EnvInfo[]>([])
const envListLoading = ref(false)

// Env create form states
const showEnvCreateForm = ref(false)
const newEnvName = ref('')
const newEnvIcon = ref('🟢')
const newEnvColor = ref('#a6e3a1')
const newEnvDesc = ref('')
const newEnvTemplate = ref('dev')
const envTemplateOpts = [
  { label: '🟢 开发环境 (宽松)', value: 'dev' },
  { label: '🟡 测试环境 (适中)', value: 'test' },
  { label: '🔵 预发布 (较严)', value: 'staging' },
  { label: '🔴 生产环境 (最严)', value: 'prod' },
  { label: '🟣 沙箱环境 (隔离)', value: 'sandbox' },
]

async function handleCreateEnv() {
  const name = newEnvName.value.trim()
  if (!name) return
  try {
    const { invoke } = await import('@tauri-apps/api/core')
    await invoke('create_environment', {
      env: {
        name,
        icon: newEnvIcon.value || '🟢',
        color: newEnvColor.value || '#a6e3a1',
        description: newEnvDesc.value,
        templateId: `env-${newEnvTemplate.value}`,
      },
    })
    newEnvName.value = ''
    newEnvIcon.value = '🟢'
    newEnvColor.value = '#a6e3a1'
    newEnvDesc.value = ''
    newEnvTemplate.value = 'dev'
    showEnvCreateForm.value = false
    await loadEnvironments()
  } catch {
    const id = `env-custom-${Date.now()}`
    const template = envDefs.find(e => e.id === `env-${newEnvTemplate.value}`)
    const policy = template?.policy ? { ...template.policy } : { ro: false, wc: false, ddl: false, drop: 'false', ac: true, rl: 0, sl: 0 }
    loadedEnvs.value.push({
      id,
      name,
      color: newEnvColor.value || '#a6e3a1',
      icon: newEnvIcon.value || '🟢',
      desc: newEnvDesc.value,
      builtin: false,
      summarySecurity: '自定义',
      summarySchema: '自动',
      summaryPerf: '默认',
      summaryAudit: '自定义',
      ui: { summaryUI: newEnvColor.value || '#a6e3a1' },
    })
    newEnvName.value = ''
    newEnvIcon.value = '🟢'
    newEnvColor.value = '#a6e3a1'
    newEnvDesc.value = ''
    newEnvTemplate.value = 'dev'
    showEnvCreateForm.value = false
  }
}

async function handleDeleteEnv(id: string) {
  try {
    const { invoke } = await import('@tauri-apps/api/core')
    await invoke('delete_environment', { id })
    await loadEnvironments()
  } catch {
    loadedEnvs.value = loadedEnvs.value.filter(e => e.id !== id)
  }
}

async function loadEnvironments() {
  envListLoading.value = true
  try {
    const { invoke } = await import('@tauri-apps/api/core')
    const remote = await invoke<Array<{ id: string; name: string; description: string; properties?: Record<string, unknown> }>>('list_environments')
    if (remote && remote.length > 0) {
      const colors = ['#a6e3a1', '#f9e2af', '#89b4fa', '#f38ba8', '#cba6f7']
      const icons = ['🟢', '🟡', '🔵', '🔴', '🟣']
      loadedEnvs.value = remote.map((e, i) => ({
        id: e.id,
        name: e.name,
        color: colors[i % colors.length],
        icon: icons[i % icons.length],
        desc: e.description || '',
        builtin: false,
        summarySecurity: (e.properties?.security as string) || '默认',
        summarySchema: (e.properties?.schema as string) || '自动',
        summaryPerf: (e.properties?.performance as string) || '默认',
        summaryAudit: (e.properties?.audit as string) || '默认',
        ui: { summaryUI: (e.properties?.uiColor as string) || colors[i % colors.length] },
      }))
    }
  } catch {
    loadedEnvs.value = envDefs as EnvInfo[]
  } finally { envListLoading.value = false }
}

onMounted(() => { loadEnvironments() })

// ========== Emit extra config ==========
watch(
  [duckdbEnabled, duckdbSync, duckdbInterval, duckdbMemory, duckdbThreads,
   polReadonly, polWriteConfirm, polDdlConfirm, polAutocommit, polDrop, polRowLimit, polSizeLimit,
   advConnectTimeout, advQueryTimeout, advHeartbeat, advMaxReconnect,
   schemaStrategy, encoding, envId],
  () => {
    const opts = {
      envId: envId.value,
      duckdb: {
        enabled: duckdbEnabled.value,
        sync: duckdbSync.value,
        interval: duckdbInterval.value,
        memory: duckdbMemory.value,
        threads: duckdbThreads.value,
      },
      security: {
        readonly: polReadonly.value,
        writeConfirm: polWriteConfirm.value,
        ddlConfirm: polDdlConfirm.value,
        autocommit: polAutocommit.value,
        dropPolicy: polDrop.value,
        rowLimit: polRowLimit.value,
        sizeLimit: polSizeLimit.value,
      },
      connection: {
        connectTimeout: advConnectTimeout.value,
        queryTimeout: advQueryTimeout.value,
        keepAlive: advHeartbeat.value,
        maxReconnect: advMaxReconnect.value,
      },
      schemaStrategy: schemaStrategy.value,
      encoding: encoding.value,
    }
    emit('extra-config', { advancedOptions: JSON.stringify(opts) })
  },
  { deep: true }
)
</script>

<style scoped>
.adv-tab { display: flex; flex-direction: column; gap: 16px; padding: 4px 0; }
.empty-hint { display: flex; align-items: center; justify-content: center; height: 120px; font-size: 13px; color: var(--color-text-muted); }

.adv-sec { display: flex; flex-direction: column; gap: 8px; }
.sec-title { font-size: 11px; font-weight: 700; text-transform: uppercase; color: var(--color-text-muted); letter-spacing: 0.5px; display: flex; align-items: center; gap: 6px; }
.sec-sub { font-size: 9px; font-weight: 400; color: var(--color-text-muted); }

.env-sec { border-bottom: 1px solid var(--color-border-subtle); padding-bottom: 12px; margin-bottom: 4px; }
.env-row { display: flex; align-items: center; gap: 10px; }

/* ===== Environment NSelect ===== */
.env-select { min-width: 180px; }
.env-label-dot { display: inline-block; width: 8px; height: 8px; border-radius: 50%; margin-right: 6px; vertical-align: middle; }
.env-label-name { vertical-align: middle; }

/* ===== Environment Tags ===== */
.env-tags { display: flex; flex-wrap: wrap; gap: 4px; }
.epi-tag { font-size: var(--font-size-xs); padding: 2px 8px; border-radius: var(--border-radius-sm); font-weight: 500; background: rgba(137,180,250,0.1); color: var(--color-text-secondary); }
.epi-tag.locked { background: rgba(243,139,168,0.12); color: var(--status-locked); }
.epi-tag.danger { background: rgba(243,139,168,0.15); color: var(--status-locked); font-weight: 700; }
.epi-tag.audit { background: rgba(249,226,175,0.12); color: var(--brand-warning); }

/* ===== Preset / Override Indicators ===== */
.env-preset-indicator { font-size: 11px; font-weight: 500; color: var(--brand-success); padding: 4px 10px; margin-top: 2px; background: rgba(0,184,148,0.08); border: 1px solid rgba(0,184,148,0.2); border-radius: 6px; }
.env-override-hint { font-size: 11px; font-weight: 500; color: var(--brand-warning); padding: 4px 10px; margin-top: 2px; background: rgba(249,226,175,0.08); border: 1px solid rgba(249,226,175,0.2); border-radius: 6px; }

/* ===== DuckDB Acceleration ===== */
.accel-card { padding: 12px; background: var(--color-bg-elevated); border: 1px solid var(--color-border-subtle); border-radius: var(--border-radius-lg); transition: border-color 0.2s; }
.accel-card.on { border-color: rgba(166,227,161,0.3); box-shadow: 0 0 0 1px rgba(166,227,161,0.15); }
.accel-row { display: flex; align-items: flex-start; gap: 12px; }
.accel-icon { font-size: 26px; line-height: 1; flex-shrink: 0; }
.accel-body { flex: 1; display: flex; flex-direction: column; gap: 2px; }
.accel-name { font-size: 13px; font-weight: 600; color: var(--color-text-primary); }
.accel-desc { font-size: 11px; color: var(--color-text-muted); }
.accel-expand { margin-top: 10px; padding-top: 10px; border-top: 1px solid var(--color-border-subtle); }
.accel-benefits { display: flex; flex-wrap: wrap; gap: 6px; margin-bottom: 8px; }
.benefit-tag { font-size: 11px; padding: 3px 8px; border-radius: var(--border-radius-sm); font-weight: 500; background: rgba(166,227,161,0.1); color: var(--brand-success); }

.accel-storage-info { display: flex; flex-wrap: wrap; align-items: baseline; gap: 6px; padding: 8px 10px; margin-bottom: 4px; background: rgba(166,227,161,0.05); border: 1px solid rgba(166,227,161,0.12); border-radius: var(--border-radius-sm); }
.storage-icon { font-size: 12px; flex-shrink: 0; }
.storage-path { font-size: 11px; font-family: var(--font-mono); color: var(--color-text-secondary); }
.storage-hint { font-size: 10px; color: var(--color-text-muted); }

/* ===== Layout Grid / Cells ===== */
.adv-grid { display: grid; grid-template-columns: 1fr 1fr; gap: 10px; }
.adv-cell { display: flex; flex-direction: column; gap: 3px; }
.adv-lbl { font-size: 11px; color: var(--color-text-secondary); }
.adv-inline { display: flex; gap: 12px; align-items: flex-end; }

/* ===== Security Policies ===== */
.policy-grid { border: 1px solid var(--color-border-subtle); border-radius: 6px; padding: 10px; background: var(--color-bg-elevated); }
.policy-row { display: flex; gap: 14px; flex-wrap: wrap; margin-bottom: 4px; }
.policy-row:last-child { margin-bottom: 0; }
.pol-item { display: flex; align-items: center; gap: 6px; font-size: 12px; color: var(--color-text-secondary); }

/* ===== Collapse Summary ===== */
.collapse-summary { font-size: var(--font-size-xs); color: var(--color-text-muted); opacity: 0.7; max-width: 240px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; margin-left: var(--spacing-sm); }

/* ===== Env Manager Modal ===== */
.env-mgr-list { display: flex; flex-direction: column; gap: 8px; }
.env-mgr-card { display: flex; align-items: center; gap: 12px; padding: 10px 14px; border-radius: var(--border-radius-lg); background: var(--color-bg-elevated); }
.env-mgr-color-dot { width: 12px; height: 12px; border-radius: 50%; flex-shrink: 0; }
.env-mgr-content { flex: 1; min-width: 0; }
.env-mgr-header { display: flex; align-items: center; gap: 6px; }
.env-mgr-icon { font-size: 14px; }
.env-mgr-name { font-size: 13px; font-weight: 600; color: var(--color-text-primary); }
.env-mgr-badge { font-size: var(--font-size-xs); padding: 1px 6px; border-radius: var(--border-radius-sm); background: rgba(137,180,250,0.1); color: var(--brand-accent); font-weight: 500; }
.env-mgr-desc { font-size: 11px; color: var(--color-text-muted); margin-top: 2px; }

.env-mgr-meta { display: flex; align-items: center; gap: 4px; margin-top: 2px; font-size: var(--font-size-xxs); color: var(--color-text-muted); }
.env-mgr-meta-lbl { font-weight: 600; }
.env-mgr-meta-dot { width: 8px; height: 8px; border-radius: 50%; flex-shrink: 0; }
.env-mgr-meta-val { font-family: var(--font-mono); }

.env-policy-tags { display: flex; flex-wrap: wrap; gap: 5px; margin-top: 6px; }
.policy-tag { font-size: var(--font-size-xs); padding: 2px 8px; border-radius: var(--border-radius-sm); font-weight: 500; }
.policy-tag.security { background: rgba(243,139,168,0.12); color: var(--status-locked); }
.policy-tag.schema { background: rgba(137,180,250,0.1); color: var(--brand-accent); }
.policy-tag.performance { background: rgba(166,227,161,0.1); color: var(--brand-success); }
.policy-tag.audit { background: rgba(249,226,175,0.12); color: var(--brand-warning); }

/* ===== Env Manager Actions ===== */
.env-mgr-actions { flex-shrink: 0; display: flex; align-items: center; }

/* ===== Env Manager Create Section ===== */
.env-mgr-create-section { margin-top: 14px; padding-top: 14px; border-top: 1px solid var(--color-border-subtle); }
.env-create-form { display: flex; flex-direction: column; gap: 10px; padding: 12px; background: var(--color-bg-elevated); border-radius: var(--border-radius-lg); border: 1px solid var(--color-border-subtle); }
.env-create-row { display: flex; align-items: center; gap: 8px; }
.env-create-lbl { font-size: var(--font-size-sm); color: var(--color-text-secondary); font-weight: 500; min-width: 56px; flex-shrink: 0; }
.env-create-actions { display: flex; gap: 8px; margin-top: 4px; }

/* ===== Color Picker (native input[type=color]) ===== */
.color-input {
  width: 32px; height: 32px; padding: 0;
  border: 1px solid var(--color-border); border-radius: var(--border-radius-sm);
  background: transparent; cursor: pointer; box-sizing: border-box; flex-shrink: 0;
}
.color-input::-webkit-color-swatch-wrapper { padding: 2px; }
.color-input::-webkit-color-swatch { border: none; border-radius: 2px; }
.color-input:focus-visible { outline: 2px solid var(--brand-accent); outline-offset: 2px; }
</style>