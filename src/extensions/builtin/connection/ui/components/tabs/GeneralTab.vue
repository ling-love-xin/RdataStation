<template>
  <div class="general-tab">
    <!-- 驱动信息横幅 -->
    <div v-if="dbType" class="info-banner">
      <div class="banner-icon" :style="{ background: dbColor }">
        {{ dbInitials }}
      </div>
      <div class="banner-body">
        <div class="banner-title">{{ dbTypeLabel }}</div>
        <div class="banner-desc">{{ infoText }}</div>
      </div>
    </div>

    <!-- 连接参数区域 -->
    <div class="params-section">
      <div class="section-title">{{ $t('navigator.tabGeneral') }}</div>

      <!-- 文件数据库模式 -->
      <template v-if="isFileDb">
        <div class="param-row">
          <span class="param-label">{{ $t('navigator.databaseFile') }}</span>
          <NInput
            :value="formData.filePath as string"
            size="small"
            :placeholder="filePathPlaceholder"
            class="param-input"
            @update:value="onUpdate('filePath', $event)"
          />
          <NButton size="small" secondary @click="onBrowse">
            <template #icon><FolderOpen :size="14" /></template>
            {{ $t('navigator.browse') }}
          </NButton>
        </div>
        <div class="param-row">
          <span class="param-label">{{ $t('navigator.formDatabase') }}</span>
          <NInput
            :value="formData.database as string"
            size="small"
            :placeholder="$t('navigator.databasePlaceholder')"
            class="param-input"
            @update:value="onUpdate('database', $event)"
          />
        </div>
      </template>

      <!-- 网络数据库模式 -->
      <template v-else>
        <div class="param-row">
          <span class="param-label">{{ $t('navigator.formHost') }}</span>
          <NInput
            :value="formData.host as string"
            size="small"
            :placeholder="$t('navigator.hostPlaceholder')"
            class="param-input host-input"
            @update:value="onUpdate('host', $event)"
          />
          <span class="param-label-spacer" />
          <span class="param-label">{{ $t('navigator.formPort') }}</span>
          <NInputNumber
            :value="formData.port as number"
            size="small"
            :placeholder="$t('navigator.portPlaceholder')"
            class="port-input"
            @update:value="onUpdate('port', $event)"
          />
        </div>
        <div class="param-row">
          <span class="param-label">{{ $t('navigator.formDatabase') }}</span>
          <NInput
            :value="formData.database as string"
            size="small"
            :placeholder="$t('navigator.databasePlaceholder')"
            class="param-input"
            @update:value="onUpdate('database', $event)"
          />
        </div>
        <div class="param-row">
          <span class="param-label">{{ $t('navigator.formUsername') }}</span>
          <NInput
            :value="formData.username as string"
            size="small"
            :placeholder="$t('navigator.usernamePlaceholder')"
            class="param-input"
            @update:value="onUpdate('username', $event)"
          />
          <span class="param-label-spacer" />
          <span class="param-label">{{ $t('navigator.formPassword') }}</span>
          <NInput
            :value="formData.password as string"
            type="password"
            size="small"
            :placeholder="$t('navigator.passwordPlaceholder')"
            class="param-input"
            show-password-on="click"
            @update:value="onUpdate('password', $event)"
          />
        </div>
      </template>
    </div>
  </div>
</template>

<script setup lang="ts">
import { FolderOpen } from 'lucide-vue-next'
import { NButton, NInput, NInputNumber } from 'naive-ui'
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'

const { t } = useI18n()

// ====== props / emits ======
interface Props {
  formData: Record<string, unknown>
  dbType?: string
  isFileDb?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  isFileDb: false,
})

interface Emits {
  (e: 'update:form-data', data: Record<string, unknown>): void
}

const emit = defineEmits<Emits>()

// ====== DB info ======
const DB_META: Record<string, { label: string; color: string; initials: string }> = {
  mysql: { label: 'MySQL', color: '#00758f', initials: 'My' },
  postgresql: { label: 'PostgreSQL', color: '#336791', initials: 'PG' },
  mariadb: { label: 'MariaDB', color: '#c0765a', initials: 'Ma' },
  sqlserver: { label: 'SQL Server', color: '#cc2927', initials: 'MS' },
  sqlite: { label: 'SQLite', color: '#003b57', initials: 'SL' },
  duckdb: { label: 'DuckDB', color: '#f9a825', initials: 'Du' },
  mongodb: { label: 'MongoDB', color: '#4db33d', initials: 'Mo' },
  redis: { label: 'Redis', color: '#d82c20', initials: 'Rd' },
  clickhouse: { label: 'ClickHouse', color: '#f9a825', initials: 'CH' },
}

const dbMeta = computed(() => {
  if (!props.dbType) return null
  return DB_META[props.dbType] || { label: props.dbType, color: '#888', initials: 'DB' }
})

const dbTypeLabel = computed(() => dbMeta.value?.label || '')
const dbColor = computed(() => dbMeta.value?.color || '#888')
const dbInitials = computed(() => dbMeta.value?.initials || 'DB')

const infoText = computed(() => {
  if (props.isFileDb) {
    return t('navigator.fileDbHint')
  }
  return t('navigator.networkDbHint')
})

const filePathPlaceholder = computed(() => {
  if (props.dbType === 'duckdb') return './data.duckdb'
  if (props.dbType === 'sqlite') return './database.db'
  return './database'
})

// ====== helpers ======
function onUpdate(key: string, value: unknown) {
  emit('update:form-data', { [key]: value })
}

function onBrowse() {
  // TODO: 调用 Tauri 文件选择对话框
  console.log('Browse file')
}
</script>

<style scoped>
.general-tab {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

/* ====== info banner ====== */
.info-banner {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px 14px;
  background: var(--color-bg-elevated, #1a1b26);
  border: 1px solid var(--color-border-subtle, rgba(255, 255, 255, 0.06));
  border-radius: 8px;
}

.banner-icon {
  width: 32px;
  height: 32px;
  border-radius: 6px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 13px;
  font-weight: 800;
  flex-shrink: 0;
  color: #fff;
}

.banner-body {
  flex: 1;
  min-width: 0;
}

.banner-title {
  font-size: var(--font-size-sm, 13px);
  font-weight: 600;
  color: var(--color-text-primary, #cdd6f4);
}

.banner-desc {
  font-size: var(--font-size-xs, 11px);
  color: var(--color-text-muted, #6c7086);
  margin-top: 2px;
}

/* ====== params ====== */
.params-section {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.section-title {
  font-size: var(--font-size-xs, 10px);
  font-weight: 700;
  text-transform: uppercase;
  color: var(--color-text-muted, #6c7086);
  letter-spacing: 0.7px;
  margin-bottom: 2px;
}

.param-row {
  display: flex;
  align-items: center;
  gap: 8px;
}

.param-label {
  font-size: var(--font-size-sm, 12px);
  font-weight: 500;
  color: var(--color-text-muted, #6c7086);
  width: 44px;
  flex-shrink: 0;
}

.param-label-spacer {
  width: var(--spacing-md, 16px);
}

.param-input {
  flex: 1;
}

.host-input {
  flex: 1;
}

.port-input {
  width: 100px;
  flex-shrink: 0;
}
</style>