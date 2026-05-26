const STATE_VERSION = 1
const STORAGE_KEY_PREFIX = 'rds:navigator:'

interface NavigatorStateEntry {
  expandedKeys: string[]
  selectedKey: string | null
  filterText: string
  lastUpdated: number
  version: number
}

function entryKey(connId: string, projectPath?: string): string {
  const scope = projectPath ? `project:${btoa(projectPath).slice(0, 32)}` : 'global'
  return `${STORAGE_KEY_PREFIX}${scope}:${connId}`
}

export function getConnectionNavigatorState(
  connId: string,
  _projectPath?: string
): NavigatorStateEntry | null {
  try {
    const raw = localStorage.getItem(entryKey(connId, _projectPath))
    if (!raw) return null

    const parsed = JSON.parse(raw) as NavigatorStateEntry & { version?: number }
    if (parsed.version !== STATE_VERSION) return null

    return parsed
  } catch {
    return null
  }
}

export function saveConnectionNavigatorState(
  connId: string,
  entry: Partial<NavigatorStateEntry>,
  _projectPath?: string
): void {
  try {
    const current = getConnectionNavigatorState(connId, _projectPath)
    const merged: NavigatorStateEntry = {
      expandedKeys: entry.expandedKeys ?? current?.expandedKeys ?? [],
      selectedKey: entry.selectedKey ?? current?.selectedKey ?? null,
      filterText: entry.filterText ?? current?.filterText ?? '',
      lastUpdated: Date.now(),
      version: STATE_VERSION,
    }

    localStorage.setItem(entryKey(connId, _projectPath), JSON.stringify(merged))
  } catch (e) {
    console.warn('[navigator-persistence] 保存失败', e)
  }
}

export function clearConnectionNavigatorState(
  connId: string,
  projectPath?: string
): void {
  localStorage.removeItem(entryKey(connId, projectPath))
}

export function clearAllNavigatorStates(): void {
  const keysToRemove: string[] = []
  for (let i = 0; i < localStorage.length; i++) {
    const key = localStorage.key(i)
    if (key?.startsWith(STORAGE_KEY_PREFIX)) {
      keysToRemove.push(key)
    }
  }
  keysToRemove.forEach((k) => localStorage.removeItem(k))
}
