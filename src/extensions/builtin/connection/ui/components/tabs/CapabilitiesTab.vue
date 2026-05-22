<template>
  <div class="cap-tab">
    <NAlert type="info" :bordered="false" class="cap-banner">
      {{ $t('connection.capabilitiesTab.desc') }}
    </NAlert>

    <div v-if="!driver" class="empty-hint">{{ $t('navigator.noDriver') }}</div>

    <div v-else class="cap-table">
      <div class="cap-thead">
        <span class="cap-th cap-name">{{ $t('navigator.capability') }}</span>
        <span class="cap-th cap-status">{{ $t('navigator.status') }}</span>
        <span class="cap-th cap-desc-col">{{ $t('navigator.description') }}</span>
      </div>
      <div v-for="cap in capList" :key="cap.key" class="cap-row">
        <span class="cap-td cap-name">{{ cap.label }}</span>
        <span class="cap-td cap-status">
          <span :class="['cap-badge', cap.has ? 'yes' : 'no']">
            {{ cap.has ? '✓ ' + $t('connection.capabilitiesTab.supported') : '✗ ' + $t('connection.capabilitiesTab.unsupported') }}
          </span>
        </span>
        <span class="cap-td cap-desc-col">{{ cap.desc }}</span>
      </div>
      <div v-if="capList.length === 0" class="cap-row-empty">
        {{ $t('navigator.noCapabilities') }}
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { NAlert } from 'naive-ui'
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'

import type { Driver } from '../../../domain/types'

const { t } = useI18n()

interface Props { driver: Driver | null }
const props = withDefaults(defineProps<Props>(), { driver: null })

interface CapItem { key: string; label: string; has: boolean; desc: string }

const CAP_META: Record<string, { label: string; desc: string }> = {
  transactions:     { label: t('connection.capabilitiesTab.transactions'),     desc: t('connection.capabilitiesTab.transactionsDesc') },
  preparedStmt:     { label: t('connection.capabilitiesTab.preparedStmt'),     desc: t('connection.capabilitiesTab.preparedStmtDesc') },
  streaming:        { label: t('connection.capabilitiesTab.streaming'),        desc: t('connection.capabilitiesTab.streamingDesc') },
  introspection:    { label: t('connection.capabilitiesTab.introspection'),    desc: t('connection.capabilitiesTab.introspectionDesc') },
  arrowExport:      { label: t('connection.capabilitiesTab.arrowExport'),      desc: t('connection.capabilitiesTab.arrowExportDesc') },
  pluginCompat:     { label: t('connection.capabilitiesTab.pluginCompat'),     desc: t('connection.capabilitiesTab.pluginCompatDesc') },
}

const parsedCaps = computed<string[]>(() => {
  if (!props.driver?.capabilities) return []
  try {
    const arr = JSON.parse(props.driver.capabilities)
    return Array.isArray(arr) ? arr.filter((c): c is string => typeof c === 'string') : []
  } catch { return [] }
})

const capList = computed<CapItem[]>(() => {
  const caps = parsedCaps.value
  const knownKeys = caps.filter((c) => c in CAP_META)
  if (knownKeys.length > 0) {
    return knownKeys.map((c) => ({
      key: c, label: CAP_META[c]?.label || c, has: true, desc: CAP_META[c]?.desc || '—',
    }))
  }
  const set = new Set(caps)
  return Object.entries(CAP_META).map(([key, meta]) => ({
    key, label: meta.label, has: set.has(key), desc: meta.desc,
  }))
})
</script>

<style scoped>
.cap-tab { display: flex; flex-direction: column; gap: 12px; padding: 4px 0; }
.cap-banner { border-radius: 6px; }
.empty-hint { display: flex; align-items: center; justify-content: center; height: 120px; font-size: 13px; color: var(--color-text-muted); }
.cap-table { border: 1px solid var(--color-border-subtle); border-radius: 8px; overflow: hidden; font-size: 12px; }
.cap-thead { display: flex; background: var(--color-bg-elevated); padding: 10px 14px; border-bottom: 1px solid var(--color-border-subtle); }
.cap-th { font-weight: 600; color: var(--color-text-muted); }
.cap-row { display: flex; padding: 8px 14px; border-bottom: 1px solid var(--color-border-subtle); }
.cap-row:last-child { border-bottom: none; }
.cap-row-empty { padding: 16px; text-align: center; color: var(--color-text-muted); }
.cap-td { display: flex; align-items: center; }
.cap-name { width: 35%; font-weight: 500; color: var(--color-text-secondary); }
.cap-status { width: 20%; }
.cap-desc-col { flex: 1; color: var(--color-text-muted); font-size: 11px; }
.cap-badge { display: inline-flex; align-items: center; gap: 3px; padding: 2px 8px; border-radius: 10px; font-size: 11px; }
.cap-badge.yes { background: rgba(166,227,161,0.08); color: var(--brand-success); }
.cap-badge.no { background: rgba(243,139,168,0.08); color: var(--brand-danger); }
</style>