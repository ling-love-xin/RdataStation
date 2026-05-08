<template>
  <div class="status-bar">
    <div class="status-left">
      <span class="status-item">
        <span class="status-dot builtin" />
        {{ t('workbench.duckdbAccelerated') }}
      </span>
      <span v-if="executionTime > 0" class="status-item">
        {{ t('workbench.duration') }}: {{ executionTime }}ms
      </span>
      <span v-if="rowCount !== undefined" class="status-item">
        {{ t('workbench.rowCount') }}: {{ rowCount }}
      </span>
      <CacheWarmingStatus />
    </div>
    <div class="status-right">
      <span class="status-item clickable" :title="t('settings.title')" @click="handleOpenSettings">
        <Settings :size="14" />
      </span>
      <span class="status-item">RdataStation • {{ t('workbench.wasmPluginVersion') }}</span>
    </div>
  </div>
</template>

<script setup lang="ts">
import { Settings } from 'lucide-vue-next'
import { useI18n } from 'vue-i18n'

import CacheWarmingStatus from '@/extensions/builtin/database/ui/components/cache-warming-status.vue'
import { useLayoutStore } from '@/extensions/builtin/workbench/ui/stores/layout-store'

const { t } = useI18n()
const layoutStore = useLayoutStore()

interface Props {
  executionTime?: number
  rowCount?: number
}

withDefaults(defineProps<Props>(), {
  executionTime: 0,
  rowCount: undefined,
})

function handleOpenSettings() {
  layoutStore.openCustomizeLayoutDialog()
}
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

.status-item.clickable {
  cursor: pointer;
  opacity: 0.8;
  transition: opacity 0.15s;
}

.status-item.clickable:hover {
  opacity: 1;
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
  background: var(--warning-color, #ff7d00);
}
</style>
