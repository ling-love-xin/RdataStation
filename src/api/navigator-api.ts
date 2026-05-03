import { invoke } from '@tauri-apps/api/core'

export interface CacheStatusResponse {
  is_valid: boolean
  last_sync: string | null
  stats: {
    database_count: number
    table_count: number
    column_count: number
  } | null
}

export interface CachedDatabase {
  id: string
  name: string
  last_sync: string
}

export interface CachedTable {
  id: string
  name: string
  comment: string | null
  last_sync: string
}

export interface CachedColumn {
  id: string
  name: string
  data_type: string
  is_nullable: boolean
  column_default: string | null
  is_primary_key: boolean
  last_sync: string
}

export interface ApiResponse<T> {
  success: boolean
  data: T
  error: string | null
}

const MAX_RETRIES = 3
const RETRY_DELAY = 1000

async function invokeWithRetry<T>(
  command: string,
  args: Record<string, unknown> = {},
  retries = MAX_RETRIES
): Promise<T> {
  let lastError: Error | null = null

  for (let i = 0; i < retries; i++) {
    try {
      const result = await invoke<T>(command, args)
      return result
    } catch (error) {
      lastError = error instanceof Error ? error : new Error(String(error))

      if (i < retries - 1) {
        await new Promise(resolve => setTimeout(resolve, RETRY_DELAY * (i + 1)))
      }
    }
  }

  throw lastError || new Error(`Command ${command} failed after ${retries} retries`)
}

export async function getMetadataCacheStatus(
  connectionId: string,
  connectionType: 'global' | 'project',
  databaseName: string,
  schemaName?: string,
  projectPath?: string
): Promise<ApiResponse<CacheStatusResponse>> {
  try {
    const data = await invokeWithRetry<CacheStatusResponse>('get_metadata_cache_status', {
      connectionId,
      connectionType,
      projectPath,
      databaseName,
      schemaName
    })

    return { success: true, data, error: null }
  } catch (error) {
    return {
      success: false,
      data: { is_valid: false, last_sync: null, stats: null },
      error: error instanceof Error ? error.message : 'Unknown error'
    }
  }
}

export async function refreshMetadataCache(
  connectionId: string,
  connectionType: 'global' | 'project',
  projectPath?: string
): Promise<ApiResponse<void>> {
  try {
    await invokeWithRetry<void>('refresh_metadata_cache', {
      connectionId,
      connectionType,
      projectPath
    })

    return { success: true, data: undefined, error: null }
  } catch (error) {
    return {
      success: false,
      data: undefined,
      error: error instanceof Error ? error.message : 'Unknown error'
    }
  }
}

export async function clearMetadataCache(
  connectionId: string,
  connectionType: 'global' | 'project',
  projectPath?: string
): Promise<ApiResponse<void>> {
  try {
    await invokeWithRetry<void>('clear_metadata_cache', {
      connectionId,
      connectionType,
      projectPath
    })

    return { success: true, data: undefined, error: null }
  } catch (error) {
    return {
      success: false,
      data: undefined,
      error: error instanceof Error ? error.message : 'Unknown error'
    }
  }
}

export async function getDatabasesFromCache(
  connectionId: string,
  connectionType: 'global' | 'project',
  projectPath?: string
): Promise<ApiResponse<CachedDatabase[]>> {
  try {
    const data = await invokeWithRetry<any[]>('get_databases_from_cache', {
      connectionId,
      connectionType,
      projectPath
    })

    return {
      success: true,
      data: data.map(item => ({
        id: item.id,
        name: item.name,
        last_sync: item.last_sync
      })),
      error: null
    }
  } catch (error) {
    return {
      success: false,
      data: [],
      error: error instanceof Error ? error.message : 'Unknown error'
    }
  }
}

export async function getTablesFromCache(
  connectionId: string,
  connectionType: 'global' | 'project',
  databaseName: string,
  schemaName?: string,
  projectPath?: string
): Promise<ApiResponse<CachedTable[]>> {
  try {
    const data = await invokeWithRetry<any[]>('get_tables_from_cache', {
      connectionId,
      connectionType,
      projectPath,
      databaseName,
      schemaName
    })

    return {
      success: true,
      data: data.map(item => ({
        id: item.id,
        name: item.name,
        comment: item.comment,
        last_sync: item.last_sync
      })),
      error: null
    }
  } catch (error) {
    return {
      success: false,
      data: [],
      error: error instanceof Error ? error.message : 'Unknown error'
    }
  }
}

export async function getColumnsFromCache(
  connectionId: string,
  connectionType: 'global' | 'project',
  databaseName: string,
  tableName: string,
  schemaName?: string,
  projectPath?: string
): Promise<ApiResponse<CachedColumn[]>> {
  try {
    const data = await invokeWithRetry<any[]>('get_columns_from_cache', {
      connectionId,
      connectionType,
      projectPath,
      databaseName,
      tableName,
      schemaName
    })

    return {
      success: true,
      data: data.map(item => ({
        id: item.id,
        name: item.name,
        data_type: item.data_type,
        is_nullable: item.is_nullable,
        column_default: item.column_default,
        is_primary_key: item.is_primary_key,
        last_sync: item.last_sync
      })),
      error: null
    }
  } catch (error) {
    return {
      success: false,
      data: [],
      error: error instanceof Error ? error.message : 'Unknown error'
    }
  }
}
