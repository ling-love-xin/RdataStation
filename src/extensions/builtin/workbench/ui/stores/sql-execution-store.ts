/**
 * SQL 执行状态管理
 * 
 * 使用 Pinia Store 管理 SQL 编辑器与结果面板之间的通信
 * 替代全局事件，提供可靠的面板 ID 绑定通信机制
 */

import { defineStore } from 'pinia'
import { ref, computed } from 'vue'

import * as queryService from '@/extensions/builtin/query/ui/services/query'

export interface ExecutionRequest {
  panelId: string
  sql: string
  connectionId: string
  timestamp: number
  status: 'pending' | 'executing' | 'completed' | 'failed'
}

export interface ExecutionResult {
  panelId: string
  result: {
    columns: string[]
    rows: any[][]
    rowCount: number
    executionTime: number
    affectedRows?: number
  } | null
  error: string | null
  timestamp: number
}

interface PendingRequest {
  request: ExecutionRequest
  resolve: (result: ExecutionResult) => void
  reject: (error: Error) => void
}

export const useSqlExecutionStore = defineStore('sqlExecution', () => {
  // ==================== State ====================
  const pendingRequests = ref<Map<string, PendingRequest>>(new Map())
  const executionResults = ref<Map<string, ExecutionResult>>(new Map())
  const activeEditorPanelId = ref<string | null>(null)

  // ==================== Getters ====================
  const hasPendingRequests = computed(() => pendingRequests.value.size > 0)
  
  const getExecutionResult = computed(() => (panelId: string) => {
    return executionResults.value.get(panelId) || null
  })

  // ==================== Actions ====================

  /**
   * 设置当前活动的编辑器面板 ID
   */
  function setActiveEditorPanelId(panelId: string | null) {
    activeEditorPanelId.value = panelId
  }

  /**
   * 执行 SQL 查询
   * 返回 Promise，调用者可以等待结果
   */
  async function executeSql(
    panelId: string,
    sql: string,
    connectionId: string
  ): Promise<ExecutionResult> {
    const request: ExecutionRequest = {
      panelId,
      sql,
      connectionId,
      timestamp: Date.now(),
      status: 'pending'
    }

    // 存储 pending 请求
    const promise = new Promise<ExecutionResult>((resolve, reject) => {
      pendingRequests.value.set(panelId, { request, resolve, reject })
    })

    // 使用 async IIFE 来执行查询
    ;(async () => {
      try {
        // 执行查询
        const result = await queryService.executeSql(sql, connectionId)
        
        // 构建结果
        // Tauri 响应结构: { result: { columns, rows, total_rows, affected_rows, is_read_only }, elapsed_ms, affected_rows }
        const qr = result.result || result
        const executionResult: ExecutionResult = {
          panelId,
          result: {
            columns: qr.columns || [],
            rows: qr.rows || [],
            rowCount: qr.total_rows || qr.rows?.length || 0,
            executionTime: result.elapsed_ms || 0,
            affectedRows: qr.affected_rows || result.affected_rows
          },
          error: null,
          timestamp: Date.now()
        }

        // 存储结果
        executionResults.value.set(panelId, executionResult)
        
        // 移除 pending 请求并解析 Promise
        const pending = pendingRequests.value.get(panelId)
        pendingRequests.value.delete(panelId)
        if (pending) {
          pending.resolve(executionResult)
        }
      } catch (error) {
        // Tauri invoke 错误是 string 类型，Error 类型需要 .message
        const errorMsg = typeof error === 'string' ? error 
          : error instanceof Error ? error.message 
          : '执行失败'

        const errorResult: ExecutionResult = {
          panelId,
          result: null,
          error: errorMsg,
          timestamp: Date.now()
        }

        executionResults.value.set(panelId, errorResult)

        // 移除 pending 请求并拒绝 Promise
        const pending = pendingRequests.value.get(panelId)
        pendingRequests.value.delete(panelId)
        if (pending) {
          pending.reject(new Error(errorMsg))
        }
      }
    })()

    return promise
  }

  /**
   * 获取指定面板的执行结果
   */
  function getResultForPanel(panelId: string): ExecutionResult | null {
    return executionResults.value.get(panelId) || null
  }

  /**
   * 清除指定面板的结果
   */
  function clearResult(panelId: string) {
    executionResults.value.delete(panelId)
  }

  /**
   * 清除所有结果
   */
  function clearAllResults() {
    executionResults.value.clear()
    pendingRequests.value.clear()
  }

  return {
    // State
    pendingRequests,
    executionResults,
    activeEditorPanelId,
    // Getters
    hasPendingRequests,
    getExecutionResult,
    // Actions
    setActiveEditorPanelId,
    executeSql,
    getResultForPanel,
    clearResult,
    clearAllResults
  }
})
