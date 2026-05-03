<template>
  <div class="navigator-status">
    <span class="status-text">{{ text }}</span>
  </div>
</template>

<script setup lang="ts">
defineProps<{
  text: string
}>()
</script>

<style scoped>
.navigator-status {
  padding: 4px 8px;
  background: var(--bg-tertiary);
  border-top: 1px solid var(--border-color);
  font-size: 11px;
  color: var(--text-secondary);
}

.status-text {
  white-space: nowrap;
}
</style>
