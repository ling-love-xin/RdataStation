import { createPinia } from 'pinia'
import { createApp } from 'vue'

import { builtinExtensions } from '@/core/builtin-extensions'
import { extensionHost } from '@/core/extension-host'

import App from './App.vue'
import router from './router'

// Import styles
import '@/shared/styles/theme.css'
import 'dockview-vue/dist/styles/dockview.css'
// dockview 主题覆盖必须在 dockview 默认样式之后导入
import '@/shared/styles/dockview-theme.css'

// Configure Monaco Editor for Tauri environment
// 完全禁用 Web Worker，在主线程中运行所有功能
// Tauri 的安全策略和 CSP 限制使得 Web Worker 加载不稳定
;(self as any).MonacoEnvironment = {
  getWorker: function () {
    // 返回一个模拟 worker 对象，包含所有 Monaco 内部可能调用的方法
    return {
      postMessage: function () {
        // 空实现，忽略所有消息
      },
      terminate: function () {
        // 空实现
      },
      addEventListener: function () {
        // 空实现
      },
      removeEventListener: function () {
        // 空实现，避免 "w.removeEventListener is not a function" 错误
      }
    }
  }
}

// 在 Monaco 加载后但在编辑器创建前，禁用颜色检测功能

// 导入扩展系统

const app = createApp(App)

app.use(createPinia())
app.use(router)

// 激活内置扩展
// 使用默认项目信息（实际项目中应该从项目存储加载）
async function activateExtensions() {
  try {
    await extensionHost.activateExtensions(builtinExtensions, {
      id: 'default',
      name: 'Default Project',
      path: '',
      description: '',
      createdAt: new Date().toISOString(),
      updatedAt: new Date().toISOString(),
    })
    console.log('[Main] All builtin extensions activated')
  } catch (error) {
    console.error('[Main] Failed to activate extensions:', error)
  }
}

// 在应用挂载后激活扩展
app.mount('#app')
activateExtensions()
