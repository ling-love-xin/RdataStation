<template>
  <NConfigProvider
    :theme="naiveTheme"
    :theme-overrides="themeOverrides"
    :locale="naiveLocale"
    :date-locale="naiveDateLocale"
  >
    <NMessageProvider>
      <NDialogProvider>
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
  NConfigProvider,
  NMessageProvider,
  NDialogProvider,
} from 'naive-ui'
import { computed, watch, onMounted, onUnmounted } from 'vue'
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
  const accent = '#E17055'
  const accentHover = '#D35400'
  return {
    common: {
      primaryColor: accent,
      primaryColorHover: accentHover,
      primaryColorPressed: accentHover,
      primaryColorSuppl: accent,
      successColor: '#00B894',
      warningColor: '#FDCB6E',
      errorColor: '#D63031',
      infoColor: accent,
    },
  }
})

watch(
  () => appStore.effectiveLanguage,
  (newLang) => {
    i18nLocale.value = newLang
  },
  { immediate: true }
)

let systemThemeListener: (() => void) | null = null

function setupSystemThemeListener() {
  const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)')
  const handler = () => {
    if (appStore.effectiveTheme === 'system') {
      appStore.applyTheme()
    }
  }
  mediaQuery.addEventListener('change', handler)
  systemThemeListener = () => mediaQuery.removeEventListener('change', handler)
}

onMounted(() => {
  setupSystemThemeListener()
  appStore.applyTheme()
})

onUnmounted(() => {
  systemThemeListener?.()
})
</script>

<style>
#app {
  width: 100vw;
  height: 100vh;
  overflow: hidden;
}
</style>
