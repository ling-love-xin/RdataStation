import { ref, computed } from 'vue'

export interface StagingSnapshot {
  name: string
  dbType: string
  driverId: string
  description: string
  formData: Record<string, unknown>
  networkConfig: Record<string, unknown>
  advancedConfig: Record<string, unknown>
  driverProps: Record<string, string>
}

export interface StagingEntry {
  id: number
  name: string
  dbType: string
  driverId: string
  snapshot: StagingSnapshot
}

export function useStagingList() {
  const entries = ref<StagingEntry[]>([])
  let nextId = 1

  const count = computed(() => entries.value.length)

  function addEntry(snapshot: StagingSnapshot) {
    // 保存当前表单快照到暂存列表
    const existing = entries.value.find(e => e.name === snapshot.name && e.dbType === snapshot.dbType)
    if (existing) {
      existing.snapshot = snapshot
      existing.driverId = snapshot.driverId
      return existing.id
    }
    const id = nextId++
    entries.value.push({
      id,
      name: snapshot.name || `连接 ${id}`,
      dbType: snapshot.dbType,
      driverId: snapshot.driverId,
      snapshot,
    })
    return id
  }

  function removeEntry(id: number) {
    const idx = entries.value.findIndex(e => e.id === id)
    if (idx !== -1) {
      entries.value.splice(idx, 1)
    }
  }

  function updateEntryName(id: number, name: string) {
    const entry = entries.value.find(e => e.id === id)
    if (entry) {
      entry.name = name
      entry.snapshot.name = name
    }
  }

  function selectEntry(id: number): StagingEntry | undefined {
    return entries.value.find(e => e.id === id)
  }

  function init() {
    entries.value = []
    nextId = 1
  }

  init()

  return {
    entries,
    count,
    addEntry,
    removeEntry,
    updateEntryName,
    selectEntry,
    init,
  }
}