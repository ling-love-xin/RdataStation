import { defineStore } from 'pinia'
import { ref, computed } from 'vue'

import type { SqlHistory, QueryTab } from '@/shared/types'

import * as queryService from '../services/query'

/**
 * 查询状态管理
 *
 * 管理 SQL 查询相关的状态，包括：
 * - 查询标签页
 * - SQL 历史记录
 * - 查询执行状态
 */
export const useQueryStore = defineStore('query', () => {
  // ==================== State ====================
  const tabs = ref<QueryTab[]>([
    {
      id: '1',
      title: '查询 1',
      name: '查询 1',
      sql: '',
      result: null,
      status: 'idle',
      isExecuting: false,
      loading: false,
      error: null,
      elapsedMs: 0,
    },
  ])
  const activeTabId = ref('1')
  const sqlHistory = ref<SqlHistory[]>([])
  const historyLoading = ref(false)

  // ==================== Getters ====================
  const activeTab = computed(
    () => tabs.value.find(t => t.id === activeTabId.value) || tabs.value[0]
  )
  const tabCount = computed(() => tabs.value.length)
  const hasResults = computed(() => activeTab.value?.result !== null)

  // ==================== Actions ====================

  /**
   * 添加新标签页
   */
  function addTab() {
    const newId = String(Date.now())
    const newTab: QueryTab = {
      id: newId,
      title: `查询 ${tabs.value.length + 1}`,
      name: `查询 ${tabs.value.length + 1}`,
      sql: '',
      result: null,
      status: 'idle',
      isExecuting: false,
      loading: false,
      error: null,
      elapsedMs: 0,
    }
    tabs.value.push(newTab)
    activeTabId.value = newId
    return newTab
  }

  /**
   * 移除标签页
   */
  function removeTab(tabId: string) {
    if (tabs.value.length <= 1) {
      // 至少保留一个标签页
      const tab = tabs.value[0]
      tab.sql = ''
      tab.result = null
      tab.error = null
      tab.elapsedMs = 0
      return
    }

    const index = tabs.value.findIndex(t => t.id === tabId)
    if (index === -1) return

    tabs.value.splice(index, 1)

    // 如果关闭的是当前活动标签，切换到相邻标签
    if (activeTabId.value === tabId) {
      const newIndex = Math.min(index, tabs.value.length - 1)
      activeTabId.value = tabs.value[newIndex].id
    }
  }

  /**
   * 切换活动标签页
   */
  function switchTab(tabId: string) {
    if (tabs.value.find(t => t.id === tabId)) {
      activeTabId.value = tabId
    }
  }

  /**
   * 更新标签页 SQL
   */
  function updateTabSql(tabId: string, sql: string) {
    const tab = tabs.value.find(t => t.id === tabId)
    if (tab) {
      tab.sql = sql
    }
  }

  /**
   * 重命名标签页
   */
  function renameTab(tabId: string, name: string) {
    const tab = tabs.value.find(t => t.id === tabId)
    if (tab) {
      tab.name = name
    }
  }

  /**
   * 执行 SQL 查询
   */
  async function executeSql(connId?: string, sql?: string, timeoutMs?: number) {
    const tab = activeTab.value
    if (!tab) return

    // 使用传入的 SQL 或当前标签页的 SQL
    const executeSql = sql || tab.sql
    if (!executeSql.trim()) return

    tab.loading = true
    tab.error = null
    tab.result = null
    tab.elapsedMs = 0

    try {
      const result = await queryService.executeSql(executeSql, connId, timeoutMs)
      if (result.result) {
        tab.result = {
          columns: result.result.columns,
          rows: result.result.rows,
          rowCount: result.rowCount || result.result.rows.length,
          executionTime: result.executionTime || result.elapsed_ms || 0,
        }
      } else if (result.rows) {
        tab.result = {
          columns: result.columns,
          rows: result.rows,
          rowCount: result.rowCount || result.rows.length,
          executionTime: result.executionTime || result.elapsed_ms || 0,
        }
      }
      tab.elapsedMs = result.elapsed_ms ?? result.executionTime ?? 0
      tab.affectedRows = result.affected_rows ?? result.affectedRows ?? undefined
    } catch (e) {
      tab.error = e instanceof Error ? e.message : '执行失败'
    } finally {
      tab.loading = false
    }
  }

  /**
   * 执行事务
   */
  async function executeTransaction(sqls: string[], connId?: string) {
    const tab = activeTab.value
    if (!tab) return

    tab.loading = true
    tab.error = null
    tab.result = null

    try {
      const result = await queryService.executeTransaction(sqls, connId)
      // 显示最后一个查询的结果
      if (result.results && result.results.length > 0) {
        const lastResult = result.results[result.results.length - 1]
        if (lastResult) {
          tab.result = {
            columns: lastResult.columns,
            rows: lastResult.rows,
            rowCount: lastResult.rowCount || lastResult.rows?.length || 0,
            executionTime: lastResult.executionTime || 0,
          }
        }
      }
      tab.elapsedMs = result.total_elapsed_ms ?? 0
    } catch (e) {
      tab.error = e instanceof Error ? e.message : '事务执行失败'
    } finally {
      tab.loading = false
    }
  }

  /**
   * 加载 SQL 历史记录
   */
  async function loadSqlHistory(limit?: number) {
    historyLoading.value = true
    try {
      const result = await queryService.getSqlHistory(limit)
      sqlHistory.value = result.map(r => ({
        id: r.id,
        sql: r.sql,
        connId: r.conn_id,
        executedAt: r.executed_at,
        executionTime: r.execution_time,
        rowCount: r.row_count,
        success: r.success,
        error: r.error,
      }))
    } catch (e) {
      console.error('加载SQL历史失败:', e)
    } finally {
      historyLoading.value = false
    }
  }

  /**
   * 搜索 SQL 历史记录
   */
  async function searchSqlHistory(keyword: string, limit?: number) {
    historyLoading.value = true
    try {
      const result = await queryService.searchSqlHistory(keyword, limit)
      sqlHistory.value = result.map(r => ({
        id: r.id,
        sql: r.sql,
        connId: r.conn_id,
        executedAt: r.executed_at,
        executionTime: r.execution_time,
        rowCount: r.row_count,
        success: r.success,
        error: r.error,
      }))
    } catch (e) {
      console.error('搜索SQL历史失败:', e)
    } finally {
      historyLoading.value = false
    }
  }

  /**
   * 清空 SQL 历史记录
   */
  async function clearSqlHistory() {
    try {
      await queryService.clearSqlHistory()
      sqlHistory.value = []
    } catch (e) {
      console.error('清空SQL历史失败:', e)
    }
  }

  /**
   * 删除指定 SQL 历史记录
   */
  async function removeSqlHistory(id: string) {
    try {
      await queryService.removeSqlHistory(id)
      sqlHistory.value = sqlHistory.value.filter(h => h.id !== id)
    } catch (e) {
      console.error('删除SQL历史失败:', e)
    }
  }

  /**
   * 清除标签页错误
   */
  function clearTabError(tabId?: string) {
    const targetTab = tabId ? tabs.value.find(t => t.id === tabId) : activeTab.value
    if (targetTab) {
      targetTab.error = null
    }
  }

  return {
    // State
    tabs,
    activeTabId,
    sqlHistory,
    historyLoading,
    // Getters
    activeTab,
    tabCount,
    hasResults,
    // Actions
    addTab,
    removeTab,
    switchTab,
    updateTabSql,
    renameTab,
    executeSql,
    executeTransaction,
    loadSqlHistory,
    searchSqlHistory,
    clearSqlHistory,
    removeSqlHistory,
    clearTabError,
  }
})
