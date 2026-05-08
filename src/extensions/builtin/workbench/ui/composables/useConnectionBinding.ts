import { ref, computed } from 'vue'

import { useConnectionStore } from '@/extensions/builtin/connection/ui/stores/connection-store'
import { useRuntimeConnectionStore } from '@/extensions/builtin/connection/ui/stores/runtime-connection-store'
import type { Connection } from '@/shared/types'
import type { DatabaseType } from '@/shared/types/sql'

export interface ConnectionBindingOptions {
  initialConnectionId?: string
}

export function useConnectionBinding(_options: ConnectionBindingOptions = {}) {
  const { initialConnectionId } = _options

  const connectionStore = useConnectionStore()
  const runtimeConnectionStore = useRuntimeConnectionStore()

  const selectedConnection = ref(initialConnectionId || '')
  const runtimeConnId = ref('')

  const connections = computed<Connection[]>(() => {
    return (connectionStore.connections as Connection[]) || []
  })

  const popselectOptions = computed(() => {
    return connections.value
      .filter(c => c.status === 'connected')
      .map(c => ({
        label: `${c.name} → ${c.dbType}${c.meta?.database ? ` / ${c.meta.database}` : ''}`,
        value: c.connId,
      }))
  })

  const connectionInfoText = computed(() => {
    if (!selectedConnection.value) return ''
    const conn = connections.value.find(c => c.connId === selectedConnection.value)
    if (!conn) return ''
    return `${conn.name} → ${conn.dbType}${conn.meta?.database ? ` / ${conn.meta.database}` : ''}`
  })

  const isDuckDbConnection = computed(() => {
    const conn = connections.value.find(c => c.connId === selectedConnection.value)
    return conn?.dbType === 'duckdb'
  })

  const currentDatabaseType = computed<DatabaseType | null>(() => {
    const conn = connections.value.find(c => c.connId === selectedConnection.value)
    return conn?.dbType ?? null
  })

  const currentDatabase = computed<string>(() => {
    const conn = connections.value.find(c => c.connId === selectedConnection.value)
    return conn?.meta?.database ?? ''
  })

  const currentConnectionName = computed<string>(() => {
    const conn = connections.value.find(c => c.connId === selectedConnection.value)
    return conn?.name ?? ''
  })

  async function ensureConnection(connId: string): Promise<boolean> {
    if (!connId) return false

    const runtimeIds = runtimeConnectionStore.runtimeConnectionIds || []
    if (runtimeIds.includes(connId)) {
      runtimeConnId.value = connId
      return true
    }

    const conn = connectionStore.getConnection(connId)
    if (!conn) {
      console.warn('[useConnectionBinding] Connection not found:', connId)
      return false
    }

    try {
      const runtimeId = await runtimeConnectionStore.establishFromConnection(connId)
      runtimeConnId.value = runtimeId
      return true
    } catch (error) {
      console.warn('[useConnectionBinding] Failed to establish runtime connection:', error)
      if (conn.status === 'connected') {
        runtimeConnId.value = connId
        return true
      }
      return false
    }
  }

  async function waitForConnection(): Promise<boolean> {
    const connId = selectedConnection.value
    if (!connId) return false

    const runtimeIds = runtimeConnectionStore.runtimeConnectionIds || []
    if (runtimeIds.includes(connId)) {
      runtimeConnId.value = connId
      return true
    }

    for (let attempt = 0; attempt < 50; attempt++) {
      const ids = runtimeConnectionStore.runtimeConnectionIds || []
      if (ids.includes(connId)) {
        runtimeConnId.value = connId
        return true
      }

      const conn = connectionStore.connections.find(c => c.connId === connId)
      if (conn && ids.includes(conn.connId)) {
        runtimeConnId.value = conn.connId
        return true
      }

      if (attempt === 4) {
        try {
          await connectionStore.loadConnections()
        } catch {
          // continue waiting
        }
      }

      if (attempt < 49) {
        await new Promise(resolve => setTimeout(resolve, 200))
      }
    }

    const fallbackIds = runtimeConnectionStore.runtimeConnectionIds || []
    if (fallbackIds.length > 0) {
      runtimeConnId.value = fallbackIds[0]
      return true
    }

    const firstConnected = connectionStore.connections.find(c => c.status === 'connected')
    if (firstConnected) {
      selectedConnection.value = firstConnected.connId
      try {
        const runtimeId = await runtimeConnectionStore.establishFromConnection(
          firstConnected.connId
        )
        runtimeConnId.value = runtimeId
        return true
      } catch {
        return false
      }
    }

    return false
  }

  function onConnectionSelected(connId: string): void {
    selectedConnection.value = connId
  }

  return {
    selectedConnection,
    runtimeConnId,
    connections,
    popselectOptions,
    connectionInfoText,
    isDuckDbConnection,
    currentDatabaseType,
    currentDatabase,
    currentConnectionName,
    ensureConnection,
    waitForConnection,
    onConnectionSelected,
  }
}
