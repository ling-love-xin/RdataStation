<template>
  <div class="tab-pane-placeholder">
    <div class="placeholder-note">{{ $t('connection.tabDriverProps') }} — {{ $t('connection.comingSoon') }}</div>
  </div>
</template>

<script setup lang="ts">
interface Props {
  driverId?: string
}

defineProps<Props>()

interface Emits {
  (e: 'update:props', props: Record<string, string>): void
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