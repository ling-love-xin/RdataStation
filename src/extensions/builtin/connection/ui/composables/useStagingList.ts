import { ref, computed } from 'vue'

export interface StagingEntry {
  id: number
  name: string
  dbType: string
  driverId: string
}

export function useStagingList() {
  const entries = ref<StagingEntry[]>([])
  let nextId = 1

  const count = computed(() => entries.value.length)

  function addEntry() {
    entries.value.push({
      id: nextId++,
      name: '新建数据源',
      dbType: '',
      driverId: '',
    })
  }

  function removeEntry(id: number) {
    const idx = entries.value.findIndex(e => e.id === id)
    if (idx !== -1) {
      entries.value.splice(idx, 1)
    }
    if (entries.value.length === 0) {
      addEntry()
    }
  }

  function updateEntry(id: number, data: Partial<StagingEntry>) {
    const entry = entries.value.find(e => e.id === id)
    if (entry) {
      Object.assign(entry, data)
    }
  }

  function selectEntry(id: number): StagingEntry | undefined {
    return entries.value.find(e => e.id === id)
  }

  function init() {
    entries.value = []
    addEntry()
  }

  init()

  return {
    entries,
    count,
    addEntry,
    removeEntry,
    updateEntry,
    selectEntry,
    init,
  }
}