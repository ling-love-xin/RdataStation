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
  const rootStyles = getComputedStyle(document.documentElement)
  const accent = rootStyles.getPropertyValue('--brand-accent').trim() || '#E17055'
  const accentHover = rootStyles.getPropertyValue('--brand-accent-hover').trim() || '#D35400'
  const success = rootStyles.getPropertyValue('--brand-success').trim() || '#00B894'
  const warning = rootStyles.getPropertyValue('--brand-warning').trim() || '#FDCB6E'
  const danger = rootStyles.getPropertyValue('--brand-danger').trim() || '#D63031'
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
