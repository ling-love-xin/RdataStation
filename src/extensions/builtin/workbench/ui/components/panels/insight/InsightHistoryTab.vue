<template>
  <div class="history-section">
    <div class="history-header">
      <span class="history-title">{{ t('resultPanel.insightHistory') }}</span>
      <NButton size="tiny" quaternary :loading="isLoading" @click="insightStore.loadHistory()">
        <template #icon><RefreshCw :size="13" /></template>
      </NButton>
    </div>

    <div v-if="insightStore.historyVersions.length === 0 && !isLoading" class="history-empty">
      {{ t('resultPanel.noHistory') }}
    </div>

    <div v-else class="history-list">
      <div
        v-for="entry in insightStore.historyVersions"
        :key="entry.version_id"
        class="history-entry"
        :class="{ 'is-active': insightStore.diffVersionId === entry.version_id }"
        @click="selectVersion(entry.version_id)"
      >
        <div class="history-entry-main">
          <span class="history-ts">{{ formatDate(entry.created_at) }}</span>
          <span class="history-type">{{ entry.rule_id || entry.analysis_type }}</span>
        </div>
        <span v-if="insightStore.diffVersionId === entry.version_id" class="history-badge">{{
          t('resultPanel.comparing')
        }}</span>
      </div>
    </div>

    <div v-if="insightStore.diffData" class="diff-panel">
      <div class="diff-header">
        <span class="diff-title">{{ t('resultPanel.diffResult') }}</span>
        <NButton size="tiny" quaternary @click="insightStore.clearDiff()">
          <template #icon><X :size="13" /></template>
        </NButton>
      </div>
      <div v-if="insightStore.diffColumns.length === 0" class="diff-empty">
        {{ t('resultPanel.noDiff') }}
      </div>
      <div v-else class="diff-grid">
        <div
          v-for="colName in insightStore.diffColumns"
          :key="colName"
          class="diff-row"
        >
          <span class="diff-key">{{ colName }}</span>
          <span class="diff-detail-text val-changed">{{
            insightStore.diffSummary[colName] || '—'
          }}</span>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { RefreshCw, X } from 'lucide-vue-next'
import { NButton } from 'naive-ui'
import { useI18n } from 'vue-i18n'

import { useInsightStore } from '../../../stores/insight-store'

const { t } = useI18n()
const insightStore = useInsightStore()

defineProps<{
  isLoading: boolean
}>()

function formatDate(ts: string): string {
  try {
    const d = new Date(ts)
    return d.toLocaleString()
  } catch {
    return ts
  }
}

function selectVersion(versionId: string) {
  insightStore.loadVersionDetail(versionId)
}
</script>

<style scoped>
.history-section { padding: 6px 0; }
.history-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 8px; }
.history-title { font-size: 12px; font-weight: 500; }
.history-empty { font-size: 11px; color: var(--text-tertiary, #666); text-align: center; padding: 20px 0; }

.history-list { max-height: 240px; overflow-y: auto; display: flex; flex-direction: column; gap: 3px; }
.history-entry {
  display: flex; justify-content: space-between; align-items: center;
  padding: 5px 8px; border-radius: 4px; cursor: pointer;
  border: 1px solid transparent;
}
.history-entry:hover { background: var(--bg-hover, rgba(255,255,255,0.05)); }
.history-entry.is-active { background: var(--bg-elevated, #2a2a2a); border-color: var(--primary-color, #0078d4); }
.history-entry-main { display: flex; gap: 8px; align-items: center; }
.history-ts { font-size: 10px; color: var(--text-tertiary, #666); }
.history-type { font-size: 11px; }
.history-badge { font-size: 10px; background: var(--primary-color, #0078d4); color: var(--color-bg-primary); padding: 1px 5px; border-radius: 3px; }

.diff-panel { margin-top: 12px; padding: 8px; background: var(--bg-elevated, #2a2a2a); border-radius: 6px; border: 1px solid var(--border-color, #333); }
.diff-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 8px; }
.diff-title { font-size: 12px; font-weight: 500; }
.diff-grid { display: flex; flex-direction: column; gap: 4px; margin-bottom: 8px; }
.diff-row { display: flex; gap: 8px; align-items: baseline; }
.diff-key { font-size: 11px; color: var(--text-secondary, #aaa); min-width: 80px; }
.diff-detail-text { font-size: 11px; font-family: var(--font-mono); }
.diff-empty { font-size: 11px; color: var(--text-tertiary, #666); text-align: center; padding: 8px 0; }
.val-changed { color: var(--brand-success); font-weight: 500; }
.val-same { color: var(--text-tertiary, #666); }
</style>