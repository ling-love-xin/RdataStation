<template>
  <NConfigProvider :theme="naiveTheme" :theme-overrides="themeOverrides">
    <NMessageProvider>
      <NDialogProvider>
        <router-view />
      </NDialogProvider>
    </NMessageProvider>
  </NConfigProvider>
</template>

<script setup lang="ts">
import { darkTheme, lightTheme, type GlobalThemeOverrides } from 'naive-ui'
import { NConfigProvider, NMessageProvider, NDialogProvider } from 'naive-ui'
import { computed, onMounted } from 'vue'

import { useUiStore } from '@/shared/stores/ui'

const uiStore = useUiStore()

// 根据当前主题选择 naive-ui 主题
const naiveTheme = computed(() => {
  return uiStore.isDark ? darkTheme : lightTheme
})

// 主题覆盖 - 统一使用企业级规范颜色
const themeOverrides = computed<GlobalThemeOverrides>(() => {
  const isDark = uiStore.isDark
  return {
    common: {
      // 主色 - 专业数据蓝 #165DFF
      primaryColor: '#165DFF',
      primaryColorHover: '#2B6DF5',
      primaryColorPressed: '#0E42D2',
      primaryColorSuppl: '#165DFF',
      // 功能色
      successColor: '#00B42A',
      warningColor: '#FF7D00',
      errorColor: '#F53F3F',
      infoColor: '#165DFF',
      // 文本色 - 3级梯度
      textColorBase: isDark ? '#cccccc' : '#333333',
      textColor1: isDark ? '#cccccc' : '#333333',
      textColor2: isDark ? '#858585' : '#666666',
      textColor3: isDark ? '#666666' : '#999999',
      // 背景色 - 3级分层
      bodyColor: isDark ? '#1e1e1e' : '#ffffff',
      cardColor: isDark ? '#252526' : '#f5f5f5',
      modalColor: isDark ? '#252526' : '#ffffff',
      popoverColor: isDark ? '#2d2d30' : '#ffffff',
      // 边框色
      borderColor: isDark ? '#3e3e42' : '#d9d9d9',
      dividerColor: isDark ? '#3e3e42' : '#d9d9d9',
    },
    Button: {
      textColor: isDark ? '#cccccc' : '#333333',
    },
    Input: {
      textColor: isDark ? '#cccccc' : '#333333',
      placeholderColor: isDark ? '#666666' : '#999999',
    },
    Tree: {
      nodeTextColor: isDark ? '#cccccc' : '#333333',
    },
    Tabs: {
      tabTextColor: isDark ? '#858585' : '#666666',
      tabTextColorActive: isDark ? '#cccccc' : '#333333',
    },
    DataTable: {
      thTextColor: isDark ? '#cccccc' : '#333333',
      tdTextColor: isDark ? '#cccccc' : '#333333',
    },
  }
})

onMounted(() => {
  uiStore.initTheme()
})
</script>

<style>
/* App-level styles */
#app {
  width: 100vw;
  height: 100vh;
  overflow: hidden;
}

/* 全局重置 */
* {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}

html {
  font-size: 14px;
}

body {
  font-family: var(--font-sans);
  background: var(--bg-primary);
  color: var(--text-primary);
  line-height: 1.5;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
}

/* 滚动条全局统一 */
::-webkit-scrollbar {
  width: 8px;
  height: 8px;
}

::-webkit-scrollbar-track {
  background: transparent;
}

::-webkit-scrollbar-thumb {
  background: var(--border-color);
  border-radius: 4px;
}

::-webkit-scrollbar-thumb:hover {
  background: var(--text-tertiary);
}

/* 选中文字样式 */
::selection {
  background: var(--primary-color);
  color: white;
}

/* 禁用文字选择（拖拽区域） */
.no-select {
  user-select: none;
  -webkit-user-select: none;
}

/* 隐藏滚动条但保持滚动功能 */
.hide-scrollbar {
  -ms-overflow-style: none;
  scrollbar-width: none;
}

.hide-scrollbar::-webkit-scrollbar {
  display: none;
}
</style>
