/**
 * 元数据缓存服务
 *
 * 提供数据库元数据缓存的读取、刷新、保存等操作
 * 优先使用缓存，缓存失效时回退到实时查询
 */

import { invoke } from '@tauri-apps/api/core'

/**
 * 缓存状态响应
 */
export interface CacheStatusResponse {
  is_valid: boolean
  last_sync: number | null
  stats: CacheStatsResponse | null
}

/**
 * 缓存统计响应
 */
export interface CacheStatsResponse {
  table_count: number
  column_count: number
  last_sync: number | null
}

/**
 * 表元数据
 */
export interface CachedTable {
  id: string
  name: string
  schema_name?: string
  comment: string | null
  last_sync: number | null
}

/**
 * 列元数据
 */
export interface CachedColumn {
  id: string
  name: string
  data_type: string
  is_nullable: boolean
  is_primary: boolean
  is_unique: boolean
  comment: string | null
  last_sync: number | null
}

/**
 * 表元数据输入
 */
export interface TableInput {
  id: string
  name: string
  comment?: string
}

/**
 * 列元数据输入
 */
export interface ColumnInput {
  id: string
  name: string
  data_type: string
  is_nullable: boolean
  is_primary: boolean
  is_unique: boolean
}

/**
 * 获取缓存状态
 */
export async function getMetadataCacheStatus(
  connectionId: string,
  connectionType: 'global' | 'project',
  databaseName: string,
  schemaName?: string,
  projectPath?: string
): Promise<CacheStatusResponse> {
  return invoke<CacheStatusResponse>('get_metadata_cache_status', {
    connectionId,
    connectionType,
    projectPath,
    databaseName,
    schemaName,
  })
}

/**
 * 刷新元数据缓存
 */
export async function refreshMetadataCache(
  connectionId: string,
  connectionType: 'global' | 'project',
  databaseName: string,
  schemaName?: string,
  projectPath?: string
): Promise<void> {
  return invoke('refresh_metadata_cache', {
    input: {
      connectionId,
      connectionType,
      projectPath,
      databaseName,
      schemaName,
    },
  })
}

/**
 * 清除元数据缓存
 */
export async function clearMetadataCache(
  connectionId: string,
  connectionType: 'global' | 'project',
  databaseName: string,
  schemaName?: string,
  projectPath?: string
): Promise<number> {
  return invoke<number>('clear_metadata_cache', {
    input: {
      connectionId,
      connectionType,
      projectPath,
      databaseName,
      schemaName,
    },
  })
}

/**
 * 从缓存获取表列表
 */
export async function getTablesFromCache(
  connectionId: string,
  connectionType: 'global' | 'project',
  databaseName: string,
  schemaName?: string,
  projectPath?: string
): Promise<CachedTable[]> {
  const result = await invoke<any[]>('get_tables_from_cache', {
    connectionId,
    connectionType,
    projectPath,
    databaseName,
    schemaName,
  })

  return result.map(item => ({
    id: item.id,
    name: item.name,
    comment: item.comment,
    last_sync: item.last_sync,
  }))
}

/**
 * 从缓存获取列列表
 */
export async function getColumnsFromCache(
  connectionId: string,
  connectionType: 'global' | 'project',
  databaseName: string,
  schemaName: string,
  tableName: string,
  projectPath?: string
): Promise<CachedColumn[]> {
  const result = await invoke<any[]>('get_columns_from_cache', {
    connectionId,
    connectionType,
    projectPath,
    databaseName,
    schemaName,
    tableName,
  })

  return result.map(item => ({
    id: item.id,
    name: item.name,
    data_type: item.data_type,
    is_nullable: item.is_nullable,
    is_primary: item.is_primary,
    is_unique: item.is_unique,
    comment: item.comment,
    last_sync: item.last_sync,
  }))
}

/**
 * 保存表元数据到缓存
 */
export async function saveTableMetadataToCache(
  connectionId: string,
  connectionType: 'global' | 'project',
  id: string,
  databaseName: string,
  schemaName: string,
  tableName: string,
  comment?: string,
  projectPath?: string
): Promise<void> {
  return invoke('save_table_metadata_to_cache', {
    connectionId,
    connectionType,
    projectPath,
    id,
    databaseName,
    schemaName,
    tableName,
    comment,
  })
}

/**
 * 保存列元数据到缓存
 */
export async function saveColumnMetadataToCache(
  connectionId: string,
  connectionType: 'global' | 'project',
  id: string,
  databaseName: string,
  schemaName: string,
  tableName: string,
  columnName: string,
  dataType: string,
  isNullable: boolean,
  isPrimary: boolean,
  isUnique: boolean,
  projectPath?: string
): Promise<void> {
  return invoke('save_column_metadata_to_cache', {
    connectionId,
    connectionType,
    projectPath,
    id,
    databaseName,
    schemaName,
    tableName,
    columnName,
    dataType,
    isNullable,
    isPrimary,
    isUnique,
  })
}

/**
 * 批量保存表元数据到缓存
 */
export async function saveTablesBatchToCache(
  connectionId: string,
  connectionType: 'global' | 'project',
  databaseName: string,
  schemaName: string,
  tables: TableInput[],
  projectPath?: string
): Promise<number> {
  return invoke<number>('save_tables_batch_to_cache', {
    connectionId,
    connectionType,
    projectPath,
    databaseName,
    schemaName,
    tables,
  })
}

/**
 * 批量保存列元数据到缓存
 */
export async function saveColumnsBatchToCache(
  connectionId: string,
  connectionType: 'global' | 'project',
  databaseName: string,
  schemaName: string,
  tableName: string,
  columns: ColumnInput[],
  projectPath?: string
): Promise<number> {
  return invoke<number>('save_columns_batch_to_cache', {
    connectionId,
    connectionType,
    projectPath,
    databaseName,
    schemaName,
    tableName,
    columns,
  })
}

/**
 * 生成稳定的缓存 ID
 *
 * 使用稳定的 ID 策略，避免每次生成新 ID 导致缓存积累
 * ID 格式：{connectionId}:{databaseName}:{schemaName}:{tableName}:{columnName}
 */
export function generateStableCacheId(
  connectionId: string,
  databaseName: string,
  schemaName: string,
  tableName: string,
  columnName?: string
): string {
  const base = `${connectionId}:${databaseName}:${schemaName}:${tableName}`
  return columnName ? `${base}:${columnName}` : base
}

/**
 * 预热进度响应
 */
export interface WarmingProgressResponse {
  connection_id: string
  is_warming: boolean
  current_step: string
  total_steps: number
  completed_steps: number
  progress_percentage: number
  current_database: string | null
  current_schema: string | null
  current_table: string | null
}

// ==================== V6: 索引表与分页 ====================

/**
 * 索引条目
 */
export interface IndexEntry {
  id: number
  schema_id: number | null
  object_type: string
  object_name: string
  parent_name: string | null
  path: string
  introspect_level: number
  is_loaded: boolean
  last_sync: number | null
  row_count_estimate: number | null
  sort_weight: number | null
}

/**
 * 分页索引结果
 */
export interface PaginatedIndexResult {
  entries: IndexEntry[]
  total: number
  page: number
  page_size: number
  total_pages: number
}

/**
 * 同步状态信息
 */
export interface SyncStatusInfo {
  connection_id: string
  status: string
  progress: number
  total_objects: number | null
  synced_objects: number | null
  current_object: string | null
  started_at: number | null
  completed_at: number | null
  last_error: string | null
}

/**
 * 获取分页索引条目
 */
export async function getIndexEntries(
  connectionId: string,
  objectType: string,
  schemaId?: number,
  page: number = 1,
  pageSize: number = 100
): Promise<PaginatedIndexResult> {
  return invoke<PaginatedIndexResult>('get_index_entries', {
    connectionId,
    objectType,
    schemaId,
    page,
    pageSize,
  })
}

/**
 * 获取同步状态
 */
export async function getSyncStatus(connectionId: string): Promise<SyncStatusInfo | null> {
  return invoke<SyncStatusInfo | null>('get_sync_status', {
    connectionId,
  })
}

/**
 * 取消同步
 */
export async function cancelSync(connectionId: string): Promise<void> {
  return invoke('cancel_sync', {
    connectionId,
  })
}

// ==================== V6: 后台同步任务队列 ====================

/**
 * 同步任务输入
 */
export interface SyncTaskInput {
  connection_id: string
  task_type: string
  object_name: string
  parent_name?: string
  priority: number
}

/**
 * 同步任务信息
 */
export interface SyncTaskInfo {
  id: number
  connection_id: string
  task_type: string
  object_name: string
  parent_name?: string
  priority: number
  status: string
  created_at: number | null
}

/**
 * 入队同步任务
 */
export async function enqueueSyncTask(
  connectionId: string,
  taskType: string,
  objectName: string,
  parentName?: string,
  priority: number = 5
): Promise<number> {
  return invoke<number>('enqueue_sync_task', {
    connectionId,
    taskType,
    objectName,
    parentName,
    priority,
  })
}

/**
 * 入队多个同步任务（批量）
 */
export async function enqueueSyncTasksBatch(tasks: SyncTaskInput[]): Promise<number> {
  return invoke<number>('enqueue_sync_tasks_batch', {
    tasks,
  })
}

/**
 * 获取待处理任务数量
 */
export async function getPendingTaskCount(connectionId: string): Promise<number> {
  return invoke<number>('get_pending_task_count', {
    connectionId,
  })
}

// ==================== V6: 分块读取 ====================

/**
 * 分块读取结果
 */
export interface ChunkResult<T> {
  items: T[]
  total: number
  offset: number
  limit: number
  has_more: boolean
}

/**
 * 分块获取表名（避免 OOM）
 */
export async function getTablesChunk(
  connectionId: string,
  schemaId?: number,
  offset: number = 0,
  limit: number = 100
): Promise<ChunkResult<IndexEntry>> {
  return invoke<ChunkResult<IndexEntry>>('get_tables_chunk', {
    connectionId,
    schemaId,
    offset,
    limit,
  })
}

/**
 * 迁移响应
 */
export interface MigrationResponse {
  from_version: number
  to_version: number
  success: boolean
  duration_ms: number | null
  message: string
}

/**
 * 启动缓存预热
 */
export async function startCacheWarming(
  connectionId: string,
  connectionType: 'global' | 'project',
  databases: string[],
  projectPath?: string
): Promise<WarmingProgressResponse> {
  return invoke<WarmingProgressResponse>('start_cache_warming', {
    input: {
      connectionId,
      connectionType,
      projectPath,
      databases,
    },
  })
}

/**
 * 取消缓存预热
 */
export async function cancelCacheWarming(connectionId: string): Promise<void> {
  return invoke('cancel_cache_warming', {
    input: {
      connectionId,
    },
  })
}

/**
 * 获取预热进度
 */
export async function getWarmingProgress(connectionId: string): Promise<WarmingProgressResponse> {
  return invoke<WarmingProgressResponse>('get_warming_progress', {
    connectionId,
  })
}

/**
 * 检查缓存版本
 */
export async function checkCacheVersion(
  connectionId: string,
  connectionType: 'global' | 'project',
  projectPath?: string
): Promise<number> {
  return invoke<number>('check_cache_version', {
    connectionId,
    connectionType,
    projectPath,
  })
}

/**
 * 执行缓存版本迁移
 */
export async function executeCacheMigration(
  connectionId: string,
  connectionType: 'global' | 'project',
  projectPath?: string
): Promise<MigrationResponse> {
  return invoke<MigrationResponse>('execute_cache_migration', {
    connectionId,
    connectionType,
    projectPath,
  })
}

/**
 * 获取缓存版本迁移历史
 */
export async function getCacheMigrationHistory(
  connectionId: string,
  connectionType: 'global' | 'project',
  projectPath?: string
): Promise<
  Array<{
    from_version: number
    to_version: number
    migrated_at: number
    reason: string | null
    duration_ms: number | null
    success: boolean
  }>
> {
  return invoke('get_cache_migration_history', {
    connectionId,
    connectionType,
    projectPath,
  })
}

// ==================== V6: DataGrip 风格内省级别 ====================

/**
 * Schema 对象数量统计
 */
export interface SchemaObjectCounts {
  table_count: number
  view_count: number
  column_count: number
  routine_count: number
  total: number
}

/**
 * 获取内省级别建议（DataGrip 风格）
 *
 * 根据对象数量自动计算合适的内省级别：
 * - Level 1: 仅索引（对象数量大）
 * - Level 2: 概要（中等数量）
 * - Level 3: 完整（少量对象）
 *
 * @param schemaId Schema ID
 * @param isCurrentSchema 是否为当前 Schema
 */
export async function getIntrospectLevelSuggestion(
  connectionId: string,
  connectionType: 'global' | 'project',
  schemaId: number,
  isCurrentSchema: boolean,
  projectPath?: string
): Promise<number> {
  return invoke<number>('get_introspect_level_suggestion', {
    connectionId,
    connectionType,
    projectPath,
    schemaId,
    isCurrentSchema,
  })
}

/**
 * 获取 Schema 对象数量统计
 */
export async function getSchemaObjectCounts(
  connectionId: string,
  connectionType: 'global' | 'project',
  schemaId: number,
  projectPath?: string
): Promise<SchemaObjectCounts> {
  return invoke<SchemaObjectCounts>('get_schema_object_counts', {
    connectionId,
    connectionType,
    projectPath,
    schemaId,
  })
}
