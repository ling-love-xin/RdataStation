<template>
  <div v-if="visible" class="config-manager-overlay" @click.self="close">
    <div class="config-manager-dialog">
      <!-- Header -->
      <div class="cm-header">
        <h3 class="cm-title">
          <Boxes :size="16" />
          {{ t('networkTab.profileManager.title') }}
        </h3>
        <button class="cm-close-btn" :title="String(t('common.close'))" @click="close">
          <X :size="16" />
        </button>
      </div>

      <!-- Tabs -->
      <div class="cm-tabs">
        <button
          class="cm-tab"
          :class="{ active: activeTab === 'ssh' }"
          @click="activeTab = 'ssh'"
        >
          {{ t('networkTab.profileManager.ssh') }}
          <span class="cm-badge">{{ sshProfiles.length }}</span>
        </button>
        <button
          class="cm-tab"
          :class="{ active: activeTab === 'ssl' }"
          @click="activeTab = 'ssl'"
        >
          {{ t('networkTab.profileManager.ssl') }}
          <span class="cm-badge">{{ sslProfiles.length }}</span>
        </button>
        <button
          class="cm-tab"
          :class="{ active: activeTab === 'proxy' }"
          @click="activeTab = 'proxy'"
        >
          {{ t('networkTab.profileManager.proxy') }}
          <span class="cm-badge">{{ proxyProfiles.length }}</span>
        </button>
      </div>

      <!-- Content -->
      <div class="cm-content">
        <!-- SSH Profiles -->
        <template v-if="activeTab === 'ssh'">
          <div v-if="sshProfiles.length === 0" class="cm-empty">
            {{ t('networkTab.profileManager.noSsh') }}
          </div>
          <div
            v-for="p in sshProfiles"
            :key="p.id"
            class="cm-card"
          >
            <div class="cm-card-info">
              <span class="cm-card-name">{{ p.name }}</span>
              <span class="cm-card-detail">
                {{ p.username }}@{{ p.host }}:{{ p.port }}
                <template v-if="p.remoteHost">→ {{ p.remoteHost }}</template>
              </span>
              <span class="cm-card-scope">{{ scopeLabel(p.scope) }}</span>
            </div>
            <div class="cm-card-actions">
              <button
                class="cm-action-btn"
                :title="String(t('common.delete'))"
                @click="onDeleteSsh(p.id)"
              >
                <Trash2 :size="14" />
              </button>
            </div>
          </div>
        </template>

        <!-- SSL Profiles -->
        <template v-if="activeTab === 'ssl'">
          <div v-if="sslProfiles.length === 0" class="cm-empty">
            {{ t('networkTab.profileManager.noSsl') }}
          </div>
          <div
            v-for="p in sslProfiles"
            :key="p.id"
            class="cm-card"
          >
            <div class="cm-card-info">
              <span class="cm-card-name">{{ p.name }}</span>
              <span class="cm-card-detail">
                {{ t('networkTab.profileManager.mode') }}: {{ p.mode }}
                <template v-if="p.ca">CA: {{ basename(p.ca) }}</template>
              </span>
              <span class="cm-card-scope">{{ scopeLabel(p.scope) }}</span>
            </div>
            <div class="cm-card-actions">
              <button
                class="cm-action-btn"
                :title="String(t('common.delete'))"
                @click="onDeleteSsl(p.id)"
              >
                <Trash2 :size="14" />
              </button>
            </div>
          </div>
        </template>

        <!-- Proxy Profiles -->
        <template v-if="activeTab === 'proxy'">
          <div v-if="proxyProfiles.length === 0" class="cm-empty">
            {{ t('networkTab.profileManager.noProxy') }}
          </div>
          <div
            v-for="p in proxyProfiles"
            :key="p.id"
            class="cm-card"
          >
            <div class="cm-card-info">
              <span class="cm-card-name">{{ p.name }}</span>
              <span class="cm-card-detail">
                {{ p.type.toUpperCase() }} {{ p.host }}:{{ p.port }}
                <template v-if="p.username">{{ p.username }}</template>
              </span>
              <span class="cm-card-scope">{{ scopeLabel(p.scope) }}</span>
            </div>
            <div class="cm-card-actions">
              <button
                class="cm-action-btn"
                :title="String(t('common.delete'))"
                @click="onDeleteProxy(p.id)"
              >
                <Trash2 :size="14" />
              </button>
            </div>
          </div>
        </template>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { Boxes, X, Trash2 } from 'lucide-vue-next'
import { ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'

import type { SshProfile, SslProfile, ProxyProfile, ProfileScope } from '../../types/network-chain'

const { t } = useI18n()

// ===== Props =====

interface Props {
  visible: boolean
  defaultTab?: 'ssh' | 'ssl' | 'proxy'
  sshProfiles: SshProfile[]
  sslProfiles: SslProfile[]
  proxyProfiles: ProxyProfile[]
}

const props = withDefaults(defineProps<Props>(), {
  defaultTab: 'ssh',
})

// ===== Emits =====

const emit = defineEmits<{
  (e: 'close'): void
  (e: 'delete-ssh', id: string): void
  (e: 'delete-ssl', id: string): void
  (e: 'delete-proxy', id: string): void
}>()

// ===== State =====

const activeTab = ref<'ssh' | 'ssl' | 'proxy'>('ssh')

// 打开对话框时同步默认标签页
watch(
  () => props.visible,
  (v) => {
    if (v) activeTab.value = props.defaultTab ?? 'ssh'
  }
)

// ===== Methods =====

function close() {
  emit('close')
}

function onDeleteSsh(id: string) {
  emit('delete-ssh', id)
}

function onDeleteSsl(id: string) {
  emit('delete-ssl', id)
}

function onDeleteProxy(id: string) {
  emit('delete-proxy', id)
}

function scopeLabel(scope: ProfileScope): string {
  return scope === 'global' ? '全局' : '项目'
}

function basename(path: string): string {
  return path.replace(/\\/g, '/').split('/').pop() || path
}
</script>

<style scoped>
.config-manager-overlay {
  position: fixed;
  inset: 0;
  z-index: 1001;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(0, 0, 0, 0.45);
}

.config-manager-dialog {
  width: 560px;
  max-height: 480px;
  background: var(--color-bg-primary, #1e1e2e);
  border: 1px solid var(--color-border, #313244);
  border-radius: 8px;
  display: flex;
  flex-direction: column;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.4);
}

.cm-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 14px 18px;
  border-bottom: 1px solid var(--color-border, #313244);
}

.cm-title {
  display: flex;
  align-items: center;
  gap: 8px;
  margin: 0;
  font-size: 14px;
  font-weight: 600;
  color: var(--color-text-primary, #cdd6f4);
}

.cm-close-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  border: none;
  border-radius: 4px;
  background: transparent;
  color: var(--color-text-muted, #6c7086);
  cursor: pointer;
}

.cm-close-btn:hover {
  background: rgba(255, 255, 255, 0.08);
  color: var(--color-text-primary, #cdd6f4);
}

.cm-tabs {
  display: flex;
  gap: 0;
  border-bottom: 1px solid var(--color-border, #313244);
  padding: 0 18px;
}

.cm-tab {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 10px 16px;
  border: none;
  border-bottom: 2px solid transparent;
  background: transparent;
  color: var(--color-text-muted, #6c7086);
  font-size: 13px;
  cursor: pointer;
  transition: color 0.15s, border-color 0.15s;
}

.cm-tab:hover {
  color: var(--color-text-secondary, #a6adc8);
}

.cm-tab.active {
  color: var(--brand-accent, #e17055);
  border-bottom-color: var(--brand-accent, #e17055);
}

.cm-badge {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-width: 18px;
  height: 18px;
  padding: 0 4px;
  border-radius: 9px;
  background: rgba(255, 255, 255, 0.08);
  font-size: 11px;
  color: var(--color-text-muted, #6c7086);
}

.cm-tab.active .cm-badge {
  background: rgba(225, 112, 85, 0.15);
  color: var(--brand-accent, #e17055);
}

.cm-content {
  flex: 1;
  overflow-y: auto;
  padding: 12px 18px;
}

.cm-empty {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 100px;
  color: var(--color-text-muted, #6c7086);
  font-size: 13px;
}

.cm-card {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 10px 12px;
  margin-bottom: 6px;
  border-radius: 6px;
  background: var(--color-bg-secondary, rgba(255, 255, 255, 0.04));
  border: 1px solid transparent;
  transition: border-color 0.15s;
}

.cm-card:hover {
  border-color: var(--color-border, #313244);
}

.cm-card-info {
  display: flex;
  flex-direction: column;
  gap: 2px;
  min-width: 0;
}

.cm-card-name {
  font-size: 13px;
  font-weight: 500;
  color: var(--color-text-primary, #cdd6f4);
}

.cm-card-detail {
  font-size: 11px;
  color: var(--color-text-muted, #6c7086);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.cm-card-scope {
  font-size: 10px;
  color: var(--color-text-muted, #6c7086);
  margin-top: 1px;
}

.cm-card-actions {
  display: flex;
  gap: 4px;
  flex-shrink: 0;
}

.cm-action-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  border: none;
  border-radius: 4px;
  background: transparent;
  color: var(--color-text-muted, #6c7086);
  cursor: pointer;
}

.cm-action-btn:hover {
  background: rgba(255, 255, 255, 0.08);
  color: var(--color-text-primary, #cdd6f4);
}

.cm-action-btn:hover :deep(svg) {
  color: #f38ba8;
}
</style>