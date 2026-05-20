<template>
  <div class="datasource-sidebar">
    <!-- 搜索框 -->
    <div class="sidebar-search">
      <NInput
        v-model:value="searchQuery"
        size="small"
        :placeholder="$t('navigator.searchDatabaseType')"
        clearable
      >
        <template #prefix>
          <Search :size="14" />
        </template>
      </NInput>
    </div>

    <!-- 暂存列表 -->
    <div class="sidebar-section saved-section">
      <div class="section-header">
        <span class="section-label">{{ $t('navigator.stagingList') }}</span>
        <NButton text size="tiny" @click="$emit('add-staging')">
          + {{ $t('navigator.add') }}
        </NButton>
      </div>
      <div v-if="stagingEntries.length === 0" class="section-empty">
        {{ $t('navigator.noStagingHint') }}
      </div>
      <div
        v-for="entry in stagingEntries"
        :key="entry.id"
        class="saved-item"
        :class="{ active: activeStagingId === entry.id }"
        @click="onSelectStaging(entry.id)"
      >
        <span class="type-dot" :style="{ background: getDbColor(entry.dbType) }" />
        <span class="db-icon-mini" :class="entry.dbType">{{ getDbInitials(entry.dbType) }}</span>
        <span class="item-name">{{ entry.name }}</span>
        <NButton text size="tiny" class="item-remove" @click.stop="$emit('remove-staging', entry.id)">
          <template #icon><X :size="10" /></template>
        </NButton>
      </div>
    </div>

    <div class="sidebar-divider" />

    <!-- 数据库类型 -->
    <div class="sidebar-section db-section">
      <div class="section-label-only">{{ $t('navigator.databaseTypes') }}</div>

      <div
        v-for="category in filteredCategories"
        :key="category.key"
        class="db-category-wrapper"
      >
        <!-- 分类头 -->
        <div
          class="db-category"
          :class="{ expanded: expandedCategories.has(category.key) }"
          @click="toggleCategory(category.key)"
        >
          <span class="category-arrow">▶</span>
          <span class="category-icon">{{ category.icon }}</span>
          <span class="category-name">{{ category.label }}</span>
          <span class="category-count">{{ category.items.length }}</span>
        </div>

        <!-- 分类子项 -->
        <div v-show="expandedCategories.has(category.key)" class="db-type-sublist">
          <div
            v-for="item in category.items"
            :key="item.type"
            class="db-type-item"
            :class="{ selected: selectedDbType === item.type }"
            @click="$emit('select-db-type', item.type)"
          >
            <span class="type-dot" :style="{ background: item.color }" />
            <span class="type-name">{{ item.label }}</span>
            <span class="driver-count">{{ getDriverCount(item.type) }} {{ $t('navigator.drivers') }}</span>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { Search, X } from 'lucide-vue-next'
import { NButton, NInput } from 'naive-ui'
import { computed, reactive, ref } from 'vue'
import { useI18n } from 'vue-i18n'

import type { DriverDescriptor } from '../types/driver'
import type { StagingEntry } from '../composables/useStagingList'

const { t } = useI18n()

// ====== props / emits ======
interface Props {
  drivers: DriverDescriptor[]
  selectedDbType?: string
  stagingEntries: StagingEntry[]
}

const props = defineProps<Props>()

interface Emits {
  (e: 'select-db-type', type: string): void
  (e: 'add-staging'): void
  (e: 'remove-staging', id: number): void
  (e: 'select-staging', id: number): void
}

defineEmits<Emits>()

// ====== state ======
const searchQuery = ref('')
const activeStagingId = ref<number | null>(null)

// 默认展开"关系型数据库"
const expandedCategories = reactive(new Set<string>(['relational']))

// ====== DB 类型分类 ======
interface DbTypeItem {
  type: string
  label: string
  color: string
}

interface DbCategory {
  key: string
  icon: string
  label: string
  items: DbTypeItem[]
}

const DB_CATEGORIES: DbCategory[] = [
  {
    key: 'relational',
    icon: '\u{1F5C4}',
    label: t('navigator.categoryRelational'),
    items: [
      { type: 'mysql', label: 'MySQL', color: '#00758f' },
      { type: 'postgresql', label: 'PostgreSQL', color: '#336791' },
      { type: 'mariadb', label: 'MariaDB', color: '#c0765a' },
      { type: 'sqlserver', label: 'SQL Server', color: '#cc2927' },
    ],
  },
  {
    key: 'file',
    icon: '\u{1F4C1}',
    label: t('navigator.categoryFile'),
    items: [
      { type: 'sqlite', label: 'SQLite', color: '#687386' },
      { type: 'duckdb', label: 'DuckDB', color: '#f9a825' },
    ],
  },
  {
    key: 'nosql',
    icon: '\u{1F4E1}',
    label: t('navigator.categoryNoSQL'),
    items: [
      { type: 'mongodb', label: 'MongoDB', color: '#4db33d' },
      { type: 'redis', label: 'Redis', color: '#d82c20' },
    ],
  },
  {
    key: 'analytics',
    icon: '\u{1F4CA}',
    label: t('navigator.categoryAnalytics'),
    items: [
      { type: 'clickhouse', label: 'ClickHouse', color: '#f9a825' },
    ],
  },
]

const DB_COLORS: Record<string, string> = {
  mysql: '#00758f',
  postgresql: '#336791',
  mariadb: '#c0765a',
  sqlserver: '#cc2927',
  sqlite: '#687386',
  duckdb: '#f9a825',
  mongodb: '#4db33d',
  redis: '#d82c20',
  clickhouse: '#f9a825',
}

const DB_INITIALS: Record<string, string> = {
  mysql: 'My',
  postgresql: 'PG',
  mariadb: 'Ma',
  sqlserver: 'MS',
  sqlite: 'SL',
  duckdb: 'Du',
  mongodb: 'Mo',
  redis: 'Rd',
  clickhouse: 'CH',
}

// ====== computed ======
const filteredCategories = computed(() => {
  if (!searchQuery.value.trim()) return DB_CATEGORIES

  const q = searchQuery.value.toLowerCase()
  return DB_CATEGORIES
    .map((cat) => ({
      ...cat,
      items: cat.items.filter(
        (item) =>
          item.label.toLowerCase().includes(q) ||
          item.type.toLowerCase().includes(q),
      ),
    }))
    .filter((cat) => cat.items.length > 0)
})

// ====== helpers ======
function getDriverCount(dbType: string): number {
  return props.drivers.filter((d) => d.id.toLowerCase().startsWith(dbType)).length
}

function getDbColor(dbType: string): string {
  return DB_COLORS[dbType] || '#888'
}

function getDbInitials(dbType: string): string {
  return DB_INITIALS[dbType] || 'DB'
}

function toggleCategory(key: string) {
  if (expandedCategories.has(key)) {
    expandedCategories.delete(key)
  } else {
    expandedCategories.add(key)
  }
}

function onSelectStaging(id: number) {
  activeStagingId.value = id
}
</script>

<style scoped>
.datasource-sidebar {
  height: 100%;
  display: flex;
  flex-direction: column;
  overflow-y: auto;
  background: var(--color-bg-secondary, #11111b);
}

/* ====== search ====== */
.sidebar-search {
  padding: 10px 12px;
}

.sidebar-search :deep(.n-input__prefix) {
  color: var(--color-text-muted, #6c7086);
}

/* ====== sections ====== */
.sidebar-section {
  padding: 0 0 6px;
}

.section-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 14px 4px;
}

.section-label {
  font-size: var(--font-size-xs, 10px);
  font-weight: 700;
  text-transform: uppercase;
  color: var(--color-text-muted, #6c7086);
  letter-spacing: 0.7px;
}

.section-label-only {
  font-size: var(--font-size-xs, 10px);
  font-weight: 700;
  text-transform: uppercase;
  color: var(--color-text-muted, #6c7086);
  letter-spacing: 0.7px;
  padding: 8px 14px 6px;
}

.section-empty {
  font-size: var(--font-size-xs, 11px);
  color: var(--color-text-muted, #6c7086);
  padding: 6px 14px;
}

/* ====== saved items ====== */
.saved-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 6px 14px;
  cursor: pointer;
  font-size: var(--font-size-sm, 12px);
  color: var(--color-text-secondary, #a6adc8);
  transition: all 0.12s;
}

.saved-item:hover {
  background: var(--color-hover, rgba(255, 255, 255, 0.05));
  color: var(--color-text-primary, #cdd6f4);
}

.saved-item.active {
  background: var(--color-selection, #2a2a3c);
  color: var(--brand-accent, #89b4fa);
  font-weight: 500;
}

.type-dot {
  width: 7px;
  height: 7px;
  border-radius: 50%;
  flex-shrink: 0;
}

.db-icon-mini {
  width: 18px;
  height: 18px;
  border-radius: 3px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 9px;
  font-weight: 700;
  flex-shrink: 0;
  color: #fff;
}

.db-icon-mini.mysql {
  background: #00758f;
}

.db-icon-mini.postgresql {
  background: #336791;
}

.db-icon-mini.mariadb {
  background: #c0765a;
}

.db-icon-mini.sqlserver {
  background: #cc2927;
}

.db-icon-mini.sqlite {
  background: #003b57;
  color: #a6e3a1;
}

.db-icon-mini.duckdb {
  background: #f9a825;
  color: #000;
}

.db-icon-mini.mongodb {
  background: #4db33d;
}

.db-icon-mini.redis {
  background: #d82c20;
}

.db-icon-mini.clickhouse {
  background: #f9a825;
  color: #000;
}

.item-name {
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.item-remove {
  flex-shrink: 0;
  opacity: 0.5;
  color: var(--color-text-muted, #6c7086);
}

.item-remove:hover {
  opacity: 1;
}

/* ====== divider ====== */
.sidebar-divider {
  height: 1px;
  background: var(--color-border, rgba(255, 255, 255, 0.07));
  margin: 4px 12px;
}

/* ====== categories ====== */
.db-category-wrapper {
  /* container for collapse */
}

.db-category {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 7px 14px;
  cursor: pointer;
  font-size: var(--font-size-sm, 12px);
  color: var(--color-text-muted, #6c7086);
  transition: all 0.12s;
  user-select: none;
}

.db-category:hover {
  background: var(--color-hover, rgba(255, 255, 255, 0.05));
  color: var(--color-text-primary, #cdd6f4);
}

.db-category.expanded {
  color: var(--color-text-primary, #cdd6f4);
}

.category-arrow {
  font-size: 9px;
  transition: transform 0.15s;
  flex-shrink: 0;
  width: 12px;
  text-align: center;
}

.db-category.expanded .category-arrow {
  transform: rotate(90deg);
}

.category-icon {
  font-size: 14px;
  width: 20px;
  text-align: center;
  flex-shrink: 0;
}

.category-name {
  flex: 1;
}

.category-count {
  font-size: var(--font-size-xs, 10px);
  color: var(--color-text-muted, #6c7086);
  padding: 1px 6px;
  background: var(--color-bg-elevated, rgba(255, 255, 255, 0.03));
  border-radius: 8px;
}

/* ====== db type items ====== */
.db-type-sublist {
  /* sub-items container */
}

.db-type-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 6px 14px 6px 32px;
  cursor: pointer;
  font-size: var(--font-size-sm, 12px);
  color: var(--color-text-muted, #6c7086);
  transition: all 0.12s;
}

.db-type-item:hover {
  background: var(--color-hover, rgba(255, 255, 255, 0.05));
  color: var(--color-text-secondary, #a6adc8);
}

.db-type-item.selected {
  background: var(--color-selection, #2a2a3c);
  color: var(--brand-accent, #89b4fa);
  font-weight: 600;
}

.db-type-item .type-dot {
  width: 6px;
  height: 6px;
}

.type-name {
  flex: 1;
}

.driver-count {
  font-size: var(--font-size-xs, 10px);
  color: var(--color-text-muted, #6c7086);
  padding: 1px 6px;
  background: var(--color-bg-elevated, rgba(255, 255, 255, 0.03));
  border-radius: 8px;
}
</style>