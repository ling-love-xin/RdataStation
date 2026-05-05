/**
 * IVM Navigator Extension
 * 基于增量视图维护的数据库导航栏
 */

import { l2Cache } from './domain/cache'
import { viewEngine } from './domain/engine/view-engine'
import NavigatorPanel from './ui/views/NavigatorPanel.vue'

import type { ExtensionContext } from '../../core/types'

export { viewEngine } from './domain/engine/view-engine'
export { useNavigator } from './ui/composables/useNavigator'
export type { NavigatorNode, NodeType, Delta } from './types'

export default {
  id: 'builtin.navigator',
  name: 'Database Navigator',
  version: '1.0.0',

  async activate(context: ExtensionContext) {
    // 初始化 L2 缓存
    await l2Cache.initialize()

    // 创建默认视图
    viewEngine.createView({
      name: 'navigator:main',
      initialData: []
    })

    // 注册面板
    context.window.registerViewProvider('navigator', {
      component: NavigatorPanel,
      title: '数据库导航',
      location: 'left',
      icon: 'Database',
      order: 0
    })

    // 注册命令
    context.commands.registerCommand('navigator.refresh', () => {
      viewEngine.refreshView('navigator:main')
    })

    context.commands.registerCommand('navigator.collapseAll', () => {
      // 发送折叠全部事件
      context.events.emit('navigator:collapseAll')
    })

    context.commands.registerCommand('navigator.expandAll', () => {
      // 发送展开全部事件
      context.events.emit('navigator:expandAll')
    })

    console.log('[Navigator] Extension activated')
  },

  deactivate() {
    // 清理资源
    viewEngine.deleteView('navigator:main')
    console.log('[Navigator] Extension deactivated')
  }
}
