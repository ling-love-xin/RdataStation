<template>
  <div class="advanced-tab">
    <div class="form-section">
      <div class="section-title">{{ t('connection.advancedTab.connectionParams') }}</div>
      <div class="adv-grid">
        <div class="form-group">
          <span class="form-label">{{ t('connection.advancedTab.connectTimeout') }}</span>
          <input v-model.number="connectTimeout" type="number" class="form-input" />
        </div>
        <div class="form-group">
          <span class="form-label">{{ t('connection.advancedTab.queryTimeout') }}</span>
          <input v-model.number="queryTimeout" type="number" class="form-input" placeholder="0=不限制" />
        </div>
        <div class="form-group">
          <span class="form-label">{{ t('connection.advancedTab.keepAlive') }}</span>
          <input v-model.number="keepAlive" type="number" class="form-input" placeholder="0=禁用" />
        </div>
        <div class="form-group">
          <span class="form-label">{{ t('connection.advancedTab.maxReconnect') }}</span>
          <input v-model.number="maxReconnect" type="number" class="form-input" />
        </div>
      </div>
    </div>

    <div class="form-section">
      <div class="section-title">{{ t('connection.advancedTab.schemaLoading') }}</div>
      <select v-model="schemaStrategy" class="form-select schema-select">
        <option value="auto">{{ t('connection.advancedTab.schemaAuto') }}</option>
        <option value="manual">{{ t('connection.advancedTab.schemaManual') }}</option>
        <option value="selected">{{ t('connection.advancedTab.schemaSelected') }}</option>
      </select>
    </div>

    <div class="form-section">
      <div class="section-title accel-title">
        <Zap :size="14" />
        {{ t('connection.advancedTab.localAccel') }}
      </div>
      <div class="accel-card">
        <div class="accel-header">
          <span class="accel-icon">🦆</span>
          <span class="accel-name">{{ t('connection.advancedTab.enableDuckdbAccel') }}</span>
          <div class="accel-switch">
            <div
              class="switch-toggle"
              :class="{ on: duckdbAccelEnabled }"
              @click="duckdbAccelEnabled = !duckdbAccelEnabled"
            />
          </div>
        </div>
        <div v-if="duckdbAccelEnabled" class="accel-body">
          <p class="accel-desc">
            {{ t('connection.advancedTab.accelDesc', { dbType: dbTypeName }) }}
            <br />
            <code>.rdata/duckdb/accel.duckdb</code>
          </p>
          <div class="form-row">
            <div class="form-group f1">
              <span class="form-label">{{ t('connection.advancedTab.syncStrategy') }}</span>
              <select v-model="syncStrategy" class="form-select">
                <option value="auto">{{ t('connection.advancedTab.syncAuto') }}</option>
                <option value="scheduled">{{ t('connection.advancedTab.syncScheduled') }}</option>
                <option value="manual">{{ t('connection.advancedTab.syncManual') }}</option>
              </select>
            </div>
            <div class="form-group f1">
              <span class="form-label">{{ t('connection.advancedTab.syncInterval') }}</span>
              <input
                v-model.number="syncInterval"
                type="number"
                class="form-input"
                :disabled="syncStrategy !== 'scheduled'"
              />
            </div>
          </div>
          <div class="form-row">
            <div class="form-group f1">
              <span class="form-label">{{ t('connection.advancedTab.memoryLimit') }}</span>
              <input v-model.number="memoryLimit" type="number" class="form-input" />
            </div>
            <div class="form-group f1">
              <span class="form-label">{{ t('connection.advancedTab.threads') }}</span>
              <input v-model.number="threads" type="number" class="form-input" />
            </div>
          </div>
        </div>
      </div>
    </div>

    <div class="form-section">
      <div class="section-title">{{ t('connection.advancedTab.encoding') }}</div>
      <select v-model="encoding" class="form-select encoding-select">
        <option value="utf8">UTF-8（默认）</option>
        <option value="gbk">GBK</option>
        <option value="latin1">Latin1</option>
        <option value="utf16">UTF-16</option>
      </select>
    </div>
  </div>
</template>

<script setup lang="ts">
import { Zap } from 'lucide-vue-next'
import { ref, computed } from 'vue'
import { useI18n } from 'vue-i18n'

const { t } = useI18n()

interface Props {
  dbType?: string
}

const props = defineProps<Props>()

const connectTimeout = ref(30)
const queryTimeout = ref(0)
const keepAlive = ref(60)
const maxReconnect = ref(3)
const schemaStrategy = ref('auto')
const encoding = ref('utf8')

const duckdbAccelEnabled = ref(false)
const syncStrategy = ref('auto')
const syncInterval = ref(15)
const memoryLimit = ref(512)
const threads = ref(4)

const DB_NAME_MAP: Record<string, string> = {
  mysql: 'MySQL',
  postgresql: 'PostgreSQL',
  mariadb: 'MariaDB',
  sqlserver: 'SQL Server',
  sqlite: 'SQLite',
  duckdb: 'DuckDB',
  mongodb: 'MongoDB',
  redis: 'Redis',
  clickhouse: 'ClickHouse',
}

const dbTypeName = computed(() => DB_NAME_MAP[props.dbType || ''] || props.dbType || '')
</script>

<style scoped>
.advanced-tab {
  padding: 0;
}
.form-section {
  margin-bottom: 20px;
}
.section-title {
  font-size: 11px;
  font-weight: 700;
  text-transform: uppercase;
  color: var(--color-text-muted, #6c7086);
  margin-bottom: 10px;
  letter-spacing: 0.5px;
}
.accel-title {
  color: var(--color-warning, #f9e2af);
  display: flex;
  align-items: center;
  gap: 6px;
}
.adv-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 12px;
}
.form-group {
  display: flex;
  flex-direction: column;
  gap: 4px;
}
.form-group.f1 {
  flex: 1;
}
.form-label {
  font-size: 12px;
  color: var(--color-text-secondary, #a6adc8);
  font-weight: 500;
}
.form-input, .form-select {
  width: 100%;
  height: 32px;
  padding: 0 10px;
  background: var(--color-bg-raised, #11111b);
  border: 1px solid rgba(255,255,255,0.05);
  border-radius: 6px;
  color: var(--color-text-primary, #cdd6f4);
  font-size: 13px;
  outline: none;
  transition: border-color 0.2s;
}
.form-input:focus, .form-select:focus {
  border-color: var(--color-accent, #89b4fa);
}
.form-input:disabled {
  opacity: 0.4;
}
.form-select {
  cursor: pointer;
}
.schema-select {
  max-width: 280px;
}
.encoding-select {
  max-width: 180px;
}

.accel-card {
  border: 1px solid rgba(249,168,37,0.2);
  border-radius: 8px;
  padding: 14px;
  background: rgba(249,168,37,0.03);
}
.accel-header {
  display: flex;
  align-items: center;
  gap: 8px;
}
.accel-icon {
  font-size: 18px;
}
.accel-name {
  font-size: 13px;
  font-weight: 600;
  color: var(--color-warning, #f9e2af);
}
.accel-switch {
  margin-left: auto;
}
.switch-toggle {
  width: 34px;
  height: 18px;
  background: rgba(255,255,255,0.08);
  border-radius: 9px;
  position: relative;
  cursor: pointer;
  transition: background 0.2s;
}
.switch-toggle.on {
  background: var(--color-accent, #89b4fa);
}
.switch-toggle::after {
  content: '';
  width: 14px;
  height: 14px;
  border-radius: 50%;
  background: #fff;
  position: absolute;
  top: 2px;
  left: 2px;
  transition: left 0.2s;
}
.switch-toggle.on::after {
  left: 18px;
}
.accel-body {
  margin-top: 12px;
}
.accel-desc {
  font-size: 11px;
  color: var(--color-text-muted, #6c7086);
  margin-bottom: 12px;
  line-height: 1.6;
}
.accel-desc code {
  font-family: 'JetBrains Mono', monospace;
  font-size: 10px;
  color: var(--color-success, #a6e3a1);
}
.form-row {
  display: flex;
  gap: 12px;
  margin-bottom: 10px;
}
</style>