/**
 * 虚拟视口
 * 实现虚拟滚动，只渲染可见区域的节点
 */

import { ref, computed, type Ref, type ComputedRef } from 'vue'

export interface VirtualViewportConfig {
  itemHeight: number
  overscan?: number
  estimateItemHeight?: (item: unknown, index: number) => number
}

export interface VirtualViewportState {
  scrollTop: Ref<number>
  containerHeight: Ref<number>
  visibleRange: ComputedRef<{ start: number; end: number }>
  totalHeight: ComputedRef<number>
}

export interface VisibleItem<T> {
  item: T
  index: number
  style: {
    position: string
    top: string
    height: string
    left: string
    right: string
  }
}

export class VirtualViewport<T> {
  private config: Required<VirtualViewportConfig>
  private items: Ref<T[]> = ref([])
  private scrollTop = ref(0)
  private containerHeight = ref(0)

  constructor(config: VirtualViewportConfig) {
    this.config = {
      itemHeight: config.itemHeight,
      overscan: config.overscan ?? 5,
      estimateItemHeight: config.estimateItemHeight ?? ((_: unknown, __: number) => config.itemHeight)
    }
  }

  /**
   * 设置数据项
   */
  setItems(items: T[]): void {
    this.items.value = items
  }

  /**
   * 获取数据项
   */
  getItems(): T[] {
    return this.items.value
  }

  /**
   * 更新滚动位置
   */
  onScroll(scrollTop: number): void {
    this.scrollTop.value = scrollTop
  }

  /**
   * 更新容器高度
   */
  onResize(height: number): void {
    this.containerHeight.value = height
  }

  /**
   * 计算可见范围
   */
  visibleRange = computed(() => {
    const itemHeight = this.config.itemHeight
    const overscan = this.config.overscan

    const start = Math.floor(this.scrollTop.value / itemHeight)
    const visibleCount = Math.ceil(this.containerHeight.value / itemHeight)

    return {
      start: Math.max(0, start - overscan),
      end: Math.min(
        this.items.value.length,
        start + visibleCount + overscan
      )
    }
  })

  /**
   * 获取可见项
   */
  visibleItems = computed<VisibleItem<T>[]>(() => {
    const { start, end } = this.visibleRange.value
    const itemHeight = this.config.itemHeight

    return this.items.value.slice(start, end).map((item, index) => ({
      item,
      index: start + index,
      style: {
        position: 'absolute',
        top: `${(start + index) * itemHeight}px`,
        height: `${itemHeight}px`,
        left: '0',
        right: '0'
      }
    }))
  })

  /**
   * 计算总高度
   */
  totalHeight = computed(() => {
    return this.items.value.length * this.config.itemHeight
  })

  /**
   * 滚动到指定索引
   */
  scrollToIndex(index: number, behavior: ScrollBehavior = 'smooth'): void {
    const top = index * this.config.itemHeight
    this.onScroll(top)
  }

  /**
   * 滚动到指定项
   */
  scrollToItem(itemId: string, getId: (item: T) => string): void {
    const index = this.items.value.findIndex(item => getId(item) === itemId)
    if (index !== -1) {
      this.scrollToIndex(index)
    }
  }

  /**
   * 获取项的偏移位置
   */
  getItemOffset(index: number): number {
    return index * this.config.itemHeight
  }

  /**
   * 获取索引在视口中的位置
   */
  getIndexAtOffset(offset: number): number {
    return Math.floor(offset / this.config.itemHeight)
  }

  /**
   * 判断索引是否在可见范围内
   */
  isIndexVisible(index: number): boolean {
    const { start, end } = this.visibleRange.value
    return index >= start && index < end
  }

  /**
   * 获取状态
   */
  getState(): VirtualViewportState {
    return {
      scrollTop: this.scrollTop,
      containerHeight: this.containerHeight,
      visibleRange: this.visibleRange,
      totalHeight: this.totalHeight
    }
  }
}

/**
 * 创建虚拟视口
 */
export function createVirtualViewport<T>(config: VirtualViewportConfig): VirtualViewport<T> {
  return new VirtualViewport<T>(config)
}
