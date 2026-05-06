import { createPinia } from 'pinia'
import { createApp } from 'vue'

import { builtinExtensions } from '@/core/builtin-extensions'
import { extensionHost } from '@/core/extension-host'
import { panelRegistry } from '@/core/panel-registry'
import ActivityBarPanel from '@/extensions/builtin/workbench/ui/components/ActivityBarPanel.vue'

// Configure Monaco Editor for Tauri environment
// 完全禁用 Web Worker，在主线程中运行所有功能
// Tauri 的安全策略和 CSP 限制使得 Web Worker 加载不稳定
;
import PanelHeaderActions from '@/extensions/builtin/workbench/ui/components/PanelHeaderActions.vue'

import App from './App.vue'
import router from './router'

// Import styles
import '@/shared/styles/theme.css'
import 'dockview-vue/dist/styles/dockview.css'
// dockview 主题覆盖必须在 dockview 默认样式之后导入
import '@/shared/styles/dockview-theme.css'

// 导入自定义组件（用于 dockview rightHeaderActionsComponent）
(self as any).MonacoEnvironment = {
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

async function main() {
  // 步骤 1：先激活所有内置扩展
  // 此时 panelRegistry 中会注册所有面板，但 Vue 应用尚未渲染
  // 确保 Dockview 的 onReady 事件触发时，面板已就绪
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

  // 步骤 2：全局注册所有面板组件（给 dockview-vue 的 findComponent 使用）
  // dockview-vue 通过 appContext.components 查找组件，必须全局注册
  const panels = panelRegistry.getAll()
  for (const panel of panels) {
    if (panel.component) {
      try {
        app.component(panel.id, panel.component as any)
        console.log(`[Main] Registered global component: ${panel.id}`)
      } catch (e) {
        console.warn(`[Main] Failed to register component '${panel.id}':`, e)
      }
    }
  }
  console.log(`[Main] Registered ${panels.length} panel components globally`)

  // 注册 Dockview 面板操作组件（最大化/弹出/钉住按钮）
  app.component('panelHeaderActions', PanelHeaderActions)
  console.log('[Main] Registered panelHeaderActions component')

  // 注册左侧活动栏组件（类似 VSCode Activity Bar）
  app.component('leftActivityBar', ActivityBarPanel)
  console.log('[Main] Registered leftActivityBar component')

  // 步骤 3：再挂载 Vue 应用
  // Dockview 的 onReady 事件将在 Vue 渲染后触发
  // 此时 panelRegistry 中已有所有面板数据，组件也已全局注册
  app.mount('#app')
}

main()
