<template>
  <footer class="modal-footer">
    <div class="footer-left">
      <button
        v-if="showSidebarToggle"
        type="button"
        class="btn-secondary"
        @click="$emit('toggle-sidebar')"
      >
        <Grid :size="14" />
        {{ sidebarVisible ? '隐藏' : '显示' }}数据库列表
      </button>
    </div>
    <div class="footer-right">
      <div
        v-if="testResult"
        class="test-result"
        :class="testResult.success ? 'success' : 'error'"
      >
        <CheckCircle v-if="testResult.success" :size="14" />
        <XCircle v-else :size="14" />
        <span class="test-result-message">{{ testResult.message }}</span>
        <span v-if="testResult.responseTimeMs" class="test-result-time"
          >{{ testResult.responseTimeMs }}ms</span
        >
      </div>

      <button type="button" class="btn-secondary" @click="$emit('cancel')"> 取消 </button>
      <button
        v-if="showTestButton"
        type="button"
        class="btn-test"
        :disabled="testing"
        @click="$emit('test')"
      >
        <Loader2 v-if="testing" :size="14" class="animate-spin" />
        <TestTube v-else :size="14" />
        {{ testing ? '测试中...' : '测试连接' }}
      </button>
      <button type="button" class="btn-primary" :disabled="saving" @click="$emit('save')">
        <Check :size="14" />
        {{ saving ? '保存中...' : saveLabel }}
      </button>
    </div>
  </footer>
</template>

<script setup lang="ts">
import { Check, CheckCircle, Grid, Loader2, TestTube, XCircle } from 'lucide-vue-next'

interface TestResultDisplay {
  success: boolean
  message: string
  responseTimeMs?: number
}

interface Props {
  showSidebarToggle?: boolean
  showTestButton?: boolean
  sidebarVisible?: boolean
  testing?: boolean
  saving?: boolean
  saveLabel?: string
  testResult?: TestResultDisplay | null
}

withDefaults(defineProps<Props>(), {
  showSidebarToggle: false,
  showTestButton: false,
  sidebarVisible: true,
  testing: false,
  saving: false,
  saveLabel: '保存连接',
  testResult: null,
})

defineEmits<{
  'toggle-sidebar': []
  cancel: []
  test: []
  save: []
}>()
</script>

<style scoped>
.modal-footer {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px 24px;
  border-top: 1px solid var(--border-color);
  background: var(--bg-secondary);
}

.footer-left,
.footer-right {
  display: flex;
  gap: 8px;
  align-items: center;
}

.test-result {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 6px 12px;
  border-radius: var(--radius-md);
  font-size: 12px;
  font-weight: 500;
  margin-right: 8px;
  transition: all 0.2s;
}

.test-result.success {
  background: rgba(34, 197, 94, 0.1);
  color: #22c55e;
  border: 1px solid rgba(34, 197, 94, 0.3);
}

.test-result.error {
  background: rgba(239, 68, 68, 0.1);
  color: #ef4444;
  border: 1px solid rgba(239, 68, 68, 0.3);
}

.test-result-message {
  max-width: 200px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.test-result-time {
  opacity: 0.7;
  font-size: 11px;
}

.btn-secondary,
.btn-primary,
.btn-test {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 8px 16px;
  border-radius: var(--radius-md);
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s;
  border: none;
}

.btn-secondary {
  background: var(--bg-tertiary);
  color: var(--text-primary);
  border: 1px solid var(--border-color);
}

.btn-secondary:hover {
  background: var(--bg-hover);
}

.btn-test {
  background: var(--bg-tertiary);
  color: var(--warning-color);
  border: 1px solid var(--warning-color);
}

.btn-test:hover:not(:disabled) {
  background: var(--warning-color);
  color: white;
}

.btn-test:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.btn-primary {
  background: var(--primary-color);
  color: white;
}

.btn-primary:hover:not(:disabled) {
  background: var(--primary-dark);
}

.btn-primary:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

@keyframes spin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}

.animate-spin {
  animation: spin 1s linear infinite;
}
</style>