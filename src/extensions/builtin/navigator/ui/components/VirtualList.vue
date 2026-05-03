<template>
  <div
    ref="containerRef"
    class="virtual-list"
    :style="{ height: `${props.height}px`, overflow: 'auto' }"
    @scroll="handleScroll"
  >
    <div
      class="virtual-list-content"
      :style="{ height: `${totalHeight}px`, position: 'relative' }"
    >
      <div
        v-for="item in visibleItems"
        :key="item.key"
        class="virtual-list-item"
        :style="item.style"
      >
        <slot
          name="item"
          :item="item.data"
          :index="item.index"
        />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts" generic="T">
import { ref, computed, onMounted, onUnmounted, watch } from 'vue'

interface Props<T> {
  items: T[]
  itemHeight: number
  height: number
  overscan?: number
  getItemKey: (item: T, index: number) => string
}

const props = withDefaults(defineProps<Props<T>>(), {
  overscan: 5
})

const emit = defineEmits<{
  scroll: [scrollTop: number]
}>()

const containerRef = ref<HTMLElement>()
const scrollTop = ref(0)
const containerHeight = ref(props.height)

// 计算可见范围
const visibleRange = computed(() => {
  const start = Math.floor(scrollTop.value / props.itemHeight)
  const visibleCount = Math.ceil(containerHeight.value / props.itemHeight)

  return {
    start: Math.max(0, start - props.overscan),
    end: Math.min(
      props.items.length,
      start + visibleCount + props.overscan
    )
  }
})

// 计算总高度
const totalHeight = computed(() => {
  return props.items.length * props.itemHeight
})

// 计算可见项
const visibleItems = computed(() => {
  const { start, end } = visibleRange.value

  return props.items.slice(start, end).map((item, index) => {
        const actualIndex = start + index
        return {
          data: item,
          index: actualIndex,
          key: props.getItemKey(item, actualIndex),
          style: {
            position: 'absolute' as const,
            top: `${actualIndex * props.itemHeight}px`,
            height: `${props.itemHeight}px`,
            left: '0',
            right: '0'
          }
        }
  })
})

// 处理滚动
const handleScroll = (e: Event) => {
  const target = e.target as HTMLElement
  scrollTop.value = target.scrollTop
  emit('scroll', scrollTop.value)
}

// 滚动到指定索引
const scrollToIndex = (index: number) => {
  if (containerRef.value) {
    containerRef.value.scrollTop = index * props.itemHeight
  }
}

// 监听 items 变化
watch(() => props.items.length, () => {
  // 可以在这里处理数据变化后的逻辑
})

// ResizeObserver 监听容器大小变化
let resizeObserver: ResizeObserver | null = null

onMounted(() => {
  if (containerRef.value && typeof ResizeObserver !== 'undefined') {
    resizeObserver = new ResizeObserver((entries) => {
      for (const entry of entries) {
        containerHeight.value = entry.contentRect.height
      }
    })
    resizeObserver.observe(containerRef.value)
  }
})

onUnmounted(() => {
  resizeObserver?.disconnect()
})

// 暴露方法
defineExpose({
  scrollToIndex
})
</script>

<style scoped>
.virtual-list {
  width: 100%;
  position: relative;
}

.virtual-list-content {
  width: 100%;
}

.virtual-list-item {
  width: 100%;
  box-sizing: border-box;
}
</style>
