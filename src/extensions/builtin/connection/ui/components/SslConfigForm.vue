<template>
  <div class="ssl-config-form">
    <div class="form-section">
      <label class="form-label">SSL 模式</label>
      <select v-model="config.mode" class="form-select">
        <option value="disabled">禁用</option>
        <option value="preferred">首选（如果可用）</option>
        <option value="required">必需</option>
        <option value="verify_ca">验证 CA</option>
        <option value="verify_identity">验证身份</option>
      </select>
    </div>

    <div v-if="config.mode !== 'disabled' && config.mode !== 'preferred'" class="ssl-options">
      <div class="form-section">
        <label class="form-label">CA 证书</label>
        <div class="file-input-wrapper">
          <input v-model="config.caCert" type="text" class="form-input" placeholder="选择 CA 证书文件" readonly />
          <button type="button" class="btn-browse" @click="selectFile('caCert')">浏览</button>
        </div>
      </div>

      <div class="form-section">
        <label class="form-label">客户端证书</label>
        <div class="file-input-wrapper">
          <input v-model="config.clientCert" type="text" class="form-input" placeholder="选择客户端证书" readonly />
          <button type="button" class="btn-browse" @click="selectFile('clientCert')">浏览</button>
        </div>
      </div>

      <div class="form-section">
        <label class="form-label">客户端私钥</label>
        <div class="file-input-wrapper">
          <input v-model="config.clientKey" type="text" class="form-input" placeholder="选择私钥文件" readonly />
          <button type="button" class="btn-browse" @click="selectFile('clientKey')">浏览</button>
        </div>
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
  mode: 'preferred',
  caCert: '',
  clientCert: '',
  clientKey: ''
})

watch(config, (val) => emit('update:modelValue', { ...val }), { deep: true })

async function selectFile(field: string) {
  // 浏览器环境：使用原生文件选择
  const input = document.createElement('input')
  input.type = 'file'
  input.onchange = (e) => {
    const file = (e.target as HTMLInputElement).files?.[0]
    if (file) {
      (config as Record<string, string>)[field] = file.name
    }
  }
  input.click()
}
</script>

<style scoped>
.ssl-config-form {
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

.form-select {
  height: 36px;
  padding: 0 12px;
  border: 1px solid var(--border-color);
  border-radius: 6px;
  font-size: 14px;
  background: var(--bg-primary);
  color: var(--text-primary);
}

.form-select:focus {
  outline: none;
  border-color: var(--primary-color);
}

.ssl-options {
  display: flex;
  flex-direction: column;
  gap: 16px;
  padding-top: 8px;
  border-top: 1px solid var(--border-color);
}

.file-input-wrapper {
  display: flex;
  gap: 8px;
}

.form-input {
  flex: 1;
  height: 36px;
  padding: 0 12px;
  border: 1px solid var(--border-color);
  border-radius: 6px;
  font-size: 14px;
  background: var(--bg-primary);
  color: var(--text-primary);
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
