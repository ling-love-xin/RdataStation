import { defineStore } from 'pinia'
import { ref, computed } from 'vue'

import {
  closeConnection,
  executeSql as executeSqlService,
} from '@/extensions/builtin/connection/ui/services/connection'

import * as databaseApi from '../api/database-api'
import {
  clearMetadataCache,
  getColumnsFromCache,
  getMetadataCacheStatus,
  getTablesFromCache,
  saveTablesBatchToCache,
  saveColumnsBatchToCache,
} from '../services/metadata-cache-service'

import type { IntrospectionLevel } from '../api/database-api'
import type { TableInput, ColumnInput } from '../services/metadata-cache-service'

export const useDatabaseNavigatorStore = defineStore('databaseNavigator', () => {
  const connectionCatalogs = ref<Map<string, CatalogNode[]>>(new Map())
  const selectedObject = ref<SelectedObject | null>(null)
  const loadingCatalogs = ref<Set<string>>(new Set())
  const loadingSchemas = ref<Set<string>>(new Set())
  const loadingTables = ref<Set<string>>(new Set())
  const loadingColumns = ref<Set<string>>(new Set())
  const error = ref<string | null>(null)
  const connectionTypes = ref<Map<string, 'global' | 'project'>>(new Map())
  const connectionProjectPaths = ref<Map<string, string | undefined>>(new Map())
  const introspectionLevels = ref<Map<string, IntrospectionLevel>>(new Map())
  const connectionDbTypes = ref<Map<string, string>>(new Map())

  const lastSyncTimes = ref<Map<string, number>>(new Map())
  const syncModes = ref<Map<string, 'full' | 'incremental'>>(new Map())

  function getCatalogs(connectionId: string): CatalogNode[] {
    return connectionCatalogs.value.get(connectionId) || []
  }

  function getLastSyncTime(
    connectionId: string,
    catalogName?: string,
    schemaName?: string
  ): number {
    const key = catalogName
      ? schemaName
        ? `${connectionId}:${catalogName}:${schemaName}`
        : `${connectionId}:${catalogName}`
      : connectionId
    return lastSyncTimes.value.get(key) || 0
  }

  function setLastSyncTime(connectionId: string, catalogName?: string, schemaName?: string) {
    const key = catalogName
      ? schemaName
        ? `${connectionId}:${catalogName}:${schemaName}`
        : `${connectionId}:${catalogName}`
      : connectionId
    lastSyncTimes.value.set(key, Date.now())
  }

  function setSyncMode(connectionId: string, mode: 'full' | 'incremental') {
    syncModes.value.set(connectionId, mode)
  }

  function getSyncMode(connectionId: string): 'full' | 'incremental' {
    return syncModes.value.get(connectionId) || 'incremental'
  }

  const isLoadingCatalogs = computed(() => {
    return (connectionId: string): boolean => {
      return loadingCatalogs.value.has(connectionId)
    }
  })

  const isLoadingSchemas = computed(() => {
    return (connectionId: string, catalogName: string): boolean => {
      return loadingSchemas.value.has(`${connectionId}:${catalogName}`)
    }
  })

  const isLoadingTables = computed(() => {
    return (connectionId: string, catalogName: string, schemaName: string): boolean => {
      return loadingTables.value.has(`${connectionId}:${catalogName}:${schemaName}`)
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

  async function loadCatalogs(connectionId: string) {
    if (loadingCatalogs.value.has(connectionId)) return

    loadingCatalogs.value.add(connectionId)
    error.value = null

    try {
      const connType = connectionTypes.value.get(connectionId) || 'global'
      const projectPath = connectionProjectPaths.value.get(connectionId)
      const dbType = connectionDbTypes.value.get(connectionId)

      const cacheStatus = await getMetadataCacheStatus(
        connectionId,
        connType,
        'all',
        undefined,
        projectPath
      ).catch(() => ({ is_valid: false, last_sync: null, stats: null }))

      if (cacheStatus.is_valid && cacheStatus.stats && cacheStatus.stats.table_count > 0) {
        const catalogs = await loadCatalogsFromCache(connectionId, connType, projectPath)
        if (catalogs.length > 0) {
          const currentMap = connectionCatalogs.value
          const newMap = new Map(currentMap)
          newMap.set(connectionId, catalogs)
          connectionCatalogs.value = newMap
          return
        }
      }

      await loadCatalogsFromDb(connectionId)
    } catch (e) {
      error.value = e instanceof Error ? e.message : '加载 Catalog 列表失败'
      console.error('加载 Catalog 列表失败:', e)
      throw e
    } finally {
      loadingCatalogs.value.delete(connectionId)
    }
  }

  async function loadCatalogsFromCache(
    connectionId: string,
    connType: 'global' | 'project',
    projectPath?: string
  ): Promise<CatalogNode[]> {
    const tables = await getTablesFromCache(
      connectionId,
      connType,
      'all',
      undefined,
      projectPath
    ).catch(() => [])

    const dbMap = new Map<string, Set<string>>()
    tables.forEach(table => {
      const parts = table.name.split('.')
      if (parts.length >= 2) {
        const catalogName = parts[0]
        const schemaName = parts[1]
        if (!dbMap.has(catalogName)) {
          dbMap.set(catalogName, new Set())
        }
        dbMap.get(catalogName)!.add(schemaName)
      }
    })

    return Array.from(dbMap.entries()).map(([name, schemas]) => ({
      name,
      schemas: Array.from(schemas).map(s => ({ name: s, tables: [], views: [] })),
    }))
  }

  /**
   * 静默从 L2 缓存加载 Catalogs（离线模式专用）
   * 不访问数据库，只从本地 SQLite 元数据缓存读取，失败时静默处理
   */
  async function loadCatalogsFromCacheSilent(connectionId: string): Promise<boolean> {
    // 如果已经有缓存数据，不重复加载
    const existing = connectionCatalogs.value.get(connectionId)
    if (existing && existing.length > 0) return true

    try {
      const connType = connectionTypes.value.get(connectionId) || 'global'
      const projectPath = connectionProjectPaths.value.get(connectionId)
      const catalogs = await loadCatalogsFromCache(connectionId, connType, projectPath)

      if (catalogs.length > 0) {
        const currentMap = connectionCatalogs.value
        const newMap = new Map(currentMap)
        newMap.set(connectionId, catalogs)
        connectionCatalogs.value = newMap
        return true
      }
    } catch {
      // 离线模式下静默处理缓存加载失败
    }
    return false
  }

  async function loadCatalogsFromDb(connectionId: string) {
    const connType = connectionTypes.value.get(connectionId) || 'global'
    const projectPath = connectionProjectPaths.value.get(connectionId)

    const catalogMetas = await databaseApi.loadCatalogs(connectionId, connType, projectPath)

    let catalogs: { name: string }[] = catalogMetas.map(d => ({ name: d.name }))

    if (catalogs.length === 0) {
      catalogs = [{ name: 'default' }]
    }

    const newCatalogs = catalogs.map((cat: { name: string }) => ({
      name: cat.name,
      schemas: [],
    }))

    const currentMap = connectionCatalogs.value
    const newMap = new Map(currentMap)
    newMap.set(connectionId, newCatalogs)
    connectionCatalogs.value = newMap

    setLastSyncTime(connectionId)
  }

  async function loadSchemas(connectionId: string, catalogName: string) {
    const key = `${connectionId}:${catalogName}`
    if (loadingSchemas.value.has(key)) return

    loadingSchemas.value.add(key)
    error.value = null

    try {
      const connType = connectionTypes.value.get(connectionId) || 'global'
      const projectPath = connectionProjectPaths.value.get(connectionId)

      const cacheStatus = await getMetadataCacheStatus(
        connectionId,
        connType,
        catalogName,
        undefined,
        projectPath
      ).catch(() => ({ is_valid: false, last_sync: null, stats: null }))

      if (cacheStatus.is_valid) {
        const schemas = await loadSchemasFromCache(connectionId, connType, catalogName, projectPath)
        if (schemas.length > 0) {
          updateCatalogSchemas(connectionId, catalogName, schemas)
          return
        }
      }

      await loadSchemasFromDb(connectionId, catalogName)
    } catch (e) {
      error.value = e instanceof Error ? e.message : '加载 Schema 列表失败'
      console.error('加载 Schema 列表失败:', e)
      const catalogs = connectionCatalogs.value.get(connectionId)
      if (catalogs) {
        const cat = catalogs.find((c: { name: string }) => c.name === catalogName)
        if (cat) {
          cat.schemas = [{ name: catalogName, tables: [], views: [] }]
        }
      }
    } finally {
      loadingSchemas.value.delete(key)
    }
  }

  async function loadSchemasFromCache(
    connectionId: string,
    connType: 'global' | 'project',
    catalogName: string,
    projectPath?: string
  ): Promise<SchemaNode[]> {
    const tables = await getTablesFromCache(
      connectionId,
      connType,
      catalogName,
      undefined,
      projectPath
    ).catch(() => [])

    const schemaSet = new Set<string>()
    tables.forEach(table => {
      if (table.schema_name) {
        schemaSet.add(table.schema_name)
      }
    })

    return Array.from(schemaSet).map(name => ({ name, tables: [], views: [] }))
  }

  async function loadSchemasFromDb(connectionId: string, catalogName: string) {
    const connType = connectionTypes.value.get(connectionId) || 'global'
    const projectPath = connectionProjectPaths.value.get(connectionId)

    const schemaMetas = await databaseApi.loadSchemas(
      connectionId,
      catalogName,
      connType,
      projectPath
    )

    let schemas: { name: string }[] = schemaMetas.map(s => ({ name: s.name }))

    if (schemas.length === 0) {
      schemas = [{ name: catalogName }]
    }

    updateCatalogSchemas(
      connectionId,
      catalogName,
      schemas.map((s: { name: string }) => ({
        name: s.name,
        tables: [],
        views: [],
      }))
    )

    setLastSyncTime(connectionId, catalogName)
  }

  function updateCatalogSchemas(connectionId: string, catalogName: string, schemas: SchemaNode[]) {
    const catalogs = connectionCatalogs.value.get(connectionId)
    if (catalogs) {
      const cat = catalogs.find((c: { name: string }) => c.name === catalogName)
      if (cat) {
        cat.schemas = schemas
        const currentMap = connectionCatalogs.value
        const newMap = new Map(currentMap)
        newMap.set(connectionId, [...catalogs])
        connectionCatalogs.value = newMap
      }
    }
  }

  async function loadTables(connectionId: string, catalogName: string, schemaName: string) {
    const key = `${connectionId}:${catalogName}:${schemaName}`
    if (loadingTables.value.has(key)) return

    loadingTables.value.add(key)
    error.value = null

    try {
      const connType = connectionTypes.value.get(connectionId) || 'global'
      const projectPath = connectionProjectPaths.value.get(connectionId)

      const cacheStatus = await getMetadataCacheStatus(
        connectionId,
        connType,
        catalogName,
        schemaName,
        projectPath
      ).catch(() => ({ is_valid: false, last_sync: null, stats: null }))

      if (cacheStatus.is_valid && cacheStatus.stats && cacheStatus.stats.table_count > 0) {
        const tables = await getTablesFromCache(
          connectionId,
          connType,
          catalogName,
          schemaName,
          projectPath
        )
        if (tables.length > 0) {
          updateSchemaTables(
            connectionId,
            catalogName,
            schemaName,
            tables.map(t => ({
              name: t.name,
              type: 'table',
              columns: [],
              rowCount: t.rowCountEstimate ?? null,
              dataLength: t.dataLength ?? null,
              indexLength: t.indexLength ?? null,
            }))
          )
          return
        }
      }

      await loadTablesFromDb(connectionId, catalogName, schemaName)
    } catch (e) {
      error.value = e instanceof Error ? e.message : '加载表列表失败'
      console.error('加载表列表失败:', e)
    } finally {
      loadingTables.value.delete(key)
    }
  }

  async function loadTablesFromDb(connectionId: string, catalogName: string, schemaName: string) {
    const connType = connectionTypes.value.get(connectionId) || 'global'
    const projectPath = connectionProjectPaths.value.get(connectionId)

    try {
      const [tableMetas, viewMetas] = await Promise.all([
        databaseApi.loadTables(connectionId, catalogName, schemaName, connType, projectPath),
        databaseApi.loadViews(connectionId, catalogName, schemaName, connType, projectPath),
      ])

      const allTables = tableMetas.map(t => ({ name: t.name, type: t.type || 'table' }))
      const allViews = viewMetas.map(v => ({ name: v.name, type: 'view' }))

      const merged = [...allTables, ...allViews]

      updateSchemaTables(
        connectionId,
        catalogName,
        schemaName,
        merged.map(t => ({ name: t.name, type: t.type, columns: [] }))
      )

      const tableInputs: TableInput[] = merged.map(t => ({
        id: `${connectionId}:${catalogName}:${schemaName}:${t.name}`,
        name: t.name,
        comment: undefined,
      }))

      if (tableInputs.length > 0) {
        try {
          await saveTablesBatchToCache(
            connectionId,
            connType,
            projectPath,
            catalogName,
            schemaName,
            tableInputs
          )
        } catch (err) {
          console.warn('保存表缓存失败（非致命）:', err)
        }
      }

      setLastSyncTime(connectionId, catalogName, schemaName)
    } catch (err) {
      console.error(`loadTablesFromDb 失败:`, err)
    }
  }

  function updateSchemaTables(
    connectionId: string,
    catalogName: string,
    schemaName: string,
    tables: TableNode[]
  ) {
    const catalogs = connectionCatalogs.value.get(connectionId)
    if (catalogs) {
      const cat = catalogs.find((c: { name: string }) => c.name === catalogName)
      if (cat) {
        // 无 Schema 的数据库（MySQL 等）：表直接存储在 CatalogNode.tables 上
        if (cat.schemas.length === 0) {
          cat.tables = tables
        } else {
          const schema = cat.schemas.find((s: { name: string }) => s.name === schemaName)
          if (schema) {
            schema.tables = tables
            computeSchemaStats(schema)
          } else {
            console.warn(`未找到 schema: ${schemaName}，回退到 catalog.tables`)
            cat.tables = tables
          }
        }
        const currentMap = connectionCatalogs.value
        const newMap = new Map(currentMap)
        newMap.set(connectionId, [...catalogs])
        connectionCatalogs.value = newMap
      } else {
        console.warn(`未找到 catalog: ${catalogName}`)
      }
    } else {
      console.warn(`未找到 connection: ${connectionId}`)
    }
  }

  async function loadViews(connectionId: string, catalogName: string, schemaName: string) {
    const key = `${connectionId}:${catalogName}:${schemaName}:views`
    if (loadingTables.value.has(key)) return

    loadingTables.value.add(key)
    error.value = null

    try {
      await loadTables(connectionId, catalogName, schemaName)
    } catch (e) {
      error.value = e instanceof Error ? e.message : '加载视图列表失败'
      console.error('加载视图列表失败:', e)
    } finally {
      loadingTables.value.delete(key)
    }
  }

  async function loadProcedures(connectionId: string, catalogName: string, schemaName: string) {
    const key = `${connectionId}:${catalogName}:${schemaName}:procedures`
    if (loadingTables.value.has(key)) return

    loadingTables.value.add(key)
    error.value = null

    try {
      const connType = connectionTypes.value.get(connectionId) || 'global'
      const projectPath = connectionProjectPaths.value.get(connectionId)

      const procedureMetas = await databaseApi.loadProcedures(connectionId, catalogName, schemaName, connType, projectPath)
      const procedures = procedureMetas.map((p: { name: string }) => ({ name: p.name }))

      const catalogs = connectionCatalogs.value.get(connectionId)
      if (catalogs) {
        const cat = catalogs.find((c: { name: string }) => c.name === catalogName)
        if (cat) {
          const schema = cat.schemas.find((s: { name: string }) => s.name === schemaName)
          if (schema) {
            schema.procedures = procedures
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

  async function loadFunctions(connectionId: string, catalogName: string, schemaName: string) {
    const key = `${connectionId}:${catalogName}:${schemaName}:functions`
    if (loadingTables.value.has(key)) return

    loadingTables.value.add(key)
    error.value = null

    try {
      const connType = connectionTypes.value.get(connectionId) || 'global'
      const projectPath = connectionProjectPaths.value.get(connectionId)

      const functionMetas = await databaseApi.loadFunctions(connectionId, catalogName, schemaName, connType, projectPath)
      const functions = functionMetas.map((f: { name: string }) => ({ name: f.name }))

      const catalogs = connectionCatalogs.value.get(connectionId)
      if (catalogs) {
        const cat = catalogs.find((c: { name: string }) => c.name === catalogName)
        if (cat) {
          const schema = cat.schemas.find((s: { name: string }) => s.name === schemaName)
          if (schema) {
            schema.functions = functions
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

  async function loadIndexes(
    connectionId: string,
    catalogName: string,
    schemaName: string,
    tableName: string
  ) {
    const key = `${connectionId}:${catalogName}:${schemaName}:${tableName}:indexes`
    if (loadingTables.value.has(key)) return

    loadingTables.value.add(key)
    error.value = null

    try {
      const connType = connectionTypes.value.get(connectionId) || 'global'
      const projectPath = connectionProjectPaths.value.get(connectionId)

      const indexMetas = await databaseApi.loadIndexes(
        connectionId,
        catalogName,
        schemaName,
        tableName,
        connType,
        projectPath
      )
      const indexes: IndexNode[] = indexMetas.map((idx: { name: string; columnNames: string[]; isUnique: boolean; isPrimary: boolean }) => ({
        name: idx.name,
        columns: idx.columnNames || [],
        isUnique: idx.isUnique || false,
        isPrimary: idx.isPrimary || false,
      }))

      // 写入 SchemaNode 的 table.indexes
      const catalogs = connectionCatalogs.value.get(connectionId)
      if (!catalogs) return
      const cat = catalogs.find((c: { name: string }) => c.name === catalogName)
      if (!cat) return
      const schema = cat.schemas.find((s: { name: string }) => s.name === schemaName)
      if (!schema) return
      const table = schema.tables.find((t: { name: string }) => t.name === tableName)
      if (table) {
        table.indexes = indexes
      }
    } catch (e) {
      error.value = e instanceof Error ? e.message : '加载索引列表失败'
      console.error('加载索引列表失败:', e)
    } finally {
      loadingTables.value.delete(key)
    }
  }

  async function loadConstraints(
    connectionId: string,
    catalogName: string,
    schemaName: string,
    tableName: string
  ) {
    const key = `${connectionId}:${catalogName}:${schemaName}:${tableName}:constraints`
    if (loadingTables.value.has(key)) return

    loadingTables.value.add(key)
    error.value = null

    try {
      const connType = connectionTypes.value.get(connectionId) || 'global'
      const projectPath = connectionProjectPaths.value.get(connectionId)

      const constraintMetas = await databaseApi.loadConstraints(
        connectionId,
        catalogName,
        schemaName,
        tableName,
        connType,
        projectPath
      )
      const constraints: ConstraintNode[] = constraintMetas.map((c: { name: string; constraintType: string; columnNames: string[] }) => ({
        name: c.name,
        type: c.constraintType as ConstraintNode['type'],
        columns: c.columnNames || [],
      }))

      const catalogs = connectionCatalogs.value.get(connectionId)
      if (!catalogs) return
      const cat = catalogs.find((c: { name: string }) => c.name === catalogName)
      if (!cat) return
      const schema = cat.schemas.find((s: { name: string }) => s.name === schemaName)
      if (!schema) return
      const table = schema.tables.find((t: { name: string }) => t.name === tableName)
      if (table) {
        table.constraints = constraints
      }
    } catch (e) {
      error.value = e instanceof Error ? e.message : '加载约束列表失败'
      console.error('加载约束列表失败:', e)
    } finally {
      loadingTables.value.delete(key)
    }
  }

  async function loadSequences(
    connectionId: string,
    catalogName: string,
    schemaName: string
  ) {
    const key = `${connectionId}:${catalogName}:${schemaName}:sequences`
    if (loadingTables.value.has(key)) return

    loadingTables.value.add(key)
    error.value = null

    try {
      const connType = connectionTypes.value.get(connectionId) || 'global'
      const projectPath = connectionProjectPaths.value.get(connectionId)

      const sequenceMetas = await databaseApi.loadSequences(
        connectionId,
        catalogName,
        schemaName,
        connType,
        projectPath
      )
      const sequences: SequenceNode[] = sequenceMetas.map((s: { name: string }) => ({
        name: s.name,
      }))

      const catalogs = connectionCatalogs.value.get(connectionId)
      if (!catalogs) return
      const cat = catalogs.find((c: { name: string }) => c.name === catalogName)
      if (!cat) return
      const schema = cat.schemas.find((s: { name: string }) => s.name === schemaName)
      if (schema) {
        schema.sequences = sequences
      }
    } catch (e) {
      error.value = e instanceof Error ? e.message : '加载序列列表失败'
      console.error('加载序列列表失败:', e)
    } finally {
      loadingTables.value.delete(key)
    }
  }

  async function loadTriggers(
    connectionId: string,
    catalogName: string,
    schemaName: string
  ) {
    const key = `${connectionId}:${catalogName}:${schemaName}:triggers`
    if (loadingTables.value.has(key)) return

    loadingTables.value.add(key)
    error.value = null

    try {
      const connType = connectionTypes.value.get(connectionId) || 'global'
      const projectPath = connectionProjectPaths.value.get(connectionId)

      const triggerMetas = await databaseApi.loadTriggers(
        connectionId,
        catalogName,
        schemaName,
        connType,
        projectPath
      )
      const triggers: TriggerNode[] = triggerMetas.map((t) => ({
        name: t.name,
        tableName: t.tableName ?? undefined,
        event: t.event ?? undefined,
      }))

      const catalogs = connectionCatalogs.value.get(connectionId)
      if (!catalogs) return
      const cat = catalogs.find((c: { name: string }) => c.name === catalogName)
      if (!cat) return
      const schema = cat.schemas.find((s: { name: string }) => s.name === schemaName)
      if (schema) {
        schema.triggers = triggers
      }
    } catch (e) {
      error.value = e instanceof Error ? e.message : '加载触发器列表失败'
      console.error('加载触发器列表失败:', e)
    } finally {
      loadingTables.value.delete(key)
    }
  }

  async function loadColumns(
    connectionId: string,
    catalogName: string,
    schemaName: string,
    tableName: string
  ) {
    const key = `${connectionId}:${catalogName}:${schemaName}:${tableName}`
    if (loadingColumns.value.has(key)) return

    loadingColumns.value.add(key)
    error.value = null

    try {
      const connType = connectionTypes.value.get(connectionId) || 'global'
      const projectPath = connectionProjectPaths.value.get(connectionId)

      const cacheStatus = await getMetadataCacheStatus(
        connectionId,
        connType,
        catalogName,
        schemaName,
        projectPath
      ).catch(() => ({ is_valid: false, last_sync: null, stats: null }))

      if (cacheStatus.is_valid) {
        const columns = await getColumnsFromCache(
          connectionId,
          connType,
          catalogName,
          schemaName,
          tableName,
          projectPath
        ).catch(() => [])

        if (columns.length > 0) {
          updateTableColumns(
            connectionId,
            catalogName,
            schemaName,
            tableName,
            columns.map(c => ({
              name: c.name,
              dataType: c.data_type,
              nullable: c.is_nullable,
              defaultValue: undefined,
              isPrimaryKey: c.is_primary,
            }))
          )
          return
        }
      }

      await loadColumnsFromDb(connectionId, catalogName, schemaName, tableName)
    } catch (e) {
      error.value = e instanceof Error ? e.message : '加载列信息失败'
      console.error('加载列信息失败:', e)
    } finally {
      loadingColumns.value.delete(key)
    }
  }

  async function loadColumnsFromDb(
    connectionId: string,
    catalogName: string,
    schemaName: string,
    tableName: string
  ) {
    const connType = connectionTypes.value.get(connectionId) || 'global'
    const projectPath = connectionProjectPaths.value.get(connectionId)

    const columnMetas = await databaseApi.loadColumns(
      connectionId,
      catalogName,
      schemaName,
      tableName,
      connType,
      projectPath
    )

    const columns = columnMetas.map(col => ({
      name: col.name,
      data_type: col.dataType,
      nullable: col.isNullable,
      default_value: col.defaultValue || undefined,
      is_primary_key: col.isPrimaryKey || false,
    }))

    const columnInputs: ColumnInput[] = columns.map(
      (col: {
        name: string
        data_type: string
        nullable: boolean
        default_value: string | undefined
        is_primary_key: boolean
      }) => ({
        id: `${connectionId}:${catalogName}:${schemaName}:${tableName}:${col.name}`,
        name: col.name,
        data_type: col.data_type,
        is_nullable: col.nullable,
        is_primary: col.is_primary_key,
        is_unique: false,
      })
    )

    if (columnInputs.length > 0) {
      try {
        await saveColumnsBatchToCache(
          connectionId,
          connType,
          projectPath,
          catalogName,
          schemaName,
          tableName,
          columnInputs
        )
      } catch (err) {
        console.warn('保存列缓存失败（非致命）:', err)
      }
    }

    updateTableColumns(
      connectionId,
      catalogName,
      schemaName,
      tableName,
      columns.map(
        (col: {
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
          isPrimaryKey: col.is_primary_key,
        })
      )
    )
  }

  function updateTableColumns(
    connectionId: string,
    catalogName: string,
    schemaName: string,
    tableName: string,
    columns: ColumnNode[]
  ) {
    const catalogs = connectionCatalogs.value.get(connectionId)
    if (catalogs) {
      const cat = catalogs.find((c: { name: string }) => c.name === catalogName)
      if (cat) {
        const schema = cat.schemas.find((s: { name: string }) => s.name === schemaName)
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

  async function refreshMetadata(connectionId: string, catalogName?: string, schemaName?: string) {
    const connType = connectionTypes.value.get(connectionId) || 'global'
    const projectPath = connectionProjectPaths.value.get(connectionId)

    await clearMetadataCache({
      connection_id: connectionId,
      connection_type: connType,
      database_name: catalogName || 'all',
      schema_name: schemaName,
      project_path: projectPath,
    }).catch(() => {})

    clearCache(connectionId)

    if (catalogName) {
      await loadCatalogs(connectionId)
      await loadSchemas(connectionId, catalogName)
      if (schemaName) {
        await loadTables(connectionId, catalogName, schemaName)
      }
    } else {
      await loadCatalogs(connectionId)
      startCacheWarming(connectionId)
    }
  }

  function setSelectedObject(object: SelectedObject | null) {
    selectedObject.value = object
  }

  function clearCache(connectionId?: string) {
    if (connectionId) {
      connectionCatalogs.value.delete(connectionId)
    } else {
      connectionCatalogs.value.clear()
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

  async function setIntrospectionLevel(
    connectionId: string,
    level: IntrospectionLevel
  ): Promise<void> {
    await databaseApi.setIntrospectionLevel(connectionId, level)
    introspectionLevels.value.set(connectionId, level)
  }

  function getIntrospectionLevel(connectionId: string): IntrospectionLevel {
    return introspectionLevels.value.get(connectionId) || 'level3'
  }

  function searchObjects(query: string): SearchResult[] {
    if (!query || query.trim().length === 0) return []

    const results: SearchResult[] = []
    const lowerQuery = query.toLowerCase()

    connectionCatalogs.value.forEach((catalogs, connectionId) => {
      catalogs.forEach(cat => {
        if (cat.name.toLowerCase().includes(lowerQuery)) {
          results.push({
            connectionId,
            type: 'catalog',
            name: cat.name,
            path: cat.name,
            matchType: 'name',
          })
        }

        cat.schemas.forEach(schema => {
          if (schema.name.toLowerCase().includes(lowerQuery)) {
            results.push({
              connectionId,
              type: 'schema',
              name: schema.name,
              path: `${cat.name}.${schema.name}`,
              matchType: 'name',
            })
          }

          schema.tables.forEach(table => {
            if (table.name.toLowerCase().includes(lowerQuery)) {
              results.push({
                connectionId,
                type: 'table',
                name: table.name,
                path: `${cat.name}.${schema.name}.${table.name}`,
                matchType: 'name',
              })
            }

            table.columns.forEach(col => {
              if (col.name.toLowerCase().includes(lowerQuery)) {
                results.push({
                  connectionId,
                  type: 'column',
                  name: `${table.name}.${col.name}`,
                  path: `${cat.name}.${schema.name}.${table.name}.${col.name}`,
                  matchType: 'name',
                  parentTable: table.name,
                })
              }
              if (col.dataType.toLowerCase().includes(lowerQuery)) {
                results.push({
                  connectionId,
                  type: 'column',
                  name: `${table.name}.${col.name}`,
                  path: `${cat.name}.${schema.name}.${table.name}.${col.name}`,
                  matchType: 'type',
                  parentTable: table.name,
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
                path: `${cat.name}.${schema.name}.${view.name}`,
                matchType: 'name',
              })
            }

            view.columns.forEach(col => {
              if (col.name.toLowerCase().includes(lowerQuery)) {
                results.push({
                  connectionId,
                  type: 'column',
                  name: `${view.name}.${col.name}`,
                  path: `${cat.name}.${schema.name}.${view.name}.${col.name}`,
                  matchType: 'name',
                  parentTable: view.name,
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

  function getCatalogSchemas(connectionId: string, catalogName: string): SchemaNode[] {
    const catalogs = connectionCatalogs.value.get(connectionId)
    if (!catalogs) return []
    const cat = catalogs.find(c => c.name === catalogName)
    if (!cat) return []
    return cat.schemas || []
  }

  function getSchemaTables(
    connectionId: string,
    catalogName: string,
    schemaName: string
  ): TableNode[] {
    const catalogs = connectionCatalogs.value.get(connectionId)
    if (!catalogs) return []
    const cat = catalogs.find(c => c.name === catalogName)
    if (!cat) return []
    if (cat.schemas.length === 0) return cat.tables || []
    const schema = cat.schemas.find(s => s.name === schemaName)
    if (!schema) return cat.tables || []
    return schema.tables || []
  }

  function getSchemaViews(
    connectionId: string,
    catalogName: string,
    schemaName: string
  ): ViewNode[] {
    const catalogs = connectionCatalogs.value.get(connectionId)
    if (!catalogs) return []
    const cat = catalogs.find(c => c.name === catalogName)
    if (!cat) return []
    const schema = cat.schemas.find(s => s.name === schemaName)
    if (!schema) return []
    return schema.views || []
  }

  function getDbType(connectionId: string): string {
    return connectionDbTypes.value.get(connectionId)?.toLowerCase() || ''
  }

  function computeSchemaStats(schema: SchemaNode) {
    schema.totalTables = schema.tables.filter(t => t.type === 'table' || !t.type).length
    schema.totalViews = schema.views.length

    let totalSize = 0
    let totalRows = 0
    for (const t of schema.tables) {
      if (t.dataLength) totalSize += t.dataLength
      if (t.indexLength) totalSize += t.indexLength
      if (t.rowCount) totalRows += t.rowCount
    }
    if (totalSize > 0) schema.totalSizeBytes = totalSize
    if (totalRows > 0) schema.rowCountTotal = totalRows
  }

  async function executeSql(
    connectionId: string,
    _catalogName: string,
    sql: string
  ): Promise<unknown> {
    return await executeSqlService(connectionId, sql)
  }

  function expandToNode(_nodeKey: string): void {
    // Tree expansion managed by database-navigator component
  }

  async function startCacheWarming(connectionId: string): Promise<void> {
    const catalogs = connectionCatalogs.value.get(connectionId)
    if (!catalogs || catalogs.length === 0) return

    const t0 = performance.now()

    const targetCatalogs = catalogs.filter(cat => cat.name !== 'default')
    if (targetCatalogs.length === 0) return

    // Phase 1: 并行加载所有 Catalog 的 Schema
    const schemaResults = await Promise.allSettled(
      targetCatalogs.map(cat => loadSchemas(connectionId, cat.name))
    )

    // Phase 2: 收集所有 Schema，并行加载表
    const tablePromises: Promise<void>[] = []
    for (const cat of targetCatalogs) {
      const schemas = getCatalogSchemas(connectionId, cat.name)
      for (const schema of schemas) {
        if (schema.name === 'default') continue
        tablePromises.push(loadTables(connectionId, cat.name, schema.name).catch(() => {}))
      }
    }
    await Promise.allSettled(tablePromises)

    let colTaskCount = 0
    // Phase 3: 根据内省级别决定是否加载列
    const colLevel = introspectionLevels.value.get(connectionId) || 'level3'
    if (colLevel !== 'level1') {
      const columnPromises: Promise<void>[] = []
      for (const cat of targetCatalogs) {
        const schemas = getCatalogSchemas(connectionId, cat.name)
        for (const schema of schemas) {
          const tables = getSchemaTables(connectionId, cat.name, schema.name)
          for (const table of tables.slice(0, 10)) {
            columnPromises.push(
              loadColumns(connectionId, cat.name, schema.name, table.name).catch(() => {})
            )
          }
        }
      }

      const BATCH_SIZE = 20
      for (let i = 0; i < columnPromises.length; i += BATCH_SIZE) {
        const batch = columnPromises.slice(i, i + BATCH_SIZE)
        await Promise.allSettled(batch)
      }
      colTaskCount = columnPromises.length
    }

    const elapsed = (performance.now() - t0).toFixed(0)
    console.log(
      `[CacheWarming] 连接 ${connectionId} 缓存预热完成 ` +
        `(耗时 ${elapsed}ms, Catalog=${targetCatalogs.length}, schema结果=${schemaResults.length}, ` +
        `table任务=${tablePromises.length}, column任务=${colTaskCount})`
    )
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
    connectionCatalogs,
    selectedObject,
    error,
    getCatalogs,
    getCatalogSchemas,
    getSchemaTables,
    getSchemaViews,
    isLoadingCatalogs,
    isLoadingSchemas,
    isLoadingTables,
    setConnectionInfo,
    getDbType,
    getConnectionType,
    getProjectPath,
    setIntrospectionLevel,
    getIntrospectionLevel,
    loadCatalogs,
    loadCatalogsFromCacheSilent,
    loadSchemas,
    loadTables,
    loadViews,
    loadProcedures,
    loadFunctions,
    loadIndexes,
    loadConstraints,
    loadSequences,
    loadTriggers,
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
    startCacheWarming,
    getLastSyncTime,
    setLastSyncTime,
    setSyncMode,
    getSyncMode,
  }
})

interface CatalogNode {
  name: string
  schemas: SchemaNode[]
  /** 无 Schema 的数据库（如 MySQL）直接在此存储表 */
  tables?: TableNode[]
}

interface SchemaNode {
  name: string
  tables: TableNode[]
  views: ViewNode[]
  procedures?: ProcedureNode[]
  functions?: FunctionNode[]
  sequences?: SequenceNode[]
  triggers?: TriggerNode[]
  totalTables?: number
  totalViews?: number
  totalSizeBytes?: number
  rowCountTotal?: number
}

interface TableNode {
  name: string
  type: string
  columns: ColumnNode[]
  indexes?: IndexNode[]
  constraints?: ConstraintNode[]
  rowCount?: number | null
  dataLength?: number | null
  indexLength?: number | null
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

interface ProcedureNode {
  name: string
}

interface FunctionNode {
  name: string
}

interface SequenceNode {
  name: string
}

interface TriggerNode {
  name: string
  tableName?: string
  event?: string
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
  kind: 'catalog' | 'schema' | 'table' | 'view' | 'column'
  catalog?: string
  schema?: string
  table?: string
  connectionId: string
  [key: string]: unknown
}

interface SearchResult {
  connectionId: string
  type: 'catalog' | 'schema' | 'table' | 'view' | 'column'
  name: string
  path: string
  matchType: 'name' | 'type'
  parentTable?: string
}
