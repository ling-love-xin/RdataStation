/**
 * 搜索索引实现
 * 
 * 使用倒排索引加速搜索
 */

export interface SearchIndexEntry {
  nodeId: string
  nodeType: string
  connectionId: string
  labels: string[]
}

export class SearchIndex {
  private index = new Map<string, Set<string>>()
  private entries = new Map<string, SearchIndexEntry>()

  add(entry: SearchIndexEntry): void {
    this.entries.set(entry.nodeId, entry)

    for (const label of entry.labels) {
      const terms = this.tokenize(label.toLowerCase())
      for (const term of terms) {
        if (!this.index.has(term)) {
          this.index.set(term, new Set())
        }
        this.index.get(term)!.add(entry.nodeId)
      }
    }
  }

  remove(nodeId: string): void {
    const entry = this.entries.get(nodeId)
    if (entry) {
      for (const label of entry.labels) {
        const terms = this.tokenize(label.toLowerCase())
        for (const term of terms) {
          const nodeIds = this.index.get(term)
          if (nodeIds) {
            nodeIds.delete(nodeId)
            if (nodeIds.size === 0) {
              this.index.delete(term)
            }
          }
        }
      }
      this.entries.delete(nodeId)
    }
  }

  search(query: string): string[] {
    if (!query.trim()) return []

    const terms = this.tokenize(query.toLowerCase())
    if (terms.length === 0) return []

    let results: Set<string> | null = null

    for (const term of terms) {
      const nodeIds = this.index.get(term)
      if (!nodeIds) return []

      if (results === null) {
        results = new Set(nodeIds)
      } else {
        const intersection = new Set<string>()
        for (const id of nodeIds) {
          if (results.has(id)) {
            intersection.add(id)
          }
        }
        results = intersection
      }

      if (results.size === 0) break
    }

    return results ? Array.from(results) : []
  }

  getEntry(nodeId: string): SearchIndexEntry | undefined {
    return this.entries.get(nodeId)
  }

  clear(): void {
    this.index.clear()
    this.entries.clear()
  }

  size(): number {
    return this.entries.size
  }

  private tokenize(text: string): string[] {
    const tokens = text
      .split(/[^a-zA-Z0-9_\u4e00-\u9fa5]+/)
      .filter(token => token.length > 0)
    
    const result: string[] = []
    for (const token of tokens) {
      result.push(token)
      for (let i = 1; i < token.length; i++) {
        result.push(token.slice(i))
      }
    }
    return result
  }
}