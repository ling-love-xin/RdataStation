/**
 * 内置扩展注册表
 *
 * 集中管理所有内置扩展的加载顺序
 * 扩展按顺序激活，确保依赖关系正确
 */

// 导入所有内置扩展
import analyticsResourceExtension from '@/extensions/builtin/analytics-resource/extension'
import connectionExtension from '@/extensions/builtin/connection/extension'
import databaseExtension from '@/extensions/builtin/database/extension'
import mysqlDriverExtension from '@/extensions/builtin/mysql-driver/extension'
import queryExtension from '@/extensions/builtin/query/extension'
import scratchpadExtension from '@/extensions/builtin/scratchpad/extension'
import workbenchExtension from '@/extensions/builtin/workbench/extension'
import type { ExtensionModule } from '@/extensions/core/types'

/**
 * 内置扩展列表
 *
 * 加载顺序很重要：
 * 1. connection - 连接管理（基础）
 * 2. database - 数据库导航（依赖 connection）
 * 3. query - SQL 查询执行（依赖 connection）
 * 4. workbench - 工作台布局（依赖所有）
 * 5. analytics-resource - 分析资源管理
 * 6. mysql-driver - 数据库驱动（可选）
 */
export interface BuiltinExtension {
  id: string
  module: ExtensionModule
}

export const builtinExtensions: BuiltinExtension[] = [
  { id: 'connection', module: connectionExtension },
  { id: 'database', module: databaseExtension },
  { id: 'query', module: queryExtension },
  { id: 'workbench', module: workbenchExtension },
  { id: 'analytics-resource', module: analyticsResourceExtension },
  { id: 'mysql-driver', module: mysqlDriverExtension },
  { id: 'scratchpad', module: scratchpadExtension },
]
