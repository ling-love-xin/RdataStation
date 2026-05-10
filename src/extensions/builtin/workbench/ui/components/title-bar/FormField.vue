<template>
  <div class="form-section">
    <label class="form-label">
      {{ label }}
      <span v-if="required" class="required">*</span>
    </label>
    <input
      v-if="type === 'input'"
      v-model="modelValue"
      type="text"
      class="form-input"
      :placeholder="placeholder"
      @keyup.enter="$emit('keyup-enter')"
    />
    <textarea
      v-else
      v-model="modelValue"
      class="form-input"
      :placeholder="placeholder"
      :rows="rows"
    />
  </div>
</template>

<script setup lang="ts">
interface Props {
  label: string
  modelValue: string
  type?: 'input' | 'textarea'
  required?: boolean
  placeholder?: string
  rows?: number
}

withDefaults(defineProps<Props>(), {
  type: 'input',
  required: false,
  placeholder: '',
  rows: 3,
})

defineEmits<{
  'update:modelValue': [value: string]
  'keyup-enter': []
}>()
</script>

<style scoped>
@import './modal.css';
</style>
