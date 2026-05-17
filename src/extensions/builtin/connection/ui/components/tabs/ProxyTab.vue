<template>
  <div class="proxy-tab">
    <div class="tab-description">
      <Globe :size="16" class="desc-icon" />
      <span>通过 HTTP / SOCKS 代理服务器连接数据库，适用于需要通过代理访问外网的网络环境</span>
    </div>

    <label class="toggle-section">
      <NSwitch
        :value="config.enabled"
        size="medium"
        @update:value="(v: boolean) => (config.enabled = v)"
      />
      <span class="toggle-label">启用代理</span>
    </label>

    <div v-if="config.enabled" class="config-body">
      <div class="form-section">
        <h4 class="section-title">代理服务器</h4>

        <div class="form-group">
          <label class="form-label">代理类型</label>
          <div class="proxy-type-selector">
            <button
              v-for="pt in proxyTypes"
              :key="pt.value"
              type="button"
              class="proxy-type-btn"
              :class="{ active: config.type === pt.value }"
              @click="config.type = pt.value"
            >
              {{ pt.label }}
            </button>
          </div>
        </div>

        <div class="form-row">
          <div class="form-group flex-2">
            <label class="form-label">代理主机 <span class="required">*</span></label>
            <input
              v-model="config.host"
              type="text"
              class="form-input"
              placeholder="proxy.example.com"
            />
          </div>
          <div class="form-group flex-1">
            <label class="form-label">端口 <span class="required">*</span></label>
            <input
              v-model.number="config.port"
              type="number"
              class="form-input"
              :placeholder="defaultPortForType.toString()"
            />
          </div>
        </div>
      </div>

      <div class="form-section">
        <h4 class="section-title">认证（可选）</h4>

        <label class="toggle-section sub">
          <NSwitch
            :value="config.requireAuth"
            size="small"
            @update:value="(v: boolean) => (config.requireAuth = v)"
          />
          <span class="toggle-label">需要认证</span>
        </label>

        <div v-if="config.requireAuth" class="auth-fields">
          <div class="form-row">
            <div class="form-group flex-1">
              <label class="form-label">用户名</label>
              <input
                v-model="config.username"
                type="text"
                class="form-input"
                placeholder="代理用户名"
              />
            </div>
            <div class="form-group flex-1">
              <label class="form-label">密码</label>
              <div class="password-wrapper">
                <input
                  v-model="config.password"
                  :type="showPassword ? 'text' : 'password'"
                  class="form-input"
                  placeholder="代理密码"
                />
                <button type="button" class="toggle-vis" @click="showPassword = !showPassword">
                  <Eye v-if="!showPassword" :size="16" />
                  <EyeOff v-else :size="16" />
                </button>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { Eye, EyeOff, Globe } from 'lucide-vue-next'
import { NSwitch } from 'naive-ui'
import { ref, computed } from 'vue'

import type { ProxyConnectionConfig } from '../../composables/useConnectionForm'

interface Props {
  config: ProxyConnectionConfig
}

const props = defineProps<Props>()

const showPassword = ref(false)

const proxyTypes = [
  { label: 'HTTP', value: 'http' },
  { label: 'HTTPS', value: 'https' },
  { label: 'SOCKS4', value: 'socks4' },
  { label: 'SOCKS5', value: 'socks5' },
] as const

const defaultPortForType = computed(() => {
  switch (props.config.type) {
    case 'http': return 8080
    case 'https': return 8443
    case 'socks4':
    case 'socks5': return 1080
    default: return 1080
  }
})
</script>

<style scoped>
.proxy-tab {
  display: flex;
  flex-direction: column;
  gap: 24px;
  max-width: 640px;
}

.tab-description {
  display: flex;
  align-items: flex-start;
  gap: 10px;
  padding: 14px 16px;
  background: rgba(34, 197, 94, 0.06);
  border: 1px solid rgba(34, 197, 94, 0.15);
  border-radius: 8px;
  font-size: 13px;
  color: var(--text-secondary);
  line-height: 1.6;
}

.desc-icon {
  color: #22c55e;
  flex-shrink: 0;
  margin-top: 1px;
}

.toggle-section {
  display: flex;
  align-items: center;
  gap: 12px;
  cursor: pointer;
}

.toggle-section.sub {
  gap: 10px;
}

.toggle-label {
  font-size: 15px;
  font-weight: 600;
  color: var(--text-primary);
}

.toggle-section.sub .toggle-label {
  font-size: 14px;
  font-weight: 500;
}

.config-body {
  display: flex;
  flex-direction: column;
  gap: 24px;
}

.form-section {
  display: flex;
  flex-direction: column;
  gap: 14px;
}

.section-title {
  font-size: 13px;
  font-weight: 600;
  color: var(--text-secondary);
  text-transform: uppercase;
  letter-spacing: 0.5px;
  margin: 0;
  padding-bottom: 8px;
  border-bottom: 1px solid var(--border-color);
}

.form-row {
  display: flex;
  gap: 14px;
}

.flex-1 {
  flex: 1;
}

.flex-2 {
  flex: 2;
}

.form-group {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.form-label {
  font-size: 13px;
  font-weight: 500;
  color: var(--text-primary);
}

.required {
  color: var(--danger-color);
}

.form-input {
  height: 36px;
  padding: 0 12px;
  background: var(--bg-primary);
  border: 1px solid var(--border-color);
  border-radius: 6px;
  font-size: 14px;
  color: var(--text-primary);
  outline: none;
  transition: all 0.2s;
}

.form-input:focus {
  border-color: var(--primary-color);
  box-shadow: 0 0 0 2px rgba(var(--primary-rgb), 0.15);
}

.auth-fields {
  padding: 16px;
  background: var(--bg-secondary);
  border: 1px solid var(--border-color);
  border-radius: 8px;
}

.proxy-type-selector {
  display: flex;
  gap: 0;
  border: 1px solid var(--border-color);
  border-radius: 6px;
  overflow: hidden;
}

.proxy-type-btn {
  flex: 1;
  padding: 8px 12px;
  background: var(--bg-primary);
  border: none;
  border-right: 1px solid var(--border-color);
  font-size: 13px;
  font-weight: 500;
  color: var(--text-secondary);
  cursor: pointer;
  transition: all 0.2s;
}

.proxy-type-btn:last-child {
  border-right: none;
}

.proxy-type-btn:hover {
  background: var(--bg-hover);
  color: var(--text-primary);
}

.proxy-type-btn.active {
  background: var(--primary-color);
  color: white;
}

.password-wrapper {
  position: relative;
}

.password-wrapper .form-input {
  width: 100%;
  padding-right: 44px;
}

.toggle-vis {
  position: absolute;
  right: 4px;
  top: 50%;
  transform: translateY(-50%);
  width: 32px;
  height: 32px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: transparent;
  border: none;
  border-radius: 4px;
  color: var(--text-tertiary);
  cursor: pointer;
  transition: all 0.2s;
}

.toggle-vis:hover {
  color: var(--text-primary);
  background: var(--bg-hover);
}
</style>