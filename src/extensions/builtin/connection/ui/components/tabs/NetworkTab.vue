<template>
  <div class="tab-pane-placeholder">
    <div class="placeholder-note">{{ $t('connection.tabNetwork') }} — {{ $t('connection.comingSoon') }}</div>
  </div>
</template>

<script setup lang="ts">
interface Props {
  dbType?: string
  formData?: Record<string, unknown>
}

defineProps<Props>()

interface Emits {
  (e: 'update:config', config: Record<string, unknown>): void
}

defineEmits<Emits>()
</script>

<style scoped>
.tab-pane-placeholder {
  padding-top: var(--spacing-sm);
}

.placeholder-note {
  font-size: var(--font-size-sm);
  color: var(--color-text-muted);
  padding: var(--spacing-md) 0;
}
</style>