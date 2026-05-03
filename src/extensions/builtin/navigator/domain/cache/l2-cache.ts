/**
 * L2 IndexedDB 缓存
 * 本地持久化缓存层
 */

import { openDB, type DBSchema, type IDBPDatabase } from 'idb'

interface NavigatorCacheSchema extends DBSchema {
  nodes: {
    key: string
    value: {
      id: string
      data: unknown
      version: number
      timestamp: number
      ttl?: number
    }
    indexes: {
      'by-timestamp': number
    }
  }
}

export class L2Cache {
  private db: IDBPDatabase<NavigatorCacheSchema> | null = null
  private dbName = 'navigator-cache-v1'
  private version = 1
  private initializing: Promise<void> | null = null

  /**
   * 初始化数据库
   */
  async initialize(): Promise<void> {
    if (this.db) return
    if (this.initializing) return this.initializing

    this.initializing = this.doInitialize()
    return this.initializing
  }

  private async doInitialize(): Promise<void> {
    try {
      this.db = await openDB<NavigatorCacheSchema>(this.dbName, this.version, {
        upgrade(db) {
          // 创建对象存储
          if (!db.objectStoreNames.contains('nodes')) {
            const store = db.createObjectStore('nodes', { keyPath: 'id' })
            store.createIndex('by-timestamp', 'timestamp')
          }
        }
      })
    } catch (error) {
      console.error('Failed to initialize L2 cache:', error)
      throw error
    }
  }

  /**
   * 获取缓存项
   */
  async get<T>(key: string): Promise<T | undefined> {
    await this.initialize()
    if (!this.db) return undefined

    try {
      const entry = await this.db.get('nodes', key)
      if (!entry) return undefined

      // 检查是否过期
      if (entry.ttl && Date.now() - entry.timestamp > entry.ttl) {
        await this.delete(key)
        return undefined
      }

      return entry.data as T
    } catch (error) {
      console.error('L2 cache get error:', error)
      return undefined
    }
  }

  /**
   * 设置缓存项
   */
  async set<T>(key: string, value: T, options?: { version?: number; ttl?: number }): Promise<void> {
    await this.initialize()
    if (!this.db) return

    try {
      await this.db.put('nodes', {
        id: key,
        data: value,
        version: options?.version || 1,
        timestamp: Date.now(),
        ttl: options?.ttl
      })
    } catch (error) {
      console.error('L2 cache set error:', error)
    }
  }

  /**
   * 删除缓存项
   */
  async delete(key: string): Promise<void> {
    await this.initialize()
    if (!this.db) return

    try {
      await this.db.delete('nodes', key)
    } catch (error) {
      console.error('L2 cache delete error:', error)
    }
  }

  /**
   * 批量获取
   */
  async batchGet<T>(keys: string[]): Promise<Map<string, T>> {
    await this.initialize()
    const result = new Map<string, T>()

    if (!this.db) return result

    try {
      const tx = this.db.transaction('nodes', 'readonly')
      const store = tx.objectStore('nodes')

      await Promise.all(
        keys.map(async key => {
          const entry = await store.get(key)
          if (entry && (!entry.ttl || Date.now() - entry.timestamp <= entry.ttl)) {
            result.set(key, entry.data as T)
          }
        })
      )

      await tx.done
    } catch (error) {
      console.error('L2 cache batchGet error:', error)
    }

    return result
  }

  /**
   * 批量设置
   */
  async batchSet<T>(entries: Array<[string, T]>, options?: { ttl?: number }): Promise<void> {
    await this.initialize()
    if (!this.db) return

    try {
      const tx = this.db.transaction('nodes', 'readwrite')
      const store = tx.objectStore('nodes')

      for (const [key, value] of entries) {
        await store.put({
          id: key,
          data: value,
          version: 1,
          timestamp: Date.now(),
          ttl: options?.ttl
        })
      }

      await tx.done
    } catch (error) {
      console.error('L2 cache batchSet error:', error)
    }
  }

  /**
   * 清理过期缓存
   */
  async cleanup(maxAge: number): Promise<number> {
    await this.initialize()
    if (!this.db) return 0

    const cutoff = Date.now() - maxAge
    let deleted = 0

    try {
      const tx = this.db.transaction('nodes', 'readwrite')
      const store = tx.objectStore('nodes')
      const index = store.index('by-timestamp')

      const range = IDBKeyRange.upperBound(cutoff)
      let cursor = await index.openCursor(range)

      while (cursor) {
        await cursor.delete()
        deleted++
        cursor = await cursor.continue()
      }

      await tx.done
    } catch (error) {
      console.error('L2 cache cleanup error:', error)
    }

    return deleted
  }

  /**
   * 清空缓存
   */
  async clear(): Promise<void> {
    await this.initialize()
    if (!this.db) return

    try {
      const tx = this.db.transaction('nodes', 'readwrite')
      await tx.objectStore('nodes').clear()
      await tx.done
    } catch (error) {
      console.error('L2 cache clear error:', error)
    }
  }
}

// 创建单例
export const l2Cache = new L2Cache()
