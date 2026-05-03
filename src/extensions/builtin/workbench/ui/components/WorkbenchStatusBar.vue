<template>
  <div class="status-bar">
    <div class="status-left">
      <span class="status-item">
        <span class="status-dot builtin" />
        DuckDB 内置加速已启用
      </span>
      <span v-if="executionTime > 0" class="status-item">
        耗时: {{ executionTime }}ms
      </span>
      <span v-if="rowCount !== undefined" class="status-item">
        行数: {{ rowCount }}
      </span>
      <CacheWarmingStatus />
    </div>
    <div class="status-right">
      <span class="status-item">RdataStation • Wasm 插件版</span>
    </div>
  </div>
</template>

<script setup lang="ts">
import CacheWarmingStatus from '@/extensions/builtin/database/ui/components/cache-warming-status.vue'

interface Props {
  executionTime?: number
  rowCount?: number
}

withDefaults(defineProps<Props>(), {
  executionTime: 0,
  rowCount: undefined
})
</script>

<style scoped>
.status-bar {
  height: 24px;
  background: var(--primary-color);
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 12px;
  font-size: 12px;
  color: white;
  flex-shrink: 0;
}

.status-left,
.status-right {
  display: flex;
  align-items: center;
  gap: 16px;
}

.status-item {
  display: flex;
  align-items: center;
  gap: 6px;
}

.status-item.connection {
  background: rgba(255, 255, 255, 0.2);
  padding: 2px 8px;
  border-radius: 3px;
}

.status-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
}

.status-dot.builtin {
  background: var(--warning-color, #f59e0b);
}
</style>
