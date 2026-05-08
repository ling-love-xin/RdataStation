<template>
  <div class="driver-select">
    <!-- 搜索和筛选 -->
    <div class="driver-header">
      <div class="search-box">
        <svg
          class="search-icon"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
        >
          <circle cx="11" cy="11" r="8" />
          <path d="m21 21-4.35-4.35" />
        </svg>
        <input
          v-model="searchQuery"
          type="text"
          class="search-input"
          placeholder="搜索数据库类型..."
        />
      </div>
      <div class="filter-tabs">
        <button
          v-for="category in categories"
          :key="category.key"
          class="filter-tab"
          :class="{ active: selectedCategory === category.key }"
          @click="selectedCategory = category.key"
        >
          {{ category.label }}
        </button>
      </div>
    </div>

    <!-- 驱动列表 -->
    <div v-if="loading" class="loading-state">
      <div class="spinner"></div>
      <p>加载驱动列表...</p>
    </div>

    <div v-else-if="filteredDrivers.length === 0" class="empty-state">
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
        <circle cx="12" cy="12" r="10" />
        <path d="M8 12h8M12 8v8" />
      </svg>
      <p>未找到匹配的数据库驱动</p>
    </div>

    <div v-else class="driver-grid">
      <div
        v-for="driver in filteredDrivers"
        :key="driver.id"
        class="driver-card"
        :class="{
          selected: selectedDriver?.id === driver.id,
          recommended: isRecommended(driver.id),
        }"
        @click="selectDriver(driver)"
      >
        <div class="driver-icon-wrapper">
          <DbIcon :type="driver.id" />
          <div v-if="isRecommended(driver.id)" class="recommended-badge">推荐</div>
        </div>
        <div class="driver-info">
          <h3 class="driver-name">{{ driver.name }}</h3>
          <p class="driver-desc">{{ driver.description }}</p>
        </div>
        <div v-if="selectedDriver?.id === driver.id" class="selected-indicator">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3">
            <polyline points="20 6 9 17 4 12" />
          </svg>
        </div>
      </div>
    </div>

    <!-- 选中驱动信息 -->
    <div v-if="selectedDriver" class="selected-info">
      <div class="info-item">
        <span class="info-label">默认端口:</span>
        <span class="info-value">{{
          selectedDriver.defaultPort || selectedDriver.default_port || '无'
        }}</span>
      </div>
      <div class="info-item">
        <span class="info-label">支持功能:</span>
        <div class="feature-tags">
          <span v-for="(feature, idx) in selectedDriver.features" :key="idx" class="feature-tag">{{
            feature
          }}</span>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'

import DbIcon from '@/shared/components/common/DbIcon.vue'

import type { DriverDescriptor } from '../types/driver'

interface Props {
  drivers: DriverDescriptor[]
  loading?: boolean
  modelValue?: DriverDescriptor | null
}

interface Emits {
  (e: 'update:modelValue', value: DriverDescriptor | null): void
  (e: 'select', value: DriverDescriptor): void
}

const props = withDefaults(defineProps<Props>(), {
  loading: false,
  modelValue: null,
})

const emit = defineEmits<Emits>()

// 搜索和筛选状态
const searchQuery = ref('')
const selectedCategory = ref('all')

// 分类定义
const categories = [
  { key: 'all', label: '全部' },
  { key: 'relational', label: '关系型' },
  { key: 'nosql', label: 'NoSQL' },
  { key: 'file', label: '文件型' },
  { key: 'cloud', label: '云数据库' },
]

// 推荐驱动列表
const recommendedDrivers = ['mysql', 'postgres', 'sqlite', 'duckdb']

// 驱动分类映射
const driverCategories: Record<string, string[]> = {
  relational: ['mysql', 'postgres', 'mariadb', 'oracle', 'sqlserver'],
  nosql: ['mongodb', 'redis', 'cassandra', 'dynamodb'],
  file: ['sqlite', 'duckdb'],
  cloud: ['snowflake', 'bigquery', 'redshift', 'athena'],
}

// 当前选中的驱动
const selectedDriver = computed({
  get: () => props.modelValue,
  set: value => emit('update:modelValue', value),
})

// 过滤后的驱动列表
const filteredDrivers = computed(() => {
  let result = props.drivers

  // 按分类筛选
  if (selectedCategory.value !== 'all') {
    const categoryDrivers = driverCategories[selectedCategory.value] || []
    result = result.filter(d => categoryDrivers.includes(d.id))
  }

  // 按搜索词筛选
  if (searchQuery.value.trim()) {
    const query = searchQuery.value.toLowerCase()
    result = result.filter(
      d =>
        d.name.toLowerCase().includes(query) ||
        (d.description || '').toLowerCase().includes(query) ||
        d.id.toLowerCase().includes(query)
    )
  }

  // 排序：推荐的在前
  return result.sort((a, b) => {
    const aRecommended = recommendedDrivers.indexOf(a.id)
    const bRecommended = recommendedDrivers.indexOf(b.id)
    if (aRecommended !== -1 && bRecommended === -1) return -1
    if (aRecommended === -1 && bRecommended !== -1) return 1
    return a.name.localeCompare(b.name)
  })
})

// 检查是否为推荐驱动
function isRecommended(driverId: string): boolean {
  return recommendedDrivers.includes(driverId)
}

// 选择驱动
function selectDriver(driver: DriverDescriptor) {
  selectedDriver.value = driver
  emit('select', driver)
}
</script>

<style scoped>
.driver-select {
  display: flex;
  flex-direction: column;
  gap: 20px;
}

/* 头部搜索和筛选 */
.driver-header {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.search-box {
  position: relative;
  display: flex;
  align-items: center;
}

.search-icon {
  position: absolute;
  left: 12px;
  width: 18px;
  height: 18px;
  color: var(--text-tertiary);
}

.search-input {
  width: 100%;
  padding: 10px 12px 10px 40px;
  border: 1px solid var(--border-color);
  border-radius: 8px;
  font-size: 14px;
  background: var(--bg-primary);
  color: var(--text-primary);
  transition: all 0.2s ease;
}

.search-input:focus {
  outline: none;
  border-color: var(--primary-color);
  box-shadow: 0 0 0 3px rgba(22, 93, 255, 0.1);
}

.search-input::placeholder {
  color: var(--text-tertiary);
}

/* 筛选标签 */
.filter-tabs {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
}

.filter-tab {
  padding: 6px 14px;
  border: 1px solid var(--border-color);
  border-radius: 6px;
  font-size: 13px;
  font-weight: 500;
  color: var(--text-secondary);
  background: var(--bg-primary);
  cursor: pointer;
  transition: all 0.2s ease;
}

.filter-tab:hover {
  border-color: var(--primary-color);
  color: var(--primary-color);
}

.filter-tab.active {
  background: var(--primary-color);
  border-color: var(--primary-color);
  color: white;
}

/* 加载状态 */
.loading-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 16px;
  padding: 60px 20px;
  color: var(--text-secondary);
}

.spinner {
  width: 40px;
  height: 40px;
  border: 3px solid var(--border-color);
  border-top-color: var(--primary-color);
  border-radius: 50%;
  animation: spin 1s linear infinite;
}

@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}

/* 空状态 */
.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 12px;
  padding: 60px 20px;
  color: var(--text-tertiary);
}

.empty-state svg {
  width: 48px;
  height: 48px;
  opacity: 0.5;
}

/* 驱动网格 */
.driver-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(180px, 1fr));
  gap: 12px;
  max-height: 400px;
  overflow-y: auto;
  padding-right: 4px;
}

.driver-card {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 10px;
  padding: 20px 16px;
  background: var(--bg-primary);
  border: 2px solid var(--border-color);
  border-radius: 10px;
  cursor: pointer;
  transition: all 0.2s ease;
  position: relative;
}

.driver-card:hover {
  border-color: var(--primary-color);
  transform: translateY(-2px);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
}

.driver-card.selected {
  border-color: var(--primary-color);
  background: rgba(22, 93, 255, 0.05);
}

.driver-card.recommended {
  border-color: var(--warning-color);
}

.driver-icon-wrapper {
  position: relative;
}

.recommended-badge {
  position: absolute;
  top: -4px;
  right: -4px;
  padding: 2px 6px;
  background: var(--warning-color);
  color: white;
  font-size: 10px;
  font-weight: 600;
  border-radius: 4px;
}

.driver-info {
  text-align: center;
}

.driver-name {
  font-size: 14px;
  font-weight: 600;
  color: var(--text-primary);
  margin: 0 0 4px;
}

.driver-desc {
  font-size: 11px;
  color: var(--text-tertiary);
  margin: 0;
  line-height: 1.3;
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
}

.selected-indicator {
  position: absolute;
  top: 8px;
  right: 8px;
  width: 20px;
  height: 20px;
  background: var(--primary-color);
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  color: white;
}

.selected-indicator svg {
  width: 12px;
  height: 12px;
}

/* 选中驱动信息 */
.selected-info {
  display: flex;
  gap: 24px;
  padding: 16px;
  background: var(--bg-secondary);
  border-radius: 8px;
  margin-top: 8px;
}

.info-item {
  display: flex;
  align-items: center;
  gap: 8px;
}

.info-label {
  font-size: 12px;
  color: var(--text-tertiary);
}

.info-value {
  font-size: 13px;
  font-weight: 500;
  color: var(--text-primary);
}

.feature-tags {
  display: flex;
  gap: 6px;
}

.feature-tag {
  padding: 2px 8px;
  background: rgba(22, 93, 255, 0.1);
  color: var(--primary-color);
  font-size: 11px;
  font-weight: 500;
  border-radius: 4px;
}

/* 滚动条 */
.driver-grid::-webkit-scrollbar {
  width: 6px;
}

.driver-grid::-webkit-scrollbar-track {
  background: transparent;
}

.driver-grid::-webkit-scrollbar-thumb {
  background: var(--border-color);
  border-radius: 3px;
}

.driver-grid::-webkit-scrollbar-thumb:hover {
  background: var(--text-tertiary);
}
</style>
