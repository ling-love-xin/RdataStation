const STORAGE_PREFIX = 'rdata:workbench:'
const DRAFT_TTL_MS = 7 * 24 * 60 * 60 * 1000
const SESSION_ID = `${Date.now()}_${Math.random().toString(36).slice(2, 9)}`
const SESSION_DRAFT_PREFIX = `${STORAGE_PREFIX}draft:${SESSION_ID}:`

;(function cleanupStaleDrafts() {
  const genericPrefix = `${STORAGE_PREFIX}draft:`
  for (let i = localStorage.length - 1; i >= 0; i--) {
    const key = localStorage.key(i)
    if (key && key.startsWith(genericPrefix) && !key.startsWith(SESSION_DRAFT_PREFIX)) {
      localStorage.removeItem(key)
    }
  }
})()

export function useEditorPersistence(panelId: string, filePath?: string) {
  const draftKey = filePath
    ? `${SESSION_DRAFT_PREFIX}${panelId}:${filePath}`
    : `${SESSION_DRAFT_PREFIX}${panelId}`

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

  for (let i = localStorage.length - 1; i >= 0; i--) {
    const key = localStorage.key(i)
    if (key && key.startsWith(SESSION_DRAFT_PREFIX)) {
      const payload = key.slice(SESSION_DRAFT_PREFIX.length)
      const firstColon = payload.indexOf(':')
      const panelId = firstColon > 0 ? payload.slice(0, firstColon) : payload

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
