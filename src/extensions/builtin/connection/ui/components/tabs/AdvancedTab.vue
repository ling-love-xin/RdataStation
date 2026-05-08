<template>
  <div class="advanced-tab">
    <!-- SSH 隧道配置 -->
    <div class="config-section">
      <div class="section-header">
        <div class="header-left">
          <Shield :size="16" class="section-icon" />
          <h4 class="section-title">{{ t('connection.advancedTab.sshTunnel') }}</h4>
        </div>
        <button
          type="button"
          class="section-switch"
          :class="{ active: enableSsh }"
          @click="enableSsh = !enableSsh"
        >
          <span class="switch-knob"></span>
        </button>
      </div>

      <div v-if="enableSsh" class="section-content">
        <div class="form-row">
          <div class="form-section flex-2">
            <label class="form-label">{{ t('connection.advancedTab.sshHost') }}</label>
            <input
              v-model="formData.sshHost"
              type="text"
              class="form-input"
              :placeholder="t('connection.advancedTab.sshHostPlaceholder')"
            />
          </div>
          <div class="form-section flex-1">
            <label class="form-label">{{ t('connection.advancedTab.sshPort') }}</label>
            <input
              v-model.number="formData.sshPort"
              type="number"
              class="form-input"
              :placeholder="t('connection.advancedTab.sshPortPlaceholder')"
            />
          </div>
        </div>

        <div class="form-row">
          <div class="form-section flex-1">
            <label class="form-label">{{ t('connection.advancedTab.sshUsername') }}</label>
            <input
              v-model="formData.sshUsername"
              type="text"
              class="form-input"
              :placeholder="t('connection.advancedTab.sshUsernamePlaceholder')"
            />
          </div>
          <div class="form-section flex-1">
            <label class="form-label">{{ t('connection.advancedTab.sshPassword') }}</label>
            <div class="password-wrapper">
              <input
                v-model="formData.sshPassword"
                :type="showSshPassword ? 'text' : 'password'"
                class="form-input"
                :placeholder="t('connection.advancedTab.sshPasswordPlaceholder')"
              />
              <button
                type="button"
                class="btn-toggle-password"
                @click="showSshPassword = !showSshPassword"
              >
                <Eye v-if="!showSshPassword" :size="14" />
                <EyeOff v-else :size="14" />
              </button>
            </div>
          </div>
        </div>

        <div class="form-section">
          <label class="form-label">{{ t('connection.advancedTab.sshKeyPath') }}</label>
          <div class="file-input-wrapper">
            <input
              v-model="formData.sshKeyPath"
              type="text"
              class="form-input"
              :placeholder="t('connection.advancedTab.sshKeyPathPlaceholder')"
            />
            <button type="button" class="btn-file" @click="selectSshKeyFile">
              <FolderOpen :size="14" />
            </button>
          </div>
        </div>
      </div>
    </div>

    <!-- SSL 加密配置 -->
    <div class="config-section">
      <div class="section-header">
        <div class="header-left">
          <Lock :size="16" class="section-icon" />
          <h4 class="section-title">{{ t('connection.advancedTab.sslTls') }}</h4>
        </div>
        <button
          type="button"
          class="section-switch"
          :class="{ active: enableSsl }"
          @click="enableSsl = !enableSsl"
        >
          <span class="switch-knob"></span>
        </button>
      </div>

      <div v-if="enableSsl" class="section-content">
        <div class="form-section">
          <label class="form-label">{{ t('connection.advancedTab.sslMode') }}</label>
          <select v-model="formData.sslMode" class="form-select">
            <option value="disable">{{ t('connection.advancedTab.sslDisable') }}</option>
            <option value="require">{{ t('connection.advancedTab.sslRequire') }}</option>
            <option value="verify-ca">{{ t('connection.advancedTab.sslVerifyCa') }}</option>
            <option value="verify-full">{{ t('connection.advancedTab.sslVerifyFull') }}</option>
          </select>
        </div>

        <div class="form-section">
          <label class="form-label">{{ t('connection.advancedTab.sslCa') }}</label>
          <div class="file-input-wrapper">
            <input
              v-model="formData.sslCa"
              type="text"
              class="form-input"
              :placeholder="t('connection.advancedTab.sslCaPlaceholder')"
            />
            <button type="button" class="btn-file" @click="selectSslFile('ca')">
              <FolderOpen :size="14" />
            </button>
          </div>
        </div>

        <div class="form-row">
          <div class="form-section flex-1">
            <label class="form-label">{{ t('connection.advancedTab.sslCert') }}</label>
            <div class="file-input-wrapper">
              <input
                v-model="formData.sslCert"
                type="text"
                class="form-input"
                :placeholder="t('connection.advancedTab.sslCertPlaceholder')"
              />
              <button type="button" class="btn-file" @click="selectSslFile('cert')">
                <FolderOpen :size="14" />
              </button>
            </div>
          </div>
          <div class="form-section flex-1">
            <label class="form-label">{{ t('connection.advancedTab.sslKey') }}</label>
            <div class="file-input-wrapper">
              <input
                v-model="formData.sslKey"
                type="text"
                class="form-input"
                :placeholder="t('connection.advancedTab.sslKeyPlaceholder')"
              />
              <button type="button" class="btn-file" @click="selectSslFile('key')">
                <FolderOpen :size="14" />
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { Shield, Lock, Eye, EyeOff, FolderOpen } from 'lucide-vue-next'
import { ref, computed } from 'vue'
import { useI18n } from 'vue-i18n'

const { t } = useI18n()

import type { DriverDescriptor, ConnectionConfig } from '../../types/connection'

const formData = defineModel<Partial<ConnectionConfig>>('formData', { required: true })

defineProps<{
  selectedDriver: DriverDescriptor | null
}>()

const emit = defineEmits<{
  'update:formData': [data: Partial<ConnectionConfig>]
}>()

const showSshPassword = ref(false)

const enableSsh = computed({
  get: () => formData.value.authMethod === 'ssh' || !!formData.value.sshHost,
  set: (val: boolean) => {
    if (val) {
      emit('update:formData', {
        ...formData.value,
        authMethod: 'ssh',
        sshPort: formData.value.sshPort || 22,
      })
    } else {
      emit('update:formData', {
        ...formData.value,
        authMethod: 'password',
        sshHost: '',
        sshPort: 22,
        sshUsername: '',
        sshPassword: '',
        sshKeyPath: '',
      })
    }
  },
})

const enableSsl = computed({
  get: () => formData.value.sslMode && formData.value.sslMode !== 'disable',
  set: (val: boolean) => {
    emit('update:formData', {
      ...formData.value,
      sslMode: val ? 'require' : 'disable',
    })
  },
})

function selectSshKeyFile() {
  // TODO: 实现文件选择
  console.log('选择 SSH 私钥文件')
}

function selectSslFile(type: 'ca' | 'cert' | 'key') {
  // TODO: 实现文件选择
  console.log('选择 SSL 文件:', type)
}
</script>

<style scoped>
.advanced-tab {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.config-section {
  background: var(--bg-secondary);
  border-radius: var(--radius-md);
  border: 1px solid var(--border-color);
  overflow: hidden;
}

.section-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 16px;
  background: var(--bg-tertiary);
  cursor: pointer;
  transition: background 0.2s;
}

.section-header:hover {
  background: var(--bg-hover);
}

.header-left {
  display: flex;
  align-items: center;
  gap: 8px;
}

.section-icon {
  color: var(--text-tertiary);
}

.section-title {
  font-size: 14px;
  font-weight: 600;
  color: var(--text-primary);
  margin: 0;
}

.section-switch {
  position: relative;
  width: 36px;
  height: 18px;
  background: var(--border-color);
  border-radius: 9px;
  border: none;
  padding: 0;
  cursor: pointer;
  transition: all 0.2s;
}

.section-switch.active {
  background: var(--primary-color);
}

.switch-knob {
  position: absolute;
  top: 2px;
  left: 2px;
  width: 14px;
  height: 14px;
  background: white;
  border-radius: 50%;
  transition: all 0.2s;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.2);
}

.section-switch.active .switch-knob {
  left: 20px;
}

.section-content {
  padding: 16px;
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.form-section {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.form-row {
  display: flex;
  gap: 12px;
}

.flex-1 {
  flex: 1;
}

.flex-2 {
  flex: 2;
}

.form-label {
  font-size: 13px;
  font-weight: 500;
  color: var(--text-primary);
}

.form-input,
.form-select {
  height: 32px;
  padding: 0 12px;
  background: var(--bg-tertiary);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-md);
  color: var(--text-primary);
  font-size: 13px;
  outline: none;
  transition: all 0.2s;
}

.form-input:focus,
.form-select:focus {
  border-color: var(--primary-color);
  background: var(--bg-primary);
}

.form-select {
  cursor: pointer;
}

.password-wrapper {
  position: relative;
  display: flex;
  align-items: center;
}

.password-wrapper .form-input {
  flex: 1;
  padding-right: 36px;
}

.btn-toggle-password {
  position: absolute;
  right: 8px;
  width: 24px;
  height: 24px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: transparent;
  border: none;
  color: var(--text-tertiary);
  cursor: pointer;
  transition: color 0.2s;
}

.btn-toggle-password:hover {
  color: var(--text-primary);
}

.file-input-wrapper {
  display: flex;
  gap: 8px;
}

.file-input-wrapper .form-input {
  flex: 1;
}

.btn-file {
  width: 32px;
  height: 32px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--bg-tertiary);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-md);
  color: var(--text-secondary);
  cursor: pointer;
  transition: all 0.2s;
}

.btn-file:hover {
  background: var(--bg-hover);
  color: var(--text-primary);
  border-color: var(--primary-color);
}
</style>
