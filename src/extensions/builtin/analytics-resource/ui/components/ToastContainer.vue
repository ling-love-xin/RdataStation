<template>
  <Teleport to="body">
    <div class="toast-container">
      <TransitionGroup name="toast">
        <div
          v-for="toast in toasts"
          :key="toast.id"
          :class="['toast', `toast-${toast.type}`]"
          @click="remove(toast.id)"
        >
          <span class="toast-icon">{{ getIcon(toast.type) }}</span>
          <span class="toast-message">{{ toast.message }}</span>
        </div>
      </TransitionGroup>
    </div>
  </Teleport>
</template>

<script setup lang="ts">
import { useToast } from '../composables/use-toast'

const { toasts, remove } = useToast()

function getIcon(type: string) {
  switch (type) {
    case 'success': return '✅'
    case 'error': return '❌'
    case 'warning': return '⚠️'
    case 'info': return 'ℹ️'
    default: return 'ℹ️'
  }
}
</script>

<style scoped>
.toast-container {
  position: fixed;
  top: 60px;
  right: 20px;
  z-index: 9999;
  display: flex;
  flex-direction: column;
  gap: 8px;
  pointer-events: none;
}

.toast {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 12px 16px;
  border-radius: 8px;
  background: var(--color-background-elevated, #fff);
  border: 1px solid var(--border-color, #d9d9d9);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
  cursor: pointer;
  pointer-events: auto;
  max-width: 360px;
  transition: all 0.3s ease;
}

.toast:hover {
  transform: translateX(-4px);
}

.toast-icon {
  font-size: 18px;
  flex-shrink: 0;
}

.toast-message {
  font-size: 14px;
  color: var(--text-primary, #333);
  line-height: 1.4;
}

.toast-success {
  border-left: 4px solid #00b42a;
}

.toast-error {
  border-left: 4px solid #f53f3f;
}

.toast-warning {
  border-left: 4px solid #ff7d00;
}

.toast-info {
  border-left: 4px solid #165dff;
}

.toast-enter-from {
  opacity: 0;
  transform: translateX(100%);
}

.toast-leave-to {
  opacity: 0;
  transform: translateX(100%);
}

.toast-move {
  transition: transform 0.3s ease;
}
</style>
