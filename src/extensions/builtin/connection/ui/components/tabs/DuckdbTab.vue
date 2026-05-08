<template>
  <div class="duckdb-tab">
    <!-- 功能说明 -->
    <div class="feature-intro">
      <div class="intro-icon">
        <Zap :size="20" />
      </div>
      <div class="intro-content">
        <h4 class="intro-title">{{ t('connection.duckdbTab.title') }}</h4>
        <p class="intro-desc">
          {{ t('connection.duckdbTab.description') }}
        </p>
      </div>
    </div>

    <!-- 启用开关 -->
    <div class="enable-section">
      <div class="enable-header">
        <label class="enable-label">{{ t('connection.duckdbTab.enable') }}</label>
        <button
          type="button"
          class="enable-switch"
          :class="{ active: formData.useDuckdbFed }"
          @click="toggleDuckdbFed"
        >
          <span class="switch-knob"></span>
        </button>
      </div>
      <p v-if="!formData.useDuckdbFed" class="enable-hint">
        {{ t('connection.duckdbTab.enableHint') }}
      </p>
    </div>

    <!-- 配置选项（启用后显示） -->
    <div v-if="formData.useDuckdbFed" class="config-section">
      <!-- 缓存策略 -->
      <div class="config-group">
        <h5 class="group-title">{{ t('connection.duckdbTab.cachePolicy') }}</h5>

        <div class="config-item">
          <label class="item-label">
            <Database :size="14" class="item-icon" />
            {{ t('connection.duckdbTab.cacheMode') }}
          </label>
          <select v-model="cacheMode" class="form-select">
            <option value="manual">{{ t('connection.duckdbTab.manual') }}</option>
            <option value="auto">{{ t('connection.duckdbTab.auto') }}</option>
            <option value="smart">{{ t('connection.duckdbTab.smart') }}</option>
          </select>
          <p class="item-hint">{{ cacheModeHint }}</p>
        </div>

        <div class="config-item">
          <label class="item-label">
            <Clock :size="14" class="item-icon" />
            {{ t('connection.duckdbTab.cacheTtl') }}
          </label>
          <div class="time-input-wrapper">
            <input v-model.number="cacheTtl" type="number" class="form-input" min="1" max="1440" />
            <select v-model="cacheTtlUnit" class="form-select">
              <option value="minutes">{{ t('connection.duckdbTab.minutes') }}</option>
              <option value="hours">{{ t('connection.duckdbTab.hours') }}</option>
              <option value="days">{{ t('connection.duckdbTab.days') }}</option>
            </select>
          </div>
        </div>
      </div>

      <!-- 存储配置 -->
      <div class="config-group">
        <h5 class="group-title">{{ t('connection.duckdbTab.storageConfig') }}</h5>

        <div class="config-item">
          <label class="item-label">
            <FolderOpen :size="14" class="item-icon" />
            {{ t('connection.duckdbTab.cachePath') }}
          </label>
          <div class="file-input-wrapper">
            <input
              v-model="cachePath"
              type="text"
              class="form-input"
              :placeholder="t('connection.duckdbTab.cachePathPlaceholder')"
            />
            <button type="button" class="btn-file" @click="selectCachePath">
              <FolderOpen :size="14" />
            </button>
          </div>
          <p class="item-hint"> {{ t('connection.duckdbTab.cachePathDefault') }} </p>
        </div>

        <div class="config-item">
          <label class="item-label">
            <HardDrive :size="14" class="item-icon" />
            {{ t('connection.duckdbTab.maxCacheSize') }}
          </label>
          <div class="size-input-wrapper">
            <input
              v-model.number="maxCacheSize"
              type="number"
              class="form-input"
              min="100"
              max="10240"
            />
            <span class="size-unit">MB</span>
          </div>
        </div>
      </div>

      <!-- 高级选项 -->
      <div class="config-group">
        <h5 class="group-title">{{ t('connection.duckdbTab.advancedOptions') }}</h5>

        <label class="checkbox-wrapper">
          <input v-model="autoSync" type="checkbox" />
          <span class="checkmark"></span>
          <span class="checkbox-label">{{ t('connection.duckdbTab.autoSync') }}</span>
        </label>

        <label class="checkbox-wrapper">
          <input v-model="compressCache" type="checkbox" />
          <span class="checkmark"></span>
          <span class="checkbox-label">{{ t('connection.duckdbTab.compressCache') }}</span>
        </label>

        <label class="checkbox-wrapper">
          <input v-model="useDirectQuery" type="checkbox" />
          <span class="checkmark"></span>
          <span class="checkbox-label">{{ t('connection.duckdbTab.useDirectQuery') }}</span>
        </label>
      </div>

      <!-- 性能预估 -->
      <div class="performance-estimate">
        <h5 class="estimate-title">
          <TrendingUp :size="14" />
          {{ t('connection.duckdbTab.performanceEstimate') }}
        </h5>
        <div class="estimate-grid">
          <div class="estimate-item">
            <span class="estimate-label">{{ t('connection.duckdbTab.querySpeedup') }}</span>
            <span class="estimate-value">5-50x</span>
          </div>
          <div class="estimate-item">
            <span class="estimate-label">{{ t('connection.duckdbTab.firstQueryLatency') }}</span>
            <span class="estimate-value">+10-30%</span>
          </div>
          <div class="estimate-item">
            <span class="estimate-label">{{
              t('connection.duckdbTab.subsequentQueryLatency')
            }}</span>
            <span class="estimate-value">-80-95%</span>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { Zap, Database, Clock, FolderOpen, HardDrive, TrendingUp } from 'lucide-vue-next'
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'

const { t } = useI18n()

import type { ConnectionConfig } from '../../types/connection'

interface Props {
  formData: Partial<ConnectionConfig>
}

const props = defineProps<Props>()

const emit = defineEmits<{
  'update:formData': [data: Partial<ConnectionConfig>]
}>()

function toggleDuckdbFed() {
  emit('update:formData', {
    ...props.formData,
    useDuckdbFed: !props.formData.useDuckdbFed,
  })
}

const cacheMode = computed({
  get: () => (props.formData.options?.duckdbCacheMode as string) || 'auto',
  set: (val: string) => {
    emit('update:formData', {
      ...props.formData,
      options: {
        ...props.formData.options,
        duckdbCacheMode: val,
      },
    })
  },
})

const cacheModeHint = computed(() => {
  const hints: Record<string, string> = {
    manual: t('connection.duckdbTab.manualHint'),
    auto: t('connection.duckdbTab.autoHint'),
    smart: t('connection.duckdbTab.smartHint'),
  }
  return hints[cacheMode.value] || ''
})

const cacheTtl = computed({
  get: () => props.formData.options?.duckdbCacheTtl || 60,
  set: (val: number) => {
    emit('update:formData', {
      ...props.formData,
      options: {
        ...props.formData.options,
        duckdbCacheTtl: val,
      },
    })
  },
})

const cacheTtlUnit = computed({
  get: () => props.formData.options?.duckdbCacheTtlUnit || 'minutes',
  set: (val: string) => {
    emit('update:formData', {
      ...props.formData,
      options: {
        ...props.formData.options,
        duckdbCacheTtlUnit: val,
      },
    })
  },
})

const cachePath = computed({
  get: () => props.formData.options?.duckdbCachePath || '',
  set: (val: string) => {
    emit('update:formData', {
      ...props.formData,
      options: {
        ...props.formData.options,
        duckdbCachePath: val,
      },
    })
  },
})

const maxCacheSize = computed({
  get: () => props.formData.options?.duckdbMaxCacheSize || 1024,
  set: (val: number) => {
    emit('update:formData', {
      ...props.formData,
      options: {
        ...props.formData.options,
        duckdbMaxCacheSize: val,
      },
    })
  },
})

const autoSync = computed({
  get: () => props.formData.options?.duckdbAutoSync || false,
  set: (val: boolean) => {
    emit('update:formData', {
      ...props.formData,
      options: {
        ...props.formData.options,
        duckdbAutoSync: val,
      },
    })
  },
})

const compressCache = computed({
  get: () => props.formData.options?.duckdbCompressCache || false,
  set: (val: boolean) => {
    emit('update:formData', {
      ...props.formData,
      options: {
        ...props.formData.options,
        duckdbCompressCache: val,
      },
    })
  },
})

const useDirectQuery = computed({
  get: () => props.formData.options?.duckdbUseDirectQuery || false,
  set: (val: boolean) => {
    emit('update:formData', {
      ...props.formData,
      options: {
        ...props.formData.options,
        duckdbUseDirectQuery: val,
      },
    })
  },
})

function selectCachePath() {
  // TODO: 实现文件夹选择
  console.log('选择缓存路径')
}
</script>

<style scoped>
.duckdb-tab {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

/* 功能说明 */
.feature-intro {
  display: flex;
  gap: 12px;
  padding: 16px;
  background: linear-gradient(135deg, rgba(22, 93, 255, 0.1) 0%, rgba(22, 93, 255, 0.05) 100%);
  border-radius: var(--radius-md);
  border: 1px solid rgba(22, 93, 255, 0.2);
}

.intro-icon {
  width: 40px;
  height: 40px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--primary-color);
  border-radius: var(--radius-md);
  color: white;
  flex-shrink: 0;
}

.intro-content {
  flex: 1;
}

.intro-title {
  font-size: 14px;
  font-weight: 600;
  color: var(--text-primary);
  margin: 0 0 4px 0;
}

.intro-desc {
  font-size: 12px;
  color: var(--text-secondary);
  line-height: 1.5;
  margin: 0;
}

/* 启用开关 */
.enable-section {
  padding: 16px;
  background: var(--bg-secondary);
  border-radius: var(--radius-md);
  border: 1px solid var(--border-color);
}

.enable-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.enable-label {
  font-size: 14px;
  font-weight: 500;
  color: var(--text-primary);
}

.enable-switch {
  position: relative;
  width: 40px;
  height: 20px;
  background: var(--border-color);
  border-radius: 10px;
  border: none;
  padding: 0;
  cursor: pointer;
  transition: all 0.2s;
}

.enable-switch.active {
  background: var(--primary-color);
}

.switch-knob {
  position: absolute;
  top: 2px;
  left: 2px;
  width: 16px;
  height: 16px;
  background: white;
  border-radius: 50%;
  transition: all 0.2s;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.2);
}

.enable-switch.active .switch-knob {
  left: 22px;
}

.enable-hint {
  margin-top: 8px;
  font-size: 12px;
  color: var(--text-tertiary);
}

/* 配置区域 */
.config-section {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.config-group {
  padding: 16px;
  background: var(--bg-secondary);
  border-radius: var(--radius-md);
  border: 1px solid var(--border-color);
}

.group-title {
  font-size: 13px;
  font-weight: 600;
  color: var(--text-primary);
  margin: 0 0 12px 0;
  padding-bottom: 8px;
  border-bottom: 1px solid var(--border-color);
}

.config-item {
  margin-bottom: 12px;
}

.config-item:last-child {
  margin-bottom: 0;
}

.item-label {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 13px;
  font-weight: 500;
  color: var(--text-primary);
  margin-bottom: 6px;
}

.item-icon {
  color: var(--text-tertiary);
}

.item-hint {
  margin-top: 4px;
  font-size: 11px;
  color: var(--text-tertiary);
}

.item-hint code {
  background: var(--bg-tertiary);
  padding: 2px 4px;
  border-radius: 3px;
  font-family: var(--font-mono);
  font-size: 11px;
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

.time-input-wrapper {
  display: flex;
  gap: 8px;
}

.time-input-wrapper .form-input {
  flex: 1;
}

.time-input-wrapper .form-select {
  width: 100px;
}

.file-input-wrapper {
  display: flex;
  gap: 8px;
}

.file-input-wrapper .form-input {
  flex: 1;
}

.btn-file {
  width: 32px;
  height: 32px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--bg-tertiary);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-md);
  color: var(--text-secondary);
  cursor: pointer;
  transition: all 0.2s;
}

.btn-file:hover {
  background: var(--bg-hover);
  color: var(--text-primary);
  border-color: var(--primary-color);
}

.size-input-wrapper {
  display: flex;
  align-items: center;
  gap: 8px;
}

.size-input-wrapper .form-input {
  flex: 1;
}

.size-unit {
  font-size: 13px;
  color: var(--text-secondary);
  white-space: nowrap;
}

/* 复选框 */
.checkbox-wrapper {
  display: flex;
  align-items: center;
  gap: 8px;
  cursor: pointer;
  font-size: 13px;
  color: var(--text-secondary);
  margin-bottom: 8px;
}

.checkbox-wrapper:last-child {
  margin-bottom: 0;
}

.checkbox-wrapper input {
  display: none;
}

.checkmark {
  position: relative;
  width: 16px;
  height: 16px;
  border: 1px solid var(--border-color);
  border-radius: 3px;
  background: var(--bg-tertiary);
  transition: all 0.2s;
  flex-shrink: 0;
}

.checkbox-wrapper input:checked + .checkmark {
  background: var(--primary-color);
  border-color: var(--primary-color);
}

.checkbox-wrapper input:checked + .checkmark::after {
  content: '';
  position: absolute;
  left: 5px;
  top: 2px;
  width: 4px;
  height: 8px;
  border: solid white;
  border-width: 0 2px 2px 0;
  transform: rotate(45deg);
}

.checkbox-label {
  user-select: none;
}

/* 性能预估 */
.performance-estimate {
  padding: 16px;
  background: linear-gradient(135deg, rgba(0, 180, 42, 0.1) 0%, rgba(0, 180, 42, 0.05) 100%);
  border-radius: var(--radius-md);
  border: 1px solid rgba(0, 180, 42, 0.2);
}

.estimate-title {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 13px;
  font-weight: 600;
  color: var(--success-color);
  margin: 0 0 12px 0;
}

.estimate-grid {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 12px;
}

.estimate-item {
  display: flex;
  flex-direction: column;
  gap: 4px;
  padding: 8px;
  background: rgba(255, 255, 255, 0.05);
  border-radius: var(--radius-sm);
}

.estimate-label {
  font-size: 11px;
  color: var(--text-secondary);
}

.estimate-value {
  font-size: 16px;
  font-weight: 600;
  color: var(--success-color);
  font-family: var(--font-mono);
}
</style>
