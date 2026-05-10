<template>
  <div class="appearance-settings">
    <!-- 外观 -->
    <div class="settings-section">
      <h3>
        <AppIcon name="Palette" :size="16" />
        {{ $t('settings.appearance') }}
      </h3>

      <div class="setting-item">
        <div class="setting-label">
          <span class="label-text">{{ $t('settings.theme') }}</span>
          <span v-if="hasProjectThemeOverride" class="label-hint"
            >({{ $t('settings.projectOverride') }})</span
          >
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
        <button v-if="hasProjectThemeOverride" class="reset-btn" @click="resetProjectTheme">
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

    <!-- 编辑器 -->
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

      <div class="setting-item">
        <div class="setting-label">
          <span class="label-text">{{ $t('settings.fontFamily') }}</span>
          <span class="label-hint">{{ $t('settings.fontFamilyHint') }}</span>
        </div>
        <input
          v-model="localEditorSettings.fontFamily"
          type="text"
          class="text-input"
          spellcheck="false"
        />
      </div>
    </div>

    <!-- 默认引擎 -->
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
  </div>
</template>

<script setup lang="ts">
import { computed, reactive, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'

import AppIcon from '@/shared/components/common/AppIcon.vue'
import { CONFIG_KEYS, DEFAULT_EDITOR_SETTINGS, DEFAULT_GLOBAL_CONFIG } from '@/stores/config'
import type { DefaultEngine, EditorSettings, Language, Theme } from '@/stores/config'
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

watch(
  () => appStore.effectiveTheme,
  val => {
    localTheme.value = val
  }
)
watch(
  () => appStore.effectiveLanguage,
  val => {
    localLanguage.value = val
  }
)
watch(
  () => appStore.effectiveEditorSettings,
  val => {
    Object.assign(localEditorSettings, val)
  },
  { deep: true }
)
watch(
  () => appStore.effectiveDefaultEngine,
  val => {
    localDefaultEngine.value = val
  }
)

function resetToDefault() {
  localTheme.value = DEFAULT_GLOBAL_CONFIG.theme
  localLanguage.value = DEFAULT_GLOBAL_CONFIG.language
  Object.assign(localEditorSettings, DEFAULT_EDITOR_SETTINGS)
  localDefaultEngine.value = DEFAULT_GLOBAL_CONFIG.defaultEngine
}

function resetToFactory() {
  localTheme.value = appStore.effectiveTheme
  localLanguage.value = appStore.effectiveLanguage
  Object.assign(localEditorSettings, appStore.effectiveEditorSettings)
  localDefaultEngine.value = appStore.effectiveDefaultEngine
}

defineExpose({
  localTheme,
  localLanguage,
  localEditorSettings,
  localDefaultEngine,
  resetToDefault,
  resetToFactory,
})
</script>

<style scoped>
.appearance-settings {
  display: flex;
  flex-direction: column;
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
  color: var(--color-bg-primary);
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

.text-input {
  width: 100%;
  padding: 8px 12px;
  border: 1px solid var(--color-border);
  border-radius: var(--border-radius-sm);
  background: var(--color-bg-primary);
  color: var(--color-text-primary);
  font-size: var(--font-size-md);
  box-sizing: border-box;
}

.text-input:focus {
  outline: none;
  border-color: var(--brand-accent);
}
</style>
