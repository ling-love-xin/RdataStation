/**
 * 虚拟滚动核心逻辑
 * 
 * 计算可见区域，只渲染可见范围内的节点
 */

import { computed, type Ref } from 'vue'

export interface UseVirtualScrollOptions {
  /** 容器高度 */
  containerHeight: Ref<number>
  /** 滚动位置 */
  scrollTop: Ref<number>
  /** 每项高度 */
  itemHeight: number
  /** 总项数 */
  totalItems: Ref<number>
  /** 缓冲区大小（上下各缓冲几项） */
  bufferSize?: number
}

export function useVirtualScroll(options: UseVirtualScrollOptions) {
  const { containerHeight, scrollTop, itemHeight, totalItems, bufferSize = 3 } = options

  // 总高度
  const totalHeight = computed(() => totalItems.value * itemHeight)

  // 可见区域起始索引（包含缓冲）
  const visibleStart = computed(() => {
    const start = Math.floor(scrollTop.value / itemHeight)
    return Math.max(0, start - bufferSize)
  })

  // 可见区域结束索引（包含缓冲）
  const visibleEnd = computed(() => {
    const end = Math.ceil((scrollTop.value + containerHeight.value) / itemHeight)
    return Math.min(totalItems.value, end + bufferSize)
  })

  // 可见项数量
  const visibleCount = computed(() => visibleEnd.value - visibleStart.value)

  // 偏移量（用于 translateY）
  const offsetY = computed(() => visibleStart.value * itemHeight)

  return {
    totalHeight,
    visibleStart,
    visibleEnd,
    visibleCount,
    offsetY
  }
}
