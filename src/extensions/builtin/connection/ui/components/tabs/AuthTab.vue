<template>
  <div class="auth-tab">
    <!-- 基础认证 -->
    <div class="auth-section">
      <h4 class="section-title">{{ t('connection.authTab.title') }}</h4>

      <div class="form-section">
        <label class="form-label">
          {{ t('connection.authTab.username') }}
          <span class="required">*</span>
        </label>
        <input
          v-model="formData.username"
          type="text"
          class="form-input"
          :placeholder="t('connection.authTab.usernamePlaceholder')"
          :class="{ error: errors.username }"
        />
        <span v-if="errors.username" class="error-text">{{ errors.username }}</span>
      </div>

      <div class="form-section">
        <label class="form-label">{{ t('connection.authTab.password') }}</label>
        <div class="password-wrapper">
          <input
            v-model="formData.password"
            :type="showPassword ? 'text' : 'password'"
            class="form-input"
            :placeholder="t('connection.authTab.passwordPlaceholder')"
          />
          <button type="button" class="btn-toggle-password" @click="showPassword = !showPassword">
            <Eye v-if="!showPassword" :size="16" />
            <EyeOff v-else :size="16" />
          </button>
        </div>
      </div>

      <label class="checkbox-wrapper">
        <input v-model="savePassword" type="checkbox" />
        <span class="checkmark"></span>
        <span class="checkbox-label">{{ t('connection.authTab.savePassword') }}</span>
      </label>
    </div>

    <!-- 认证方式选择 -->
    <div class="auth-method-section">
      <h4 class="section-title">{{ t('connection.authTab.authMethod') }}</h4>

      <div class="method-options">
        <label class="method-option">
          <input v-model="authMethod" type="radio" value="password" />
          <span class="method-radio"></span>
          <div class="method-info">
            <span class="method-name">{{ t('connection.authTab.passwordAuth') }}</span>
            <span class="method-desc">{{ t('connection.authTab.passwordAuthDesc') }}</span>
          </div>
        </label>

        <label class="method-option">
          <input v-model="authMethod" type="radio" value="trust" />
          <span class="method-radio"></span>
          <div class="method-info">
            <span class="method-name">{{ t('connection.authTab.trustAuth') }}</span>
            <span class="method-desc">{{ t('connection.authTab.trustAuthDesc') }}</span>
          </div>
        </label>

        <label v-if="selectedDriver?.supportsSsl" class="method-option">
          <input v-model="authMethod" type="radio" value="ssl" />
          <span class="method-radio"></span>
          <div class="method-info">
            <span class="method-name">{{ t('connection.authTab.sslAuth') }}</span>
            <span class="method-desc">{{ t('connection.authTab.sslAuthDesc') }}</span>
          </div>
        </label>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { Eye, EyeOff } from 'lucide-vue-next'
import { ref, computed, watch } from 'vue'
import { useI18n } from 'vue-i18n'

const { t } = useI18n()

import type { DriverDescriptor, ConnectionConfig } from '../../types/connection'

const formData = defineModel<Partial<ConnectionConfig>>('formData', { required: true })

defineProps<{
  selectedDriver: DriverDescriptor | null
  errors: Record<string, string>
}>()

const emit = defineEmits<{
  'update:formData': [data: Partial<ConnectionConfig>]
}>()

const showPassword = ref(false)

const authMethod = computed({
  get: () => formData.value.authMethod || 'password',
  set: (val: 'password' | 'trust' | 'ssh' | 'ssl') => {
    emit('update:formData', {
      ...formData.value,
      authMethod: val,
    })
  },
})

const savePassword = computed({
  get: () => formData.value.options?.savePassword || false,
  set: (val: boolean) => {
    emit('update:formData', {
      ...formData.value,
      options: {
        ...formData.value.options,
        savePassword: val,
      },
    })
  },
})

// 监听认证方式变化，自动更新表单
watch(authMethod, newMethod => {
  if (newMethod === 'trust') {
    emit('update:formData', {
      ...formData.value,
      username: '',
      password: '',
    })
  }
})
</script>

<style scoped>
.auth-tab {
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.auth-section,
.auth-method-section {
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

.form-section {
  display: flex;
  flex-direction: column;
  gap: 6px;
  margin-bottom: 12px;
}

.form-label {
  font-size: 13px;
  font-weight: 500;
  color: var(--text-primary);
}

.form-label .required {
  color: var(--danger-color);
  margin-left: 2px;
}

.form-input {
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

.form-input:focus {
  border-color: var(--primary-color);
  background: var(--bg-primary);
}

.form-input.error {
  border-color: var(--danger-color);
}

.error-text {
  font-size: 12px;
  color: var(--danger-color);
}

.password-wrapper {
  position: relative;
  display: flex;
  align-items: center;
}

.password-wrapper .form-input {
  flex: 1;
  padding-right: 40px;
}

.btn-toggle-password {
  position: absolute;
  right: 8px;
  width: 24px;
  height: 24px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: transparent;
  border: none;
  color: var(--text-tertiary);
  cursor: pointer;
  transition: color 0.2s;
}

.btn-toggle-password:hover {
  color: var(--text-primary);
}

.checkbox-wrapper {
  display: flex;
  align-items: center;
  gap: 8px;
  cursor: pointer;
  font-size: 13px;
  color: var(--text-secondary);
}

.checkbox-wrapper input {
  display: none;
}

.checkmark {
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

/* 认证方式选项 */
.method-options {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.method-option {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 10px 12px;
  background: var(--bg-tertiary);
  border-radius: var(--radius-md);
  cursor: pointer;
  transition: all 0.2s;
  border: 1px solid transparent;
}

.method-option:hover {
  background: var(--bg-hover);
}

.method-option input {
  display: none;
}

.method-option input:checked ~ .method-radio {
  border-color: var(--primary-color);
}

.method-option input:checked ~ .method-radio::after {
  content: '';
  position: absolute;
  left: 3px;
  top: 3px;
  width: 8px;
  height: 8px;
  background: var(--primary-color);
  border-radius: 50%;
}

.method-radio {
  position: relative;
  width: 16px;
  height: 16px;
  border: 2px solid var(--border-color);
  border-radius: 50%;
  flex-shrink: 0;
  transition: all 0.2s;
}

.method-info {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.method-name {
  font-size: 13px;
  font-weight: 500;
  color: var(--text-primary);
}

.method-desc {
  font-size: 11px;
  color: var(--text-tertiary);
}
</style>
