<template>
  <div class="driver-tab-content">
    <div class="driver-info-section">
      <h4 class="section-title">驱动信息</h4>
      <div class="driver-details">
        <div class="detail-item">
          <span class="detail-label">驱动名称</span>
          <span class="detail-value">{{ driver.name }}</span>
        </div>
        <div class="detail-item">
          <span class="detail-label">版本</span>
          <span class="detail-value">{{ driver.version || '最新' }}</span>
        </div>
        <div class="detail-item">
          <span class="detail-label">特性</span>
          <div class="feature-tags">
            <span
              v-for="feature in driver.features"
              :key="feature"
              class="feature-tag"
            >
              {{ feature }}
            </span>
          </div>
        </div>
      </div>
    </div>

    <div v-if="extraOptions.length > 0" class="driver-options-section">
      <h4 class="section-title">驱动选项</h4>
      <div class="options-list">
        <div
          v-for="option in extraOptions"
          :key="option.name"
          class="option-item"
        >
          <label class="option-label">{{ option.label }}</label>
          <input
            v-if="option.type === 'string' || option.type === 'number'"
            v-model="(options as Record<string, unknown>)[option.name]"
            :type="option.type"
            class="form-input"
            :placeholder="option.description"
          />
          <select
            v-else-if="option.type === 'select'"
            v-model="(options as Record<string, unknown>)[option.name]"
            class="form-select"
          >
            <option
              v-for="opt in option.options"
              :key="opt.value"
              :value="opt.value"
            >
              {{ opt.label }}
            </option>
          </select>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'

import type { DriverDescriptor } from '../types/connection'
import type { DriverOption } from '../types/driver'

interface Props {
  driver: DriverDescriptor
  options: Record<string, unknown>
}

const props = defineProps<Props>()

const extraOptions = computed<DriverOption[]>(() => {
  return props.driver.extraOptions || props.driver.extra_options || []
})
</script>

<style scoped>
.driver-tab-content {
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.driver-info-section,
.driver-options-section {
  padding: 16px;
  background: var(--bg-secondary);
  border-radius: var(--radius-md);
  border: 1px solid var(--border-color);
}

.section-title {
  font-size: 14px;
  font-weight: 600;
  color: var(--text-primary);
  margin: 0 0 12px 0;
}

.driver-details {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.detail-item {
  display: flex;
  align-items: flex-start;
  gap: 12px;
}

.detail-label {
  min-width: 80px;
  font-size: 13px;
  color: var(--text-secondary);
}

.detail-value {
  flex: 1;
  font-size: 13px;
  color: var(--text-primary);
}

.feature-tags {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}

.feature-tag {
  padding: 4px 8px;
  background: var(--bg-tertiary);
  border-radius: var(--radius-sm);
  font-size: 12px;
  color: var(--text-secondary);
}

.options-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.option-item {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.option-label {
  font-size: 13px;
  font-weight: 500;
  color: var(--text-primary);
}

.form-input,
.form-select {
  height: 32px;
  padding: 0 12px;
  background: var(--bg-tertiary);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-md);
  color: var(--text-primary);
  font-size: 13px;
  outline: none;
  transition: all 0.2s;
}

.form-input:focus,
.form-select:focus {
  border-color: var(--primary-color);
  background: var(--bg-primary);
}

.form-select {
  cursor: pointer;
}
</style>