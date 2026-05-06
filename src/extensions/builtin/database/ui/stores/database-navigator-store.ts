import { defineStore } from 'pinia'
import { ref, computed } from 'vue'

import { closeConnection, executeSql as executeSqlService } from '@/extensions/builtin/connection/ui/services/connection'

import {
  clearMetadataCache,
  getColumnsFromCache,
  getMetadataCacheStatus,
  getTablesFromCache,
  saveTablesBatchToCache,
  saveColumnsBatchToCache,
  generateStableCacheId
} from '../services/metadata-cache-service'

import type { TableInput, ColumnInput } from '../services/metadata-cache-service'

function escapeSql(value: string): string {
  return value.replace(/'/g, "''")
}

function quoteIdentifier(value: string): string {
  return `"${value.replace(/"/g, '""')}"`
}

export const useDatabaseNavigatorStore = defineStore('databaseNavigator', () => {
  const connectionDatabases = ref<Map<string, DatabaseNode[]>>(new Map())
  const selectedObject = ref<SelectedObject | null>(null)
  const loadingDatabases = ref<Set<string>>(new Set())
  const loadingSchemas = ref<Set<string>>(new Set())
  const loadingTables = ref<Set<string>>(new Set())
  const loadingColumns = ref<Set<string>>(new Set())
  const error = ref<string | null>(null)
  const connectionTypes = ref<Map<string, 'global' | 'project'>>(new Map())
  const connectionProjectPaths = ref<Map<string, string | undefined>>(new Map())
  const connectionDbTypes = ref<Map<string, string>>(new Map())
  
  const lastSyncTimes = ref<Map<string, number>>(new Map())
  const syncModes = ref<Map<string, 'full' | 'incremental'>>(new Map())

  function getDatabases(connectionId: string): DatabaseNode[] {
    return connectionDatabases.value.get(connectionId) || []
  }

  function getLastSyncTime(connectionId: string, dbName?: string, schemaName?: string): number {
    const key = dbName ? (schemaName ? `${connectionId}:${dbName}:${schemaName}` : `${connectionId}:${dbName}`) : connectionId
    return lastSyncTimes.value.get(key) || 0
  }

  function setLastSyncTime(connectionId: string, dbName?: string, schemaName?: string) {
    const key = dbName ? (schemaName ? `${connectionId}:${dbName}:${schemaName}` : `${connectionId}:${dbName}`) : connectionId
    lastSyncTimes.value.set(key, Date.now())
  }

  function setSyncMode(connectionId: string, mode: 'full' | 'incremental') {
    syncModes.value.set(connectionId, mode)
  }

  function getSyncMode(connectionId: string): 'full' | 'incremental' {
    return syncModes.value.get(connectionId) || 'incremental'
  }

  const isLoadingDatabases = computed(() => {
    return (connectionId: string): boolean => {
      return loadingDatabases.value.has(connectionId)
    }
  })

  const isLoadingSchemas = computed(() => {
    return (connectionId: string, dbName: string): boolean => {
      return loadingSchemas.value.has(`${connectionId}:${dbName}`)
    }
  })

  const isLoadingTables = computed(() => {
    return (connectionId: string, dbName: string, schemaName: string): boolean => {
      return loadingTables.value.has(`${connectionId}:${dbName}:${schemaName}`)
    }
  })

  function setConnectionInfo(
    connectionId: string,
    type: 'global' | 'project',
    projectPath?: string,
    dbType?: string
  ) {
    connectionTypes.value.set(connectionId, type)
    connectionProjectPaths.value.set(connectionId, projectPath)
    if (dbType) {
      connectionDbTypes.value.set(connectionId, dbType)
    }
  }

  async function loadDatabases(connectionId: string) {
    if (loadingDatabases.value.has(connectionId)) return

    loadingDatabases.value.add(connectionId)
    error.value = null

    try {
      const connType = connectionTypes.value.get(connectionId) || 'global'
      const projectPath = connectionProjectPaths.value.get(connectionId)
      const dbType = connectionDbTypes.value.get(connectionId)
      console.log(`loadDatabases: connectionId=${connectionId}, connType=${connType}, dbType=${dbType}`)

      const cacheStatus = await getMetadataCacheStatus(
        connectionId,
        connType,
        'all',
        undefined,
        projectPath
      ).catch(() => ({ is_valid: false, last_sync: null, stats: null }))

      if (cacheStatus.is_valid && cacheStatus.stats && cacheStatus.stats.table_count > 0) {
        const databases = await loadDatabasesFromCache(connectionId, connType, projectPath)
        if (databases.length > 0) {
          const currentMap = connectionDatabases.value
          const newMap = new Map(currentMap)
          newMap.set(connectionId, databases)
          connectionDatabases.value = newMap
          return
        }
      }

      await loadDatabasesFromDb(connectionId)
      console.log(`loadDatabases 完成，当前 databases:`, connectionDatabases.value.get(connectionId))
    } catch (e) {
      error.value = e instanceof Error ? e.message : '加载数据库列表失败'
      console.error('加载数据库列表失败:', e)
      const currentMap = connectionDatabases.value
      const newMap = new Map(currentMap)
      newMap.set(connectionId, [{ name: 'default', schemas: [] }])
      connectionDatabases.value = newMap
    } finally {
      loadingDatabases.value.delete(connectionId)
    }
  }

  async function loadDatabasesFromCache(
    connectionId: string,
    connType: 'global' | 'project',
    projectPath?: string
  ): Promise<DatabaseNode[]> {
    const tables = await getTablesFromCache(connectionId, connType, 'all', undefined, projectPath)
      .catch(() => [])

    const dbMap = new Map<string, Set<string>>()
    tables.forEach(table => {
      const parts = table.name.split('.')
      if (parts.length >= 2) {
        const dbName = parts[0]
        const schemaName = parts[1]
        if (!dbMap.has(dbName)) {
          dbMap.set(dbName, new Set())
        }
        dbMap.get(dbName)!.add(schemaName)
      }
    })

    return Array.from(dbMap.entries()).map(([name, schemas]) => ({
      name,
      schemas: Array.from(schemas).map(s => ({ name: s, tables: [], views: [] }))
    }))
  }

  async function loadDatabasesFromDb(connectionId: string) {
    const dbType = connectionDbTypes.value.get(connectionId)?.toLowerCase() || ''
    console.log(`loadDatabasesFromDb: connectionId=${connectionId}, dbType=${dbType}`)
    
    let sql: string
    let databases: { name: string }[]

    if (dbType === 'sqlite') {
      // SQLite 只有一个 main 数据库
      console.log('SQLite: 创建 main 数据库')
      databases = [{ name: 'main' }]
    } else if (dbType === 'duckdb') {
      // DuckDB 使用 information_schema
      sql = `
        SELECT DISTINCT catalog_name as name
        FROM information_schema.schemata
        ORDER BY name
      `
      const result = await executeSqlService(connectionId, sql)
      const rows = result?.result?.rows || []
      databases = rows.map((row: Record<string, unknown>) => ({
        name: row.name as string
      }))
      if (databases.length === 0) {
        databases = [{ name: 'memory' }]
      }
    } else {
      // MySQL/PostgreSQL 使用 information_schema.tables
      sql = `
        SELECT DISTINCT table_catalog as name
        FROM information_schema.tables
        WHERE table_schema NOT IN ('information_schema', 'pg_catalog')
        ORDER BY name
      `
      const result = await executeSqlService(connectionId, sql)
      const rows = result?.result?.rows || []
      databases = rows.map((row: Record<string, unknown>) => ({
        name: row.name as string
      }))
      if (databases.length === 0) {
        databases = [{ name: 'default' }]
      }
    }

    const newDatabases = databases.map((db: { name: string }) => ({
      name: db.name,
      schemas: []
    }))
    
    // 创建新的 Map 来触发 Vue 响应式更新
    const currentMap = connectionDatabases.value
    const newMap = new Map(currentMap)
    newMap.set(connectionId, newDatabases)
    connectionDatabases.value = newMap
    
    setLastSyncTime(connectionId)
    
    console.log(`loadDatabasesFromDb 完成，databases:`, newDatabases)
  }

  async function loadSchemas(connectionId: string, dbName: string) {
    const key = `${connectionId}:${dbName}`
    if (loadingSchemas.value.has(key)) return

    loadingSchemas.value.add(key)
    error.value = null

    try {
      const connType = connectionTypes.value.get(connectionId) || 'global'
      const projectPath = connectionProjectPaths.value.get(connectionId)

      const cacheStatus = await getMetadataCacheStatus(
        connectionId,
        connType,
        dbName,
        undefined,
        projectPath
      ).catch(() => ({ is_valid: false, last_sync: null, stats: null }))

      if (cacheStatus.is_valid) {
        const schemas = await loadSchemasFromCache(connectionId, connType, dbName, projectPath)
        if (schemas.length > 0) {
          updateDatabaseSchemas(connectionId, dbName, schemas)
          return
        }
      }

      await loadSchemasFromDb(connectionId, dbName)
    } catch (e) {
      error.value = e instanceof Error ? e.message : '加载 Schema 列表失败'
      console.error('加载 Schema 列表失败:', e)
      const databases = connectionDatabases.value.get(connectionId)
      if (databases) {
        const db = databases.find((d: { name: string }) => d.name === dbName)
        if (db) {
          db.schemas = [{ name: 'public', tables: [], views: [] }]
        }
      }
    } finally {
      loadingSchemas.value.delete(key)
    }
  }

  async function loadSchemasFromCache(
    connectionId: string,
    connType: 'global' | 'project',
    dbName: string,
    projectPath?: string
  ): Promise<SchemaNode[]> {
    const tables = await getTablesFromCache(connectionId, connType, dbName, undefined, projectPath)
      .catch(() => [])

    const schemaSet = new Set<string>()
    tables.forEach(table => {
      if (table.schema_name) {
        schemaSet.add(table.schema_name)
      }
    })

    return Array.from(schemaSet).map(name => ({ name, tables: [], views: [] }))
  }

  async function loadSchemasFromDb(connectionId: string, dbName: string) {
    const dbType = connectionDbTypes.value.get(connectionId)?.toLowerCase() || ''
    console.log(`loadSchemasFromDb: connectionId=${connectionId}, dbName=${dbName}, dbType=${dbType}`)
    const safeDbName = escapeSql(dbName)
    
    let sql: string
    let schemas: { name: string }[]

    if (dbType === 'sqlite') {
      // SQLite 只有一个 main schema
      console.log('SQLite: 创建 main schema')
      schemas = [{ name: 'main' }]
    } else if (dbType === 'duckdb') {
      sql = `
        SELECT DISTINCT schema_name as name
        FROM information_schema.schemata
        WHERE catalog_name = '${safeDbName}'
        AND schema_name NOT IN ('information_schema', 'pg_catalog')
        ORDER BY name
      `
      const result = await executeSqlService(connectionId, sql)
      const rows = result?.result?.rows || []
      schemas = rows.map((row: Record<string, unknown>) => ({
        name: row.name as string
      }))
      if (schemas.length === 0) {
        schemas = [{ name: 'main' }]
      }
    } else {
      sql = `
        SELECT DISTINCT table_schema as name
        FROM information_schema.tables
        WHERE table_catalog = '${safeDbName}'
        AND table_schema NOT IN ('information_schema', 'pg_catalog')
        ORDER BY name
      `
      const result = await executeSqlService(connectionId, sql)
      const rows = result?.result?.rows || []
      schemas = rows.map((row: Record<string, unknown>) => ({
        name: row.name as string
      }))
      if (schemas.length === 0) {
        schemas = [{ name: 'public' }]
      }
    }

    updateDatabaseSchemas(connectionId, dbName, schemas.map((s: { name: string }) => ({
      name: s.name,
      tables: [],
      views: []
    })))
    
    setLastSyncTime(connectionId, dbName)
  }

  function updateDatabaseSchemas(connectionId: string, dbName: string, schemas: SchemaNode[]) {
    const databases = connectionDatabases.value.get(connectionId)
    if (databases) {
      const db = databases.find((d: { name: string }) => d.name === dbName)
      if (db) {
        db.schemas = schemas
        // 触发响应式更新
        const currentMap = connectionDatabases.value
        const newMap = new Map(currentMap)
        newMap.set(connectionId, [...databases])
        connectionDatabases.value = newMap
      }
    }
  }

  async function loadTables(connectionId: string, dbName: string, schemaName: string) {
    const key = `${connectionId}:${dbName}:${schemaName}`
    console.log(`loadTables 被调用: connectionId=${connectionId}, dbName=${dbName}, schemaName=${schemaName}`)
    if (loadingTables.value.has(key)) return

    loadingTables.value.add(key)
    error.value = null

    try {
      const connType = connectionTypes.value.get(connectionId) || 'global'
      const projectPath = connectionProjectPaths.value.get(connectionId)

      const cacheStatus = await getMetadataCacheStatus(
        connectionId,
        connType,
        dbName,
        schemaName,
        projectPath
      ).catch(() => ({ is_valid: false, last_sync: null, stats: null }))

      if (cacheStatus.is_valid && cacheStatus.stats && cacheStatus.stats.table_count > 0) {
        const tables = await getTablesFromCache(connectionId, connType, dbName, schemaName, projectPath)
        if (tables.length > 0) {
          updateSchemaTables(connectionId, dbName, schemaName, tables.map(t => ({
            name: t.name,
            type: 'table',
            columns: []
          })))
          return
        }
      }

      await loadTablesFromDb(connectionId, dbName, schemaName)
    } catch (e) {
      error.value = e instanceof Error ? e.message : '加载表列表失败'
      console.error('加载表列表失败:', e)
    } finally {
      loadingTables.value.delete(key)
    }
  }

  async function loadTablesFromDb(connectionId: string, dbName: string, schemaName: string) {
    const dbType = connectionDbTypes.value.get(connectionId)?.toLowerCase() || ''
    const safeDbName = escapeSql(dbName)
    const safeSchemaName = escapeSql(schemaName)
    
    let sql: string

    if (dbType === 'sqlite') {
      // SQLite 使用 sqlite_master 表
      sql = `
        SELECT name, type
        FROM sqlite_master
        WHERE type IN ('table', 'view')
        AND name NOT LIKE 'sqlite_%'
        ORDER BY name
      `
    } else if (dbType === 'duckdb') {
      sql = `
        SELECT table_name as name, table_type as type
        FROM information_schema.tables
        WHERE table_catalog = '${safeDbName}'
        AND table_schema = '${safeSchemaName}'
        ORDER BY table_name
      `
    } else {
      sql = `
        SELECT table_name as name, 'table' as type
        FROM information_schema.tables
        WHERE table_catalog = '${safeDbName}'
        AND table_schema = '${safeSchemaName}'
        ORDER BY table_name
      `
    }

    const result = await executeSqlService(connectionId, sql)
    console.log(`loadTablesFromDb 结果 (${dbType}):`, JSON.stringify(result, null, 2))
    const rows = result?.result?.rows || []
    // rows 来自 Tauri IPC，格式为 unknown[][]（数组的数组，按列顺序）
    // SELECT table_name as name, 'table' as type → [0]=name, [1]=type
    // SELECT name, type (sqlite)               → [0]=name, [1]=type
    console.log(`解析后的 rows (${dbType}):`, rows)
    const tables = rows.map((row: any) => ({
      name: typeof row === 'object' && !Array.isArray(row) ? (row as Record<string, unknown>).name as string : String(row[0] ?? ''),
      type: typeof row === 'object' && !Array.isArray(row) ? ((row as Record<string, unknown>).type as string) || 'table' : String(row[1] ?? 'table')
    }))
    console.log(`解析后的 tables (${dbType}):`, tables)

    updateSchemaTables(connectionId, dbName, schemaName, tables.map((t: { name: string; type: string }) => ({
      name: t.name,
      type: t.type || 'table',
      columns: []
    })))

    const connType = connectionTypes.value.get(connectionId) || 'global'
    const projectPath = connectionProjectPaths.value.get(connectionId)
    
    const tableInputs: TableInput[] = tables.map((t: { name: string; type: string }) => ({
      id: generateStableCacheId(connectionId, dbName, schemaName, t.name),
      name: t.name,
      comment: null
    }))

    if (tableInputs.length > 0) {
      try {
        await saveTablesBatchToCache(
          connectionId,
          connType,
          dbName,
          schemaName,
          tableInputs,
          projectPath
        )
      } catch (err) {
        console.warn('保存表缓存失败（非致命）:', err)
      }
    }
    
    setLastSyncTime(connectionId, dbName, schemaName)
  }

  function updateSchemaTables(
    connectionId: string,
    dbName: string,
    schemaName: string,
    tables: TableNode[]
  ) {
    console.log(`updateSchemaTables: connectionId=${connectionId}, dbName=${dbName}, schemaName=${schemaName}, tables=${tables.length}`)
    const databases = connectionDatabases.value.get(connectionId)
    console.log(`databases:`, databases)
    if (databases) {
      const db = databases.find((d: { name: string }) => d.name === dbName)
      console.log(`db:`, db)
      if (db) {
        // 无 Schema 的数据库（MySQL 等）：表直接存储在 DatabaseNode.tables 上
        if (db.schemas.length === 0) {
          db.tables = tables
          console.log(`db.tables 已更新 (no-schema mode):`, db.tables)
        } else {
          const schema = db.schemas.find((s: { name: string }) => s.name === schemaName)
          console.log(`schema:`, schema)
          if (schema) {
            schema.tables = tables
            console.log(`schema.tables 已更新:`, schema.tables)
          } else {
            console.warn(`未找到 schema: ${schemaName}，回退到 db.tables`)
            db.tables = tables
          }
        }
        // 触发响应式更新
        const currentMap = connectionDatabases.value
        const newMap = new Map(currentMap)
        newMap.set(connectionId, [...databases])
        connectionDatabases.value = newMap
      } else {
        console.warn(`未找到 database: ${dbName}`)
      }
    } else {
      console.warn(`未找到 connection: ${connectionId}`)
    }
  }

  async function loadViews(connectionId: string, dbName: string, schemaName: string) {
    const key = `${connectionId}:${dbName}:${schemaName}:views`
    if (loadingTables.value.has(key)) return

    loadingTables.value.add(key)
    error.value = null

    try {
      const dbType = connectionDbTypes.value.get(connectionId)?.toLowerCase() || ''
      const safeDbName = escapeSql(dbName)
      const safeSchemaName = escapeSql(schemaName)
      
      let sql: string
      let views: { name: string }[]

      if (dbType === 'sqlite') {
        // SQLite 的视图已经在 loadTablesFromDb 中通过 sqlite_master 获取了
        views = []
      } else if (dbType === 'duckdb') {
        sql = `
          SELECT table_name as name
          FROM information_schema.views
          WHERE table_catalog = '${safeDbName}'
          AND table_schema = '${safeSchemaName}'
          ORDER BY table_name
        `
        const result = await executeSqlService(connectionId, sql)
        const rows = result?.result?.rows || []
        views = rows.map((row: Record<string, unknown>) => ({
          name: row.name as string
        }))
      } else {
        sql = `
          SELECT table_name as name
          FROM information_schema.views
          WHERE table_catalog = '${safeDbName}'
          AND table_schema = '${safeSchemaName}'
          ORDER BY table_name
        `
        const result = await executeSqlService(connectionId, sql)
        const rows = result?.result?.rows || []
        views = rows.map((row: Record<string, unknown>) => ({
          name: row.name as string
        }))
      }

      const databases = connectionDatabases.value.get(connectionId)
      if (databases) {
        const db = databases.find((d: { name: string }) => d.name === dbName)
        if (db) {
          const schema = db.schemas.find((s: { name: string }) => s.name === schemaName)
          if (schema) {
            schema.views = views.map((view: { name: string }) => ({
              name: view.name,
              type: 'view',
              columns: []
            }))
          }
        }
      }
    } catch (e) {
      error.value = e instanceof Error ? e.message : '加载视图列表失败'
      console.error('加载视图列表失败:', e)
    } finally {
      loadingTables.value.delete(key)
    }
  }

  async function loadProcedures(connectionId: string, dbName: string, schemaName: string) {
    const key = `${connectionId}:${dbName}:${schemaName}:procedures`
    if (loadingTables.value.has(key)) return

    loadingTables.value.add(key)
    error.value = null

    try {
      const dbType = connectionDbTypes.value.get(connectionId)
      const safeDbName = escapeSql(dbName)
      const safeSchemaName = escapeSql(schemaName)
      let procedures: Array<{ name: string }> = []

      if (dbType === 'mysql') {
        const sql = `
          SELECT ROUTINE_NAME as name
          FROM INFORMATION_SCHEMA.ROUTINES
          WHERE ROUTINE_TYPE = 'PROCEDURE'
          AND ROUTINE_SCHEMA = '${safeSchemaName}'
          ORDER BY ROUTINE_NAME
        `
        const result = await executeSqlService(connectionId, sql)
        const rows = result?.result?.rows || []
        procedures = rows.map((row: Record<string, unknown>) => ({
          name: row.name as string
        }))
      } else if (dbType === 'postgres') {
        const sql = `
          SELECT proname as name
          FROM pg_proc
          JOIN pg_namespace ON pg_proc.pronamespace = pg_namespace.oid
          WHERE pg_namespace.nspname = '${safeSchemaName}'
          AND pg_proc.prokind = 'p'
          ORDER BY proname
        `
        const result = await executeSqlService(connectionId, sql)
        const rows = result?.result?.rows || []
        procedures = rows.map((row: Record<string, unknown>) => ({
          name: row.name as string
        }))
      } else if (dbType === 'sqlite') {
        procedures = []
      } else {
        const sql = `
          SELECT ROUTINE_NAME as name
          FROM INFORMATION_SCHEMA.ROUTINES
          WHERE ROUTINE_TYPE = 'PROCEDURE'
          AND ROUTINE_SCHEMA = '${safeSchemaName}'
          ORDER BY ROUTINE_NAME
        `
        const result = await executeSqlService(connectionId, sql)
        const rows = result?.result?.rows || []
        procedures = rows.map((row: Record<string, unknown>) => ({
          name: row.name as string
        }))
      }

      const databases = connectionDatabases.value.get(connectionId)
      if (databases) {
        const db = databases.find((d: { name: string }) => d.name === dbName)
        if (db) {
          const schema = db.schemas.find((s: { name: string }) => s.name === schemaName)
          if (schema) {
            schema.procedures = procedures.map((p: { name: string }) => ({
              name: p.name
            }))
          }
        }
      }
    } catch (e) {
      error.value = e instanceof Error ? e.message : '加载存储过程列表失败'
      console.error('加载存储过程列表失败:', e)
    } finally {
      loadingTables.value.delete(key)
    }
  }

  async function loadFunctions(connectionId: string, dbName: string, schemaName: string) {
    const key = `${connectionId}:${dbName}:${schemaName}:functions`
    if (loadingTables.value.has(key)) return

    loadingTables.value.add(key)
    error.value = null

    try {
      const dbType = connectionDbTypes.value.get(connectionId)
      const safeDbName = escapeSql(dbName)
      const safeSchemaName = escapeSql(schemaName)
      let functions: Array<{ name: string }> = []

      if (dbType === 'mysql') {
        const sql = `
          SELECT ROUTINE_NAME as name
          FROM INFORMATION_SCHEMA.ROUTINES
          WHERE ROUTINE_TYPE = 'FUNCTION'
          AND ROUTINE_SCHEMA = '${safeSchemaName}'
          ORDER BY ROUTINE_NAME
        `
        const result = await executeSqlService(connectionId, sql)
        const rows = result?.result?.rows || []
        functions = rows.map((row: Record<string, unknown>) => ({
          name: row.name as string
        }))
      } else if (dbType === 'postgres') {
        const sql = `
          SELECT proname as name
          FROM pg_proc
          JOIN pg_namespace ON pg_proc.pronamespace = pg_namespace.oid
          WHERE pg_namespace.nspname = '${safeSchemaName}'
          AND pg_proc.prokind IN ('f', 'a')
          ORDER BY proname
        `
        const result = await executeSqlService(connectionId, sql)
        const rows = result?.result?.rows || []
        functions = rows.map((row: Record<string, unknown>) => ({
          name: row.name as string
        }))
      } else if (dbType === 'sqlite') {
        functions = []
      } else {
        const sql = `
          SELECT ROUTINE_NAME as name
          FROM INFORMATION_SCHEMA.ROUTINES
          WHERE ROUTINE_TYPE = 'FUNCTION'
          AND ROUTINE_SCHEMA = '${safeSchemaName}'
          ORDER BY ROUTINE_NAME
        `
        const result = await executeSqlService(connectionId, sql)
        const rows = result?.result?.rows || []
        functions = rows.map((row: Record<string, unknown>) => ({
          name: row.name as string
        }))
      }

      const databases = connectionDatabases.value.get(connectionId)
      if (databases) {
        const db = databases.find((d: { name: string }) => d.name === dbName)
        if (db) {
          const schema = db.schemas.find((s: { name: string }) => s.name === schemaName)
          if (schema) {
            schema.functions = functions.map((f: { name: string }) => ({
              name: f.name
            }))
          }
        }
      }
    } catch (e) {
      error.value = e instanceof Error ? e.message : '加载函数列表失败'
      console.error('加载函数列表失败:', e)
    } finally {
      loadingTables.value.delete(key)
    }
  }

  async function loadColumns(connectionId: string, dbName: string, schemaName: string, tableName: string) {
    const key = `${connectionId}:${dbName}:${schemaName}:${tableName}`
    if (loadingColumns.value.has(key)) return

    loadingColumns.value.add(key)
    error.value = null

    try {
      const connType = connectionTypes.value.get(connectionId) || 'global'
      const projectPath = connectionProjectPaths.value.get(connectionId)

      const cacheStatus = await getMetadataCacheStatus(
        connectionId,
        connType,
        dbName,
        schemaName,
        projectPath
      ).catch(() => ({ is_valid: false, last_sync: null, stats: null }))

      if (cacheStatus.is_valid) {
        const columns = await getColumnsFromCache(
          connectionId,
          connType,
          dbName,
          schemaName,
          tableName,
          projectPath
        ).catch(() => [])

        if (columns.length > 0) {
          updateTableColumns(connectionId, dbName, schemaName, tableName, columns.map(c => ({
            name: c.name,
            dataType: c.data_type,
            nullable: c.is_nullable,
            defaultValue: undefined,
            isPrimaryKey: c.is_primary
          })))
          return
        }
      }

      await loadColumnsFromDb(connectionId, dbName, schemaName, tableName)
    } catch (e) {
      error.value = e instanceof Error ? e.message : '加载列信息失败'
      console.error('加载列信息失败:', e)
    } finally {
      loadingColumns.value.delete(key)
    }
  }

  async function loadColumnsFromDb(connectionId: string, dbName: string, schemaName: string, tableName: string) {
    const dbType = connectionDbTypes.value.get(connectionId)?.toLowerCase() || ''
    const safeDbName = escapeSql(dbName)
    const safeSchemaName = escapeSql(schemaName)
    const safeTableName = escapeSql(tableName)
    
    let sql: string
    let columns: {
      name: string
      data_type: string
      nullable: boolean
      default_value: string | undefined
      is_primary_key: boolean
    }[]

    if (dbType === 'sqlite') {
      // SQLite 使用 PRAGMA table_info
      sql = `PRAGMA table_info(${safeTableName})`
      const result = await executeSqlService(connectionId, sql)
      const rows = result?.result?.rows || []
      
      // SQLite PRAGMA table_info 返回的 pk 字段表示是否为主键
      // Tauri IPC: rows 为 unknown[][], 列顺序: [cid, name, type, notnull, dflt_value, pk]
      const tableRows = rows
      columns = tableRows.map((row: any) => {
        const r = Array.isArray(row) ? row : [row]
        return {
          name: String(r[1] ?? ''),
          data_type: String(r[2] ?? ''),
          nullable: !Number(r[3] ?? 0),
          default_value: r[4] != null ? String(r[4]) : undefined,
          is_primary_key: Number(r[5] ?? 0) === 1
        }
      })
    } else if (dbType === 'duckdb') {
      sql = `
        SELECT 
          column_name as name,
          data_type,
          is_nullable = 'YES' as nullable,
          column_default as default_value,
          column_name IN (
            SELECT column_name
            FROM information_schema.key_column_usage
            WHERE table_catalog = '${safeDbName}'
            AND table_schema = '${safeSchemaName}'
            AND table_name = '${safeTableName}'
          ) as is_primary_key
        FROM information_schema.columns
        WHERE table_catalog = '${safeDbName}'
        AND table_schema = '${safeSchemaName}'
        AND table_name = '${safeTableName}'
        ORDER BY ordinal_position
      `
      const result = await executeSqlService(connectionId, sql)
      const rows = result?.result?.rows || []
      // Tauri IPC: rows 为 unknown[][], 列顺序: [name, data_type, nullable, default_value, is_primary_key]
      columns = rows.map((row: any) => {
        const r = Array.isArray(row) ? row : [row]
        return {
          name: String(r[0] ?? ''),
          data_type: String(r[1] ?? ''),
          nullable: r[2] === true || String(r[2]).toLowerCase() === 'yes',
          default_value: r[3] != null ? String(r[3]) : undefined,
          is_primary_key: r[4] === true || String(r[4]).toLowerCase() === 'true'
        }
      })
    } else {
      // MySQL/PostgreSQL
      sql = `
        SELECT 
          column_name as name,
          data_type,
          is_nullable = 'YES' as nullable,
          column_default as default_value,
          column_name IN (
            SELECT column_name
            FROM information_schema.key_column_usage
            WHERE table_catalog = '${safeDbName}'
            AND table_schema = '${safeSchemaName}'
            AND table_name = '${safeTableName}'
            AND constraint_name LIKE '%_pkey'
          ) as is_primary_key
        FROM information_schema.columns
        WHERE table_catalog = '${safeDbName}'
        AND table_schema = '${safeSchemaName}'
        AND table_name = '${safeTableName}'
        ORDER BY ordinal_position
      `
      const result = await executeSqlService(connectionId, sql)
      const rows = result?.result?.rows || []
      // Tauri IPC: rows 为 unknown[][], 列顺序: [name, data_type, nullable, default_value, is_primary_key]
      columns = rows.map((row: any) => {
        const r = Array.isArray(row) ? row : [row]
        return {
          name: String(r[0] ?? ''),
          data_type: String(r[1] ?? ''),
          nullable: r[2] === true || String(r[2]).toLowerCase() === 'yes' || String(r[2]) === '1',
          default_value: r[3] != null ? String(r[3]) : undefined,
          is_primary_key: r[4] === true || String(r[4]).toLowerCase() === 'true' || String(r[4]) === '1'
        }
      })
    }

    const connType = connectionTypes.value.get(connectionId) || 'global'
    const projectPath = connectionProjectPaths.value.get(connectionId)

    const columnInputs: ColumnInput[] = columns.map((col: {
      name: string
      data_type: string
      nullable: boolean
      default_value: string | undefined
      is_primary_key: boolean
    }) => ({
      id: generateStableCacheId(connectionId, dbName, schemaName, tableName, col.name),
      name: col.name,
      data_type: col.data_type,
      is_nullable: col.nullable,
      is_primary: col.is_primary_key,
      is_unique: false
    }))

    if (columnInputs.length > 0) {
      try {
        await saveColumnsBatchToCache(
          connectionId,
          connType,
          dbName,
          schemaName,
          tableName,
          columnInputs,
          projectPath
        )
      } catch (err) {
        console.warn('保存列缓存失败（非致命）:', err)
      }
    }

    updateTableColumns(connectionId, dbName, schemaName, tableName, columns.map((col: {
      name: string
      data_type: string
      nullable: boolean
      default_value: string | undefined
      is_primary_key: boolean
    }) => ({
      name: col.name,
      dataType: col.data_type,
      nullable: col.nullable,
      defaultValue: col.default_value ?? undefined,
      isPrimaryKey: col.is_primary_key
    })))
  }

  function updateTableColumns(
    connectionId: string,
    dbName: string,
    schemaName: string,
    tableName: string,
    columns: ColumnNode[]
  ) {
    const databases = connectionDatabases.value.get(connectionId)
    if (databases) {
      const db = databases.find((d: { name: string }) => d.name === dbName)
      if (db) {
        const schema = db.schemas.find((s: { name: string }) => s.name === schemaName)
        if (schema) {
          const table = schema.tables.find((t: { name: string }) => t.name === tableName)
          if (table) {
            table.columns = columns
          } else {
            const view = schema.views.find((v: { name: string }) => v.name === tableName)
            if (view) {
              view.columns = columns
            }
          }
        }
      }
    }
  }

  async function refreshMetadata(connectionId: string, dbName?: string, schemaName?: string) {
    const connType = connectionTypes.value.get(connectionId) || 'global'
    const projectPath = connectionProjectPaths.value.get(connectionId)

    await clearMetadataCache(
      connectionId,
      connType,
      dbName || 'all',
      schemaName,
      projectPath
    ).catch(() => {})

    clearCache(connectionId)

    if (dbName) {
      await loadDatabases(connectionId)
      await loadSchemas(connectionId, dbName)
      if (schemaName) {
        await loadTables(connectionId, dbName, schemaName)
      }
    } else {
      await loadDatabases(connectionId)
    }
  }

  function setSelectedObject(object: SelectedObject | null) {
    selectedObject.value = object
  }

  function clearCache(connectionId?: string) {
    if (connectionId) {
      connectionDatabases.value.delete(connectionId)
    } else {
      connectionDatabases.value.clear()
    }
  }

  function clearError() {
    error.value = null
  }

  function getConnectionType(connectionId: string): 'global' | 'project' | undefined {
    return connectionTypes.value.get(connectionId)
  }

  function getProjectPath(connectionId: string): string | undefined {
    return connectionProjectPaths.value.get(connectionId)
  }

  function searchObjects(query: string): SearchResult[] {
    if (!query || query.trim().length === 0) return []

    const results: SearchResult[] = []
    const lowerQuery = query.toLowerCase()

    connectionDatabases.value.forEach((databases, connectionId) => {
      databases.forEach(db => {
        if (db.name.toLowerCase().includes(lowerQuery)) {
          results.push({
            connectionId,
            type: 'database',
            name: db.name,
            path: db.name,
            matchType: 'name'
          })
        }

        db.schemas.forEach(schema => {
          if (schema.name.toLowerCase().includes(lowerQuery)) {
            results.push({
              connectionId,
              type: 'schema',
              name: schema.name,
              path: `${db.name}.${schema.name}`,
              matchType: 'name'
            })
          }

          schema.tables.forEach(table => {
            if (table.name.toLowerCase().includes(lowerQuery)) {
              results.push({
                connectionId,
                type: 'table',
                name: table.name,
                path: `${db.name}.${schema.name}.${table.name}`,
                matchType: 'name'
              })
            }

            table.columns.forEach(col => {
              if (col.name.toLowerCase().includes(lowerQuery)) {
                results.push({
                  connectionId,
                  type: 'column',
                  name: `${table.name}.${col.name}`,
                  path: `${db.name}.${schema.name}.${table.name}.${col.name}`,
                  matchType: 'name',
                  parentTable: table.name
                })
              }
              if (col.dataType.toLowerCase().includes(lowerQuery)) {
                results.push({
                  connectionId,
                  type: 'column',
                  name: `${table.name}.${col.name}`,
                  path: `${db.name}.${schema.name}.${table.name}.${col.name}`,
                  matchType: 'type',
                  parentTable: table.name
                })
              }
            })
          })

          schema.views.forEach(view => {
            if (view.name.toLowerCase().includes(lowerQuery)) {
              results.push({
                connectionId,
                type: 'view',
                name: view.name,
                path: `${db.name}.${schema.name}.${view.name}`,
                matchType: 'name'
              })
            }

            view.columns.forEach(col => {
              if (col.name.toLowerCase().includes(lowerQuery)) {
                results.push({
                  connectionId,
                  type: 'column',
                  name: `${view.name}.${col.name}`,
                  path: `${db.name}.${schema.name}.${view.name}.${col.name}`,
                  matchType: 'name',
                  parentTable: view.name
                })
              }
            })
          })
        })
      })
    })

    return results
  }

  async function disconnectConnection(connectionId: string) {
    await closeConnection(connectionId).catch(() => {})
    clearCache(connectionId)
  }

  function getDatabaseSchemas(connectionId: string, dbName: string): SchemaNode[] {
    const databases = connectionDatabases.value.get(connectionId)
    if (!databases) return []
    const db = databases.find(d => d.name === dbName)
    if (!db) return []
    return db.schemas || []
  }

  function getSchemaTables(connectionId: string, dbName: string, schemaName: string): TableNode[] {
    const databases = connectionDatabases.value.get(connectionId)
    if (!databases) return []
    const db = databases.find(d => d.name === dbName)
    if (!db) return []
    // 无 Schema 的数据库：返回 db.tables
    if (db.schemas.length === 0) return db.tables || []
    const schema = db.schemas.find(s => s.name === schemaName)
    if (!schema) return db.tables || []
    return schema.tables || []
  }

  function getSchemaViews(connectionId: string, dbName: string, schemaName: string): ViewNode[] {
    const databases = connectionDatabases.value.get(connectionId)
    if (!databases) return []
    const db = databases.find(d => d.name === dbName)
    if (!db) return []
    const schema = db.schemas.find(s => s.name === schemaName)
    if (!schema) return []
    return schema.views || []
  }

  function getDbType(connectionId: string): string {
    return connectionDbTypes.value.get(connectionId)?.toLowerCase() || ''
  }

  async function executeSql(connectionId: string, _dbName: string, sql: string): Promise<any> {
    return await executeSqlService(connectionId, sql)
  }

  function expandToNode(_nodeKey: string): void {
    // Tree expansion managed by database-navigator component
  }

  function selectNode(nodeKey: string): void {
    const parts = nodeKey.split('_')
    if (parts.length >= 3) {
      setSelectedObject({
        name: parts[parts.length - 1],
        kind: 'table',
        connectionId: parts[1],
      } as SelectedObject)
    }
  }

  return {
    connectionDatabases,
    selectedObject,
    error,
    getDatabases,
    getDatabaseSchemas,
    getSchemaTables,
    getSchemaViews,
    isLoadingDatabases,
    isLoadingSchemas,
    isLoadingTables,
    setConnectionInfo,
    getDbType,
    getConnectionType,
    getProjectPath,
    loadDatabases,
    loadSchemas,
    loadTables,
    loadViews,
    loadProcedures,
    loadFunctions,
    loadColumns,
    refreshMetadata,
    searchObjects,
    setSelectedObject,
    clearCache,
    clearError,
    disconnectConnection,
    executeSql,
    expandToNode,
    selectNode,
    getLastSyncTime,
    setLastSyncTime,
    setSyncMode,
    getSyncMode
  }
})

interface DatabaseNode {
  name: string
  schemas: SchemaNode[]
  /** 无 Schema 的数据库（如 MySQL）直接在此存储表 */
  tables?: TableNode[]
}

interface SchemaNode {
  name: string
  tables: TableNode[]
  views: ViewNode[]
}

interface TableNode {
  name: string
  type: string
  columns: ColumnNode[]
  indexes?: IndexNode[]
  constraints?: ConstraintNode[]
}

interface IndexNode {
  name: string
  columns: string[]
  isUnique: boolean
  isPrimary: boolean
}

interface ConstraintNode {
  name: string
  type: 'PRIMARY KEY' | 'FOREIGN KEY' | 'UNIQUE' | 'CHECK'
  columns: string[]
}

interface ViewNode {
  name: string
  type: string
  columns: ColumnNode[]
}

interface ColumnNode {
  name: string
  dataType: string
  nullable?: boolean
  defaultValue?: string
  isPrimaryKey?: boolean
}

interface SelectedObject {
  name: string
  kind: 'database' | 'schema' | 'table' | 'view' | 'column'
  database?: string
  schema?: string
  table?: string
  connectionId: string
  [key: string]: unknown
}

interface SearchResult {
  connectionId: string
  type: 'database' | 'schema' | 'table' | 'view' | 'column'
  name: string
  path: string
  matchType: 'name' | 'type'
  parentTable?: string
}
