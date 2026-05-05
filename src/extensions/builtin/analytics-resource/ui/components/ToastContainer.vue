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
  gap: var(--size-sm);
  pointer-events: none;
}

.toast {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: var(--size-md) var(--size-lg);
  border-radius: var(--radius-md);
  background: var(--bg-primary);
  border: 1px solid var(--border-color);
  box-shadow: var(--shadow-md);
  cursor: pointer;
  pointer-events: auto;
  max-width: 360px;
  transition: all 0.3s ease;
}

.toast:hover {
  transform: translateX(-4px);
}

.toast-icon {
  font-size: 16px;
  flex-shrink: 0;
}

.toast-message {
  font-size: 13px;
  color: var(--text-primary);
  line-height: 1.4;
}

.toast-success {
  border-left: 3px solid var(--success-color);
}

.toast-error {
  border-left: 3px solid var(--danger-color);
}

.toast-warning {
  border-left: 3px solid var(--warning-color);
}

.toast-info {
  border-left: 3px solid var(--info-color);
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
