<template>
  <div class="ssh-tunnel-tab">
    <div class="tab-description">
      <Server :size="16" class="desc-icon" />
      <span>使用 SSH 隧道安全连接到数据库服务器，所有通信将经过加密的 SSH 通道传输</span>
    </div>

    <label class="toggle-section">
      <NSwitch
        :value="config.enabled"
        size="medium"
        @update:value="(v: boolean) => (config.enabled = v)"
      />
      <span class="toggle-label">启用 SSH 隧道</span>
    </label>

    <div v-if="config.enabled" class="config-body">
      <div class="form-section">
        <h4 class="section-title">基本配置</h4>

        <div class="form-row">
          <div class="form-group flex-2">
            <label class="form-label">SSH 主机 <span class="required">*</span></label>
            <input
              v-model="config.host"
              type="text"
              class="form-input"
              placeholder="192.168.1.100"
            />
          </div>
          <div class="form-group flex-1">
            <label class="form-label">端口</label>
            <input
              v-model.number="config.port"
              type="number"
              class="form-input"
              placeholder="22"
            />
          </div>
        </div>

        <div class="form-row">
          <div class="form-group flex-1">
            <label class="form-label">用户名 <span class="required">*</span></label>
            <input
              v-model="config.username"
              type="text"
              class="form-input"
              placeholder="root"
            />
          </div>
          <div class="form-group flex-1">
            <label class="form-label">认证方式</label>
            <select v-model="config.authType" class="form-select">
              <option value="password">密码</option>
              <option value="keyFile">密钥文件</option>
            </select>
          </div>
        </div>
      </div>

      <div class="form-section">
        <h4 class="section-title">认证配置</h4>

        <div v-if="config.authType === 'password'" class="form-group">
          <label class="form-label">密码 <span class="required">*</span></label>
          <div class="password-wrapper">
            <input
              v-model="config.password"
              :type="showPassword ? 'text' : 'password'"
              class="form-input"
              placeholder="SSH 密码"
            />
            <button type="button" class="toggle-vis" @click="showPassword = !showPassword">
              <Eye v-if="!showPassword" :size="16" />
              <EyeOff v-else :size="16" />
            </button>
          </div>
        </div>

        <div v-else class="form-group">
          <label class="form-label">私钥文件 <span class="required">*</span></label>
          <div class="file-input-row">
            <input
              v-model="config.privateKey"
              type="text"
              class="form-input"
              placeholder="~/.ssh/id_rsa"
            />
            <button type="button" class="btn-browse" @click="selectKeyFile">浏览</button>
          </div>
          <p class="form-hint">支持 OpenSSH 格式私钥 (RSA/DSA/ECDSA/Ed25519)</p>
        </div>
      </div>

      <div class="form-section">
        <h4 class="section-title">高级选项</h4>

        <div class="form-row">
          <div class="form-group flex-1">
            <label class="form-label">本地端口</label>
            <input
              v-model.number="config.localPort"
              type="number"
              class="form-input"
              placeholder="自动分配"
            />
            <p class="form-hint">留空或填 0 则自动分配</p>
          </div>
          <div class="form-group flex-1">
            <label class="form-label">保活间隔 (秒)</label>
            <input
              v-model.number="config.keepAlive"
              type="number"
              class="form-input"
              placeholder="0 = 禁用"
            />
            <p class="form-hint">0 表示不发送保活包</p>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { open } from '@tauri-apps/plugin-dialog'
import { Eye, EyeOff, Server } from 'lucide-vue-next'
import { NSwitch } from 'naive-ui'
import { ref } from 'vue'

import type { SshTunnelConfig } from '../../composables/useConnectionForm'

interface Props {
  config: SshTunnelConfig
}

const props = defineProps<Props>()

const showPassword = ref(false)

async function selectKeyFile() {
  const selected = await open({
    multiple: false,
    title: '选择 SSH 私钥文件',
    filters: [
      { name: 'SSH Keys', extensions: ['pem', 'key', ''] },
      { name: 'All Files', extensions: ['*'] },
    ],
  })
  if (selected) {
    props.config.privateKey = selected as string
  }
}
</script>

<style scoped>
.ssh-tunnel-tab {
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
  background: rgba(99, 102, 241, 0.06);
  border: 1px solid rgba(99, 102, 241, 0.15);
  border-radius: 8px;
  font-size: 13px;
  color: var(--text-secondary);
  line-height: 1.6;
}

.desc-icon {
  color: #6366f1;
  flex-shrink: 0;
  margin-top: 1px;
}

.toggle-section {
  display: flex;
  align-items: center;
  gap: 12px;
  cursor: pointer;
}

.toggle-label {
  font-size: 15px;
  font-weight: 600;
  color: var(--text-primary);
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

.form-input,
.form-select {
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

.form-input:focus,
.form-select:focus {
  border-color: var(--primary-color);
  box-shadow: 0 0 0 2px rgba(var(--primary-rgb), 0.15);
}

.form-select {
  cursor: pointer;
}

.form-hint {
  font-size: 12px;
  color: var(--text-tertiary);
  margin: 0;
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

.file-input-row {
  display: flex;
  gap: 8px;
}

.file-input-row .form-input {
  flex: 1;
}

.btn-browse {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 0 14px;
  height: 36px;
  background: var(--bg-secondary);
  border: 1px solid var(--border-color);
  border-radius: 6px;
  font-size: 13px;
  color: var(--text-primary);
  cursor: pointer;
  transition: all 0.2s;
  white-space: nowrap;
}

.btn-browse:hover {
  background: var(--bg-hover);
  border-color: var(--primary-color);
}
</style>