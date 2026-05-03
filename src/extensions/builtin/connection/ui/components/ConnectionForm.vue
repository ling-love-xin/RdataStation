<template>
  <div class="connection-form">
    <!-- 连接名称 - 所有驱动都需要 -->
    <div class="form-section" :class="{ 'has-error': errors.name }">
      <label class="form-label">
        连接名称
        <span class="required">*</span>
      </label>
      <input
        v-model="formData.name"
        type="text"
        class="form-input"
        :class="{ error: errors.name }"
        :placeholder="connectionNamePlaceholder"
        @blur="validateField('name')"
      />
      <span v-if="errors.name" class="error-text">{{ errors.name }}</span>
    </div>

    <!-- 动态渲染驱动特定字段 -->
    <template v-for="(field, index) in driverFields" :key="index">
      <!-- 文件选择字段（SQLite/DuckDB） -->
      <div
        v-if="getFieldType(field) === 'file'"
        class="form-section"
        :class="{ 'has-error': errors[getFieldName(field)] }"
      >
        <label class="form-label">
          {{ field.label }}
          <span v-if="field.required" class="required">*</span>
        </label>
        <div class="file-input-wrapper">
          <input
            v-model="formData[getFieldName(field)]"
            type="text"
            class="form-input"
            :class="{ error: errors[getFieldName(field)] }"
            :placeholder="field.placeholder || '选择数据库文件'"
            readonly
          />
          <button type="button" class="btn-browse" @click="selectFile(getFieldName(field))">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/>
              <polyline points="14 2 14 8 20 8"/>
            </svg>
            浏览
          </button>
        </div>
        <span v-if="errors[getFieldName(field)]" class="error-text">{{ errors[getFieldName(field)] }}</span>
      </div>

      <!-- 主机地址字段 -->
      <div
        v-else-if="getFieldType(field) === 'text' && getFieldName(field) === 'host'"
        class="form-section"
        :class="{ 'has-error': errors[getFieldName(field)] }"
      >
        <label class="form-label">
          {{ field.label }}
          <span v-if="field.required" class="required">*</span>
        </label>
        <input
          v-model="formData[getFieldName(field)]"
          type="text"
          class="form-input"
          :class="{ error: errors[getFieldName(field)] }"
          :placeholder="field.placeholder || 'localhost'"
          @blur="validateField(getFieldName(field))"
        />
        <span v-if="errors[getFieldName(field)]" class="error-text">{{ errors[getFieldName(field)] }}</span>
      </div>

      <!-- 端口字段 -->
      <div
        v-else-if="getFieldType(field) === 'number' && getFieldName(field) === 'port'"
        class="form-section"
        :class="{ 'has-error': errors[getFieldName(field)] }"
      >
        <label class="form-label">
          {{ field.label }}
          <span v-if="field.required" class="required">*</span>
        </label>
        <input
          v-model.number="formData[getFieldName(field)]"
          type="number"
          class="form-input"
          :class="{ error: errors[getFieldName(field)] }"
          :placeholder="String(driver?.defaultPort || '')"
          min="1"
          max="65535"
          @blur="validateField(getFieldName(field))"
        />
        <span v-if="errors[getFieldName(field)]" class="error-text">{{ errors[getFieldName(field)] }}</span>
      </div>

      <!-- 数据库名字段 -->
      <div
        v-else-if="getFieldType(field) === 'text' && getFieldName(field) === 'database'"
        class="form-section"
        :class="{ 'has-error': errors[getFieldName(field)] }"
      >
        <label class="form-label">
          {{ field.label }}
          <span v-if="field.required" class="required">*</span>
        </label>
        <input
          v-model="formData[getFieldName(field)]"
          type="text"
          class="form-input"
          :class="{ error: errors[getFieldName(field)] }"
          :placeholder="field.placeholder || '数据库名称'"
          @blur="validateField(getFieldName(field))"
        />
        <span v-if="errors[getFieldName(field)]" class="error-text">{{ errors[getFieldName(field)] }}</span>
      </div>

      <!-- 用户名字段 -->
      <div
        v-else-if="getFieldType(field) === 'text' && getFieldName(field) === 'username'"
        class="form-section"
        :class="{ 'has-error': errors[getFieldName(field)] }"
      >
        <label class="form-label">
          {{ field.label }}
          <span v-if="field.required" class="required">*</span>
        </label>
        <input
          v-model="formData[getFieldName(field)]"
          type="text"
          class="form-input"
          :class="{ error: errors[getFieldName(field)] }"
          :placeholder="field.placeholder || '用户名'"
          @blur="validateField(getFieldName(field))"
        />
        <span v-if="errors[getFieldName(field)]" class="error-text">{{ errors[getFieldName(field)] }}</span>
      </div>

      <!-- 密码字段 -->
      <div
        v-else-if="getFieldType(field) === 'password'"
        class="form-section"
        :class="{ 'has-error': errors[getFieldName(field)] }"
      >
        <label class="form-label">
          {{ field.label }}
          <span v-if="field.required" class="required">*</span>
        </label>
        <div class="password-wrapper">
          <input
            v-model="formData[getFieldName(field)]"
            :type="showPassword ? 'text' : 'password'"
            class="form-input"
            :class="{ error: errors[getFieldName(field)] }"
            :placeholder="field.placeholder || '密码'"
            @blur="validateField(getFieldName(field))"
          />
          <button type="button" class="toggle-password" @click="showPassword = !showPassword">
            <svg v-if="showPassword" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z"/>
              <circle cx="12" cy="12" r="3"/>
            </svg>
            <svg v-else viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M17.94 17.94A10.07 10.07 0 0 1 12 20c-7 0-11-8-11-8a18.45 18.45 0 0 1 5.06-5.94M9.9 4.24A9.12 9.12 0 0 1 12 4c7 0 11 8 11 8a18.5 18.5 0 0 1-2.16 3.19m-6.72-1.07a3 3 0 1 1-4.24-4.24"/>
              <line x1="1" y1="1" x2="23" y2="23"/>
            </svg>
          </button>
        </div>
        <span v-if="errors[getFieldName(field)]" class="error-text">{{ errors[getFieldName(field)] }}</span>
      </div>

      <!-- 其他文本字段 -->
      <div
        v-else-if="getFieldType(field) === 'text'"
        class="form-section"
        :class="{ 'has-error': errors[getFieldName(field)] }"
      >
        <label class="form-label">
          {{ field.label }}
          <span v-if="field.required" class="required">*</span>
        </label>
        <input
          v-model="formData[getFieldName(field)]"
          type="text"
          class="form-input"
          :class="{ error: errors[getFieldName(field)] }"
          :placeholder="field.placeholder || ''"
          @blur="validateField(getFieldName(field))"
        />
        <span v-if="errors[getFieldName(field)]" class="error-text">{{ errors[getFieldName(field)] }}</span>
      </div>

      <!-- 数字字段 -->
      <div
        v-else-if="getFieldType(field) === 'number'"
        class="form-section"
        :class="{ 'has-error': errors[getFieldName(field)] }"
      >
        <label class="form-label">
          {{ field.label }}
          <span v-if="field.required" class="required">*</span>
        </label>
        <input
          v-model.number="formData[getFieldName(field)]"
          type="number"
          class="form-input"
          :class="{ error: errors[getFieldName(field)] }"
          :placeholder="field.placeholder || ''"
          @blur="validateField(getFieldName(field))"
        />
        <span v-if="errors[getFieldName(field)]" class="error-text">{{ errors[getFieldName(field)] }}</span>
      </div>
    </template>

    <!-- 高级选项 -->
    <div v-if="hasAdvancedOptions" class="advanced-options">
      <div class="section-header" @click="showAdvanced = !showAdvanced">
        <div class="header-left">
          <svg class="section-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <circle cx="12" cy="12" r="3"/>
            <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z"/>
          </svg>
          <span>高级选项</span>
        </div>
        <svg class="toggle-icon" :class="{ expanded: showAdvanced }" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <polyline points="6 9 12 15 18 9"/>
        </svg>
      </div>

      <div v-show="showAdvanced" class="advanced-content">
        <div v-for="(option, optIndex) in driver?.extraOptions || []" :key="optIndex" class="form-section">
          <label class="form-label">
            {{ option.label }}
            <span v-if="option.required" class="required">*</span>
          </label>

          <!-- 字符串选项 -->
          <template v-if="getOptionType(option) === 'string'">
            <input
              v-model="formData.options[getFieldName(option)]"
              type="text"
              class="form-input"
              :placeholder="option.description || ''"
            />
          </template>

          <!-- 数字选项 -->
          <template v-else-if="getOptionType(option) === 'number'">
            <input
              v-model.number="formData.options[getFieldName(option)]"
              type="number"
              class="form-input"
              :placeholder="option.description || ''"
            />
          </template>

          <!-- 布尔选项 -->
          <template v-else-if="getOptionType(option) === 'boolean'">
            <label class="toggle-switch">
              <input v-model="formData.options[getFieldName(option)]" type="checkbox" />
              <span class="toggle-slider"></span>
              <span class="toggle-label">{{ option.description || option.label }}</span>
            </label>
          </template>

          <!-- 选择选项 -->
          <template v-else-if="getOptionType(option) === 'select'">
            <select v-model="formData.options[getFieldName(option)]" class="form-select">
              <option v-for="(opt, idx) in getOptionChoices(option)" :key="idx" :value="opt">
                {{ opt }}
              </option>
            </select>
          </template>

          <!-- 文件选项 -->
          <template v-else-if="getOptionType(option) === 'file'">
            <div class="file-input-wrapper">
              <input
                v-model="formData.options[getFieldName(option)]"
                type="text"
                class="form-input"
                :placeholder="option.description || '选择文件'"
                readonly
              />
              <button type="button" class="btn-browse" @click="selectOptionFile(getFieldName(option))">
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/>
                  <polyline points="14 2 14 8 20 8"/>
                </svg>
                浏览
              </button>
            </div>
          </template>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, watch, computed } from 'vue'

import type { DriverDescriptor, ConnectionConfig } from '../types/driver'

// 安全地导入 Tauri API
let openDialog: typeof import('@tauri-apps/plugin-dialog').open | null = null
if (typeof window !== 'undefined' && (window as any).__TAURI__) {
  import('@tauri-apps/plugin-dialog').then(m => {
    openDialog = m.open
  }).catch(() => {
    openDialog = null
  })
}

interface Props {
  driver?: DriverDescriptor | null
  modelValue?: Partial<ConnectionConfig>
  existingConnections?: Array<{ name: string; db_type: string }>
}

interface Emits {
  (e: 'update:modelValue', value: Partial<ConnectionConfig>): void
  (e: 'validate', isValid: boolean): void
}

const props = withDefaults(defineProps<Props>(), {
  existingConnections: () => []
})
const emit = defineEmits<Emits>()

// 表单数据 - 动态字段
const formData = reactive<Record<string, any>>({
  name: '',
  options: {}
})

// 错误信息
const errors = reactive<Record<string, string>>({})

// 状态
const showPassword = ref(false)
const showAdvanced = ref(false)

// 计算属性：驱动字段列表
const driverFields = computed(() => {
  return props.driver?.fields || []
})

// 计算属性：是否有高级选项
const hasAdvancedOptions = computed(() => {
  return (props.driver?.extraOptions?.length || 0) > 0
})

// 辅助函数：获取字段名称
function getFieldName(field: any): string {
  return field.name || field.key || ''
}

// 辅助函数：获取字段类型
function getFieldType(field: any): string {
  return field.fieldType || field.field_type || field.type || 'text'
}

// 辅助函数：获取选项类型
function getOptionType(option: any): string {
  if (option.optionType) return option.optionType
  if (option.option_type?.type) return option.option_type.type
  return option.type || 'string'
}

// 辅助函数：获取选项选择列表
function getOptionChoices(option: any): string[] {
  if (option.option_type?.options) return option.option_type.options
  if (option.options) return option.options.map((o: any) => o.value || o)
  return []
}

// 计算属性：是否是文件型数据库
const isFileDatabase = computed(() => {
  return props.driver?.requireFile === true
})

// 计算属性：连接名称占位符
const connectionNamePlaceholder = computed(() => {
  const driverName = props.driver?.name || '数据库'
  return `例如：生产环境 ${driverName}`
})

// 监听驱动变化，重置表单
watch(() => props.driver, (newDriver) => {
  if (newDriver) {
    // 重置表单数据
    Object.keys(formData).forEach(key => {
      if (key !== 'options') delete (formData as any)[key]
    })
    formData.name = ''
    formData.options = {}

    // 根据驱动字段初始化默认值
    newDriver.fields?.forEach(field => {
      const fieldName = getFieldName(field)
      if (fieldName === 'port') {
        ;(formData as any)[fieldName] = newDriver.defaultPort || newDriver.default_port
      } else {
        ;(formData as any)[fieldName] = ''
      }
    })

    clearErrors()
  }
}, { immediate: true })

// 监听表单变化
watch(formData, (newValue) => {
  emit('update:modelValue', { ...newValue })
  validateAll()
}, { deep: true })

// 验证单个字段
function validateField(field: string): boolean {
  delete errors[field]
  const value = (formData as any)[field]

  // 查找字段配置
  const fieldConfig = props.driver?.fields?.find((f: any) => getFieldName(f) === field)
  if (!fieldConfig) return true

  // 必填验证
  if (fieldConfig.required) {
    if (value === undefined || value === null || String(value).trim() === '') {
      errors[field] = `${fieldConfig.label}不能为空`
      return false
    }
  }

  // 特定字段验证
  switch (field) {
    case 'name':
      if (value && String(value).length < 2) {
        errors.name = '连接名称至少需要2个字符'
        return false
      }
      // 检查名称+驱动类型是否重复
      if (value && props.driver) {
        const driverId = props.driver.id
        const isDuplicate = props.existingConnections.some(
          conn => conn.name === value && conn.db_type === driverId
        )
        if (isDuplicate) {
          errors.name = `已存在同名 ${props.driver.name} 连接`
          return false
        }
      }
      break

    case 'port':
      if (value !== undefined && value !== null && value !== '') {
        const portNum = Number(value)
        if (isNaN(portNum) || portNum < 1 || portNum > 65535) {
          errors.port = '端口号必须在 1-65535 之间'
          return false
        }
      }
      break
  }

  return true
}

// 验证所有字段
function validateAll(): boolean {
  const fields = props.driver?.fields?.map((f: any) => getFieldName(f)) || []
  fields.push('name') // 连接名称始终需要验证

  let isValid = true
  for (const field of fields) {
    if (!validateField(field)) {
      isValid = false
    }
  }

  emit('validate', isValid)
  return isValid
}

// 清除错误
function clearErrors() {
  Object.keys(errors).forEach(key => delete errors[key])
}

// 选择文件
async function selectFile(fieldKey: string) {
  if (!openDialog) {
    // 浏览器环境：使用原生文件选择
    const input = document.createElement('input')
    input.type = 'file'
    input.onchange = (e) => {
      const file = (e.target as HTMLInputElement).files?.[0]
      if (file) {
        formData[fieldKey] = file.name
        delete errors[fieldKey]
      }
    }
    input.click()
    return
  }

  try {
    const selected = await openDialog({
      multiple: false,
      directory: false
    })
    if (selected) {
      formData[fieldKey] = selected as string
      delete errors[fieldKey]
    }
  } catch (error) {
    console.error('选择文件失败:', error)
  }
}

// 选择选项文件
async function selectOptionFile(optionKey: string) {
  if (!openDialog) {
    // 浏览器环境：使用原生文件选择
    const input = document.createElement('input')
    input.type = 'file'
    input.onchange = (e) => {
      const file = (e.target as HTMLInputElement).files?.[0]
      if (file) {
        if (!formData.options) formData.options = {}
        formData.options[optionKey] = file.name
      }
    }
    input.click()
    return
  }

  try {
    const selected = await openDialog({
      multiple: false,
      directory: false
    })
    if (selected) {
      if (!formData.options) formData.options = {}
      formData.options[optionKey] = selected as string
    }
  } catch (error) {
    console.error('选择文件失败:', error)
  }
}

// 暴露方法
defineExpose({
  validate: validateAll,
  clearErrors
})
</script>

<style scoped>
.connection-form {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.form-section {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.form-section.has-error .form-input {
  border-color: var(--error-color);
  background: rgba(245, 63, 63, 0.05);
}

.form-label {
  font-size: 13px;
  font-weight: 500;
  color: var(--text-secondary);
  display: flex;
  align-items: center;
  gap: 4px;
}

.required {
  color: var(--danger-color);
}

.form-input,
.form-select {
  height: 36px;
  padding: 0 12px;
  border: 1px solid var(--border-color);
  border-radius: 6px;
  font-size: 14px;
  background: var(--bg-primary);
  color: var(--text-primary);
  transition: all 0.2s ease;
}

.form-input:focus,
.form-select:focus {
  outline: none;
  border-color: var(--primary-color);
  box-shadow: 0 0 0 3px rgba(22, 93, 255, 0.1);
}

.form-input::placeholder {
  color: var(--text-tertiary);
}

.error-text {
  font-size: 12px;
  color: var(--danger-color);
  display: flex;
  align-items: center;
  gap: 4px;
}

.error-text::before {
  content: '⚠';
}

/* 密码输入 */
.password-wrapper {
  position: relative;
  display: flex;
  align-items: center;
}

.password-wrapper .form-input {
  width: 100%;
  padding-right: 40px;
}

.toggle-password {
  position: absolute;
  right: 8px;
  top: 50%;
  transform: translateY(-50%);
  background: none;
  border: none;
  padding: 4px;
  cursor: pointer;
  color: var(--text-tertiary);
  display: flex;
  align-items: center;
  justify-content: center;
  transition: color 0.2s ease;
}

.toggle-password:hover {
  color: var(--text-secondary);
}

.toggle-password svg {
  width: 18px;
  height: 18px;
}

/* 文件输入 */
.file-input-wrapper {
  display: flex;
  gap: 8px;
}

.file-input-wrapper .form-input {
  flex: 1;
}

.btn-browse {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 0 16px;
  height: 36px;
  background: var(--bg-secondary);
  border: 1px solid var(--border-color);
  border-radius: 6px;
  font-size: 13px;
  font-weight: 500;
  color: var(--text-secondary);
  cursor: pointer;
  transition: all 0.2s ease;
  white-space: nowrap;
}

.btn-browse:hover {
  background: var(--bg-hover);
  border-color: var(--border-hover);
}

.btn-browse svg {
  width: 16px;
  height: 16px;
}

/* 高级选项 */
.advanced-options {
  border-top: 1px solid var(--border-color);
  padding-top: 16px;
  margin-top: 8px;
}

.section-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  cursor: pointer;
  padding: 8px 0;
  user-select: none;
}

.header-left {
  display: flex;
  align-items: center;
  gap: 8px;
  color: var(--text-secondary);
  font-size: 14px;
  font-weight: 500;
}

.section-icon {
  width: 18px;
  height: 18px;
}

.toggle-icon {
  width: 18px;
  height: 18px;
  color: var(--text-tertiary);
  transition: transform 0.2s ease;
}

.toggle-icon.expanded {
  transform: rotate(180deg);
}

.advanced-content {
  display: flex;
  flex-direction: column;
  gap: 16px;
  padding-top: 8px;
  animation: slideDown 0.2s ease;
}

@keyframes slideDown {
  from {
    opacity: 0;
    transform: translateY(-10px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

/* Toggle Switch */
.toggle-switch {
  display: flex;
  align-items: center;
  gap: 10px;
  cursor: pointer;
}

.toggle-switch input {
  display: none;
}

.toggle-slider {
  position: relative;
  width: 44px;
  height: 24px;
  background: var(--bg-tertiary);
  border-radius: 12px;
  transition: background 0.2s ease;
}

.toggle-slider::before {
  content: '';
  position: absolute;
  top: 2px;
  left: 2px;
  width: 20px;
  height: 20px;
  background: white;
  border-radius: 50%;
  transition: transform 0.2s ease;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.2);
}

.toggle-switch input:checked + .toggle-slider {
  background: var(--primary-color);
}

.toggle-switch input:checked + .toggle-slider::before {
  transform: translateX(20px);
}

.toggle-label {
  font-size: 13px;
  color: var(--text-secondary);
}
</style>