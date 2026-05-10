<template>
  <div class="settings-panel">
    <div class="settings-header">
      <AppIcon name="Settings" :size="20" accent class="settings-icon" />
      <h2>{{ $t('settings.title') }}</h2>
    </div>

    <div class="settings-tabs">
      <button
        v-for="tab in tabs"
        :key="tab.id"
        :class="['tab-btn', { active: activeTab === tab.id }]"
        @click="activeTab = tab.id"
      >
        <AppIcon :name="tab.icon" :size="14" />
        {{ tab.label }}
      </button>
    </div>

    <div class="settings-content">
      <AppearanceSettings v-if="activeTab === 'appearance'" ref="appearanceRef" />
      <InterfaceSettings v-if="activeTab === 'interface'" ref="interfaceRef" />

      <!-- 操作按钮 -->
      <div class="settings-section">
        <h3>
          <AppIcon name="RefreshCw" :size="16" />
          {{ $t('settings.actions') }}
        </h3>

        <div class="action-buttons">
          <button class="action-btn primary" @click="applyAllSettings">
            <AppIcon name="Check" :size="16" />
            {{ $t('settings.applyAll') }}
          </button>
          <button class="action-btn" @click="resetToDefault">
            <AppIcon name="RotateCcw" :size="16" />
            {{ $t('settings.resetDefault') }}
          </button>
          <button class="action-btn danger" @click="resetToFactory">
            <AppIcon name="Trash2" :size="16" />
            {{ $t('settings.resetFactory') }}
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { useMessage } from 'naive-ui'
import { onMounted, ref } from 'vue'
import { useI18n } from 'vue-i18n'

import AppIcon from '@/shared/components/common/AppIcon.vue'
import { CONFIG_KEYS } from '@/stores/config'
import { useAppStore } from '@/stores/useAppStore'

import AppearanceSettings from './AppearanceSettings.vue'
import InterfaceSettings from './InterfaceSettings.vue'

const appStore = useAppStore()
const { t } = useI18n()
const message = useMessage()

const activeTab = ref('appearance')
const appearanceRef = ref<InstanceType<typeof AppearanceSettings> | null>(null)
const interfaceRef = ref<InstanceType<typeof InterfaceSettings> | null>(null)

const tabs = [
  { id: 'appearance', label: t('settings.appearanceTab'), icon: 'Palette' as const },
  { id: 'interface', label: t('settings.interfaceTab'), icon: 'LayoutTemplate' as const },
]

async function applyAllSettings() {
  const batch: Array<{ key: string; value: unknown; scope: 'global' | 'project' }> = []

  if (appearanceRef.value) {
    batch.push(
      { key: CONFIG_KEYS.THEME, value: appearanceRef.value.localTheme, scope: 'global' as const },
      { key: CONFIG_KEYS.LANGUAGE, value: appearanceRef.value.localLanguage, scope: 'global' as const },
      {
        key: CONFIG_KEYS.EDITOR_SETTINGS,
        value: { ...appearanceRef.value.localEditorSettings },
        scope: 'global' as const,
      },
      { key: CONFIG_KEYS.DEFAULT_ENGINE, value: appearanceRef.value.localDefaultEngine, scope: 'global' as const }
    )
  }

  if (interfaceRef.value) {
    batch.push(
      { key: CONFIG_KEYS.TITLE_BAR_SETTINGS, value: { ...interfaceRef.value.localTitleBarSettings }, scope: 'global' as const },
      { key: CONFIG_KEYS.STATUS_BAR_SETTINGS, value: { ...interfaceRef.value.localStatusBarSettings }, scope: 'global' as const },
      { key: CONFIG_KEYS.COMMAND_PALETTE_SETTINGS, value: { ...interfaceRef.value.localCommandPaletteSettings }, scope: 'global' as const }
    )
  }

  const results = await appStore.saveBatch(batch)

  const failures = results.filter(r => !r.success)
  if (failures.length > 0) {
    console.error('[SettingsPanel] Failed to apply settings:', failures)
    message.error(t('settings.saveFailed'))
  } else {
    message.success(t('settings.saveSuccess'))
  }

  appStore.applyTheme()
}

function resetToDefault() {
  appearanceRef.value?.resetToDefault()
  interfaceRef.value?.resetToDefault()
  applyAllSettings()
}

async function resetToFactory() {
  const results = await appStore.resetToFactory()
  const failures = results.filter(r => !r.success)
  if (failures.length === 0) {
    appearanceRef.value?.resetToFactory()
    interfaceRef.value?.resetToFactory()
    console.log('[SettingsPanel] Settings reset to factory')
  } else {
    console.error('[SettingsPanel] resetToFactory failed:', failures)
  }
}

onMounted(() => {
  console.log('[SettingsPanel] mounted')
})
</script>

<style scoped>
.settings-panel {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: var(--color-bg-primary);
  color: var(--color-text-primary);
}

.settings-header {
  display: flex;
  align-items: center;
  gap: var(--spacing-md);
  padding: var(--spacing-lg) 20px;
  border-bottom: 1px solid var(--color-border);
  background: var(--color-bg-secondary);
}

.settings-header h2 {
  margin: 0;
  font-size: var(--font-size-lg);
  font-weight: 600;
}

.settings-icon {
  color: var(--brand-accent);
}

.settings-tabs {
  display: flex;
  gap: var(--spacing-xs);
  padding: var(--spacing-sm) 20px;
  border-bottom: 1px solid var(--color-border);
  background: var(--color-bg-secondary);
}

.tab-btn {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 6px 12px;
  border: 1px solid transparent;
  border-radius: var(--border-radius-sm);
  background: transparent;
  color: var(--color-text-secondary);
  font-size: var(--font-size-sm);
  cursor: pointer;
  transition: all 0.2s ease;
}

.tab-btn:hover {
  background: var(--color-hover);
  color: var(--color-text-primary);
}

.tab-btn.active {
  background: var(--brand-accent);
  border-color: var(--brand-accent);
  color: var(--color-bg-primary);
}

.settings-content {
  flex: 1;
  overflow-y: auto;
  padding: var(--spacing-lg) 20px;
}

.settings-section {
  margin-bottom: var(--spacing-lg);
  padding-bottom: var(--spacing-lg);
  border-bottom: 1px solid var(--color-border);
}

.settings-section:last-child {
  border-bottom: none;
}

.settings-section h3 {
  display: flex;
  align-items: center;
  gap: var(--spacing-sm);
  margin: 0 0 var(--spacing-md) 0;
  font-size: var(--font-size-md);
  font-weight: 600;
  color: var(--color-text-secondary);
}

.action-buttons {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-sm);
}

.action-btn {
  display: flex;
  align-items: center;
  gap: var(--spacing-sm);
  padding: 10px 16px;
  border: 1px solid var(--color-border);
  border-radius: var(--border-radius-md);
  background: var(--color-bg-secondary);
  color: var(--color-text-primary);
  font-size: var(--font-size-md);
  cursor: pointer;
  transition: all 0.2s ease;
}

.action-btn:hover {
  background: var(--color-hover);
}

.action-btn.primary {
  background: var(--brand-accent);
  border-color: var(--brand-accent);
  color: var(--color-bg-primary);
}

.action-btn.primary:hover {
  background: var(--brand-accent-hover);
}

.action-btn.danger {
  border-color: var(--brand-danger);
  color: var(--brand-danger);
}

.action-btn.danger:hover {
  background: var(--brand-danger);
  color: var(--color-bg-primary);
}
</style>
