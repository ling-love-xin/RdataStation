<template>
  <div class="proxy-config-form">
    <div class="form-section">
      <label class="form-label">代理类型</label>
      <select v-model="config.type" class="form-select">
        <option value="http">HTTP</option>
        <option value="https">HTTPS</option>
        <option value="socks4">SOCKS4</option>
        <option value="socks5">SOCKS5</option>
      </select>
    </div>

    <div class="form-row">
      <div class="form-section flex-2">
        <label class="form-label">代理主机 <span class="required">*</span></label>
        <input v-model="config.host" type="text" class="form-input" placeholder="代理服务器地址" />
      </div>
      <div class="form-section flex-1">
        <label class="form-label">端口 <span class="required">*</span></label>
        <input v-model.number="config.port" type="number" class="form-input" placeholder="8080" />
      </div>
    </div>

    <div class="form-section">
      <label class="toggle-label">
        <input v-model="config.auth" type="checkbox" class="toggle-input" />
        <span class="toggle-text">需要认证</span>
      </label>
    </div>

    <div v-if="config.auth" class="auth-section">
      <div class="form-section">
        <label class="form-label">用户名</label>
        <input v-model="config.username" type="text" class="form-input" placeholder="代理用户名" />
      </div>
      <div class="form-section">
        <label class="form-label">密码</label>
        <input
          v-model="config.password"
          type="password"
          class="form-input"
          placeholder="代理密码"
        />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { reactive, watch } from 'vue'

interface Props {
  modelValue?: Record<string, unknown>
}

// eslint-disable-next-line @typescript-eslint/no-unused-vars
const _props = defineProps<Props>()
const emit = defineEmits<{
  (e: 'update:modelValue', value: Record<string, unknown>): void
}>()

const config = reactive({
  type: 'http',
  host: '',
  port: 8080,
  auth: false,
  username: '',
  password: '',
})

watch(config, val => emit('update:modelValue', { ...val }), { deep: true })
</script>

<style scoped>
.proxy-config-form {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.form-row {
  display: flex;
  gap: 16px;
}

.flex-1 {
  flex: 1;
}

.flex-2 {
  flex: 2;
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
}

.form-input:focus,
.form-select:focus {
  outline: none;
  border-color: var(--primary-color);
}

.toggle-label {
  display: flex;
  align-items: center;
  gap: 8px;
  cursor: pointer;
}

.toggle-input {
  width: 18px;
  height: 18px;
  accent-color: var(--primary-color);
}

.toggle-text {
  font-size: 13px;
  color: var(--text-secondary);
}

.auth-section {
  display: flex;
  flex-direction: column;
  gap: 16px;
  padding-top: 8px;
  border-top: 1px solid var(--border-color);
}
</style>
