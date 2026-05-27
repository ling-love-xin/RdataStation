<template>
  <div class="cap-inline">
    <table class="tab-table">
      <tbody>
        <tr v-for="cap in capabilityList" :key="cap.key">
          <td class="cap-key">
            <span class="cap-badge" :class="cap.supported ? 'supported' : 'unsupported'">
              {{ cap.supported ? '✓' : '✗' }}
            </span>
            {{ cap.label }}
          </td>
          <td class="cap-desc">{{ cap.description }}</td>
        </tr>
      </tbody>
    </table>
  </div>
</template>

<script setup lang="ts">
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

const CAPABILITY_DEFS: Record<string, {
  labelKey: string
  deriveFrom: (d: DriverDescriptor) => boolean
  descKey: string
}> = {
  tree: {
    labelKey: 'connection.capabilitiesTab.tree',
    deriveFrom: () => true,
    descKey: 'connection.capabilitiesTab.treeDesc',
  },
  health_check: {
    labelKey: 'connection.capabilitiesTab.healthCheck',
    deriveFrom: () => true,
    descKey: 'connection.capabilitiesTab.healthCheckDesc',
  },
  transactions: {
    labelKey: 'connection.capabilitiesTab.transactions',
    deriveFrom: () => true,
    descKey: 'connection.capabilitiesTab.transactionsDesc',
  },
  index_analysis: {
    labelKey: 'connection.capabilitiesTab.indexAnalysis',
    deriveFrom: () => true,
    descKey: 'connection.capabilitiesTab.indexAnalysisDesc',
  },
  sql_autocomplete: {
    labelKey: 'connection.capabilitiesTab.sqlAutocomplete',
    deriveFrom: () => true,
    descKey: 'connection.capabilitiesTab.sqlAutocompleteDesc',
  },
  table_editor: {
    labelKey: 'connection.capabilitiesTab.tableEditor',
    deriveFrom: () => true,
    descKey: 'connection.capabilitiesTab.tableEditorDesc',
  },
  schema_browser: {
    labelKey: 'connection.capabilitiesTab.schemaBrowser',
    deriveFrom: () => true,
    descKey: 'connection.capabilitiesTab.schemaBrowserDesc',
  },
  analytics: {
    labelKey: 'connection.capabilitiesTab.analytics',
    deriveFrom: () => true,
    descKey: 'connection.capabilitiesTab.analyticsDesc',
  },
  federation: {
    labelKey: 'connection.capabilitiesTab.federation',
    deriveFrom: () => true,
    descKey: 'connection.capabilitiesTab.federationDesc',
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
.cap-inline {
  overflow-x: auto;
}

.cap-inline .tab-table th,
.cap-inline .tab-table td {
  padding: var(--spacing-xs) var(--spacing-md);
}

.cap-key {
  display: flex;
  align-items: center;
  gap: var(--spacing-xs);
  font-weight: 500;
}

.cap-desc {
  color: var(--color-text-muted);
  font-size: var(--font-size-sm);
}

.cap-badge {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 18px;
  height: 18px;
  border-radius: 50%;
  font-size: var(--font-size-xs);
  font-weight: 700;
  flex-shrink: 0;
}

.cap-badge.supported {
  background: var(--color-bg-elevated);
  color: var(--brand-success);
}

.cap-badge.unsupported {
  background: var(--color-bg-elevated);
  color: var(--color-text-muted);
}
</style>