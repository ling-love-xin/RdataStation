<template>
  <div class="ssh-config-form">
    <div class="form-row">
      <div class="form-section flex-2">
        <label class="form-label">SSH 主机 <span class="required">*</span></label>
        <input v-model="config.host" type="text" class="form-input" placeholder="SSH 服务器地址" />
      </div>
      <div class="form-section flex-1">
        <label class="form-label">端口</label>
        <input v-model.number="config.port" type="number" class="form-input" placeholder="22" />
      </div>
    </div>
    
    <div class="form-row">
      <div class="form-section flex-1">
        <label class="form-label">用户名 <span class="required">*</span></label>
        <input v-model="config.username" type="text" class="form-input" placeholder="SSH 用户名" />
      </div>
      <div class="form-section flex-1">
        <label class="form-label">认证方式</label>
        <select v-model="config.authType" class="form-select">
          <option value="password">密码</option>
          <option value="key">私钥</option>
        </select>
      </div>
    </div>

    <div v-if="config.authType === 'password'" class="form-section">
      <label class="form-label">密码 <span class="required">*</span></label>
      <input v-model="config.password" type="password" class="form-input" placeholder="SSH 密码" />
    </div>

    <div v-else class="form-section">
      <label class="form-label">私钥文件 <span class="required">*</span></label>
      <div class="file-input-wrapper">
        <input v-model="config.privateKey" type="text" class="form-input" placeholder="选择私钥文件" readonly />
        <button type="button" class="btn-browse" @click="selectKeyFile">浏览</button>
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
  host: '',
  port: 22,
  username: '',
  authType: 'password',
  password: '',
  privateKey: ''
})

watch(config, (val) => emit('update:modelValue', { ...val }), { deep: true })

async function selectKeyFile() {
  // 浏览器环境：使用原生文件选择
  const input = document.createElement('input')
  input.type = 'file'
  input.onchange = (e) => {
    const file = (e.target as HTMLInputElement).files?.[0]
    if (file) {
      config.privateKey = file.name
    }
  }
  input.click()
}
</script>

<style scoped>
.ssh-config-form {
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

.file-input-wrapper {
  display: flex;
  gap: 8px;
}

.file-input-wrapper .form-input {
  flex: 1;
}

.btn-browse {
  padding: 0 16px;
  height: 36px;
  background: var(--bg-secondary);
  border: 1px solid var(--border-color);
  border-radius: 6px;
  font-size: 13px;
  cursor: pointer;
}

.btn-browse:hover {
  background: var(--bg-hover);
}
</style>
