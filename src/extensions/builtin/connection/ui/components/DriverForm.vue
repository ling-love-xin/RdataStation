<template>
  <div class="driver-form">
    <!-- 驱动选择 -->
    <div class="form-section">
      <label class="form-label">数据库类型</label>
      <select v-model="selectedDriverId" class="form-select" @change="onDriverChange">
        <option value="">请选择数据库类型</option>
        <option v-for="driver in drivers" :key="driver.id" :value="driver.id">
          {{ driver.name }}
        </option>
      </select>
    </div>

    <!-- 连接名称 -->
    <div class="form-section">
      <label class="form-label">连接名称</label>
      <input
        v-model="formData.name"
        type="text"
        class="form-input"
        placeholder="给这个连接起个名字"
      />
    </div>

    <!-- 驱动特定字段 -->
    <template v-if="selectedDriver">
      <div v-for="(field, idx) in selectedDriver.fields || []" :key="idx" class="form-section">
        <label class="form-label">
          {{ field.label }}
          <span v-if="field.required" class="required">*</span>
        </label>

        <!-- 文本输入 -->
        <input
          v-if="(field.fieldType || field.type) === 'text'"
          v-model="(formData as any)[field.name]"
          type="text"
          class="form-input"
          :placeholder="field.placeholder"
          :required="field.required"
        />

        <!-- 密码输入 -->
        <input
          v-else-if="(field.fieldType || field.type) === 'password'"
          v-model="(formData as any)[field.name]"
          type="password"
          class="form-input"
          :placeholder="field.placeholder"
        />

        <!-- 数字输入 -->
        <input
          v-else-if="(field.fieldType || field.type) === 'number'"
          v-model.number="(formData as any)[field.name]"
          type="number"
          class="form-input"
          :placeholder="field.placeholder"
          :required="field.required"
        />

        <!-- 文件选择 -->
        <div v-else-if="(field.fieldType || field.type) === 'file'" class="file-input-wrapper">
          <input
            v-model="(formData as any)[field.name]"
            type="text"
            class="form-input"
            :placeholder="field.placeholder"
            readonly
          />
          <button type="button" class="btn-file" @click="selectFile(field.name)"> 浏览... </button>
        </div>

        <!-- 下拉选择 -->
        <select
          v-else-if="(field.fieldType || field.type) === 'select'"
          v-model="(formData as any)[field.name]"
          class="form-select"
          :required="field.required"
        >
          <option
            v-for="(option, optIdx) in field.options || []"
            :key="optIdx"
            :value="option.value"
          >
            {{ option.label }}
          </option>
        </select>
      </div>
    </template>

    <!-- 额外选项 -->
    <div
      v-if="selectedDriver?.extraOptions?.length || selectedDriver?.extra_options?.length"
      class="form-section"
    >
      <button type="button" class="btn-toggle" @click="showAdvanced = !showAdvanced">
        <span class="toggle-icon">{{ showAdvanced ? '▼' : '▶' }}</span>
        高级选项
      </button>

      <div v-show="showAdvanced" class="advanced-options">
        <div
          v-for="(option, optIdx) in selectedDriver?.extraOptions ||
          selectedDriver?.extra_options ||
          []"
          :key="optIdx"
          class="form-section"
        >
          <label class="form-label">{{ option.label }}</label>

          <!-- 字符串选项 -->
          <input
            v-if="(option.optionType || option.option_type) === 'string'"
            v-model="(formData as any).options![option.name]"
            type="text"
            class="form-input"
            :placeholder="option.description"
          />

          <!-- 数字选项 -->
          <input
            v-else-if="(option.optionType || option.option_type) === 'number'"
            v-model.number="(formData as any).options![option.name]"
            type="number"
            class="form-input"
          />

          <!-- 布尔选项 -->
          <label
            v-else-if="(option.optionType || option.option_type) === 'boolean'"
            class="checkbox-label"
          >
            <input v-model="(formData as any).options![option.name]" type="checkbox" />
            {{ option.description }}
          </label>

          <!-- 选择选项 -->
          <select
            v-else-if="(option.optionType || option.option_type) === 'select'"
            v-model="(formData as any).options![option.name]"
            class="form-select"
          >
            <option
              v-for="(opt, oIdx) in option.options || []"
              :key="oIdx"
              :value="opt.value || opt"
            >
              {{ opt.label || opt }}
            </option>
          </select>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue'

import type { DriverDescriptor, ConnectionConfig } from '../types/connection'

interface Props {
  drivers: DriverDescriptor[]
  modelValue: Partial<ConnectionConfig>
}

interface Emits {
  (e: 'update:modelValue', value: Partial<ConnectionConfig>): void
  (e: 'driverChange', driver: DriverDescriptor | null): void
}

const props = defineProps<Props>()
const emit = defineEmits<Emits>()

const selectedDriverId = ref('')
const showAdvanced = ref(false)

const selectedDriver = computed(() => {
  return props.drivers.find(d => d.id === selectedDriverId.value) || null
})

const formData = computed({
  get: () => props.modelValue,
  set: value => emit('update:modelValue', value),
})

function onDriverChange() {
  const driver = selectedDriver.value
  emit('driverChange', driver)

  if (driver) {
    // 初始化表单数据
    const newFormData: Partial<ConnectionConfig> = {
      driver: driver.id,
      name: formData.value.name || '',
      options: {},
    }

    // 设置字段默认值
    ;(driver.fields || []).forEach((field: any) => {
      const defaultValue = field.defaultValue || field.default
      if (defaultValue !== undefined) {
        ;(newFormData as any)[field.name] = defaultValue
      }
    })

    // 设置额外选项默认值
    const extraOptions = driver.extraOptions || (driver as any).extra_options || []
    extraOptions.forEach((option: any) => {
      const defaultValue = option.defaultValue || option.default
      if (defaultValue !== undefined) {
        newFormData.options![option.name] = String(defaultValue)
      }
    })

    formData.value = newFormData
  }
}

async function selectFile(fieldName: string) {
  // 使用 Tauri 的文件选择对话框
  try {
    const { open } = await import('@tauri-apps/plugin-dialog')
    const selected = await open({
      multiple: false,
      filters: [
        { name: '数据库文件', extensions: ['db', 'sqlite', 'duckdb'] },
        { name: '所有文件', extensions: ['*'] },
      ],
    })
    if (selected && typeof selected === 'string') {
      formData.value[fieldName as keyof ConnectionConfig] = selected as any
    }
  } catch (error) {
    console.error('选择文件失败:', error)
  }
}

// 监听外部 modelValue 变化
watch(
  () => props.modelValue.driver,
  driverId => {
    if (driverId && driverId !== selectedDriverId.value) {
      selectedDriverId.value = driverId
    }
  },
  { immediate: true }
)
</script>

<style scoped>
.driver-form {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.form-section {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.form-label {
  font-size: 13px;
  font-weight: 500;
  color: var(--text-secondary);
}

.form-label .required {
  color: var(--error-color);
  margin-left: 2px;
}

.form-input,
.form-select {
  padding: 8px 12px;
  border: 1px solid var(--border-color);
  border-radius: 6px;
  font-size: 14px;
  background: var(--bg-primary);
  color: var(--text-primary);
  transition: border-color 0.2s;
}

.form-input:focus,
.form-select:focus {
  outline: none;
  border-color: var(--primary-color);
}

.file-input-wrapper {
  display: flex;
  gap: 8px;
}

.file-input-wrapper .form-input {
  flex: 1;
}

.btn-file {
  padding: 8px 16px;
  background: var(--bg-secondary);
  border: 1px solid var(--border-color);
  border-radius: 6px;
  cursor: pointer;
  font-size: 13px;
  white-space: nowrap;
}

.btn-file:hover {
  background: var(--bg-hover);
}

.btn-toggle {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 8px 0;
  background: none;
  border: none;
  color: var(--text-secondary);
  font-size: 13px;
  cursor: pointer;
}

.btn-toggle:hover {
  color: var(--text-primary);
}

.toggle-icon {
  font-size: 10px;
}

.advanced-options {
  padding: 16px;
  background: var(--bg-secondary);
  border-radius: 8px;
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.checkbox-label {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 13px;
  cursor: pointer;
}

.checkbox-label input[type='checkbox'] {
  width: 16px;
  height: 16px;
}
</style>
