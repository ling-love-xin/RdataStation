<template>
  <div class="search-bar">
    <div class="search-input-wrapper">
      <span class="search-icon">🔍</span>
      <input
        ref="inputRef"
        v-model="localQuery"
        type="text"
        class="search-input"
        placeholder="搜索资源名称..."
        @input="handleInput"
        @keydown="handleKeyDown"
      />
      <button
        v-if="localQuery"
        class="clear-btn"
        @click="clearSearch"
      >
        ✕
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch, onMounted, onUnmounted } from 'vue'

const props = withDefaults(defineProps<{
  modelValue?: string
  debounceMs?: number
}>(), {
  modelValue: '',
  debounceMs: 300,
})

const emit = defineEmits<{
  'update:modelValue': [value: string]
  'search': [value: string]
  'clear': []
}>()

const inputRef = ref<HTMLInputElement | null>(null)
const localQuery = ref(props.modelValue)
let debounceTimer: ReturnType<typeof setTimeout> | null = null

watch(() => props.modelValue, (newVal) => {
  localQuery.value = newVal
})

function handleInput() {
  if (debounceTimer) {
    clearTimeout(debounceTimer)
  }
  debounceTimer = setTimeout(() => {
    emit('update:modelValue', localQuery.value)
    emit('search', localQuery.value)
  }, props.debounceMs)
}

function clearSearch() {
  localQuery.value = ''
  emit('update:modelValue', '')
  emit('clear')
  inputRef.value?.focus()
}

function handleKeyDown(e: KeyboardEvent) {
  if (e.key === 'Escape') {
    clearSearch()
  }
}

function focus() {
  inputRef.value?.focus()
}

function handleGlobalKeyDown(e: KeyboardEvent) {
  if ((e.ctrlKey || e.metaKey) && e.key === 'f') {
    e.preventDefault()
    focus()
  }
}

onMounted(() => {
  document.addEventListener('keydown', handleGlobalKeyDown)
})

onUnmounted(() => {
  document.removeEventListener('keydown', handleGlobalKeyDown)
  if (debounceTimer) {
    clearTimeout(debounceTimer)
  }
})

defineExpose({ focus })
</script>

<style scoped>
.search-bar {
  padding: var(--size-md) var(--size-lg);
  border-bottom: 1px solid var(--border-color);
}

.search-input-wrapper {
  position: relative;
  display: flex;
  align-items: center;
}

.search-icon {
  position: absolute;
  left: 12px;
  font-size: 14px;
  color: var(--text-tertiary);
  pointer-events: none;
}

.search-input {
  width: 100%;
  padding: 6px 36px;
  border: 1px solid var(--border-color);
  border-radius: var(--radius-md);
  background: var(--bg-secondary);
  font-size: 13px;
  color: var(--text-primary);
  transition: all 0.2s;
  height: var(--height-input);
}

.search-input:focus {
  outline: none;
  border-color: var(--primary-color);
  background: var(--bg-primary);
  box-shadow: 0 0 0 2px var(--primary-light);
}

.search-input::placeholder {
  color: var(--text-tertiary);
}

.clear-btn {
  position: absolute;
  right: 8px;
  width: 24px;
  height: 24px;
  display: flex;
  align-items: center;
  justify-content: center;
  border: none;
  background: transparent;
  color: var(--text-tertiary);
  cursor: pointer;
  border-radius: var(--radius-sm);
  transition: all 0.15s;
}

.clear-btn:hover {
  background: var(--bg-hover);
  color: var(--text-primary);
}
</style>
