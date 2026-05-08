<template>
  <div ref="containerRef" class="three-column-layout">
    <div
      class="column column-a"
      :style="{
        width: `${columnAWidth}px`,
        minWidth: `${minWidthA}px`,
        maxWidth: `${maxWidthA}px`,
      }"
    >
      <slot name="column-a"></slot>
    </div>

    <div
      v-if="showLeftDivider"
      class="divider divider-left"
      @mousedown="startResize('left', $event)"
    >
      <div class="divider-handle"></div>
    </div>

    <div class="column column-b" :style="{ width: `${columnBWidth}px` }">
      <slot name="column-b"></slot>
    </div>

    <div
      v-if="showRightDivider"
      class="divider divider-right"
      @mousedown="startResize('right', $event)"
    >
      <div class="divider-handle"></div>
    </div>

    <div
      class="column column-c"
      :style="{
        width: `${columnCWidth}px`,
        minWidth: `${minWidthC}px`,
        maxWidth: `${maxWidthC}px`,
      }"
    >
      <slot name="column-c"></slot>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted, watch } from 'vue'

interface Props {
  minWidthA?: number
  maxWidthA?: number
  minWidthB?: number
  maxWidthB?: number
  minWidthC?: number
  maxWidthC?: number
  defaultRatio?: [number, number, number]
  showLeftDivider?: boolean
  showRightDivider?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  minWidthA: 150,
  maxWidthA: 500,
  minWidthB: 400,
  maxWidthB: 1200,
  minWidthC: 150,
  maxWidthC: 500,
  defaultRatio: () => [1, 2, 1],
  showLeftDivider: true,
  showRightDivider: true,
})

const containerRef = ref<HTMLElement | null>(null)
const containerWidth = ref(0)

// Column widths in pixels
const columnAWidth = ref(0)
const columnBWidth = ref(0)
const columnCWidth = ref(0)

// Resize state
const isResizing = ref(false)
const resizeTarget = ref<'left' | 'right' | null>(null)
const resizeStartX = ref(0)
const resizeStartWidth = ref(0)

// Initialize widths based on ratio
function initializeWidths() {
  if (!containerRef.value) return

  containerWidth.value = containerRef.value.offsetWidth
  const totalRatio = props.defaultRatio[0] + props.defaultRatio[1] + props.defaultRatio[2]
  const unitWidth = containerWidth.value / totalRatio

  columnAWidth.value = Math.round(unitWidth * props.defaultRatio[0])
  columnBWidth.value = Math.round(unitWidth * props.defaultRatio[1])
  columnCWidth.value = Math.round(unitWidth * props.defaultRatio[2])

  // Apply constraints
  columnAWidth.value = Math.max(props.minWidthA, Math.min(props.maxWidthA, columnAWidth.value))
  columnCWidth.value = Math.max(props.minWidthC, Math.min(props.maxWidthC, columnCWidth.value))
  columnBWidth.value = Math.max(props.minWidthB, Math.min(props.maxWidthB, columnBWidth.value))
}

function startResize(target: 'left' | 'right', event: MouseEvent) {
  isResizing.value = true
  resizeTarget.value = target
  resizeStartX.value = event.clientX

  if (target === 'left') {
    resizeStartWidth.value = columnAWidth.value
  } else {
    resizeStartWidth.value = columnCWidth.value
  }

  document.addEventListener('mousemove', handleMouseMove)
  document.addEventListener('mouseup', stopResize)
  document.body.style.cursor = 'col-resize'
  document.body.style.userSelect = 'none'
}

function handleMouseMove(event: MouseEvent) {
  if (!isResizing.value || !resizeTarget.value) return

  const deltaX = event.clientX - resizeStartX.value

  if (resizeTarget.value === 'left') {
    const newWidth = resizeStartWidth.value + deltaX
    columnAWidth.value = Math.max(props.minWidthA, Math.min(props.maxWidthA, newWidth))
  } else {
    const newWidth = resizeStartWidth.value + deltaX
    columnCWidth.value = Math.max(props.minWidthC, Math.min(props.maxWidthC, newWidth))
  }
}

function stopResize() {
  isResizing.value = false
  resizeTarget.value = null
  document.removeEventListener('mousemove', handleMouseMove)
  document.removeEventListener('mouseup', stopResize)
  document.body.style.cursor = ''
  document.body.style.userSelect = ''
}

onMounted(() => {
  initializeWidths()
  window.addEventListener('resize', initializeWidths)
})

onUnmounted(() => {
  window.removeEventListener('resize', initializeWidths)
})

watch(
  () => props.defaultRatio,
  () => {
    initializeWidths()
  }
)
</script>

<style scoped>
.three-column-layout {
  display: flex;
  width: 100%;
  height: 100%;
  overflow: hidden;
}

.column {
  height: 100%;
  overflow: hidden;
  flex-shrink: 0;
}

.column-a {
  background-color: var(--bg-secondary, #252526);
}

.column-b {
  flex: 1;
  background-color: var(--bg-primary, #1e1e1e);
}

.column-c {
  background-color: var(--bg-secondary, #252526);
}

.divider {
  width: 4px;
  height: 100%;
  background-color: var(--border-color, #3e3e42);
  cursor: col-resize;
  flex-shrink: 0;
  position: relative;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: background-color 0.15s ease;
}

.divider:hover {
  background-color: var(--primary-color, #165dff);
}

.divider-handle {
  width: 2px;
  height: 32px;
  background-color: var(--text-tertiary, #858585);
  border-radius: 1px;
  opacity: 0.5;
}

.divider:hover .divider-handle {
  background-color: white;
  opacity: 1;
}
</style>
