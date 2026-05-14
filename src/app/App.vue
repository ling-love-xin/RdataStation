<template>
  <NConfigProvider
    :theme="naiveTheme"
    :theme-overrides="themeOverrides"
    :locale="naiveLocale"
    :date-locale="naiveDateLocale"
  >
    <NMessageProvider>
      <NDialogProvider>
        <NAlert
          v-if="appStore.initError"
          type="error"
          :title="$t('config.initError.title', '配置初始化失败')"
          :bordered="false"
          closable
          class="init-error-banner"
          @close="appStore.clearInitError()"
        >
          <template #default>
            {{ appStore.initError }}
          </template>
        </NAlert>
        <router-view />
      </NDialogProvider>
    </NMessageProvider>
  </NConfigProvider>
</template>

<script setup lang="ts">
import {
  darkTheme,
  lightTheme,
  zhCN,
  dateZhCN,
  enUS,
  dateEnUS,
  NAlert,
  NConfigProvider,
  NMessageProvider,
  NDialogProvider,
} from 'naive-ui'
import { computed, watch, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'

import { useAppStore } from '@/stores/useAppStore'

const appStore = useAppStore()
const { locale: i18nLocale } = useI18n()

const naiveTheme = computed(() => {
  return appStore.isDark ? darkTheme : lightTheme
})

const naiveLocale = computed(() => {
  return appStore.effectiveLanguage === 'zh-CN' ? zhCN : enUS
})

const naiveDateLocale = computed(() => {
  return appStore.effectiveLanguage === 'zh-CN' ? dateZhCN : dateEnUS
})

const themeOverrides = computed(() => {
  const dark = appStore.isDark
  const rootStyles = getComputedStyle(document.documentElement)
  const bodyStyles = getComputedStyle(document.body)

  const accent = rootStyles.getPropertyValue('--brand-accent').trim() || '#E17055'
  const accentHover = rootStyles.getPropertyValue('--brand-accent-hover').trim() || '#D35400'
  const success = rootStyles.getPropertyValue('--brand-success').trim() || '#00B894'
  const warning = rootStyles.getPropertyValue('--brand-warning').trim() || '#FDCB6E'
  const danger = rootStyles.getPropertyValue('--brand-danger').trim() || '#D63031'

  const fall = (v: string, dk: string, lt: string) => v || (dark ? dk : lt)

  const bgPrimary = fall(bodyStyles.getPropertyValue('--color-bg-primary').trim(), '#1e1f22', '#ffffff')
  const bgSecondary = fall(bodyStyles.getPropertyValue('--color-bg-secondary').trim(), '#2b2d30', '#f5f5f5')
  const bgElevated = fall(bodyStyles.getPropertyValue('--color-bg-elevated').trim(), '#3d4446', '#ffffff')
  const bgTertiary = fall(bodyStyles.getPropertyValue('--color-bg-tertiary').trim(), '#2d3436', '#e5e7eb')
  const textPrimary = fall(bodyStyles.getPropertyValue('--color-text-primary').trim(), '#e5e7eb', '#1f2937')
  const textSecondary = fall(bodyStyles.getPropertyValue('--color-text-secondary').trim(), '#9ca3af', '#4b5563')
  const textMuted = fall(bodyStyles.getPropertyValue('--color-text-muted').trim(), '#6b7280', '#9ca3af')
  const borderSubtle = fall(bodyStyles.getPropertyValue('--color-border-subtle').trim(), '#3c3f41', '#e5e7eb')
  const hoverBg = fall(bodyStyles.getPropertyValue('--color-hover').trim(), '#454545', '#e5e7e9')

  return {
    common: {
      primaryColor: accent,
      primaryColorHover: accentHover,
      primaryColorPressed: accentHover,
      primaryColorSuppl: accent,
      successColor: success,
      warningColor: warning,
      errorColor: danger,
      infoColor: accent,
      bodyColor: bgPrimary,
      cardColor: bgElevated,
      modalColor: bgElevated,
      popoverColor: bgElevated,
      tableColor: bgSecondary,
      dividerColor: borderSubtle,
      borderColor: borderSubtle,
      hoverColor: hoverBg,
    },
    Card: {
      color: bgElevated,
      borderColor: borderSubtle,
      titleTextColor: textPrimary,
      closeColor: textMuted,
      closeColorHover: textPrimary,
      titleFontWeight: '600',
    },
    Modal: {
      color: bgElevated,
      textColor: textPrimary,
    },
    Drawer: {
      color: bgPrimary,
      textColor: textPrimary,
      headerBorderColor: borderSubtle,
      footerBorderColor: borderSubtle,
    },
  }
})

watch(
  () => appStore.effectiveLanguage,
  newLang => {
    i18nLocale.value = newLang
  },
  { immediate: true }
)

onMounted(() => {
  appStore.applyTheme()
})
</script>

<style>
#app {
  width: 100vw;
  height: 100vh;
  overflow: hidden;
}

.init-error-banner {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  z-index: 10000;
}
</style>
