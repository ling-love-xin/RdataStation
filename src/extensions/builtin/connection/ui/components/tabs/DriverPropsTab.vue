<template>
  <div class="driver-props-tab">
    <div class="info-banner">
      <Settings :size="15" class="banner-icon" />
      <span>{{ t('connection.driverPropsTab.desc') }}</span>
    </div>
    <table class="props-table">
      <thead>
        <tr>
          <th style="width: 30%">{{ t('connection.driverPropsTab.property') }}</th>
          <th style="width: 35%">{{ t('connection.driverPropsTab.value') }}</th>
          <th>{{ t('connection.driverPropsTab.description') }}</th>
        </tr>
      </thead>
      <tbody>
        <tr v-if="!propertyList || propertyList.length === 0">
          <td colspan="3" style="text-align:center;padding:24px;color:var(--color-text-muted)">
            {{ t('connection.driverPropsTab.noProperties') }}
          </td>
        </tr>
        <tr v-for="prop in propertyList" :key="prop.name">
          <td><code>{{ prop.name }}</code></td>
          <td>
            <input
              v-model="propValues[prop.name]"
              type="text"
              class="prop-input"
              :placeholder="String(prop.defaultValue || '')"
            />
          </td>
          <td>{{ prop.description || '—' }}</td>
        </tr>
      </tbody>
    </table>
  </div>
</template>

<script setup lang="ts">
import { Settings } from 'lucide-vue-next'
import { computed, reactive, watch } from 'vue'
import { useI18n } from 'vue-i18n'

import type { DriverDescriptor, DriverOption } from '../../types/connection'

const { t } = useI18n()

interface Props {
  selectedDriver: DriverDescriptor | null
}

const props = defineProps<Props>()

const propValues = reactive<Record<string, string>>({})

interface PropertyItem {
  name: string
  defaultValue?: string
  description?: string
  optionType?: string
}

const propertyList = computed<PropertyItem[]>(() => {
  const driver = props.selectedDriver
  if (!driver) return []
  const extraOptions = (driver.extraOptions || driver.extra_options || []) as DriverOption[]
  return extraOptions.map((opt) => ({
    name: opt.name,
    defaultValue: opt.defaultValue,
    description: opt.description,
    optionType: opt.optionType,
  }))
})

watch(
  () => propertyList.value,
  (list) => {
    const vals: Record<string, string> = {}
    for (const prop of list) {
      vals[prop.name] = propValues[prop.name] || prop.defaultValue || ''
    }
    Object.keys(propValues).forEach((k) => {
      if (!list.find((p) => p.name === k)) {
        delete propValues[k]
      }
    })
    Object.assign(propValues, vals)
  },
  { immediate: true }
)
</script>

<style scoped>
.driver-props-tab {
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
.props-table {
  width: 100%;
  border-collapse: collapse;
  border: 1px solid var(--color-border, rgba(255,255,255,0.07));
  border-radius: 8px;
  overflow: hidden;
  font-size: 12px;
}
.props-table th {
  text-align: left;
  padding: 9px 14px;
  background: var(--color-bg-raised, #11111b);
  color: var(--color-text-muted, #6c7086);
  font-weight: 600;
  border-bottom: 1px solid var(--color-border, rgba(255,255,255,0.07));
}
.props-table td {
  padding: 7px 14px;
  border-bottom: 1px solid var(--color-border, rgba(255,255,255,0.07));
  color: var(--color-text-secondary, #a6adc8);
}
.props-table tr:last-child td {
  border-bottom: none;
}
.props-table td code {
  font-family: 'JetBrains Mono', monospace;
  font-size: 11px;
  color: var(--color-text-primary, #cdd6f4);
}
.prop-input {
  width: 100%;
  height: 28px;
  padding: 0 8px;
  background: var(--color-bg-raised, #11111b);
  border: 1px solid rgba(255,255,255,0.06);
  border-radius: 4px;
  color: var(--color-text-primary, #cdd6f4);
  font-size: 12px;
  outline: none;
}
.prop-input:focus {
  border-color: var(--color-accent, #89b4fa);
}
</style>