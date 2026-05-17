<template>
  <div class="datasource-sidebar">
    <div class="sidebar-search">
      <Search :size="14" class="search-icon" />
      <input
        v-model="searchQuery"
        type="text"
        class="search-input"
        :placeholder="String(t('navigator.searchDatabaseType'))"
      />
    </div>

    <div class="sidebar-section">
      <div class="section-header">
        <span class="section-title">{{ t('navigator.stagingList') }}</span>
        <button class="add-btn" :title="String(t('navigator.addStagingEntry'))" @click="staging.addEntry()">
          +
        </button>
      </div>
      <div
        v-for="entry in staging.entries.value"
        :key="entry.id"
        class="staging-item"
        :class="{ active: stagingActiveId === entry.id }"
        @click="$emit('select-staging', entry.id)"
      >
        <span class="staging-dot" :style="{ background: entry.dbType ? getDbColor(entry.dbType) : 'var(--color-text-muted)' }" />
        <DbIcon v-if="entry.dbType" :type="entry.dbType" :size="14" class="staging-db-icon" />
        <span class="staging-name">{{ entry.name }}</span>
        <button
          class="remove-btn"
          :title="String(t('navigator.removeStagingEntry'))"
          @click.stop="staging.removeEntry(entry.id)"
        >
          ✕
        </button>
      </div>
    </div>

    <div class="sidebar-divider" />

    <div class="sidebar-section">
      <span class="section-title">{{ t('navigator.databaseTypes') }}</span>

      <div
        v-for="category in categorizedDrivers"
        :key="category.name"
        class="db-category-group"
      >
        <div
          class="db-category"
          :class="{ expanded: expandedCategories.has(category.name) }"
          @click="toggleCategory(category.name)"
        >
          <ChevronRight :size="12" class="category-arrow" />
          <span class="category-name">{{ getCategoryLabel(category.name) }}</span>
          <span class="category-count">{{ category.items.length }}</span>
        </div>
        <div v-if="expandedCategories.has(category.name)" class="db-type-list">
          <div
            v-for="dbType in filteredDbTypes(category)"
            :key="dbType.type"
            class="db-type-item"
            :class="{ selected: selectedDbType === dbType.type }"
            @click="selectDbType(dbType.type)"
          >
            <span class="db-type-dot" :style="{ background: dbType.color }" />
            <span class="db-type-name">{{ dbType.label }}</span>
            <span class="db-type-driver-count">{{ dbType.driverCount }} {{ t('navigator.drivers') }}</span>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { Search, ChevronRight } from 'lucide-vue-next'
import { ref, computed, onMounted, watch } from 'vue'
import { useI18n } from 'vue-i18n'

import DbIcon from '@/shared/components/common/DbIcon.vue'

import { useStagingList } from '../composables/useStagingList'

import type { DriverDescriptor } from '../types/connection'

const { t } = useI18n()

interface Props {
  drivers: DriverDescriptor[]
  selectedDbType?: string
  selectedDriverId?: string
}

const props = defineProps<Props>()

const emit = defineEmits<{
  (e: 'select-db-type', type: string): void
  (e: 'select-staging', id: number): void
}>()

const staging = useStagingList()
const stagingActiveId = ref<number>(staging.entries.value[0]?.id ?? 1)
const searchQuery = ref('')
const expandedCategories = ref<Set<string>>(new Set(['relational']))
const selectedDbType = ref<string>(props.selectedDbType || '')

interface DbTypeInfo {
  type: string
  label: string
  color: string
  driverCount: number
}

interface CategoryGroup {
  name: string
  label: string
  items: DbTypeInfo[]
}

const DB_CATEGORIES: Record<string, { label: string; types: string[] }> = {
  relational: { label: '关系型数据库', types: ['mysql', 'postgresql', 'mariadb', 'sqlserver'] },
  file: { label: '文件数据库', types: ['sqlite', 'duckdb'] },
  nosql: { label: 'NoSQL', types: ['mongodb', 'redis'] },
  analytics: { label: '分析型数据库', types: ['clickhouse'] },
}

const DB_COLORS: Record<string, string> = {
  mysql: '#00758f', postgresql: '#336791', mariadb: '#c0765a', sqlserver: '#d54d3d',
  sqlite: '#687386', duckdb: '#f9a825', mongodb: '#4db33d', redis: '#d82c20',
  clickhouse: '#f9a825',
}

function getDbColor(type: string): string {
  return DB_COLORS[type] || '#6c7086'
}

function getCategoryLabel(name: string): string {
  return DB_CATEGORIES[name]?.label || name
}

function toggleCategory(name: string) {
  if (expandedCategories.value.has(name)) {
    expandedCategories.value.delete(name)
  } else {
    expandedCategories.value.add(name)
  }
}

const categorizedDrivers = computed<CategoryGroup[]>(() => {
  const dbTypeMap = new Map<string, Set<string>>()
  for (const d of props.drivers) {
    const dbType = d.id.replace(/-.*$/, '')
    if (!dbTypeMap.has(dbType)) {
      dbTypeMap.set(dbType, new Set())
    }
    dbTypeMap.get(dbType)!.add(d.id)
  }

  const result: CategoryGroup[] = []
  for (const [catName, catConfig] of Object.entries(DB_CATEGORIES)) {
    const items: DbTypeInfo[] = []
    for (const dbType of catConfig.types) {
      const drivers = dbTypeMap.get(dbType)
      if (drivers && drivers.size > 0) {
        items.push({
          type: dbType,
          label: dbType.charAt(0).toUpperCase() + dbType.slice(1),
          color: DB_COLORS[dbType] || '#6c7086',
          driverCount: drivers.size,
        })
      }
    }
    if (items.length > 0) {
      result.push({ name: catName, label: catConfig.label, items })
    }
  }

  const categorized = new Set(result.flatMap(c => c.items.map(i => i.type)))
  for (const [dbType, drivers] of dbTypeMap) {
    if (!categorized.has(dbType)) {
      const existing = result.find(c => c.name === 'other')
      const item: DbTypeInfo = {
        type: dbType,
        label: dbType.charAt(0).toUpperCase() + dbType.slice(1),
        color: DB_COLORS[dbType] || '#6c7086',
        driverCount: drivers.size,
      }
      if (existing) {
        existing.items.push(item)
      } else {
        result.push({ name: 'other', label: '其他', items: [item] })
      }
    }
  }

  return result
})

function filteredDbTypes(category: CategoryGroup): DbTypeInfo[] {
  if (!searchQuery.value) return category.items
  const q = searchQuery.value.toLowerCase()
  return category.items.filter(item => item.label.toLowerCase().includes(q))
}

function selectDbType(type: string) {
  selectedDbType.value = type
  emit('select-db-type', type)
}

watch(
  () => props.selectedDbType,
  (val) => {
    if (val) selectedDbType.value = val
  }
)

onMounted(() => {
  if (props.selectedDbType) {
    selectedDbType.value = props.selectedDbType
  }
})
</script>

<style scoped>
.datasource-sidebar {
  width: 240px;
  min-width: 240px;
  background: var(--color-bg-raised, #11111b);
  border-right: 1px solid var(--color-border, rgba(255,255,255,0.07));
  display: flex;
  flex-direction: column;
  overflow-y: auto;
  height: 100%;
}

/* 搜索 */
.sidebar-search {
  padding: 10px 12px;
  position: relative;
}
.search-icon {
  position: absolute;
  left: 22px;
  top: 50%;
  transform: translateY(-50%);
  color: var(--color-text-muted, #6c7086);
}
.search-input {
  width: 100%;
  height: 32px;
  padding: 0 12px 0 32px;
  background: rgba(255,255,255,0.03);
  border: 1px solid rgba(255,255,255,0.06);
  border-radius: 6px;
  color: var(--color-text-primary, #cdd6f4);
  font-size: 12px;
  outline: none;
  transition: border-color 0.2s;
}
.search-input:focus {
  border-color: var(--color-accent, #89b4fa);
}

/* 暂存列表 */
.sidebar-section {
  padding: 4px 0;
}
.section-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 14px 6px;
}
.section-title {
  font-size: 10px;
  font-weight: 700;
  text-transform: uppercase;
  color: var(--color-text-muted, #6c7086);
  letter-spacing: 0.7px;
}
.add-btn {
  width: 20px;
  height: 20px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: transparent;
  border: none;
  border-radius: 4px;
  color: var(--color-text-muted, #6c7086);
  font-size: 16px;
  cursor: pointer;
  transition: all 0.12s;
}
.add-btn:hover {
  background: var(--color-bg-hover, rgba(255,255,255,0.05));
  color: var(--color-text-primary, #cdd6f4);
}

.staging-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 5px 14px;
  cursor: pointer;
  font-size: 12px;
  color: var(--color-text-secondary, #a6adc8);
  transition: all 0.12s;
}
.staging-item:hover {
  background: var(--color-bg-hover, rgba(255,255,255,0.05));
}
.staging-item.active {
  background: var(--color-bg-active, #2a2a3c);
  color: var(--color-accent, #89b4fa);
  font-weight: 500;
}
.staging-dot {
  width: 7px;
  height: 7px;
  border-radius: 50%;
  flex-shrink: 0;
}
.staging-db-icon {
  flex-shrink: 0;
}
.staging-name {
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.remove-btn {
  background: none;
  border: none;
  color: var(--color-text-muted, #6c7086);
  font-size: 12px;
  cursor: pointer;
  padding: 0 2px;
  flex-shrink: 0;
  opacity: 0;
  transition: opacity 0.15s;
}
.staging-item:hover .remove-btn {
  opacity: 1;
}

.sidebar-divider {
  height: 1px;
  background: var(--color-border, rgba(255,255,255,0.07));
  margin: 6px 12px;
}

/* 数据库分类 */
.db-category-group {
  margin-bottom: 0;
}
.db-category {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 6px 14px;
  cursor: pointer;
  font-size: 12px;
  color: var(--color-text-muted, #6c7086);
  transition: all 0.12s;
}
.db-category:hover {
  background: var(--color-bg-hover, rgba(255,255,255,0.05));
  color: var(--color-text-primary, #cdd6f4);
}
.db-category.expanded {
  color: var(--color-text-primary, #cdd6f4);
}
.category-arrow {
  color: var(--color-text-muted, #6c7086);
  transition: transform 0.15s;
  flex-shrink: 0;
}
.db-category.expanded .category-arrow {
  transform: rotate(90deg);
}
.category-name {
  flex: 1;
}
.category-count {
  font-size: 10px;
  color: var(--color-text-muted, #6c7086);
  padding: 1px 6px;
  background: rgba(255,255,255,0.03);
  border-radius: 8px;
}

.db-type-list {
  padding: 0;
}
.db-type-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 5px 14px 5px 32px;
  cursor: pointer;
  font-size: 12px;
  color: var(--color-text-muted, #6c7086);
  transition: all 0.12s;
}
.db-type-item:hover {
  background: var(--color-bg-hover, rgba(255,255,255,0.05));
  color: var(--color-text-secondary, #a6adc8);
}
.db-type-item.selected {
  background: var(--color-bg-active, #2a2a3c);
  color: var(--color-accent, #89b4fa);
  font-weight: 600;
}
.db-type-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  flex-shrink: 0;
}
.db-type-name {
  flex: 1;
}
.db-type-driver-count {
  font-size: 10px;
  color: var(--color-text-muted, #6c7086);
  padding: 1px 6px;
  background: rgba(255,255,255,0.03);
  border-radius: 8px;
}
</style>