/**
 * SQL 执行历史增强服务
 * 提供历史记录管理、收藏、搜索等功能
 */

export interface SqlHistoryItem {
  id: string
  sql: string
  connectionId: string
  connectionName: string
  databaseType: string
  executedAt: number
  executionTime: number
  rowCount: number
  success: boolean
  error?: string
  isFavorite: boolean
  tags?: string[]
  note?: string
}

const STORAGE_KEY = 'sql-execution-history'
const FAVORITES_KEY = 'sql-favorites'

/**
 * 获取所有历史记录
 */
export function getHistory(limit: number = 100): SqlHistoryItem[] {
  try {
    const stored = localStorage.getItem(STORAGE_KEY)
    if (stored) {
      const history: SqlHistoryItem[] = JSON.parse(stored)
      return history.slice(0, limit)
    }
  } catch (error) {
    console.error('Failed to load history:', error)
  }
  return []
}

/**
 * 添加历史记录
 */
export function addHistory(item: Omit<SqlHistoryItem, 'id' | 'executedAt'>): SqlHistoryItem {
  const history = getHistory(1000)
  
  const newItem: SqlHistoryItem = {
    ...item,
    id: `history-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`,
    executedAt: Date.now()
  }
  
  history.unshift(newItem)
  
  // 限制历史记录数量
  if (history.length > 1000) {
    history.splice(1000)
  }
  
  saveHistory(history)
  return newItem
}

/**
 * 删除历史记录
 */
export function deleteHistory(id: string): boolean {
  const history = getHistory(1000)
  const filtered = history.filter(item => item.id !== id)
  
  if (filtered.length === history.length) {
    return false
  }
  
  saveHistory(filtered)
  return true
}

/**
 * 清空历史记录
 */
export function clearHistory(): void {
  localStorage.removeItem(STORAGE_KEY)
}

/**
 * 切换收藏状态
 */
export function toggleFavorite(id: string): boolean {
  const history = getHistory(1000)
  const index = history.findIndex(item => item.id === id)
  
  if (index === -1) {
    return false
  }
  
  history[index].isFavorite = !history[index].isFavorite
  saveHistory(history)
  return true
}

/**
 * 获取收藏的 SQL
 */
export function getFavorites(): SqlHistoryItem[] {
  const history = getHistory(1000)
  return history.filter(item => item.isFavorite)
}

/**
 * 搜索历史记录
 */
export function searchHistory(query: string): SqlHistoryItem[] {
  const history = getHistory(1000)
  const lowerQuery = query.toLowerCase()
  
  return history.filter(
    item =>
      item.sql.toLowerCase().includes(lowerQuery) ||
      item.connectionName.toLowerCase().includes(lowerQuery) ||
      (item.tags && item.tags.some(tag => tag.toLowerCase().includes(lowerQuery))) ||
      (item.note && item.note.toLowerCase().includes(lowerQuery))
  )
}

/**
 * 按连接过滤历史记录
 */
export function filterByConnection(connectionId: string): SqlHistoryItem[] {
  const history = getHistory(1000)
  return history.filter(item => item.connectionId === connectionId)
}

/**
 * 按数据库类型过滤历史记录
 */
export function filterByDatabaseType(databaseType: string): SqlHistoryItem[] {
  const history = getHistory(1000)
  return history.filter(item => item.databaseType === databaseType)
}

/**
 * 按时间范围过滤历史记录
 */
export function filterByDateRange(startDate: number, endDate: number): SqlHistoryItem[] {
  const history = getHistory(1000)
  return history.filter(
    item => item.executedAt >= startDate && item.executedAt <= endDate
  )
}

/**
 * 获取执行统计信息
 */
export function getStatistics(): {
  totalExecutions: number
  successRate: number
  averageExecutionTime: number
  totalFavorites: number
  topConnections: Array<{ connectionName: string; count: number }>
} {
  const history = getHistory(1000)
  
  const totalExecutions = history.length
  const successCount = history.filter(item => item.success).length
  const successRate = totalExecutions > 0 ? (successCount / totalExecutions) * 100 : 0
  const averageExecutionTime =
    totalExecutions > 0
      ? history.reduce((sum, item) => sum + item.executionTime, 0) / totalExecutions
      : 0
  const totalFavorites = history.filter(item => item.isFavorite).length
  
  // 统计最常用的连接
  const connectionCounts = new Map<string, number>()
  history.forEach(item => {
    connectionCounts.set(
      item.connectionName,
      (connectionCounts.get(item.connectionName) || 0) + 1
    )
  })
  
  const topConnections = Array.from(connectionCounts.entries())
    .map(([connectionName, count]) => ({ connectionName, count }))
    .sort((a, b) => b.count - a.count)
    .slice(0, 10)
  
  return {
    totalExecutions,
    successRate,
    averageExecutionTime,
    totalFavorites,
    topConnections
  }
}

/**
 * 添加标签
 */
export function addTag(id: string, tag: string): boolean {
  const history = getHistory(1000)
  const index = history.findIndex(item => item.id === id)
  
  if (index === -1) {
    return false
  }
  
  if (!history[index].tags) {
    history[index].tags = []
  }
  
  if (!history[index].tags.includes(tag)) {
    history[index].tags.push(tag)
    saveHistory(history)
  }
  
  return true
}

/**
 * 移除标签
 */
export function removeTag(id: string, tag: string): boolean {
  const history = getHistory(1000)
  const index = history.findIndex(item => item.id === id)
  
  if (index === -1) {
    return false
  }
  
  if (history[index].tags) {
    history[index].tags = history[index].tags.filter(t => t !== tag)
    saveHistory(history)
  }
  
  return true
}

/**
 * 添加备注
 */
export function addNote(id: string, note: string): boolean {
  const history = getHistory(1000)
  const index = history.findIndex(item => item.id === id)
  
  if (index === -1) {
    return false
  }
  
  history[index].note = note
  saveHistory(history)
  return true
}

/**
 * 保存历史记录到 localStorage
 */
function saveHistory(history: SqlHistoryItem[]): void {
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(history))
  } catch (error) {
    console.error('Failed to save history:', error)
  }
}

/**
 * 导出历史记录
 */
export function exportHistory(): string {
  const history = getHistory(1000)
  return JSON.stringify(history, null, 2)
}

/**
 * 导入历史记录
 */
export function importHistory(json: string): boolean {
  try {
    const history: SqlHistoryItem[] = JSON.parse(json)
    if (!Array.isArray(history)) {
      return false
    }
    
    const existingHistory = getHistory(1000)
    const mergedHistory = [...history, ...existingHistory]
    
    // 去重（基于 id）
    const uniqueIds = new Set<string>()
    const deduplicatedHistory = mergedHistory.filter(item => {
      if (uniqueIds.has(item.id)) {
        return false
      }
      uniqueIds.add(item.id)
      return true
    })
    
    saveHistory(deduplicatedHistory)
    return true
  } catch (error) {
    console.error('Failed to import history:', error)
    return false
  }
}

/**
 * 获取常用 SQL（执行次数最多的）
 */
export function getFrequentSql(limit: number = 10): Array<{ sql: string; count: number }> {
  const history = getHistory(1000)
  const sqlCounts = new Map<string, number>()
  
  history.forEach(item => {
    const normalizedSql = item.sql.trim().toLowerCase()
    sqlCounts.set(normalizedSql, (sqlCounts.get(normalizedSql) || 0) + 1)
  })
  
  return Array.from(sqlCounts.entries())
    .map(([sql, count]) => ({ sql, count }))
    .sort((a, b) => b.count - a.count)
    .slice(0, limit)
}

/**
 * 获取最近执行的 SQL
 */
export function getRecentSql(limit: number = 10): SqlHistoryItem[] {
  return getHistory(limit)
}
