<template>
  <div class="connection-sidebar">
    <!-- 搜索框 -->
    <div class="sidebar-search">
      <Search :size="14" class="search-icon" />
      <input
        v-model="searchQuery"
        type="text"
        placeholder="搜索数据库类型..."
        class="search-input"
      />
    </div>

    <!-- 数据库分类树 -->
    <div class="database-tree">
      <div v-for="category in filteredCategories" :key="category.key" class="tree-category">
        <!-- 分类头部 -->
        <div class="category-header" @click="toggleCategory(category)">
          <ChevronRight :size="14" class="category-icon" :class="{ expanded: category.expanded }" />
          <span class="category-label">{{ category.label }}</span>
          <span class="category-count">{{ category.databases.length }}</span>
        </div>

        <!-- 分类内容 -->
        <div v-show="category.expanded" class="category-items">
          <div
            v-for="db in category.databases"
            :key="db.id"
            class="database-item"
            :class="{ active: selectedDriver?.id === db.id }"
            @click="selectDatabase(db)"
          >
            <DbIcon :type="db.id" class="db-icon" />
            <div class="db-info">
              <span class="db-name">{{ db.name }}</span>
              <span class="db-desc">{{ db.description }}</span>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- 最近使用 -->
    <div v-if="recentDrivers.length > 0" class="recent-section">
      <div class="section-header">
        <Clock :size="14" />
        <span>最近使用</span>
      </div>
      <div class="recent-list">
        <div
          v-for="driver in recentDrivers"
          :key="driver.id"
          class="recent-item"
          :class="{ active: selectedDriver?.id === driver.id }"
          @click="selectDatabase(driver)"
        >
          <DbIcon :type="driver.id" class="db-icon" />
          <span>{{ driver.name }}</span>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { Search, ChevronRight, Clock } from 'lucide-vue-next'
import { ref, computed } from 'vue'

import DbIcon from '@/shared/components/common/DbIcon.vue'

import type { DriverDescriptor } from '../types/connection'

interface DatabaseCategory {
  key: string
  label: string
  expanded: boolean
  databases: DriverDescriptor[]
}

interface Props {
  drivers: DriverDescriptor[]
  selectedDriver: DriverDescriptor | null
  recentDriverIds?: string[]
}

const props = withDefaults(defineProps<Props>(), {
  recentDriverIds: () => [],
})

const emit = defineEmits<{
  select: [driver: DriverDescriptor]
}>()

const searchQuery = ref('')

// 数据库分类
const categories = computed<DatabaseCategory[]>(() => {
  const allDrivers = props.drivers

  return [
    {
      key: 'relational',
      label: '关系型数据库',
      expanded: true,
      databases: allDrivers.filter(d => ['mysql', 'postgres', 'mariadb'].includes(d.id)),
    },
    {
      key: 'file-based',
      label: '文件数据库',
      expanded: true,
      databases: allDrivers.filter(d => ['sqlite', 'duckdb'].includes(d.id)),
    },
    {
      key: 'nosql',
      label: 'NoSQL',
      expanded: false,
      databases: allDrivers.filter(d => ['mongodb', 'redis'].includes(d.id)),
    },
  ].filter(cat => cat.databases.length > 0)
})

// 最近使用的驱动
const recentDrivers = computed(() => {
  return props.recentDriverIds
    .map(id => props.drivers.find(d => d.id === id))
    .filter((d): d is DriverDescriptor => d !== undefined)
    .slice(0, 5)
})

// 过滤后的分类（支持搜索）
const filteredCategories = computed(() => {
  if (!searchQuery.value) return categories.value

  const query = searchQuery.value.toLowerCase()
  return categories.value
    .map(cat => ({
      ...cat,
      databases: cat.databases.filter(
        db => db.name.toLowerCase().includes(query) || db.description?.toLowerCase().includes(query)
      ),
    }))
    .filter(cat => cat.databases.length > 0)
})

function toggleCategory(category: DatabaseCategory) {
  category.expanded = !category.expanded
}

function selectDatabase(driver: DriverDescriptor) {
  emit('select', driver)
}
</script>

<style scoped>
.connection-sidebar {
  width: 260px;
  background: var(--bg-secondary);
  border-right: 1px solid var(--border-color);
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

/* 搜索框 */
.sidebar-search {
  padding: 12px;
  border-bottom: 1px solid var(--border-color);
  position: relative;
}

.search-icon {
  position: absolute;
  left: 20px;
  top: 50%;
  transform: translateY(-50%);
  color: var(--text-tertiary);
  pointer-events: none;
}

.search-input {
  width: 100%;
  height: 32px;
  padding: 0 12px 0 32px;
  background: var(--bg-tertiary);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-md);
  color: var(--text-primary);
  font-size: 13px;
  outline: none;
  transition: all 0.2s;
}

.search-input:focus {
  border-color: var(--primary-color);
  background: var(--bg-primary);
}

.search-input::placeholder {
  color: var(--text-tertiary);
}

/* 数据库树 */
.database-tree {
  flex: 1;
  overflow-y: auto;
  padding: 8px 0;
}

.tree-category {
  margin-bottom: 4px;
}

.category-header {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 8px 12px;
  cursor: pointer;
  user-select: none;
  transition: background 0.2s;
}

.category-header:hover {
  background: var(--bg-tertiary);
}

.category-icon {
  color: var(--text-tertiary);
  transition: transform 0.2s;
  flex-shrink: 0;
}

.category-icon.expanded {
  transform: rotate(90deg);
}

.category-label {
  flex: 1;
  font-size: 12px;
  font-weight: 600;
  color: var(--text-secondary);
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.category-count {
  font-size: 11px;
  color: var(--text-tertiary);
  background: var(--bg-tertiary);
  padding: 2px 6px;
  border-radius: 10px;
  min-width: 20px;
  text-align: center;
}

.category-items {
  padding: 2px 0;
}

.database-item {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 8px 12px 8px 24px;
  cursor: pointer;
  transition: all 0.2s;
  border-left: 2px solid transparent;
}

.database-item:hover {
  background: var(--bg-tertiary);
}

.database-item.active {
  background: var(--primary-light);
  border-left-color: var(--primary-color);
}

.db-icon {
  width: 24px;
  height: 24px;
  flex-shrink: 0;
}

.db-info {
  flex: 1;
  min-width: 0;
}

.db-name {
  display: block;
  font-size: 13px;
  font-weight: 500;
  color: var(--text-primary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.db-desc {
  display: block;
  font-size: 11px;
  color: var(--text-tertiary);
  margin-top: 2px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

/* 最近使用 */
.recent-section {
  border-top: 1px solid var(--border-color);
  padding: 12px;
}

.section-header {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 12px;
  font-weight: 600;
  color: var(--text-secondary);
  margin-bottom: 8px;
}

.recent-list {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.recent-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 6px 8px;
  border-radius: var(--radius-md);
  cursor: pointer;
  transition: all 0.2s;
  font-size: 13px;
  color: var(--text-primary);
}

.recent-item:hover {
  background: var(--bg-tertiary);
}

.recent-item.active {
  background: var(--primary-light);
  color: var(--primary-color);
}
</style>
