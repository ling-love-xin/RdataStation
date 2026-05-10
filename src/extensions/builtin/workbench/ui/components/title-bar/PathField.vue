<template>
  <div class="form-section">
    <label class="form-label">
      {{ label }}
      <span v-if="required" class="required">*</span>
    </label>
    <div class="path-input-wrapper">
      <input
        v-model="modelValue"
        type="text"
        class="form-input"
        :placeholder="placeholder"
        readonly
      />
      <button class="btn-browse" @click="$emit('browse')">
        {{ t('workbench.browse') }}
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { useI18n } from 'vue-i18n'

interface Props {
  label: string
  modelValue: string
  required?: boolean
  placeholder?: string
}

withDefaults(defineProps<Props>(), {
  required: false,
  placeholder: '',
})

defineEmits<{
  'update:modelValue': [value: string]
  browse: []
}>()

const { t } = useI18n()
</script>

<style scoped>
@import './modal.css';
</style>
