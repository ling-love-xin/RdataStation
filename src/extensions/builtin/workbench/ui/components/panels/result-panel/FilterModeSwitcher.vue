<template>
  <div class="filter-mode-switcher">
    <button
      v-for="mode in modes"
      :key="mode.key"
      :class="['mode-btn', { active: modelValue === mode.key }]"
      @click="emit('update:modelValue', mode.key)"
    >
      {{ mode.icon }} {{ mode.label }}
    </button>
  </div>
</template>

<script setup lang="ts">
export type FilterMode = 'quick' | 'sql' | 'duckdb'

interface ModeItem { key: FilterMode; icon: string; label: string }

defineProps<{ modelValue: FilterMode }>()
const emit = defineEmits<{ 'update:modelValue': [FilterMode] }>()

const modes: ModeItem[] = [
  { key: 'quick', icon: '🔍', label: '即时过滤' },
  { key: 'sql', icon: '🗄️', label: 'SQL过滤' },
  { key: 'duckdb', icon: '🧠', label: 'DuckDB分析' },
]
</script>

<style scoped>
.filter-mode-switcher {
  display: flex;
  align-items: center;
  height: 30px;
  padding: 0 4px;
  gap: 2px;
  background: var(--bg-secondary, #252526);
  border-bottom: 1px solid var(--border-color, #333);
  flex-shrink: 0;
}
.mode-btn {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 3px 10px;
  font-size: 12px;
  border: none;
  background: transparent;
  color: var(--text-secondary, #888);
  cursor: pointer;
  border-radius: 3px;
  transition: all 0.15s;
}
.mode-btn:hover { background: var(--bg-hover, #333); color: var(--text-primary); }
.mode-btn.active {
  background: var(--primary-color, #0078d4);
  color: #fff;
}
</style>
