<template>
  <div class="duckdb-acceleration-tab">
    <div class="tab-description">
      <p class="description-text">
        将当前连接信息保存为 DuckDB 的秘密（Secret），用于联邦查询时访问远程数据源。
      </p>
    </div>

    <!-- 加速开关 -->
    <div class="acceleration-toggle">
      <label class="toggle-wrapper">
        <input v-model="useDuckdbFed" type="checkbox" />
        <span class="toggle-slider"></span>
        <div class="toggle-info">
          <span class="toggle-label">启用 DuckDB 本地加速</span>
          <span class="toggle-desc">将连接信息存储为 DuckDB Secret，支持联邦查询</span>
        </div>
      </label>
    </div>

    <!-- 加速配置 -->
    <div v-if="useDuckdbFed" class="acceleration-config">
      <div class="config-section">
        <h4 class="section-title">
          <Key :size="16" class="section-icon" />
          连接信息（将存储为 DuckDB Secret）
        </h4>
        <div class="info-grid">
          <div class="info-item">
            <span class="info-label">驱动类型</span>
            <span class="info-value">{{ formData.driver || '未选择' }}</span>
          </div>
          <div class="info-item">
            <span class="info-label">主机</span>
            <span class="info-value">{{ formData.host || '未配置' }}</span>
          </div>
          <div class="info-item">
            <span class="info-label">端口</span>
            <span class="info-value">{{ formData.port || '未配置' }}</span>
          </div>
          <div class="info-item">
            <span class="info-label">数据库</span>
            <span class="info-value">{{ formData.database || '未配置' }}</span>
          </div>
          <div class="info-item">
            <span class="info-label">用户名</span>
            <span class="info-value">{{ formData.username || '未配置' }}</span>
          </div>
        </div>
      </div>

      <div class="config-section">
        <h4 class="section-title">
          <Zap :size="16" class="section-icon" />
          缓存策略
        </h4>
        <div class="strategy-options">
          <label class="strategy-option">
            <input v-model="cacheStrategy" type="radio" value="manual" />
            <span class="radio-mark"></span>
            <div class="option-info">
              <span class="option-name">手动缓存</span>
              <span class="option-desc">手动触发数据导入和刷新</span>
            </div>
          </label>
          <label class="strategy-option">
            <input v-model="cacheStrategy" type="radio" value="auto" />
            <span class="radio-mark"></span>
            <div class="option-info">
              <span class="option-name">自动缓存</span>
              <span class="option-desc">查询时自动导入最新数据</span>
            </div>
          </label>
          <label class="strategy-option">
            <input v-model="cacheStrategy" type="radio" value="sync" />
            <span class="radio-mark"></span>
            <div class="option-info">
              <span class="option-name">实时同步</span>
              <span class="option-desc">监听源数据库变更并同步</span>
            </div>
          </label>
        </div>
      </div>

      <div class="config-section">
        <h4 class="section-title">
          <Settings :size="16" class="section-icon" />
          性能设置
        </h4>
        <div class="form-grid">
          <div class="form-field">
            <label class="field-label">内存限制 (GB)</label>
            <input
              v-model.number="memoryLimit"
              type="number"
              class="field-input"
              placeholder="0 表示不限制"
              min="0"
              max="64"
            />
          </div>
          <div class="form-field">
            <label class="field-label">线程数</label>
            <select v-model="threadCount" class="field-select">
              <option value="auto">自动</option>
              <option value="1">1</option>
              <option value="2">2</option>
              <option value="4">4</option>
              <option value="8">8</option>
              <option value="16">16</option>
            </select>
          </div>
          <div class="form-field">
            <label class="field-label">批量大小</label>
            <input
              v-model.number="batchSize"
              type="number"
              class="field-input"
              placeholder="每次导入的行数"
              min="1000"
              max="1000000"
              step="1000"
            />
          </div>
        </div>
      </div>

      <div class="config-section">
        <h4 class="section-title">
          <Clock :size="16" class="section-icon" />
          缓存过期策略
        </h4>
        <div class="form-grid">
          <div class="form-field">
            <label class="field-label">缓存有效期 (分钟)</label>
            <input
              v-model.number="cacheTtl"
              type="number"
              class="field-input"
              placeholder="0 表示永不过期"
              min="0"
            />
          </div>
          <div class="form-field">
            <label class="field-label">自动清理</label>
            <label class="checkbox-wrapper">
              <input v-model="autoCleanup" type="checkbox" />
              <span class="checkmark"></span>
              <span class="checkbox-label">过期后自动清理缓存</span>
            </label>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { Key, Zap, Settings, Clock } from 'lucide-vue-next'
import { computed } from 'vue'

import type { ConnectionConfig } from '../../types/connection'

interface Props {
  formData: Partial<ConnectionConfig>
}

const props = defineProps<Props>()

const emit = defineEmits<{
  'update:formData': [data: Partial<ConnectionConfig>]
}>()

const useDuckdbFed = computed({
  get: () => props.formData.useDuckdbFed || false,
  set: (val: boolean) => {
    emit('update:formData', {
      ...props.formData,
      useDuckdbFed: val
    })
  }
})

const cacheStrategy = computed({
  get: () => (props.formData.options?.cacheStrategy as string) || 'manual',
  set: (val: string) => {
    emit('update:formData', {
      ...props.formData,
      options: {
        ...props.formData.options,
        cacheStrategy: val
      }
    })
  }
})

const memoryLimit = computed({
  get: () => (props.formData.options?.memoryLimit as number) || 0,
  set: (val: number) => {
    emit('update:formData', {
      ...props.formData,
      options: {
        ...props.formData.options,
        memoryLimit: val
      }
    })
  }
})

const threadCount = computed({
  get: () => (props.formData.options?.threadCount as string) || 'auto',
  set: (val: string) => {
    emit('update:formData', {
      ...props.formData,
      options: {
        ...props.formData.options,
        threadCount: val
      }
    })
  }
})

const batchSize = computed({
  get: () => (props.formData.options?.batchSize as number) || 10000,
  set: (val: number) => {
    emit('update:formData', {
      ...props.formData,
      options: {
        ...props.formData.options,
        batchSize: val
      }
    })
  }
})

const cacheTtl = computed({
  get: () => (props.formData.options?.cacheTtl as number) || 0,
  set: (val: number) => {
    emit('update:formData', {
      ...props.formData,
      options: {
        ...props.formData.options,
        cacheTtl: val
      }
    })
  }
})

const autoCleanup = computed({
  get: () => (props.formData.options?.autoCleanup as boolean) || false,
  set: (val: boolean) => {
    emit('update:formData', {
      ...props.formData,
      options: {
        ...props.formData.options,
        autoCleanup: val
      }
    })
  }
})
</script>

<style scoped>
.duckdb-acceleration-tab {
  display: flex;
  flex-direction: column;
  gap: 24px;
}

.tab-description {
  padding: 12px 16px;
  background: var(--bg-secondary);
  border-radius: var(--radius-md);
  border-left: 3px solid var(--primary-color);
}

.description-text {
  font-size: 13px;
  color: var(--text-secondary);
  margin: 0;
  line-height: 1.6;
}

.acceleration-toggle {
  padding: 16px;
  background: var(--bg-secondary);
  border-radius: var(--radius-md);
}

.toggle-wrapper {
  display: flex;
  align-items: center;
  gap: 12px;
  cursor: pointer;
}

.toggle-wrapper input {
  display: none;
}

.toggle-slider {
  width: 40px;
  height: 22px;
  background: var(--bg-tertiary);
  border-radius: 11px;
  position: relative;
  transition: background 0.2s;
  flex-shrink: 0;
}

.toggle-slider::after {
  content: '';
  position: absolute;
  width: 18px;
  height: 18px;
  background: white;
  border-radius: 50%;
  top: 2px;
  left: 2px;
  transition: transform 0.2s;
}

.toggle-wrapper input:checked + .toggle-slider {
  background: var(--primary-color);
}

.toggle-wrapper input:checked + .toggle-slider::after {
  transform: translateX(18px);
}

.toggle-info {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.toggle-label {
  font-size: 14px;
  font-weight: 500;
  color: var(--text-primary);
}

.toggle-desc {
  font-size: 12px;
  color: var(--text-tertiary);
}

.acceleration-config {
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.config-section {
  padding: 16px;
  background: var(--bg-secondary);
  border-radius: var(--radius-md);
}

.section-title {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 14px;
  font-weight: 600;
  color: var(--text-primary);
  margin: 0 0 16px 0;
}

.section-icon {
  color: var(--primary-color);
}

.info-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
  gap: 12px;
}

.info-item {
  display: flex;
  flex-direction: column;
  gap: 4px;
  padding: 10px 12px;
  background: var(--bg-tertiary);
  border-radius: var(--radius-sm);
}

.info-label {
  font-size: 11px;
  color: var(--text-tertiary);
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.info-value {
  font-size: 13px;
  color: var(--text-primary);
  font-family: var(--font-mono);
}

.strategy-options {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.strategy-option {
  display: flex;
  align-items: flex-start;
  gap: 10px;
  padding: 10px 12px;
  border-radius: var(--radius-sm);
  cursor: pointer;
  transition: background 0.15s;
}

.strategy-option:hover {
  background: var(--bg-tertiary);
}

.strategy-option input {
  display: none;
}

.radio-mark {
  width: 18px;
  height: 18px;
  border: 2px solid var(--border-color);
  border-radius: 50%;
  position: relative;
  flex-shrink: 0;
  margin-top: 1px;
  transition: border-color 0.15s;
}

.strategy-option input:checked ~ .radio-mark {
  border-color: var(--primary-color);
}

.strategy-option input:checked ~ .radio-mark::after {
  content: '';
  position: absolute;
  width: 10px;
  height: 10px;
  background: var(--primary-color);
  border-radius: 50%;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
}

.option-info {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.option-name {
  font-size: 13px;
  font-weight: 500;
  color: var(--text-primary);
}

.option-desc {
  font-size: 12px;
  color: var(--text-tertiary);
}

.form-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(180px, 1fr));
  gap: 12px;
}

.form-field {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.field-label {
  font-size: 12px;
  font-weight: 500;
  color: var(--text-secondary);
}

.field-input,
.field-select {
  padding: 8px 12px;
  border: 1px solid var(--border-color);
  border-radius: var(--radius-sm);
  background: var(--bg-tertiary);
  color: var(--text-primary);
  font-size: 13px;
  transition: border-color 0.15s;
}

.field-input:focus,
.field-select:focus {
  outline: none;
  border-color: var(--primary-color);
}

.checkbox-wrapper {
  display: flex;
  align-items: center;
  gap: 8px;
  cursor: pointer;
}

.checkbox-wrapper input {
  display: none;
}

.checkmark {
  width: 18px;
  height: 18px;
  border: 2px solid var(--border-color);
  border-radius: 4px;
  position: relative;
  flex-shrink: 0;
  transition: all 0.15s;
}

.checkbox-wrapper input:checked ~ .checkmark {
  background: var(--primary-color);
  border-color: var(--primary-color);
}

.checkbox-wrapper input:checked ~ .checkmark::after {
  content: '';
  position: absolute;
  width: 5px;
  height: 10px;
  border: solid white;
  border-width: 0 2px 2px 0;
  transform: rotate(45deg);
  top: 2px;
  left: 6px;
}

.checkbox-label {
  font-size: 13px;
  color: var(--text-secondary);
}
</style>
