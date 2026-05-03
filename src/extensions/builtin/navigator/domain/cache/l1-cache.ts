/**
 * L1 内存缓存
 * 最快的缓存层，使用 LRU 淘汰策略
 */

interface CacheEntry<T> {
  value: T
  timestamp: number
  ttl?: number
}

export class L1Cache<T> {
  private cache = new Map<string, CacheEntry<T>>()
  private maxSize: number
  private accessOrder: string[] = []

  constructor(maxSize: number = 1000) {
    this.maxSize = maxSize
  }

  /**
   * 获取缓存项
   */
  get(key: string): T | undefined {
    const entry = this.cache.get(key)
    if (!entry) return undefined

    // 检查是否过期
    if (entry.ttl && Date.now() - entry.timestamp > entry.ttl) {
      this.delete(key)
      return undefined
    }

    // 更新访问顺序
    this.updateAccessOrder(key)

    return entry.value
  }

  /**
   * 设置缓存项
   */
  set(key: string, value: T, options?: { ttl?: number }): void {
    // 检查容量
    if (this.cache.size >= this.maxSize && !this.cache.has(key)) {
      this.evictLRU()
    }

    const entry: CacheEntry<T> = {
      value,
      timestamp: Date.now(),
      ttl: options?.ttl
    }

    this.cache.set(key, entry)
    this.updateAccessOrder(key)
  }

  /**
   * 删除缓存项
   */
  delete(key: string): boolean {
    this.removeFromAccessOrder(key)
    return this.cache.delete(key)
  }

  /**
   * 清空缓存
   */
  clear(): void {
    this.cache.clear()
    this.accessOrder = []
  }

  /**
   * 批量获取
   */
  batchGet(keys: string[]): Map<string, T> {
    const result = new Map<string, T>()

    for (const key of keys) {
      const value = this.get(key)
      if (value !== undefined) {
        result.set(key, value)
      }
    }

    return result
  }

  /**
   * 批量设置
   */
  batchSet(entries: Array<[string, T]>, options?: { ttl?: number }): void {
    for (const [key, value] of entries) {
      this.set(key, value, options)
    }
  }

  /**
   * 检查是否存在
   */
  has(key: string): boolean {
    const entry = this.cache.get(key)
    if (!entry) return false

    // 检查是否过期
    if (entry.ttl && Date.now() - entry.timestamp > entry.ttl) {
      this.delete(key)
      return false
    }

    return true
  }

  /**
   * 获取缓存大小
   */
  size(): number {
    return this.cache.size
  }

  /**
   * 获取所有键
   */
  keys(): string[] {
    return Array.from(this.cache.keys())
  }

  /**
   * 清理过期项
   */
  evictExpired(): number {
    let count = 0
    const now = Date.now()

    for (const [key, entry] of this.cache.entries()) {
      if (entry.ttl && now - entry.timestamp > entry.ttl) {
        this.delete(key)
        count++
      }
    }

    return count
  }

  // 私有方法
  private updateAccessOrder(key: string): void {
    const index = this.accessOrder.indexOf(key)
    if (index > -1) {
      this.accessOrder.splice(index, 1)
    }
    this.accessOrder.push(key)
  }

  private removeFromAccessOrder(key: string): void {
    const index = this.accessOrder.indexOf(key)
    if (index > -1) {
      this.accessOrder.splice(index, 1)
    }
  }

  private evictLRU(): void {
    if (this.accessOrder.length === 0) return

    const lruKey = this.accessOrder[0]
    this.delete(lruKey)
  }
}

// 创建单例
export const l1Cache = new L1Cache<unknown>(1000)
