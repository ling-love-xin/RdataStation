<template>
  <div class="capabilities-tab">
    <div class="info-banner">
      <Info :size="15" class="banner-icon" />
      <span>{{ t('connection.capabilitiesTab.desc') }}</span>
    </div>
    <table class="caps-table">
      <thead>
        <tr>
          <th style="width: 35%">{{ t('connection.capabilitiesTab.capability') }}</th>
          <th style="width: 20%">{{ t('connection.capabilitiesTab.status') }}</th>
          <th>{{ t('connection.capabilitiesTab.remark') }}</th>
        </tr>
      </thead>
      <tbody>
        <tr v-for="cap in capabilityList" :key="cap.key">
          <td>{{ cap.label }}</td>
          <td>
            <span class="cap-badge" :class="cap.supported ? 'yes' : 'no'">
              {{ cap.supported ? '✓ ' + t('connection.capabilitiesTab.supported') : '✗ ' + t('connection.capabilitiesTab.unsupported') }}
            </span>
          </td>
          <td>{{ cap.description || '—' }}</td>
        </tr>
      </tbody>
    </table>
  </div>
</template>

<script setup lang="ts">
import { Info } from 'lucide-vue-next'
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'

import type { DriverDescriptor } from '../../types/connection'

const { t } = useI18n()

interface Props {
  selectedDriver: DriverDescriptor | null
}

const props = defineProps<Props>()

interface CapabilityItem {
  key: string
  label: string
  supported: boolean
  description: string
}

const CAPABILITY_DEFS: Record<string, { labelKey: string; deriveFrom: (d: DriverDescriptor) => boolean; descKey: string }> = {
  transactions: {
    labelKey: 'connection.capabilitiesTab.transactions',
    deriveFrom: () => true,
    descKey: 'connection.capabilitiesTab.transactionsDesc',
  },
  preparedStmt: {
    labelKey: 'connection.capabilitiesTab.preparedStmt',
    deriveFrom: () => true,
    descKey: 'connection.capabilitiesTab.preparedStmtDesc',
  },
  streaming: {
    labelKey: 'connection.capabilitiesTab.streaming',
    deriveFrom: (d) => !!d.features?.includes('streaming'),
    descKey: 'connection.capabilitiesTab.streamingDesc',
  },
  introspection: {
    labelKey: 'connection.capabilitiesTab.introspection',
    deriveFrom: (d) => !!d.features?.includes('introspection'),
    descKey: 'connection.capabilitiesTab.introspectionDesc',
  },
  arrowExport: {
    labelKey: 'connection.capabilitiesTab.arrowExport',
    deriveFrom: (d) => !!d.features?.includes('arrow'),
    descKey: 'connection.capabilitiesTab.arrowExportDesc',
  },
  federated: {
    labelKey: 'connection.capabilitiesTab.federated',
    deriveFrom: (d) => !!d.features?.includes('federated'),
    descKey: 'connection.capabilitiesTab.federatedDesc',
  },
}

const capabilityList = computed<CapabilityItem[]>(() => {
  const driver = props.selectedDriver
  if (!driver) {
    return Object.entries(CAPABILITY_DEFS).map(([key, def]) => ({
      key,
      label: t(def.labelKey),
      supported: false,
      description: t(def.descKey),
    }))
  }
  return Object.entries(CAPABILITY_DEFS).map(([key, def]) => ({
    key,
    label: t(def.labelKey),
    supported: def.deriveFrom(driver),
    description: t(def.descKey),
  }))
})
</script>

<style scoped>
.capabilities-tab {
  padding: 0;
}
.info-banner {
  display: flex;
  align-items: flex-start;
  gap: 8px;
  padding: 10px 14px;
  background: rgba(137,180,250,0.04);
  border: 1px solid rgba(137,180,250,0.1);
  border-radius: 6px;
  font-size: 12px;
  color: var(--color-accent, #89b4fa);
  margin-bottom: 16px;
  line-height: 1.5;
}
.banner-icon {
  flex-shrink: 0;
  margin-top: 1px;
}
.caps-table {
  width: 100%;
  border-collapse: collapse;
  border: 1px solid var(--color-border, rgba(255,255,255,0.07));
  border-radius: 8px;
  overflow: hidden;
  font-size: 12px;
}
.caps-table th {
  text-align: left;
  padding: 9px 14px;
  background: var(--color-bg-raised, #11111b);
  color: var(--color-text-muted, #6c7086);
  font-weight: 600;
  border-bottom: 1px solid var(--color-border, rgba(255,255,255,0.07));
}
.caps-table td {
  padding: 8px 14px;
  border-bottom: 1px solid var(--color-border, rgba(255,255,255,0.07));
  color: var(--color-text-secondary, #a6adc8);
}
.caps-table tr:last-child td {
  border-bottom: none;
}
.cap-badge {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  padding: 2px 8px;
  border-radius: 10px;
  font-size: 11px;
}
.cap-badge.yes {
  background: rgba(166,227,161,0.08);
  color: var(--color-success, #a6e3a1);
}
.cap-badge.no {
  background: rgba(243,139,168,0.08);
  color: var(--color-danger, #f38ba8);
}
</style>