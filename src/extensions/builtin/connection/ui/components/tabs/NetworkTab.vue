<template>
  <div class="network-tab">
    <!-- 文件数据库提示 -->
    <div v-if="isFileDb" class="network-file-hint">
      <div class="hint-icon">🗄</div>
      <h3 class="hint-title">{{ t('connection.networkTab.fileDbHint') }}</h3>
      <p class="hint-desc">{{ t('connection.networkTab.fileDbHintDesc') }}</p>
    </div>

    <!-- 网络数据库 — 协议链引擎 -->
    <div v-else class="network-content">
      <!-- 说明横幅 -->
      <div class="network-hint">
        <strong>{{ t('connection.networkTab.dynamicChain') }}</strong>
        <p class="network-hint-desc">{{ t('connection.networkTab.chainDesc') }}</p>
      </div>

      <!-- 协议链列表 -->
      <ProtocolChainList
        :chain="chain"
        :menu-open="menuOpen"
        :ssh-profiles="sshProfiles"
        :ssl-profiles="sslProfiles"
        :proxy-profiles="proxyProfiles"
        :add-hop-options="addHopOptions"
        :network-hop-count="networkHopCount"
        :enabled-network-hop-count="enabledNetworkHopCount"
        :is-max-network-hops="isMaxNetworkHops"
        :has-ssl="hasSsl"
        @toggle="toggleHop"
        @delete="onDeleteHop"
        @switch-mode="switchHopMode"
        @save-new="onSaveNewHop"
        @manage="onManageProfiles"
        @select-profile="selectProfile"
        @drag-start="onDragStart"
        @drag-end="onDragEnd"
        @drop="onDrop"
        @toggle-menu="toggleMenu"
        @add-hop="onAddHop"
      >
        <!-- 新建表单：根据协议类型渲染不同字段 -->
        <template #new-form="{ hop }">
          <!-- SSH 新建表单 -->
          <template v-if="hop.protocol === 'ssh'">
            <div class="form-row">
              <div class="form-group f1">
                <span class="form-label">名称</span>
                <input
                  v-model="newFormData[hop.id].name"
                  type="text"
                  class="form-input"
                  placeholder="新SSH配置"
                />
              </div>
              <div class="form-group f1">
                <span class="form-label">范围归属</span>
                <span class="profile-scope-badge" :class="scopeClass">
                  {{ scopeLabel }}
                </span>
              </div>
            </div>
            <div class="form-section-label">跳板机连接</div>
            <div class="form-row">
              <div class="form-group f2">
                <span class="form-label">{{ t('connection.networkTab.sshHost') }}</span>
                <input
                  v-model="newFormData[hop.id].host"
                  type="text"
                  class="form-input"
                  placeholder="192.168.1.1"
                />
              </div>
              <div class="form-group f1">
                <span class="form-label">{{ t('connection.networkTab.port') }}</span>
                <input
                  v-model.number="newFormData[hop.id].port"
                  type="number"
                  class="form-input"
                />
              </div>
            </div>
            <div class="form-row">
              <div class="form-group f1">
                <span class="form-label">{{ t('connection.networkTab.username') }}</span>
                <input
                  v-model="newFormData[hop.id].username"
                  type="text"
                  class="form-input"
                  placeholder="root"
                />
              </div>
              <div class="form-group f1">
                <span class="form-label">{{ t('connection.networkTab.authMethod') }}</span>
                <select v-model="newFormData[hop.id].authType" class="form-select">
                  <option value="key">密钥</option>
                  <option value="password">密码</option>
                </select>
              </div>
            </div>
            <div class="form-row">
              <div class="form-group f1">
                <span class="form-label">{{ t('connection.networkTab.privateKey') }}</span>
                <input
                  v-model="newFormData[hop.id].keyPath"
                  type="text"
                  class="form-input"
                  placeholder="~/.ssh/id_rsa"
                />
              </div>
              <div class="form-group f1">
                <span class="form-label">{{ t('connection.networkTab.keepAlive') }}</span>
                <input
                  v-model.number="newFormData[hop.id].keepAlive"
                  type="number"
                  class="form-input"
                />
              </div>
            </div>
            <div class="form-section-label">端口转发 (本地:远端)</div>
            <div class="form-row">
              <div class="form-group f1">
                <span class="form-label">{{ t('connection.networkTab.localPort') }}</span>
                <input
                  v-model.number="newFormData[hop.id].localPort"
                  type="number"
                  class="form-input"
                  placeholder="自动分配"
                />
              </div>
              <div class="form-group f1">
                <span class="form-label">远程目标地址</span>
                <input
                  v-model="newFormData[hop.id].remoteHost"
                  type="text"
                  class="form-input"
                  placeholder="目标DB地址"
                />
              </div>
              <div class="form-group" style="width: 80px">
                <span class="form-label">{{ t('connection.networkTab.port') }}</span>
                <input
                  v-model.number="newFormData[hop.id].remotePort"
                  type="number"
                  class="form-input"
                  placeholder="3306"
                />
              </div>
            </div>
          </template>

          <!-- SSL 新建表单 -->
          <template v-else-if="hop.protocol === 'ssl'">
            <div class="form-row">
              <div class="form-group f1">
                <span class="form-label">名称</span>
                <input
                  v-model="newFormData[hop.id].name"
                  type="text"
                  class="form-input"
                  placeholder="新SSL配置"
                />
              </div>
              <div class="form-group f1">
                <span class="form-label">范围归属</span>
                <span class="profile-scope-badge" :class="scopeClass">
                  {{ scopeLabel }}
                </span>
              </div>
            </div>
            <div class="form-row">
              <div class="form-group f1">
                <span class="form-label">{{ t('connection.networkTab.sslMode') }}</span>
                <select v-model="newFormData[hop.id].mode" class="form-select">
                  <option value="verify-full">verify-full</option>
                  <option value="verify-ca">verify-ca</option>
                  <option value="require">require</option>
                </select>
              </div>
            </div>
            <div class="form-row">
              <div class="form-group f1">
                <span class="form-label">CA {{ t('connection.networkTab.cert') }}</span>
                <input
                  v-model="newFormData[hop.id].ca"
                  type="text"
                  class="form-input"
                  placeholder="ca.pem"
                />
              </div>
              <div class="form-group f1">
                <span class="form-label">{{ t('connection.networkTab.clientCert') }}</span>
                <input
                  v-model="newFormData[hop.id].cert"
                  type="text"
                  class="form-input"
                  placeholder="client.pem"
                />
              </div>
            </div>
          </template>

          <!-- Proxy 新建表单 -->
          <template v-else-if="hop.protocol === 'proxy'">
            <div class="form-row">
              <div class="form-group f1">
                <span class="form-label">名称</span>
                <input
                  v-model="newFormData[hop.id].name"
                  type="text"
                  class="form-input"
                  placeholder="新代理配置"
                />
              </div>
              <div class="form-group f1">
                <span class="form-label">范围归属</span>
                <span class="profile-scope-badge" :class="scopeClass">
                  {{ scopeLabel }}
                </span>
              </div>
            </div>
            <div class="form-row">
              <div class="form-group f1">
                <span class="form-label">{{ t('connection.networkTab.proxyType') }}</span>
                <select v-model="newFormData[hop.id].type" class="form-select">
                  <option value="socks5">SOCKS5</option>
                  <option value="http">HTTP</option>
                  <option value="socks4">SOCKS4</option>
                </select>
              </div>
            </div>
            <div class="form-row">
              <div class="form-group f2">
                <span class="form-label">{{ t('connection.networkTab.proxyHost') }}</span>
                <input
                  v-model="newFormData[hop.id].host"
                  type="text"
                  class="form-input"
                  placeholder="proxy.example.com"
                />
              </div>
              <div class="form-group f1">
                <span class="form-label">{{ t('connection.networkTab.port') }}</span>
                <input
                  v-model.number="newFormData[hop.id].port"
                  type="number"
                  class="form-input"
                />
              </div>
            </div>
          </template>
        </template>
      </ProtocolChainList>

      <!-- 跳数警告 -->
      <ChainWarning
        :visible="showHopWarning"
        :hop-count="enabledNetworkHopCount"
        :estimated-latency="estimatedLatency"
      />

      <!-- 拓扑预览 -->
      <TopologyPreview
        :nodes="topologyNodes"
        :target-label="targetLabel"
        :empty="isEmpty"
      />
    </div>

    <!-- 配置文件管理器对话框 -->
    <NetworkConfigManager
      :visible="showConfigManager"
      :default-tab="configManagerTab"
      :ssh-profiles="sshProfiles"
      :ssl-profiles="sslProfiles"
      :proxy-profiles="proxyProfiles"
      @close="showConfigManager = false"
      @delete-ssh="(id: string) => onDeleteProfile('ssh', id)"
      @delete-ssl="(id: string) => onDeleteProfile('ssl', id)"
      @delete-proxy="(id: string) => onDeleteProfile('proxy', id)"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, watch, computed, onMounted, onUnmounted } from 'vue'
import { useI18n } from 'vue-i18n'

import { useNetworkChain } from '../../composables/useNetworkChain'
import ChainWarning from '../network/ChainWarning.vue'
import NetworkConfigManager from '../network/NetworkConfigManager.vue'
import ProtocolChainList from '../network/ProtocolChainList.vue'
import TopologyPreview from '../network/TopologyPreview.vue'

import type { ProtocolType } from '../../types/network-chain'

const { t } = useI18n()

const props = defineProps<{
  isFileDb?: boolean
  dbType?: string
  defaultPort?: number
  scope?: 'global' | 'project'
}>()

const emit = defineEmits<{
  'update:config': [config: Record<string, unknown>]
}>()

// ===== 协议链引擎 =====

const {
  chain,
  menuOpen,
  sshProfiles,
  sslProfiles,
  proxyProfiles,
  networkHopCount,
  enabledNetworkHopCount,
  isMaxNetworkHops,
  hasSsl,
  showHopWarning,
  estimatedLatency,
  addHopOptions,
  topologyNodes,
  isEmpty,
  networkConfig,
  toggleHop,
  deleteHop,
  switchHopMode,
  selectProfile,
  addHop,
  saveNewHop,
  onDragStart,
  onDragEnd,
  onDrop,
  loadProfilesFromDb,
  saveChainToDb,
  deleteProfileInDb,
} = useNetworkChain()

// ===== 挂载时加载已有配置文件 =====

onMounted(() => {
  void loadProfilesFromDb()
})

// ===== 暴露给父组件（供保存时调用） =====

const savedNetworkConfigId = ref<string | null>(null)

const showConfigManager = ref(false)
const configManagerTab = ref<'ssh' | 'ssl' | 'proxy'>('ssh')

async function saveNetworkChain(scope: 'global' | 'project'): Promise<string | null> {
  // 如果已有保存的 ID 且 scope 一致，直接返回（避免重复创建）
  if (savedNetworkConfigId.value) {
    return savedNetworkConfigId.value
  }
  const id = await saveChainToDb(scope)
  if (id) {
    savedNetworkConfigId.value = id
  }
  return id
}

defineExpose({ saveNetworkChain })

// ===== 新建表单临时数据 =====

interface NewFormState {
  [hopId: string]: Record<string, unknown>
}

const newFormData = reactive<NewFormState>({})

function initNewFormData(hopId: string, protocol: ProtocolType) {
  if (protocol === 'ssh') {
    newFormData[hopId] = {
      name: '',
      scope: props.scope || 'project',
      host: '',
      port: 22,
      username: 'root',
      authType: 'key',
      keyPath: '~/.ssh/id_rsa',
      keepAlive: 60,
      localPort: null,
      remoteHost: '',
      remotePort: null,
    }
  } else if (protocol === 'ssl') {
    newFormData[hopId] = {
      name: '',
      scope: props.scope || 'project',
      mode: 'verify-full',
      ca: '',
      cert: '',
    }
  } else {
    newFormData[hopId] = {
      name: '',
      scope: props.scope || 'project',
      type: 'socks5',
      host: '',
      port: 1080,
    }
  }
}

// ===== 范围 =====

const scopeLabel = computed(() => {
  if (props.scope === 'global') return '全局'
  return '项目'
})

const scopeClass = computed(() => {
  if (props.scope === 'global') return 'global'
  return 'project'
})

// ===== 目标标签 =====

const targetLabel = computed(() => {
  const db = (props.dbType || 'DB').toUpperCase()
  const port = props.defaultPort ? `:${props.defaultPort}` : ''
  return `🗄 ${db}${port}`
})

// ===== 操作处理 =====

function onDeleteHop(hopId: string) {
  const hop = chain.value.find(h => h.id === hopId)
  if (!hop) return
  if (!deleteHop(hopId)) {
    // 每种协议至少保留一个
    return
  }
}

async function onSaveNewHop(hopId: string) {
  const data = newFormData[hopId]
  if (!data) return
  const result = await saveNewHop(hopId, data)
  if (result) {
    // 清理表单数据
    delete newFormData[hopId]
  }
}

function onManageProfiles(_protocol: ProtocolType) {
  showConfigManager.value = true
  configManagerTab.value = _protocol === 'ssl' ? 'ssl' : _protocol === 'proxy' ? 'proxy' : 'ssh'
}

async function onDeleteProfile(protocol: ProtocolType, id: string) {
  await deleteProfileInDb(protocol, id)
}

function onAddHop(protocol: ProtocolType) {
  if (isMaxNetworkHops.value && protocol !== 'ssl') return

  const hopId = addHop(protocol)
  if (hopId) {
    menuOpen.value = false
    // 如果新建模式需要表单，初始化数据
    if (protocol !== 'ssl' || !hasSsl.value) {
      initNewFormData(hopId, protocol)
    }
  }
}

function toggleMenu() {
  if (isMaxNetworkHops.value && hasSsl.value) return
  if (isMaxNetworkHops.value && !hasSsl.value) {
    // 只允许添加 SSL
    onAddHop('ssl')
    return
  }
  menuOpen.value = !menuOpen.value
}

// ===== 点击外部关闭菜单 =====

function handleClickOutside(event: MouseEvent) {
  const target = event.target as HTMLElement
  if (!target.closest('.add-hop-section') && !target.closest('.hop-type-menu')) {
    menuOpen.value = false
  }
}

onMounted(() => {
  document.addEventListener('click', handleClickOutside)
})

onUnmounted(() => {
  document.removeEventListener('click', handleClickOutside)
})

// ===== 监听变更 → emit =====

watch(networkConfig, (config) => {
  emit('update:config', config)
}, { deep: true })
</script>

<style scoped>
.network-tab {
  padding: 0;
}

/* 文件数据库提示 */
.network-file-hint {
  text-align: center;
  padding: 40px 20px;
}

.hint-icon {
  font-size: 42px;
  margin-bottom: 14px;
  opacity: 0.3;
}

.hint-title {
  font-size: var(--font-size-xl);
  color: var(--color-text-secondary);
  margin-bottom: var(--spacing-sm);
  font-weight: 600;
}

.hint-desc {
  font-size: var(--font-size-sm);
  color: var(--color-text-muted);
  line-height: 1.6;
  max-width: 360px;
  margin: 0 auto;
}

/* 网络内容 */
.network-content {
  display: flex;
  flex-direction: column;
}

/* 说明横幅 */
.network-hint {
  margin-bottom: var(--spacing-md);
  padding: 10px 14px;
  background: rgba(137, 180, 250, 0.04);
  border: 1px solid rgba(137, 180, 250, 0.1);
  border-radius: var(--border-radius-md);
  font-size: var(--font-size-sm);
  color: var(--color-text-secondary);
  line-height: 1.6;
}

.network-hint strong {
  color: #89b4fa;
}

.network-hint-desc {
  margin-top: 4px;
  font-size: var(--font-size-xs);
  color: var(--color-text-muted);
  margin-bottom: 0;
}

/* 表单组件 */
.form-row {
  display: flex;
  gap: 12px;
  margin-bottom: 10px;
}

.form-group {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.form-group.f2 {
  flex: 2;
}

.form-group.f1 {
  flex: 1;
}

.form-label {
  font-size: var(--font-size-sm);
  color: var(--color-text-secondary);
  font-weight: 500;
}

.form-section-label {
  font-size: var(--font-size-xxs);
  font-weight: 600;
  color: var(--brand-success);
  letter-spacing: 0.5px;
  margin-bottom: 6px;
}

.form-input,
.form-select {
  width: 100%;
  height: var(--height-input);
  padding: 0 10px;
  background: var(--color-bg-raised, #11111b);
  border: 1px solid rgba(255, 255, 255, 0.05);
  border-radius: var(--border-radius-sm);
  color: var(--color-text-primary);
  font-size: var(--font-size-md);
  outline: none;
  transition: border-color 0.2s;
}

.form-input:focus,
.form-select:focus {
  border-color: #89b4fa;
}

.form-select {
  cursor: pointer;
}

/* 范围标签 */
.profile-scope-badge {
  font-size: var(--font-size-xxs);
  padding: 2px 8px;
  border-radius: 10px;
  font-weight: 600;
  margin-top: 4px;
  display: inline-block;
}

.profile-scope-badge.global {
  background: rgba(166, 227, 161, 0.1);
  color: var(--brand-success);
}

.profile-scope-badge.project {
  background: rgba(203, 166, 247, 0.1);
  color: #cba6f7;
}
</style>