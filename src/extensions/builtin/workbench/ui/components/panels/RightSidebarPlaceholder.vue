<template>
  <div class="right-sidebar-placeholder">
    <div class="placeholder-content">
      <div class="placeholder-icon">
        <PanelRight :size="48" />
      </div>
      <h3>{{ t('workbench.auxiliaryPanel') }}</h3>
      <p>{{ t('workbench.auxiliaryPanelDesc') }}</p>
      <div class="placeholder-tips">
        <p>{{ t('workbench.panelTip') }}</p>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { PanelRight } from 'lucide-vue-next'
import { useI18n } from 'vue-i18n'

const { t } = useI18n()
</script>

<style scoped>
.right-sidebar-placeholder {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 100%;
  padding: 24px;
  background: var(--bg-secondary);
}

.placeholder-content {
  text-align: center;
  max-width: 280px;
}

.placeholder-icon {
  width: 64px;
  height: 64px;
  margin: 0 auto 16px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 12px;
  background: var(--bg-tertiary);
  color: var(--text-tertiary);
}

.placeholder-content h3 {
  margin: 0 0 8px 0;
  font-size: 16px;
  font-weight: 600;
  color: var(--text-primary);
}

.placeholder-content p {
  margin: 0 0 16px 0;
  font-size: 13px;
  color: var(--text-secondary);
  line-height: 1.5;
}

.placeholder-tips {
  padding: 12px;
  background: var(--bg-tertiary);
  border-radius: 6px;
  border: 1px solid var(--border-color);
}

.placeholder-tips p {
  margin: 0;
  font-size: 12px;
  color: var(--text-tertiary);
}
</style>
