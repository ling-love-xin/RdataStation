import { ref, watch, type Ref, type WatchStopHandle } from 'vue'

import {
  getCurrentDialect as getHighlightDialect,
  setCurrentDialect as setHighlightDialect,
} from '@/extensions/builtin/workbench/services/sql-dialect-highlight'
import {
  registerDatabaseCompletionProvider,
  unregisterCompletionProvider,
} from '@/extensions/builtin/workbench/services/sql-editor-service'
import type { SqlDialect } from '@/shared/types/sql'

export interface DialectSyncOptions {
  dbType: Ref<string | null>
  editorReady: Ref<boolean>
}

export function useDialectSync(options: DialectSyncOptions) {
  const { dbType, editorReady } = options

  const currentDialect = ref<SqlDialect>('generic')
  let stopWatcher: WatchStopHandle | null = null

  function getCurrentDialect(): SqlDialect {
    return currentDialect.value
  }

  function mapDbTypeToDialect(dbTypeValue: string | null): SqlDialect {
    if (!dbTypeValue) return 'generic'
    const validDialects: SqlDialect[] = [
      'generic',
      'mysql',
      'postgres',
      'sqlite',
      'duckdb',
      'mssql',
      'oracle',
      'snowflake',
      'bigquery',
      'redshift',
    ]
    return validDialects.includes(dbTypeValue as SqlDialect)
      ? (dbTypeValue as SqlDialect)
      : 'generic'
  }

  function updateDialectHighlight(): void {
    const dialect = currentDialect.value
    const prevDialect = getHighlightDialect()
    if (prevDialect === dialect) return

    unregisterCompletionProvider(currentDialect.value || '')
    setHighlightDialect(dialect)
    registerDatabaseCompletionProvider(dialect, dialect, undefined, dialect)
  }

  function startSync(): void {
    stopWatcher = watch(
      [dbType, editorReady],
      ([newDbType, ready]) => {
        if (!ready) return
        const dialect = mapDbTypeToDialect(newDbType)
        if (currentDialect.value !== dialect) {
          currentDialect.value = dialect
          updateDialectHighlight()
        }
      },
      { immediate: true }
    )
  }

  function stopSync(): void {
    stopWatcher?.()
    stopWatcher = null
  }

  return {
    currentDialect,
    getCurrentDialect,
    updateDialectHighlight,
    startSync,
    stopSync,
  }
}
