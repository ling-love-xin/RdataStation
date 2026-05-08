const STORAGE_PREFIX = 'rdata:workbench:'
const DRAFT_TTL_MS = 7 * 24 * 60 * 60 * 1000

export function useEditorPersistence(panelId: string) {
  const draftKey = `${STORAGE_PREFIX}draft:${panelId}`

  const draft = {
    save: (sql: string) => {
      try {
        const payload = JSON.stringify({ sql, timestamp: Date.now() })
        localStorage.setItem(draftKey, payload)
      } catch (error) {
        console.warn('[EditorPersistence] Failed to save draft:', error)
      }
    },

    load: (): string | null => {
      try {
        const raw = localStorage.getItem(draftKey)
        if (!raw) return null
        const { sql, timestamp } = JSON.parse(raw)
        if (Date.now() - timestamp > DRAFT_TTL_MS) {
          localStorage.removeItem(draftKey)
          return null
        }
        return sql
      } catch {
        localStorage.removeItem(draftKey)
        return null
      }
    },

    remove: () => {
      localStorage.removeItem(draftKey)
    },
  }

  return { draft }
}

export function clearOrphanDrafts(activePanelIds: string[]) {
  const activeSet = new Set(activePanelIds)
  const prefix = `${STORAGE_PREFIX}draft:`

  for (let i = localStorage.length - 1; i >= 0; i--) {
    const key = localStorage.key(i)
    if (key && key.startsWith(prefix)) {
      const panelId = key.slice(prefix.length)
      if (!activeSet.has(panelId)) {
        try {
          const raw = localStorage.getItem(key)
          if (raw) {
            const { timestamp } = JSON.parse(raw)
            if (Date.now() - timestamp > DRAFT_TTL_MS) {
              localStorage.removeItem(key)
            }
          }
        } catch {
          localStorage.removeItem(key)
        }
      }
    }
  }
}
