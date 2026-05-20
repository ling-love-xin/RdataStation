<template>
  <div class="advanced-tab">
    <!-- 环境选择 -->
    <div class="adv-section">
      <div class="section-title">{{ $t('navigator.advancedEnv') }}</div>
      <div class="section-row">
        <span class="row-label">{{ $t('navigator.advancedEnvSelect') }}</span>
        <NSelect
          :value="envValue"
          :options="envOptions"
          size="small"
          class="env-select"
          @update:value="onEnvChange"
        />
      </div>
    </div>

    <!-- DuckDB 联邦加速 -->
    <div class="adv-section">
      <div class="section-title">{{ $t('navigator.advancedDuckAccel') }}</div>
      <div class="duck-card">
        <div class="duck-card-header">
          <span class="duck-icon">&#x1F986;</span>
          <span class="duck-title">DuckDB</span>
          <NSwitch
            :value="duckEnabled"
            size="small"
            @update:value="onDuckToggle"
          />
        </div>
        <div class="duck-card-body">
          <div class="slot-item">
            <span class="slot-label">{{ $t('navigator.advancedDuckSlot') }}</span>
            <span class="slot-value">16</span>
          </div>
          <div class="slot-item">
            <span class="slot-label">{{ $t('navigator.advancedDuckThreads') }}</span>
            <span class="slot-value">8</span>
          </div>
        </div>
        <div class="duck-card-hint">
          {{ $t('navigator.advancedDuckHint') }}
        </div>
      </div>
    </div>

    <!-- 安全策略（折叠） -->
    <NCollapse class="security-collapse">
      <NCollapseItem :title="$t('navigator.advancedSecurity')" name="security">
        <div class="security-content">
          <div class="section-row">
            <span class="row-label">{{ $t('navigator.advancedSslMode') }}</span>
            <NSelect
              :value="sslMode"
              :options="sslOptions"
              size="small"
              class="param-select"
              @update:value="sslMode = $event"
            />
          </div>
          <div class="section-row">
            <span class="row-label">{{ $t('navigator.advancedReadOnly') }}</span>
            <NSwitch
              :value="readOnly"
              size="small"
              @update:value="readOnly = $event"
            />
          </div>
        </div>
      </NCollapseItem>
    </NCollapse>

    <!-- 连接参数 -->
    <div class="adv-section">
      <div class="section-title">{{ $t('navigator.advancedConnParams') }}</div>
      <div class="params-grid">
        <div class="param-cell">
          <span class="param-cell-label">{{ $t('navigator.advancedTimeout') }}</span>
          <NInputNumber
            :value="connTimeout"
            size="small"
            :placeholder="'30'"
            class="cell-input"
            @update:value="connTimeout = $event ?? undefined"
          />
          <span class="param-cell-unit">s</span>
        </div>
        <div class="param-cell">
          <span class="param-cell-label">{{ $t('navigator.advancedPoolMax') }}</span>
          <NInputNumber
            :value="poolMax"
            size="small"
            :placeholder="'10'"
            class="cell-input"
            @update:value="poolMax = $event ?? undefined"
          />
        </div>
      </div>
    </div>

    <!-- Schema + 编码 -->
    <div class="adv-section">
      <div class="params-grid-bottom">
        <div class="section-row">
          <span class="row-label">{{ $t('navigator.advancedSchema') }}</span>
          <NInput
            :value="schemaValue"
            size="small"
            :placeholder="'public'"
            class="schema-input"
            @update:value="schemaValue = $event"
          />
        </div>
        <div class="section-row">
          <span class="row-label">{{ $t('navigator.advancedEncoding') }}</span>
          <NSelect
            :value="encodingValue"
            :options="encodingOptions"
            size="small"
            class="encoding-select"
            @update:value="encodingValue = $event"
          />
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { NCollapse, NCollapseItem, NInput, NInputNumber, NSelect, NSwitch } from 'naive-ui'
import { ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'

const { t } = useI18n()

// ====== props / emits ======
interface Props {
  dbType?: string
  driverId?: string
}

defineProps<Props>()

interface Emits {
  (e: 'update:config', config: Record<string, unknown>): void
}

const emit = defineEmits<Emits>()

// ====== state ======
const envValue = ref('production')
const duckEnabled = ref(false)
const sslMode = ref('disable')
const readOnly = ref(false)
const connTimeout = ref<number | undefined>(30)
const poolMax = ref<number | undefined>(10)
const schemaValue = ref('public')
const encodingValue = ref('utf8')

// ====== options ======
const envOptions = [
  { label: t('navigator.advancedEnvProd'), value: 'production' },
  { label: t('navigator.advancedEnvDev'), value: 'development' },
  { label: t('navigator.advancedEnvTest'), value: 'test' },
]

const sslOptions = [
  { label: 'disable', value: 'disable' },
  { label: 'prefer', value: 'prefer' },
  { label: 'require', value: 'require' },
  { label: 'verify-ca', value: 'verify-ca' },
  { label: 'verify-full', value: 'verify-full' },
]

const encodingOptions = [
  { label: 'UTF-8', value: 'utf8' },
  { label: 'UTF-16', value: 'utf16' },
  { label: 'Latin1', value: 'latin1' },
  { label: 'GBK', value: 'gbk' },
]

// ====== emit config on change ======
function emitConfig() {
  emit('update:config', {
    env: envValue.value,
    duckEnabled: duckEnabled.value,
    sslMode: sslMode.value,
    readOnly: readOnly.value,
    connTimeout: connTimeout.value,
    poolMax: poolMax.value,
    schema: schemaValue.value,
    encoding: encodingValue.value,
  })
}

function onEnvChange(value: string) {
  envValue.value = value
  emitConfig()
}

function onDuckToggle(value: boolean) {
  duckEnabled.value = value
  emitConfig()
}

watch([sslMode, readOnly, connTimeout, poolMax, schemaValue, encodingValue], emitConfig)
</script>

<style scoped>
.advanced-tab {
  display: flex;
  flex-direction: column;
  gap: 18px;
}

/* ====== sections ====== */
.adv-section {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.section-title {
  font-size: var(--font-size-xs, 10px);
  font-weight: 700;
  text-transform: uppercase;
  color: var(--color-text-muted, #6c7086);
  letter-spacing: 0.7px;
}

.section-row {
  display: flex;
  align-items: center;
  gap: 8px;
}

.row-label {
  font-size: var(--font-size-sm, 12px);
  font-weight: 500;
  color: var(--color-text-secondary, #a6adc8);
  width: 60px;
  flex-shrink: 0;
}

.env-select {
  width: 160px;
}

/* ====== duck card ====== */
.duck-card {
  background: var(--color-bg-elevated, #1a1b26);
  border: 1px solid var(--color-border-subtle, rgba(255, 255, 255, 0.06));
  border-radius: 8px;
  padding: 12px 14px;
}

.duck-card-header {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 8px;
}

.duck-icon {
  font-size: 16px;
}

.duck-title {
  font-size: var(--font-size-sm, 13px);
  font-weight: 600;
  color: var(--color-text-primary, #cdd6f4);
  flex: 1;
}

.duck-card-body {
  display: flex;
  gap: 24px;
  margin-bottom: 6px;
}

.slot-item {
  display: flex;
  align-items: center;
  gap: 6px;
}

.slot-label {
  font-size: var(--font-size-xs, 11px);
  color: var(--color-text-muted, #6c7086);
}

.slot-value {
  font-size: var(--font-size-sm, 12px);
  font-weight: 600;
  color: var(--color-text-secondary, #a6adc8);
  padding: 1px 7px;
  background: var(--color-bg-secondary, #11111b);
  border-radius: 4px;
}

.duck-card-hint {
  font-size: var(--font-size-xs, 11px);
  color: var(--color-text-muted, #6c7086);
  line-height: 1.4;
}

/* ====== security collapse ====== */
.security-collapse {
  border-radius: 6px;
  background: var(--color-bg-elevated, #1a1b26);
  border: 1px solid var(--color-border-subtle, rgba(255, 255, 255, 0.06));
}

.security-collapse :deep(.n-collapse-item__header) {
  font-size: var(--font-size-sm, 12px);
  font-weight: 600;
}

.security-content {
  display: flex;
  flex-direction: column;
  gap: 8px;
  padding-bottom: 4px;
}

.param-select {
  width: 160px;
}

/* ====== params grid ====== */
.params-grid {
  display: flex;
  gap: 24px;
}

.param-cell {
  display: flex;
  align-items: center;
  gap: 6px;
}

.param-cell-label {
  font-size: var(--font-size-sm, 12px);
  color: var(--color-text-secondary, #a6adc8);
  flex-shrink: 0;
}

.cell-input {
  width: 80px;
}

.param-cell-unit {
  font-size: var(--font-size-xs, 11px);
  color: var(--color-text-muted, #6c7086);
}

/* ====== schema + encoding ====== */
.params-grid-bottom {
  display: flex;
  gap: 32px;
}

.schema-input {
  width: 140px;
}

.encoding-select {
  width: 130px;
}
</style>