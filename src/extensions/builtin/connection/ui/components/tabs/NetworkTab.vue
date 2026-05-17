<template>
  <div class="network-tab">
    <div class="collapse-panel">
      <div class="collapse-header" @click="sshExpanded = !sshExpanded">
        <ChevronRight :size="12" class="carrow" :class="{ open: sshExpanded }" />
        <span>{{ t('connection.networkTab.sshTunnel') }}</span>
        <div class="cswitch" @click.stop>
          <span class="switch-label">{{ sshEnabled ? t('connection.networkTab.enabled') : t('connection.networkTab.disabled') }}</span>
          <div class="switch-toggle" :class="{ on: sshEnabled }" @click="sshEnabled = !sshEnabled" />
        </div>
      </div>
      <div v-if="sshExpanded" class="collapse-body">
        <div class="form-row">
          <div class="form-group f2">
            <span class="form-label">{{ t('connection.networkTab.sshHost') }}</span>
            <input v-model="sshHost" type="text" class="form-input" placeholder="192.168.1.100" />
          </div>
          <div class="form-group f1">
            <span class="form-label">{{ t('connection.networkTab.port') }}</span>
            <input v-model="sshPort" type="number" class="form-input" />
          </div>
        </div>
        <div class="form-row">
          <div class="form-group f1">
            <span class="form-label">{{ t('connection.networkTab.username') }}</span>
            <input v-model="sshUsername" type="text" class="form-input" placeholder="root" />
          </div>
          <div class="form-group f1">
            <span class="form-label">{{ t('connection.networkTab.authMethod') }}</span>
            <select v-model="sshAuthType" class="form-select">
              <option value="password">{{ t('connection.networkTab.password') }}</option>
              <option value="key">{{ t('connection.networkTab.keyFile') }}</option>
            </select>
          </div>
        </div>
        <div v-if="sshAuthType === 'password'" class="form-row">
          <div class="form-group f1">
            <span class="form-label">{{ t('connection.networkTab.sshPassword') }}</span>
            <input v-model="sshPassword" type="password" class="form-input" />
          </div>
        </div>
        <div v-if="sshAuthType === 'key'" class="form-row">
          <div class="form-group f1">
            <span class="form-label">{{ t('connection.networkTab.privateKey') }}</span>
            <input v-model="sshKeyPath" type="text" class="form-input" placeholder="~/.ssh/id_rsa" />
          </div>
        </div>
        <div class="form-row">
          <div class="form-group f1">
            <span class="form-label">{{ t('connection.networkTab.localPort') }}</span>
            <input v-model="sshLocalPort" type="number" class="form-input" placeholder="自动分配" />
          </div>
          <div class="form-group f1">
            <span class="form-label">{{ t('connection.networkTab.keepAlive') }}</span>
            <input v-model="sshKeepAlive" type="number" class="form-input" placeholder="60" />
          </div>
        </div>
      </div>
    </div>

    <div class="collapse-panel">
      <div class="collapse-header" @click="sslExpanded = !sslExpanded">
        <ChevronRight :size="12" class="carrow" :class="{ open: sslExpanded }" />
        <span>{{ t('connection.networkTab.sslTls') }}</span>
        <div class="cswitch" @click.stop>
          <span class="switch-label">{{ sslEnabled ? t('connection.networkTab.enabled') : t('connection.networkTab.disabled') }}</span>
          <div class="switch-toggle" :class="{ on: sslEnabled }" @click="sslEnabled = !sslEnabled" />
        </div>
      </div>
      <div v-if="sslExpanded" class="collapse-body">
        <div class="form-row">
          <div class="form-group f1">
            <span class="form-label">{{ t('connection.networkTab.sslMode') }}</span>
            <select v-model="sslMode" class="form-select">
              <option value="disable">disable</option>
              <option value="require">require</option>
              <option value="verify-ca">verify-ca</option>
              <option value="verify-full">verify-full</option>
            </select>
          </div>
        </div>
        <div class="form-row">
          <div class="form-group f1">
            <span class="form-label">CA {{ t('connection.networkTab.cert') }}</span>
            <input v-model="sslCa" type="text" class="form-input" placeholder="ca.pem" />
          </div>
          <div class="form-group f1">
            <span class="form-label">{{ t('connection.networkTab.clientCert') }}</span>
            <input v-model="sslCert" type="text" class="form-input" placeholder="client-cert.pem" />
          </div>
        </div>
      </div>
    </div>

    <div class="collapse-panel">
      <div class="collapse-header" @click="proxyExpanded = !proxyExpanded">
        <ChevronRight :size="12" class="carrow" :class="{ open: proxyExpanded }" />
        <span>{{ t('connection.networkTab.proxy') }}</span>
        <div class="cswitch" @click.stop>
          <span class="switch-label">{{ proxyEnabled ? t('connection.networkTab.enabled') : t('connection.networkTab.disabled') }}</span>
          <div class="switch-toggle" :class="{ on: proxyEnabled }" @click="proxyEnabled = !proxyEnabled" />
        </div>
      </div>
      <div v-if="proxyExpanded" class="collapse-body">
        <div class="form-row">
          <div class="form-group f1">
            <span class="form-label">{{ t('connection.networkTab.proxyType') }}</span>
            <select v-model="proxyType" class="form-select">
              <option value="socks5">SOCKS5</option>
              <option value="http">HTTP</option>
              <option value="socks4">SOCKS4</option>
              <option value="https">HTTPS</option>
            </select>
          </div>
        </div>
        <div class="form-row">
          <div class="form-group f2">
            <span class="form-label">{{ t('connection.networkTab.proxyHost') }}</span>
            <input v-model="proxyHost" type="text" class="form-input" placeholder="proxy.example.com" />
          </div>
          <div class="form-group f1">
            <span class="form-label">{{ t('connection.networkTab.port') }}</span>
            <input v-model="proxyPort" type="number" class="form-input" />
          </div>
        </div>
        <div class="form-row">
          <div class="form-group f1">
            <span class="form-label">{{ t('connection.networkTab.proxyUsername') }}</span>
            <input v-model="proxyUsername" type="text" class="form-input" />
          </div>
          <div class="form-group f1">
            <span class="form-label">{{ t('connection.networkTab.proxyPassword') }}</span>
            <input v-model="proxyPassword" type="password" class="form-input" />
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ChevronRight } from 'lucide-vue-next'
import { ref } from 'vue'
import { useI18n } from 'vue-i18n'

const { t } = useI18n()

const sshExpanded = ref(false)
const sshEnabled = ref(false)
const sshHost = ref('')
const sshPort = ref(22)
const sshUsername = ref('')
const sshAuthType = ref('password')
const sshPassword = ref('')
const sshKeyPath = ref('')
const sshLocalPort = ref<number | null>(null)
const sshKeepAlive = ref(60)

const sslExpanded = ref(false)
const sslEnabled = ref(false)
const sslMode = ref('disable')
const sslCa = ref('')
const sslCert = ref('')

const proxyExpanded = ref(false)
const proxyEnabled = ref(false)
const proxyType = ref('socks5')
const proxyHost = ref('')
const proxyPort = ref(1080)
const proxyUsername = ref('')
const proxyPassword = ref('')
</script>

<style scoped>
.network-tab {
  padding: 0;
}
.collapse-panel {
  border: 1px solid var(--color-border, rgba(255,255,255,0.07));
  border-radius: 8px;
  margin-bottom: 10px;
  overflow: hidden;
}
.collapse-header {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 10px 14px;
  background: var(--color-bg-raised, #11111b);
  cursor: pointer;
  font-size: 13px;
  font-weight: 600;
  color: var(--color-text-secondary, #a6adc8);
  user-select: none;
  transition: background 0.15s;
}
.collapse-header:hover {
  background: #181825;
}
.carrow {
  color: var(--color-text-muted, #6c7086);
  transition: transform 0.2s;
  flex-shrink: 0;
}
.carrow.open {
  transform: rotate(90deg);
}
.cswitch {
  margin-left: auto;
  display: flex;
  align-items: center;
  gap: 6px;
}
.switch-label {
  font-size: 11px;
  color: var(--color-text-muted, #6c7086);
  font-weight: 400;
}
.switch-toggle {
  width: 34px;
  height: 18px;
  background: rgba(255,255,255,0.08);
  border-radius: 9px;
  position: relative;
  cursor: pointer;
  transition: background 0.2s;
}
.switch-toggle.on {
  background: var(--color-accent, #89b4fa);
}
.switch-toggle::after {
  content: '';
  width: 14px;
  height: 14px;
  border-radius: 50%;
  background: #fff;
  position: absolute;
  top: 2px;
  left: 2px;
  transition: left 0.2s;
}
.switch-toggle.on::after {
  left: 18px;
}
.collapse-body {
  padding: 14px;
  background: var(--color-bg-surface, #1a1b26);
}
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
  font-size: 12px;
  color: var(--color-text-secondary, #a6adc8);
  font-weight: 500;
}
.form-input, .form-select {
  width: 100%;
  height: 32px;
  padding: 0 10px;
  background: var(--color-bg-raised, #11111b);
  border: 1px solid rgba(255,255,255,0.05);
  border-radius: 5px;
  color: var(--color-text-primary, #cdd6f4);
  font-size: 13px;
  outline: none;
  transition: border-color 0.2s;
}
.form-input:focus, .form-select:focus {
  border-color: var(--color-accent, #89b4fa);
}
.form-select {
  cursor: pointer;
}
</style>