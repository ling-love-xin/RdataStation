<template>
  <div class="settings-panel">
    <div class="settings-header">
      <AppIcon name="Settings" :size="20" accent class="settings-icon" />
      <h2>{{ $t('settings.title') }}</h2>
    </div>

    <div class="settings-content">
      <div class="settings-section">
        <h3>
          <AppIcon name="Palette" :size="16" />
          {{ $t('settings.appearance') }}
        </h3>

        <div class="setting-item">
          <div class="setting-label">
            <span class="label-text">{{ $t('settings.theme') }}</span>
            <span v-if="hasProjectThemeOverride" class="label-hint">({{ $t('settings.projectOverride') }})</span>
          </div>
          <div class="theme-selector">
            <button
              v-for="opt in themeOptions"
              :key="opt.value"
              :class="['theme-btn', { active: localTheme === opt.value }]"
              @click="localTheme = opt.value"
            >
              <AppIcon :name="opt.icon" :size="16" />
              {{ opt.label }}
            </button>
          </div>
          <button
            v-if="hasProjectThemeOverride"
            class="reset-btn"
            @click="resetProjectTheme"
          >
            <AppIcon name="RotateCcw" :size="14" />
            {{ $t('settings.resetToGlobal') }}
          </button>
        </div>

        <div class="setting-item">
          <div class="setting-label">
            <span class="label-text">{{ $t('settings.language') }}</span>
            <span class="label-hint">{{ $t('settings.restartHint') }}</span>
          </div>
          <div class="theme-selector">
            <button
              v-for="opt in languageOptions"
              :key="opt.value"
              :class="['theme-btn', { active: localLanguage === opt.value }]"
              @click="localLanguage = opt.value"
            >
              {{ opt.label }}
            </button>
          </div>
        </div>
      </div>

      <div class="settings-section">
        <h3>
          <AppIcon name="FileCode" :size="16" />
          {{ $t('settings.editor') }}
        </h3>

        <div class="setting-item">
          <div class="setting-label">
            <span class="label-text">{{ $t('settings.fontSize') }}</span>
            <span class="label-value">{{ localEditorSettings.fontSize }}px</span>
          </div>
          <input
            v-model.number="localEditorSettings.fontSize"
            type="range"
            min="10"
            max="24"
            step="1"
            class="slider"
          />
          <div class="slider-labels">
            <span>10px</span>
            <span>24px</span>
          </div>
        </div>

        <div class="setting-item">
          <div class="setting-label">
            <span class="label-text">{{ $t('settings.tabSize') }}</span>
            <span class="label-value">{{ localEditorSettings.tabSize }}</span>
          </div>
          <div class="theme-selector">
            <button
              v-for="size in [2, 4, 8]"
              :key="size"
              :class="['theme-btn', { active: localEditorSettings.tabSize === size }]"
              @click="localEditorSettings.tabSize = size"
            >
              {{ size }}
            </button>
          </div>
        </div>

        <div class="setting-item">
          <div class="setting-label">
            <span class="label-text">{{ $t('settings.wordWrap') }}</span>
          </div>
          <label class="switch">
            <input v-model="localEditorSettings.wordWrap" type="checkbox" />
            <span class="slider-switch"></span>
          </label>
        </div>

        <div class="setting-item">
          <div class="setting-label">
            <span class="label-text">{{ $t('settings.lineNumbers') }}</span>
          </div>
          <label class="switch">
            <input v-model="localEditorSettings.lineNumbers" type="checkbox" />
            <span class="slider-switch"></span>
          </label>
        </div>

        <div class="setting-item">
          <div class="setting-label">
            <span class="label-text">{{ $t('settings.minimap') }}</span>
          </div>
          <label class="switch">
            <input v-model="localEditorSettings.minimap" type="checkbox" />
            <span class="slider-switch"></span>
          </label>
        </div>
      </div>

      <div class="settings-section">
        <h3>
          <AppIcon name="Database" :size="16" />
          {{ $t('settings.defaultEngine') }}
        </h3>

        <div class="setting-item">
          <div class="setting-label">
            <span class="label-text">{{ $t('settings.engine') }}</span>
            <span class="label-hint">{{ $t('settings.engineHint') }}</span>
          </div>
          <div class="theme-selector">
            <button
              v-for="opt in engineOptions"
              :key="opt.value"
              :class="['theme-btn', { active: localDefaultEngine === opt.value }]"
              @click="localDefaultEngine = opt.value"
            >
              {{ opt.label }}
            </button>
          </div>
        </div>
      </div>

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
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, reactive } from 'vue'
import { useI18n } from 'vue-i18n'

import AppIcon from '@/shared/components/common/AppIcon.vue'
import { CONFIG_KEYS } from '@/stores/config'
import type { Theme, Language, EditorSettings, DefaultEngine } from '@/stores/config'
import { useAppStore } from '@/stores/useAppStore'

const appStore = useAppStore()
const { t } = useI18n()

const themeOptions = [
  { value: 'dark' as Theme, label: t('settings.dark'), icon: 'Moon' as const },
  { value: 'light' as Theme, label: t('settings.light'), icon: 'Sun' as const },
  { value: 'system' as Theme, label: t('settings.system'), icon: 'Monitor' as const },
]

const languageOptions = [
  { value: 'zh-CN' as Language, label: t('settings.simplifiedChinese') },
  { value: 'en' as Language, label: t('settings.english') },
]

const engineOptions = [
  { value: 'native' as DefaultEngine, label: t('settings.nativeEngine') },
  { value: 'duckdb' as DefaultEngine, label: t('settings.duckdbEngine') },
]

const localTheme = ref<Theme>(appStore.effectiveTheme)
const localLanguage = ref<Language>(appStore.effectiveLanguage)
const localEditorSettings = reactive<EditorSettings>({ ...appStore.effectiveEditorSettings })
const localDefaultEngine = ref<DefaultEngine>(appStore.effectiveDefaultEngine)

const hasProjectThemeOverride = computed(() => appStore.hasProjectOverride(CONFIG_KEYS.THEME))

function resetProjectTheme() {
  appStore.resetProjectOverride(CONFIG_KEYS.THEME)
  localTheme.value = appStore.effectiveTheme
}

async function applyAllSettings() {
  await appStore.setTheme(localTheme.value)
  await appStore.setLanguage(localLanguage.value)
  await appStore.setEditorSettings({ ...localEditorSettings })
  await appStore.setDefaultEngine(localDefaultEngine.value)
  appStore.applyTheme()
  console.log('[SettingsPanel] All settings applied')
}

function resetToDefault() {
  localTheme.value = 'dark'
  localLanguage.value = 'zh-CN'
  Object.assign(localEditorSettings, {
    fontSize: 14,
    tabSize: 2,
    wordWrap: true,
    minimap: true,
    lineNumbers: true,
    fontFamily: "'Cascadia Code', 'Fira Code', 'Consolas', monospace",
  })
  localDefaultEngine.value = 'native'
  applyAllSettings()
  console.log('[SettingsPanel] Settings reset to default')
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

.setting-item {
  margin-bottom: 18px;
}

.setting-label {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: var(--spacing-sm);
}

.label-text {
  font-size: var(--font-size-md);
  color: var(--color-text-primary);
}

.label-hint {
  font-size: var(--font-size-sm);
  color: var(--color-text-muted);
  margin-left: var(--spacing-sm);
}

.label-value {
  font-size: var(--font-size-sm);
  color: var(--brand-accent);
  font-weight: 500;
}

.theme-selector {
  display: flex;
  gap: var(--spacing-sm);
  flex-wrap: wrap;
}

.theme-btn {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: var(--spacing-sm) 14px;
  border: 1px solid var(--color-border);
  border-radius: var(--border-radius-md);
  background: var(--color-bg-secondary);
  color: var(--color-text-primary);
  font-size: var(--font-size-sm);
  cursor: pointer;
  transition: all 0.2s ease;
}

.theme-btn:hover {
  background: var(--color-hover);
}

.theme-btn.active {
  background: var(--brand-accent);
  border-color: var(--brand-accent);
  color: #FFFFFF;
}

.reset-btn {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  margin-top: var(--spacing-sm);
  padding: 4px 10px;
  border: 1px dashed var(--color-border);
  border-radius: var(--border-radius-sm);
  background: transparent;
  color: var(--color-text-muted);
  font-size: var(--font-size-sm);
  cursor: pointer;
  transition: all 0.2s ease;
}

.reset-btn:hover {
  border-color: var(--brand-accent);
  color: var(--brand-accent);
}

.slider {
  width: 100%;
  height: 4px;
  border-radius: 2px;
  background: var(--color-bg-secondary);
  outline: none;
  -webkit-appearance: none;
}

.slider::-webkit-slider-thumb {
  -webkit-appearance: none;
  width: 14px;
  height: 14px;
  border-radius: 50%;
  background: var(--brand-accent);
  cursor: pointer;
}

.slider-labels {
  display: flex;
  justify-content: space-between;
  margin-top: 4px;
  font-size: var(--font-size-sm);
  color: var(--color-text-muted);
}

.switch {
  position: relative;
  display: inline-block;
  width: 36px;
  height: 20px;
}

.switch input {
  opacity: 0;
  width: 0;
  height: 0;
}

.slider-switch {
  position: absolute;
  cursor: pointer;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background-color: var(--color-bg-secondary);
  transition: 0.3s;
  border-radius: 20px;
}

.slider-switch:before {
  position: absolute;
  content: '';
  height: 14px;
  width: 14px;
  left: 3px;
  bottom: 3px;
  background-color: var(--color-bg-primary);
  transition: 0.3s;
  border-radius: 50%;
}

.switch input:checked + .slider-switch {
  background-color: var(--brand-accent);
}

.switch input:checked + .slider-switch:before {
  transform: translateX(16px);
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
  color: #FFFFFF;
}

.action-btn.primary:hover {
  background: var(--brand-accent-hover);
}
</style>
