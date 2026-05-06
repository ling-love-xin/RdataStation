<template>
  <div class="right-sidebar-placeholder">
    <div class="placeholder-content">
      <div class="placeholder-icon">
        <PanelRight :size="48" />
      </div>
      <h3>辅助面板</h3>
      <p>此处将显示 SQL 历史、列洞察等辅助信息</p>
      <div class="placeholder-tips">
        <p>提示：点击右侧活动栏图标切换面板</p>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { PanelRight } from 'lucide-vue-next'
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
