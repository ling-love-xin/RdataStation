<template>
  <div class="adv-tab">
    <div v-if="!driver" class="empty-hint">{{ $t('navigator.noDriver') }}</div>

    <template v-else>
      <!-- Environment selector (extracted) -->
      <div class="adv-sec env-sec">
        <EnvironmentSelector
          v-model="envId"
          :options="envSelectOpts"
          :label="$t('navigator.advancedEnv')"
          :manage-label="$t('navigator.manage')"
          @update:model-value="onEnvChange"
          @manage="showEnvMgr = true"
        />
        <div v-if="envPolicyTags.length" class="env-tags">
          <span v-for="tag in envPolicyTags" :key="tag.key" :class="['epi-tag', tag.kind]">{{ tag.label }}</span>
        </div>
        <div v-if="!isPolicyOverridden" class="env-preset-indicator">← 🟢 {{ currentEnvDef.name }} 预设</div>
        <div v-if="isPolicyOverridden" class="env-override-hint">⚠ {{ $t('navigator.envOverrideHint', { name: currentEnvDef.name }) }}</div>
        <div v-if="envSnapshotting" class="env-snapshot-hint">📸 正在快照全局环境...</div>
        <div v-else-if="envSnapshotId" class="env-snapshot-hint">📸 已快照为 {{ envSnapshotId }}</div>
      </div>

      <!-- DuckDB acceleration (extracted) -->
      <div class="adv-sec">
        <div class="sec-title" style="color:var(--brand-warning)">
          ⚡ {{ $t('connection.advancedTab.localAccel') }} <span class="sec-sub">DuckDB</span>
        </div>
        <DuckDBAccelSection
          v-model:enabled="duckdbEnabled"
          v-model:sync="duckdbSync"
          v-model:interval="duckdbInterval"
          v-model:memory="duckdbMemory"
          v-model:threads="duckdbThreads"
          :sync-options="syncOpts"
          :title="$t('connection.advancedTab.enableDuckdbAccel')"
          :description="$t('connection.advancedTab.accelDesc', { dbType: driver?.name || 'DB' })"
          :sync-label="$t('connection.advancedTab.syncStrategy')"
          :interval-label="$t('connection.advancedTab.syncInterval')"
          :memory-label="$t('connection.advancedTab.memoryLimit')"
          :threads-label="$t('connection.advancedTab.threads')"
        />
      </div>

      <!-- Security policies (extracted) -->
      <div class="adv-sec">
        <SecurityPolicySection
          v-model:readonly="polReadonly"
          v-model:write-confirm="polWriteConfirm"
          v-model:ddl-confirm="polDdlConfirm"
          v-model:autocommit="polAutocommit"
          v-model:drop-policy="polDrop"
          v-model:row-limit="polRowLimit"
          v-model:size-limit="polSizeLimit"
          :title="$t('navigator.advancedSecurity')"
          :summary="securitySummary"
          :readonly-label="$t('navigator.advancedReadOnly')"
          :write-confirm-label="$t('navigator.writeConfirm')"
          :ddl-confirm-label="$t('navigator.advancedDdlConfirm') || 'DDL确认'"
          :drop-op-label="$t('navigator.advancedDropOp') || 'DROP操作'"
          :autocommit-label="$t('navigator.autoCommit') || '自动提交'"
          :row-limit-label="$t('navigator.rowLimit')"
          :size-limit-label="$t('navigator.sizeLimit')"
          :drop-options="dropOpts"
          @override="checkPolicyOverride"
        />
      </div>

      <!-- Schema policies -->
      <div class="adv-sec">
        <NCollapse>
          <NCollapseItem name="schema">
            <template #header>
              <span class="sec-title" style="margin-bottom:0">📋 {{ $t('navigator.advancedSchema') || 'Schema 策略' }}</span>
              <span class="collapse-summary">{{ schemaSummary }}</span>
            </template>
            <div class="policy-grid">
              <div class="policy-row">
                <span class="pol-item">自动加载 <NSwitch v-model:value="schAutoLoad" size="small" /></span>
                <span class="pol-item">加载深度 <NInputNumber v-model:value="schLoadDepth" size="small" :min="1" :max="10" style="width:80px" /></span>
                <span class="pol-item">显示系统表 <NSwitch v-model:value="schShowSystem" size="small" /></span>
              </div>
              <div class="policy-row">
                <span class="pol-item">刷新间隔(秒) <NInputNumber v-model:value="schRefreshInterval" size="small" :min="0" :max="3600" style="width:80px" /></span>
              </div>
            </div>
          </NCollapseItem>
        </NCollapse>
      </div>

      <!-- Performance policies -->
      <div class="adv-sec">
        <NCollapse>
          <NCollapseItem name="perf">
            <template #header>
              <span class="sec-title" style="margin-bottom:0">⚡ {{ $t('navigator.performance') || '性能策略' }}</span>
              <span class="collapse-summary">{{ perfSummary }}</span>
            </template>
            <div class="policy-grid">
              <div class="policy-row">
                <span class="pol-item">连接池大小 <NInputNumber v-model:value="perfPoolSize" size="small" :min="1" :max="100" style="width:80px" /></span>
                <span class="pol-item">查询超时(秒) <NInputNumber v-model:value="advQueryTimeout" size="small" :min="0" :max="3600" style="width:80px" /></span>
                <span class="pol-item">连接超时(秒) <NInputNumber v-model:value="advConnectTimeout" size="small" :min="1" :max="300" style="width:80px" /></span>
              </div>
              <div class="policy-row">
                <span class="pol-item">心跳间隔(秒) <NInputNumber v-model:value="advHeartbeat" size="small" :min="10" :max="600" style="width:80px" /></span>
                <span class="pol-item">最大重连 <NInputNumber v-model:value="advMaxReconnect" size="small" :min="0" :max="20" style="width:80px" /></span>
              </div>
            </div>
          </NCollapseItem>
        </NCollapse>
      </div>

      <!-- Audit policies -->
      <div class="adv-sec">
        <NCollapse>
          <NCollapseItem name="audit">
            <template #header>
              <span class="sec-title" style="margin-bottom:0">📝 {{ $t('navigator.audit') || '审计策略' }}</span>
              <span class="collapse-summary">{{ auditSummary }}</span>
            </template>
            <div class="policy-grid">
              <div class="policy-row">
                <span class="pol-item">SQL 日志 <NSwitch v-model:value="audSqlLog" size="small" /></span>
                <span class="pol-item">操作记录 <NSwitch v-model:value="audOperationRecord" size="small" /></span>
                <span class="pol-item">敏感表告警 <NSwitch v-model:value="audSensitiveTableAlert" size="small" /></span>
              </div>
            </div>
          </NCollapseItem>
        </NCollapse>
      </div>

      <!-- UI policies -->
      <div class="adv-sec">
        <NCollapse>
          <NCollapseItem name="ui">
            <template #header>
              <span class="sec-title" style="margin-bottom:0">🎨 {{ $t('navigator.uiPolicy') || 'UI 策略' }}</span>
              <span class="collapse-summary">{{ uiSummary }}</span>
            </template>
            <div class="policy-grid">
              <div class="policy-row">
                <span class="pol-item">顶栏颜色 <input v-model="uiTopBarColor" type="color" class="color-input-sm" /></span>
                <span class="pol-item">标签指示符 <NInput v-model:value="uiTabIndicator" size="small" style="width:100px" placeholder="🔴" /></span>
              </div>
              <div class="policy-row">
                <span class="pol-item">SQL 警告横幅 <NSwitch v-model:value="uiSqlWarningBanner" size="small" /></span>
                <span class="pol-item">写入按钮样式
                  <NSelect v-model:value="uiWriteBtnStyle" size="small" :options="writeBtnStyleOpts" style="width:120px" />
                </span>
              </div>
            </div>
          </NCollapseItem>
        </NCollapse>
      </div>

      <!-- Connection params (basic) -->
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

      <!-- Environment Manager Modal (extracted) -->
      <EnvironmentManager
        v-model="showEnvMgr"
        :title="$t('navigator.envManager') || '环境管理器'"
        :loading="envListLoading"
        :loading-text="$t('dataPreview.loading')"
        :environments="loadedEnvs"
        :builtin-badge="$t('navigator.builtinBadge') || '内置'"
        :show-create-form="showEnvCreateForm"
        :editing="!!editingEnvId"
        :create-label="$t('navigator.createEnv')"
        :name-label="$t('navigator.envName')"
        :name-placeholder="$t('navigator.envNamePlaceholder') || '输入环境名称'"
        :icon-label="$t('navigator.envIcon')"
        :color-label="$t('navigator.envColor')"
        :desc-label="$t('navigator.envDesc')"
        :desc-placeholder="$t('navigator.envDescPlaceholder') || '输入环境描述'"
        :template-label="$t('navigator.envTemplate')"
        :save-label="editingEnvId ? ($t('common.save') || '保存修改') : ($t('common.save') || '保存')"
        :cancel-label="$t('common.cancel')"
        :new-name="newEnvName"
        :new-icon="newEnvIcon"
        :new-color="newEnvColor"
        :new-desc="newEnvDesc"
        :new-template="newEnvTemplate"
        :template-options="envTemplateOpts"
        @update:model-value="v => showEnvMgr = v"
        @update:new-name="v => newEnvName = v"
        @update:new-icon="v => newEnvIcon = v"
        @update:new-color="v => newEnvColor = v"
        @update:new-desc="v => newEnvDesc = v"
        @update:new-template="v => newEnvTemplate = v"
        @toggle-create="toggleEnvForm"
        @create="handleCreateEnv"
        @edit="handleEditEnv"
        @delete="handleDeleteEnv"
      />
    </template>
  </div>
</template>

<script setup lang="ts">
import {
  NCollapse, NCollapseItem, NSelect, NSwitch, NInputNumber, NInput,
} from 'naive-ui'
import { ref, computed, watch, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'

import DuckDBAccelSection from './DuckDBAccelSection.vue'
import EnvironmentManager from './EnvironmentManager.vue'
import EnvironmentSelector from './EnvironmentSelector.vue'
import SecurityPolicySection from './SecurityPolicySection.vue'

import type { EnvInfo } from './EnvironmentManager.vue'
import type { Driver } from '../../../domain/types'
import type { SelectOption } from 'naive-ui'

const { t } = useI18n()
const props = defineProps<{
  driver?: Driver | null
  formData?: Record<string, unknown>
  /** 连接作用域 — 决定环境列表的过滤和快照行为 */
  scope?: { global: boolean; project: boolean }
}>()

const emit = defineEmits<{
  'update:formData': [data: Record<string, unknown>]
  'extra-config': [config: Record<string, unknown>]
}>()

// ========== Environment ==========
const envId = ref('env-dev')
const selectedEnvId = ref<string | null>(null)
const envSnapshotting = ref(false)
const envSnapshotId = ref<string | null>(null)
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

function checkPolicyOverride() { /* computed auto-updates */ }

function onEnvChange(id: string) {
  // 项目级连接引用全局环境 → 触发快照
  if (props.scope?.project && id.startsWith('G_') && !id.startsWith('GP_')) {
    envSnapshotting.value = true
    import('@tauri-apps/api/core').then(async ({ invoke }) => {
      try {
        const { useProjectStore } = await import('@/core/project/stores/project')
        const pp = useProjectStore().currentProject?.path
        const r = await invoke<{ snapshot_id: string }>('snapshot_global_env', { globalEnvId: id, projectPath: pp })
        const gpId = r.snapshot_id
        envSnapshotId.value = gpId
        selectedEnvId.value = gpId // 替换为快照 ID
        applyEnvDefaults(id)
        loadEnvironments() // 刷新列表显示新快照
      } finally { envSnapshotting.value = false }
    })
    return
  }
  selectedEnvId.value = id
  applyEnvDefaults(id)
}

function applyEnvDefaults(id: string) {
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
  // Reset schema/audit/ui to default for the environment
  const isProd = id === 'env-prod'
  schAutoLoad.value = !isProd
  schShowSystem.value = !isProd
  schLoadDepth.value = isProd ? 1 : 3
  schRefreshInterval.value = isProd ? 60 : 0
  audSqlLog.value = !(id === 'env-dev' || id === 'env-sandbox')
  audOperationRecord.value = isProd || id === 'env-staging'
  audSensitiveTableAlert.value = isProd
  uiSqlWarningBanner.value = isProd || id === 'env-staging'
  uiTopBarColor.value = currentEnvDef.value.color
  uiWriteBtnStyle.value = isProd ? 'danger' : 'default'
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

// ========== Schema policies (NEW) ==========
const schAutoLoad = ref(true)
const schLoadDepth = ref(3)
const schShowSystem = ref(false)
const schRefreshInterval = ref(0)

const schemaSummary = computed(() => {
  const parts: string[] = []
  if (schAutoLoad.value) parts.push('自动加载')
  else parts.push('手动加载')
  if (schShowSystem.value) parts.push('系统表')
  if (schLoadDepth.value > 0) parts.push(`深度${schLoadDepth.value}`)
  if (schRefreshInterval.value > 0) parts.push(`刷新${schRefreshInterval.value}s`)
  return parts.join('·') || '默认'
})

// ========== Performance policies (NEW) ==========
const perfPoolSize = ref(10)

const perfSummary = computed(() => {
  const parts: string[] = []
  parts.push(`池${perfPoolSize.value}`)
  if (advQueryTimeout.value > 0) parts.push(`查询${advQueryTimeout.value}s`)
  parts.push(`超时${advConnectTimeout.value}s`)
  parts.push(`重连${advMaxReconnect.value}`)
  return parts.join('·')
})

// ========== Audit policies (NEW) ==========
const audSqlLog = ref(false)
const audOperationRecord = ref(false)
const audSensitiveTableAlert = ref(false)

const auditSummary = computed(() => {
  const parts: string[] = []
  if (audSqlLog.value) parts.push('SQL日志')
  if (audOperationRecord.value) parts.push('操作记录')
  if (audSensitiveTableAlert.value) parts.push('敏感告警')
  return parts.join('·') || '无审计'
})

// ========== UI policies (NEW) ==========
const uiTopBarColor = ref('#a6e3a1')
const uiTabIndicator = ref('')
const uiSqlWarningBanner = ref(false)
const uiWriteBtnStyle = ref('default')
const writeBtnStyleOpts = [
  { label: '默认', value: 'default' },
  { label: '警告', value: 'warning' },
  { label: '危险', value: 'danger' },
  { label: '虚线', value: 'dashed' },
]

const uiSummary = computed(() => {
  const parts: string[] = []
  parts.push(uiTopBarColor.value)
  if (uiSqlWarningBanner.value) parts.push('警告横幅')
  if (uiWriteBtnStyle.value !== 'default') parts.push(`按钮:${uiWriteBtnStyle.value}`)
  return parts.join('·') || '默认'
})

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

// ========== Env Manager state ==========
const loadedEnvs = ref<EnvInfo[]>([])
const envListLoading = ref(false)
const showEnvCreateForm = ref(false)
const editingEnvId = ref<string | null>(null)
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

function resetEnvForm() {
  newEnvName.value = ''
  newEnvIcon.value = '🟢'
  newEnvColor.value = '#a6e3a1'
  newEnvDesc.value = ''
  newEnvTemplate.value = 'dev'
  editingEnvId.value = null
}

function toggleEnvForm() {
  if (showEnvCreateForm.value) {
    showEnvCreateForm.value = false
    resetEnvForm()
  } else {
    resetEnvForm()
    showEnvCreateForm.value = true
  }
}

function handleEditEnv(env: EnvInfo) {
  editingEnvId.value = env.id
  newEnvName.value = env.name
  newEnvIcon.value = env.icon
  newEnvColor.value = env.color
  newEnvDesc.value = env.desc
  newEnvTemplate.value = 'dev'
  showEnvCreateForm.value = true
}

async function handleCreateEnv() {
  const name = newEnvName.value.trim()
  if (!name) return
  const isEdit = !!editingEnvId.value
  try {
    const { invoke } = await import('@tauri-apps/api/core')
    if (isEdit) {
      await invoke('update_environment', {
        env: {
          id: editingEnvId.value,
          name,
          description: newEnvDesc.value,
          color: newEnvColor.value || '#a6e3a1',
          sort_order: 0,
          origin: 'project',
          source_id: null,
        },
      })
    } else {
      await invoke('create_environment', {
        env: {
          name,
          icon: newEnvIcon.value || '🟢',
          color: newEnvColor.value || '#a6e3a1',
          description: newEnvDesc.value,
          templateId: `env-${newEnvTemplate.value}`,
        },
      })
    }
    resetEnvForm()
    showEnvCreateForm.value = false
    await loadEnvironments()
  } catch {
    if (isEdit) {
      // Update locally on error as fallback
      const idx = loadedEnvs.value.findIndex(e => e.id === editingEnvId.value)
      if (idx >= 0) {
        loadedEnvs.value[idx] = {
          ...loadedEnvs.value[idx],
          name,
          color: newEnvColor.value || '#a6e3a1',
          icon: newEnvIcon.value || '🟢',
          desc: newEnvDesc.value,
          ui: { summaryUI: newEnvColor.value || '#a6e3a1' },
        }
      }
    } else {
      const id = `env-custom-${Date.now()}`
      const template = envDefs.find(e => e.id === `env-${newEnvTemplate.value}`)
      loadedEnvs.value.push({
        id, name,
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
    }
    resetEnvForm()
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
    const scopeType = props.scope?.global ? 'global' : 'project'
    // 按 scope 过滤：global 只看 G_，project 合并 G_ + P_ + GP_
    const remote = await invoke<Array<{ id: string; name: string; description: string; properties?: Record<string, unknown> }>>('list_environments')
    if (remote && remote.length > 0) {
      const colors = ['#a6e3a1', '#f9e2af', '#89b4fa', '#f38ba8', '#cba6f7']
      const icons = ['🟢', '🟡', '🔵', '🔴', '🟣']
      loadedEnvs.value = remote
        .filter(e => {
          // global scope → only G_ (not GP_)
          if (scopeType === 'global') return e.id.startsWith('G_') && !e.id.startsWith('GP_')
          // project scope → P_ or GP_ (快照), plus G_ (可引用但会触发快照)
          return e.id.startsWith('P_') || e.id.startsWith('GP_') || e.id.startsWith('G_')
        })
        .map((e, i) => ({
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
   schAutoLoad, schLoadDepth, schShowSystem, schRefreshInterval,
   perfPoolSize,
   audSqlLog, audOperationRecord, audSensitiveTableAlert,
   uiTopBarColor, uiTabIndicator, uiSqlWarningBanner, uiWriteBtnStyle,
   advConnectTimeout, advQueryTimeout, advHeartbeat, advMaxReconnect,
   schemaStrategy, encoding, envId, selectedEnvId, envSnapshotId],
  () => {
    const opts = {
      envId: envSnapshotId.value || selectedEnvId.value || envId.value,
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
      schema: {
        autoLoad: schAutoLoad.value,
        loadDepth: schLoadDepth.value,
        showSystem: schShowSystem.value,
        refreshInterval: schRefreshInterval.value,
      },
      performance: {
        poolSize: perfPoolSize.value,
        queryTimeout: advQueryTimeout.value,
        connectTimeout: advConnectTimeout.value,
        heartbeat: advHeartbeat.value,
        maxReconnect: advMaxReconnect.value,
      },
      audit: {
        sqlLog: audSqlLog.value,
        operationRecord: audOperationRecord.value,
        sensitiveTableAlert: audSensitiveTableAlert.value,
      },
      ui: {
        topBarColor: uiTopBarColor.value,
        tabIndicator: uiTabIndicator.value,
        sqlWarningBanner: uiSqlWarningBanner.value,
        writeBtnStyle: uiWriteBtnStyle.value,
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

.env-tags { display: flex; flex-wrap: wrap; gap: 4px; }
.epi-tag { font-size: var(--font-size-xs); padding: 2px 8px; border-radius: var(--border-radius-sm); font-weight: 500; background: rgba(137,180,250,0.1); color: var(--color-text-secondary); }
.epi-tag.locked { background: rgba(243,139,168,0.12); color: var(--status-locked); }
.epi-tag.danger { background: rgba(243,139,168,0.15); color: var(--status-locked); font-weight: 700; }
.epi-tag.audit { background: rgba(249,226,175,0.12); color: var(--brand-warning); }

.env-preset-indicator { font-size: 11px; font-weight: 500; color: var(--brand-success); padding: 4px 10px; margin-top: 2px; background: rgba(0,184,148,0.08); border: 1px solid rgba(0,184,148,0.2); border-radius: 6px; }
.env-override-hint { font-size: 11px; font-weight: 500; color: var(--brand-warning); padding: 4px 10px; margin-top: 2px; background: rgba(249,226,175,0.08); border: 1px solid rgba(249,226,175,0.2); border-radius: 6px; }
.env-snapshot-hint { font-size: 11px; font-weight: 500; color: #cba6f7; padding: 4px 10px; margin-top: 2px; background: rgba(245,194,231,0.08); border: 1px solid rgba(245,194,231,0.2); border-radius: 6px; font-style: italic; }

.collapse-summary { font-size: var(--font-size-xs); color: var(--color-text-muted); opacity: 0.7; max-width: 240px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; margin-left: var(--spacing-sm); }
.policy-grid { border: 1px solid var(--color-border-subtle); border-radius: 6px; padding: 10px; background: var(--color-bg-elevated); }
.policy-row { display: flex; gap: 14px; flex-wrap: wrap; margin-bottom: 4px; }
.policy-row:last-child { margin-bottom: 0; }
.pol-item { display: flex; align-items: center; gap: 6px; font-size: 12px; color: var(--color-text-secondary); }

.adv-grid { display: grid; grid-template-columns: 1fr 1fr; gap: 10px; }
.adv-cell { display: flex; flex-direction: column; gap: 3px; }
.adv-lbl { font-size: 11px; color: var(--color-text-secondary); }
.adv-inline { display: flex; gap: 12px; align-items: flex-end; }

.color-input-sm { width: 28px; height: 28px; padding: 0; border: 1px solid var(--color-border); border-radius: var(--border-radius-sm); background: transparent; cursor: pointer; box-sizing: border-box; flex-shrink: 0; }
.color-input-sm::-webkit-color-swatch-wrapper { padding: 1px; }
.color-input-sm::-webkit-color-swatch { border: none; border-radius: 2px; }
</style>