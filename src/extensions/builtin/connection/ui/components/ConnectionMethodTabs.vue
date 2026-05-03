<template>
  <div class="connection-method-tabs">
    <div class="tabs-header">
      <button
        v-for="tab in availableTabs"
        :key="tab.key"
        class="tab-button"
        :class="{ active: modelValue === tab.key }"
        @click="selectTab(tab.key)"
      >
        <svg class="tab-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <component :is="tab.icon" />
        </svg>
        <span>{{ tab.label }}</span>
      </button>
    </div>

    <div class="tabs-content">
      <!-- 直接连接 -->
      <div v-show="modelValue === 'direct'" class="tab-panel">
        <div class="panel-info">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <circle cx="12" cy="12" r="10"/>
            <line x1="12" y1="16" x2="12" y2="12"/>
            <line x1="12" y1="8" x2="12.01" y2="8"/>
          </svg>
          <p>直接连接到数据库服务器，适用于本地开发或内网环境。</p>
        </div>
      </div>

      <!-- SSH 隧道 -->
      <div v-show="modelValue === 'ssh'" class="tab-panel">
        <SshConfigForm v-model="sshConfig" />
      </div>

      <!-- SSL -->
      <div v-show="modelValue === 'ssl'" class="tab-panel">
        <SslConfigForm v-model="sslConfig" />
      </div>

      <!-- 代理 -->
      <div v-show="modelValue === 'proxy'" class="tab-panel">
        <ProxyConfigForm v-model="proxyConfig" />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, watch, ref } from 'vue'

import ProxyConfigForm from './ProxyConfigForm.vue'
import SshConfigForm from './SshConfigForm.vue'
import SslConfigForm from './SslConfigForm.vue'

interface Props {
  modelValue: string | Record<string, any>
  driverSupportsSsh?: boolean
  driverSupportsSsl?: boolean
  driverSupportsProxy?: boolean
}

interface Emits {
  (e: 'update:modelValue', value: string | Record<string, any>): void
  (e: 'update:sshConfig', value: Record<string, unknown>): void
  (e: 'update:sslConfig', value: Record<string, unknown>): void
  (e: 'update:proxyConfig', value: Record<string, unknown>): void
}

const props = withDefaults(defineProps<Props>(), {
  driverSupportsSsh: false,
  driverSupportsSsl: false,
  driverSupportsProxy: false
})

const emit = defineEmits<Emits>()

// 配置数据
const sshConfig = ref<Record<string, unknown>>({})
const sslConfig = ref<Record<string, unknown>>({})
const proxyConfig = ref<Record<string, unknown>>({})

// 监听配置变化
watch(sshConfig, (value) => emit('update:sshConfig', value), { deep: true })
watch(sslConfig, (value) => emit('update:sslConfig', value), { deep: true })
watch(proxyConfig, (value) => emit('update:proxyConfig', value), { deep: true })

// 可用的标签页
const availableTabs = computed(() => {
  const tabs = [
    { key: 'direct', label: '直接连接', icon: 'DirectIcon' }
  ]

  if (props.driverSupportsSsh) {
    tabs.push({ key: 'ssh', label: 'SSH 隧道', icon: 'SshIcon' })
  }

  if (props.driverSupportsSsl) {
    tabs.push({ key: 'ssl', label: 'SSL/TLS', icon: 'SslIcon' })
  }

  if (props.driverSupportsProxy) {
    tabs.push({ key: 'proxy', label: '代理', icon: 'ProxyIcon' })
  }

  return tabs
})

// 选择标签
function selectTab(key: string) {
  emit('update:modelValue', key)
}
</script>



<style scoped>
.connection-method-tabs {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.tabs-header {
  display: flex;
  gap: 8px;
  padding: 4px;
  background: var(--bg-secondary);
  border-radius: 8px;
}

.tab-button {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 8px 16px;
  border: none;
  border-radius: 6px;
  font-size: 13px;
  font-weight: 500;
  color: var(--text-secondary);
  background: transparent;
  cursor: pointer;
  transition: all 0.2s ease;
}

.tab-button:hover {
  color: var(--text-primary);
  background: var(--bg-hover);
}

.tab-button.active {
  color: var(--primary-color);
  background: var(--bg-primary);
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
}

.tab-icon {
  width: 16px;
  height: 16px;
}

.tabs-content {
  min-height: 200px;
}

.tab-panel {
  animation: fadeIn 0.2s ease;
}

@keyframes fadeIn {
  from {
    opacity: 0;
    transform: translateY(-5px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

.panel-info {
  display: flex;
  align-items: flex-start;
  gap: 12px;
  padding: 16px;
  background: rgba(22, 93, 255, 0.05);
  border: 1px solid rgba(22, 93, 255, 0.1);
  border-radius: 8px;
}

.panel-info svg {
  width: 20px;
  height: 20px;
  color: var(--primary-color);
  flex-shrink: 0;
  margin-top: 2px;
}

.panel-info p {
  margin: 0;
  font-size: 13px;
  color: var(--text-secondary);
  line-height: 1.5;
}
</style>
