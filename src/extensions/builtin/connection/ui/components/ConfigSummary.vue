<template>
  <div class="config-summary">
    <h3 class="summary-title">配置摘要</h3>

    <div class="summary-card">
      <div class="summary-section">
        <h4>基本信息</h4>
        <div class="summary-row">
          <span class="label">连接名称:</span>
          <span class="value">{{ config.name || '-' }}</span>
        </div>
        <div class="summary-row">
          <span class="label">数据库类型:</span>
          <span class="value">{{ driverName }}</span>
        </div>
      </div>

      <div class="summary-section">
        <h4>连接参数</h4>
        <div class="summary-row">
          <span class="label">主机:</span>
          <span class="value">{{ config.host || '-' }}</span>
        </div>
        <div class="summary-row">
          <span class="label">端口:</span>
          <span class="value">{{ config.port || '-' }}</span>
        </div>
        <div v-if="config.database" class="summary-row">
          <span class="label">数据库:</span>
          <span class="value">{{ config.database }}</span>
        </div>
        <div v-if="(config as any).file_path" class="summary-row">
          <span class="label">文件路径:</span>
          <span class="value file-path">{{ (config as any).file_path }}</span>
        </div>
      </div>

      <div class="summary-section">
        <h4>认证信息</h4>
        <div class="summary-row">
          <span class="label">用户名:</span>
          <span class="value">{{ config.username || '-' }}</span>
        </div>
        <div class="summary-row">
          <span class="label">密码:</span>
          <span class="value">{{ config.password ? '••••••••' : '-' }}</span>
        </div>
      </div>

      <div v-if="hasAdvancedConfig" class="summary-section">
        <h4>高级选项</h4>
        <div v-if="connectionMethod !== 'direct'" class="summary-row">
          <span class="label">连接方式:</span>
          <span class="value method-tag" :class="connectionMethod">{{ methodLabel }}</span>
        </div>
        <div v-if="Object.keys(config.options || {}).length > 0" class="summary-row">
          <span class="label">自定义选项:</span>
          <span class="value">{{ Object.keys(config.options || {}).length }} 个</span>
        </div>
      </div>
    </div>

    <!-- 连接测试区域 -->
    <div class="test-section">
      <button
        class="btn-test"
        :class="{
          testing: isTesting,
          success: testStatus === 'success',
          error: testStatus === 'error',
        }"
        :disabled="isTesting || !canTest"
        @click="testConnection"
      >
        <span v-if="isTesting" class="spinner"></span>
        <svg
          v-else-if="testStatus === 'success'"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="3"
        >
          <polyline points="20 6 9 17 4 12" />
        </svg>
        <svg
          v-else-if="testStatus === 'error'"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
        >
          <circle cx="12" cy="12" r="10" />
          <line x1="15" y1="9" x2="9" y2="15" />
          <line x1="9" y1="9" x2="15" y2="15" />
        </svg>
        <svg v-else viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M22 11.08V12a10 10 0 1 1-5.93-9.14" />
          <polyline points="22 4 12 14.01 9 11.01" />
        </svg>
        {{ testButtonText }}
      </button>

      <div v-if="testMessage" class="test-message" :class="testStatus">
        {{ testMessage }}
      </div>

      <div v-if="testDetails" class="test-details">
        <div class="detail-row">
          <span class="detail-label">响应时间:</span>
          <span class="detail-value">{{ testDetails.responseTime }}ms</span>
        </div>
        <div class="detail-row">
          <span class="detail-label">服务器版本:</span>
          <span class="detail-value">{{ testDetails.serverVersion }}</span>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'

import type { ConnectionConfig } from '../types/connection'
import type { DriverDescriptor } from '../types/driver'

interface Props {
  config: Partial<ConnectionConfig>
  driver: DriverDescriptor | null
  connectionMethod: string
  canTest: boolean
}

const props = defineProps<Props>()
// eslint-disable-next-line @typescript-eslint/no-unused-vars
const _emit = defineEmits<{
  (e: 'test'): void
}>()

// 测试状态
const isTesting = ref(false)
const testStatus = ref<'idle' | 'success' | 'error'>('idle')
const testMessage = ref('')
const testDetails = ref<{ responseTime: number; serverVersion: string } | null>(null)

// 计算属性
const driverName = computed(() => props.driver?.name || '-')

const hasAdvancedConfig = computed(() => {
  return props.connectionMethod !== 'direct' || Object.keys(props.config.options || {}).length > 0
})

const methodLabel = computed(() => {
  const labels: Record<string, string> = {
    direct: '直接连接',
    ssh: 'SSH 隧道',
    ssl: 'SSL/TLS',
    proxy: '代理',
  }
  return labels[props.connectionMethod] || props.connectionMethod
})

const testButtonText = computed(() => {
  if (isTesting.value) return '测试中...'
  if (testStatus.value === 'success') return '连接成功'
  if (testStatus.value === 'error') return '连接失败'
  return '测试连接'
})

// 获取数据库版本信息
const getServerVersion = computed(() => {
  const driverId = props.driver?.id
  const versions: Record<string, string> = {
    mysql: 'MySQL 8.0.32',
    postgres: 'PostgreSQL 15.2',
    sqlite: 'SQLite 3.40.0',
    duckdb: 'DuckDB 0.9.2',
  }
  return versions[driverId || ''] || 'Unknown'
})

// 测试连接
async function testConnection() {
  if (!props.canTest || !props.driver) return

  isTesting.value = true
  testStatus.value = 'idle'
  testMessage.value = ''
  testDetails.value = null

  try {
    // 构建连接 URL
    const { invoke } = await import('@tauri-apps/api/core')

    let url = ''
    const config = props.config as any

    if (props.driver.requireFile) {
      // 文件型数据库
      url = `${props.driver.id}:///${config.file_path || config.filePath || ''}`
    } else {
      // 网络型数据库
      const port = config.port || props.driver.default_port || 3306
      const auth = config.username
        ? config.password
          ? `${encodeURIComponent(config.username)}:${encodeURIComponent(config.password)}@`
          : `${encodeURIComponent(config.username)}@`
        : ''
      url = `${props.driver.id}://${auth}${config.host || 'localhost'}:${port}/${encodeURIComponent(config.database || '')}`
    }

    // 调用真实的测试 API
    const result = await invoke<{
      success: boolean
      message: string
      server_version: string
      response_time_ms: number
    }>('test_connection', {
      dbType: props.driver.id,
      url,
    })

    // 显示成功信息
    const driverName = props.driver?.name || '数据库'
    testStatus.value = 'success'
    testMessage.value = `连接成功！${driverName} 服务器响应正常。`
    testDetails.value = {
      responseTime: result.response_time_ms,
      serverVersion: result.server_version,
    }
  } catch (error) {
    testStatus.value = 'error'
    testMessage.value = error instanceof Error ? error.message : '连接失败，请检查配置'
  } finally {
    isTesting.value = false
  }
}
</script>

<style scoped>
.config-summary {
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.summary-title {
  font-size: 16px;
  font-weight: 600;
  color: var(--text-primary);
  margin: 0;
}

.summary-card {
  background: var(--bg-secondary);
  border-radius: 10px;
  padding: 20px;
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.summary-section {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.summary-section h4 {
  font-size: 13px;
  font-weight: 600;
  color: var(--text-secondary);
  margin: 0 0 4px;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.summary-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  font-size: 14px;
}

.summary-row .label {
  color: var(--text-tertiary);
}

.summary-row .value {
  color: var(--text-primary);
  font-weight: 500;
}

.summary-row .value.file-path {
  max-width: 200px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.method-tag {
  padding: 2px 10px;
  border-radius: 4px;
  font-size: 12px;
  font-weight: 500;
}

.method-tag.direct {
  background: rgba(22, 93, 255, 0.1);
  color: var(--primary-color);
}

.method-tag.ssh {
  background: rgba(0, 180, 42, 0.1);
  color: var(--success-color);
}

.method-tag.ssl {
  background: rgba(255, 125, 0, 0.1);
  color: var(--warning-color);
}

.method-tag.proxy {
  background: rgba(102, 102, 102, 0.1);
  color: var(--text-secondary);
}

/* 测试区域 */
.test-section {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.btn-test {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  padding: 12px 24px;
  background: var(--primary-color);
  border: none;
  border-radius: 8px;
  font-size: 14px;
  font-weight: 600;
  color: white;
  cursor: pointer;
  transition: all 0.2s ease;
}

.btn-test:hover:not(:disabled) {
  background: var(--primary-hover);
  transform: translateY(-1px);
  box-shadow: 0 4px 12px rgba(22, 93, 255, 0.3);
}

.btn-test:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.btn-test.success {
  background: var(--success-color);
}

.btn-test.error {
  background: var(--danger-color);
}

.btn-test svg {
  width: 18px;
  height: 18px;
}

.spinner {
  width: 18px;
  height: 18px;
  border: 2px solid rgba(255, 255, 255, 0.3);
  border-top-color: white;
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
}

@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}

.test-message {
  padding: 12px 16px;
  border-radius: 8px;
  font-size: 13px;
  text-align: center;
}

.test-message.success {
  background: rgba(0, 180, 42, 0.1);
  color: var(--success-color);
  border: 1px solid rgba(0, 180, 42, 0.2);
}

.test-message.error {
  background: rgba(245, 63, 63, 0.1);
  color: var(--danger-color);
  border: 1px solid rgba(245, 63, 63, 0.2);
}

.test-details {
  display: flex;
  flex-direction: column;
  gap: 8px;
  padding: 12px 16px;
  background: var(--bg-secondary);
  border-radius: 8px;
}

.detail-row {
  display: flex;
  justify-content: space-between;
  font-size: 13px;
}

.detail-label {
  color: var(--text-tertiary);
}

.detail-value {
  color: var(--text-primary);
  font-weight: 500;
}
</style>
