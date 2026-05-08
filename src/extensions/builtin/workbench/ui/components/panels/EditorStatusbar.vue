<template>
  <div class="editor-statusbar">
    <div class="status-left">
      <span class="status-item status-location">
        {{ $t('sqlEditor.statusLine') }} {{ cursorPosition }}
      </span>
      <span v-if="selectedTextInfo" class="status-item status-selection">
        {{ selectedTextInfo }}
      </span>
      <span class="status-divider" />
      <span class="status-item status-mode">{{ editorMode }}</span>
      <span v-if="isDirty" class="status-item status-dirty">● {{ $t('sqlEditor.unsaved') }}</span>
      <span v-if="executing" class="status-item status-executing">
        <span class="executing-dot" />
        {{ $t('sqlEditor.executing') }}
        <NButton
          v-if="canCancel"
          quaternary
          size="tiny"
          class="cancel-btn"
          @click="$emit('cancel')"
        >
          <X :size="12" />
        </NButton>
      </span>
      <span v-else-if="lastExecutionTime !== null" class="status-item status-time">
        {{ lastExecutionTime }}ms
      </span>
      <span v-if="statementCount > 0" class="status-item status-statements">
        {{ statementCount }} {{ statementCount === 1 ? 'statement' : 'statements' }}
      </span>
    </div>

    <div class="status-center">
      <NPopselect
        :options="popselectOptions"
        :value="selectedConnection"
        size="tiny"
        :render-label="renderConnectionLabel"
        trigger="click"
        @update:value="$emit('connectionChange', $event)"
      >
        <NButton quaternary size="tiny" class="status-connection-btn">
          {{ connectionInfoText || $t('sqlEditor.statusNoConnection') }}
        </NButton>
      </NPopselect>
    </div>

    <div class="status-right">
      <template v-if="inTransaction">
        <span class="status-item status-transaction">
          <span class="tx-dot" />
          TX
        </span>
        <NButton quaternary size="tiny" class="tx-btn tx-commit" @click="$emit('commit')">
          {{ $t('sqlEditor.commit') }}
        </NButton>
        <NButton quaternary size="tiny" class="tx-btn tx-rollback" @click="$emit('rollback')">
          {{ $t('sqlEditor.rollback') }}
        </NButton>
      </template>
      <slot name="right" />
    </div>
  </div>
</template>

<script setup lang="ts">
import { X } from 'lucide-vue-next'
import { NButton, NPopselect } from 'naive-ui'

interface Props {
  cursorPosition: string
  selectedTextInfo: string
  editorMode: string
  executing: boolean
  canCancel: boolean
  lastExecutionTime: number | null
  connectionInfoText: string
  popselectOptions: Array<{ label: string; value: string }>
  selectedConnection: string
  inTransaction: boolean
  statementCount: number
  isDirty?: boolean
}

defineProps<Props>()

interface Emits {
  (e: 'connectionChange', connId: string): void
  (e: 'cancel'): void
  (e: 'commit'): void
  (e: 'rollback'): void
}

defineEmits<Emits>()

function renderConnectionLabel(option: { label: string; value: string }): string {
  return option.label
}
</script>

<style scoped>
.editor-statusbar {
  display: flex;
  align-items: center;
  height: 24px;
  padding: 0 8px;
  background: var(--bg-secondary, #252526);
  border-top: 1px solid var(--border-color, #3e3e42);
  font-size: 12px;
  color: var(--text-secondary, #858585);
  gap: 0;
}

.status-left {
  display: flex;
  align-items: center;
  gap: 8px;
  flex: 1;
}

.status-center {
  display: flex;
  align-items: center;
}

.status-right {
  display: flex;
  align-items: center;
  gap: 8px;
}

.status-item {
  white-space: nowrap;
}

.status-divider {
  width: 1px;
  height: 14px;
  background: var(--border-color, #3e3e42);
  margin: 0 2px;
}

.status-executing {
  display: flex;
  align-items: center;
  gap: 4px;
  color: #e17055;
}

.executing-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: #e17055;
  animation: pulse 1.5s ease-in-out infinite;
}

@keyframes pulse {
  0%,
  100% {
    opacity: 1;
  }
  50% {
    opacity: 0.3;
  }
}

.status-connection-btn {
  font-size: 12px;
  color: var(--text-secondary, #858585);
}

.status-connection-btn:hover {
  color: var(--text-primary, #cccccc);
}

.status-transaction {
  display: flex;
  align-items: center;
  gap: 4px;
  color: #00b894;
  font-weight: 600;
}

.tx-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: #00b894;
}

.tx-btn {
  font-size: 11px;
  padding: 0 4px;
  height: 18px;
}

.tx-commit {
  color: #00b894;
}

.tx-rollback {
  color: #e17055;
}

.status-dirty {
  color: #f0c040;
  font-weight: 600;
}
</style>
