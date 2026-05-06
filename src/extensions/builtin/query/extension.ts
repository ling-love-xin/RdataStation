/**
 * Query Extension
 *
 * 提供 SQL 查询执行能力
 */

import { sqlApi } from '@/shared/api'
import type { ExecuteSqlResponse, SqlHistoryResponse } from '@/shared/api'
import type { QueryResult } from '@/shared/types'

import ColumnInsightsPanel from '../workbench/ui/components/panels/ColumnInsightsPanel.vue'
import MultiTabResults from '../workbench/ui/components/panels/MultiTabResults.vue'
import QueryResultPanel from '../workbench/ui/components/panels/QueryResultPanel.vue'
import SqlEditorPanel from '../workbench/ui/components/panels/SqlEditorPanel.vue'

import type {
  ExtensionContext,
  ExtensionAPI,
  ExtensionModule,
  Disposable,
} from '../../core/types'

interface QueryExtensionAPI extends ExtensionAPI {
  query: {
    execute(connectionId: string, sql: string): Promise<QueryResult>
    cancel(queryId: string): Promise<void>
    getHistory(): Promise<QueryHistoryItem[]>
  }
}

interface QueryHistoryItem {
  id: string
  sql: string
  connectionId: string
  executedAt: Date
  executionTime: number
  rowCount: number
}

const activate = (context: ExtensionContext): QueryExtensionAPI => {
  console.log('[Query] Activating for project:', context.project.name)

  const activeQueries = new Map<string, AbortController>()

  const execute = async (connectionId: string, sql: string): Promise<QueryResult> => {
    const queryId = `query_${Date.now()}`
    const controller = new AbortController()
    activeQueries.set(queryId, controller)

    const response: ExecuteSqlResponse = await sqlApi.executeSql({
      conn_id: connectionId || undefined,
      sql,
    })

    const result: QueryResult = {
      columns: response.result.columns.map((c) => c.name),
      rows: response.result.rows,
      rowCount: response.result.row_count,
      executionTime: response.elapsed_ms,
      affectedRows: response.affected_rows,
    }

    activeQueries.delete(queryId)
    return result
  }

  const cancel = async (queryId: string): Promise<void> => {
    const controller = activeQueries.get(queryId)
    if (controller) {
      controller.abort()
      activeQueries.delete(queryId)
    }
  }

  const getHistory = async (): Promise<QueryHistoryItem[]> => {
    const history: SqlHistoryResponse[] = await sqlApi.getSqlHistory(100)
    return history.map((item) => ({
      id: item.id,
      sql: item.sql,
      connectionId: item.conn_id || '',
      executedAt: new Date(item.executed_at),
      executionTime: 0,
      rowCount: 0,
    }))
  }

  // 注册 SQL 编辑器面板
  const sqlEditorDisposable = context.window.registerViewProvider('sqlEditor', {
    component: SqlEditorPanel,
    title: 'SQL 编辑器',
    location: 'center',
    icon: 'Code',
    order: 1
  })

  // 注册查询结果面板（单语句结果）
  const resultPanelDisposable = context.window.registerViewProvider('queryResult', {
    component: QueryResultPanel,
    title: '查询结果',
    location: 'bottom',
    icon: 'Table2',
    order: 2
  })

  // 注册多 Tab 结果面板（多语句结果）
  const multiTabResultDisposable = context.window.registerViewProvider('multiTabResult', {
    component: MultiTabResults,
    title: '查询结果',
    location: 'bottom',
    icon: 'Table2',
    order: 3
  })

  // 注册列洞察面板（右侧可选面板）
  const columnInsightsDisposable = context.window.registerViewProvider('columnInsights', {
    component: ColumnInsightsPanel,
    title: '列洞察',
    location: 'right',
    icon: 'Eye',
    order: 4
  })

  const disposables: Disposable[] = [
    sqlEditorDisposable,
    resultPanelDisposable,
    multiTabResultDisposable,
    columnInsightsDisposable,
    context.commands.registerCommand('query.execute', (...args: unknown[]) =>
      execute(args[0] as string, args[1] as string)
    ),
    context.commands.registerCommand('query.cancel', (...args: unknown[]) =>
      cancel(args[0] as string)
    ),
    context.commands.registerCommand('query.history', (...args: unknown[]) => getHistory()),
  ]

  return {
    version: '1.0.0',
    project: context.project,
    commands: context.commands,
    window: context.window,
    workspace: context.workspace,
    database: context.database,
    sqlEditor: context.sqlEditor,
    events: context.events,
    configuration: context.configuration,
    utils: context.utils,

    query: {
      execute,
      cancel,
      getHistory,
    },

    dispose: () => {
      disposables.forEach((d) => d.dispose())
      activeQueries.forEach((controller) => controller.abort())
      activeQueries.clear()
    },
  }
}

const deactivate = (): void => {
  console.log('[Query] Deactivated')
}

const extension: ExtensionModule = {
  activate: activate as (context: ExtensionContext) => ExtensionAPI,
  deactivate,
}

export default extension
