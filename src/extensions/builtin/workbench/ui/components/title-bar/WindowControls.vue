<template>
  <div class="window-controls">
    <button
      class="window-btn minimize"
      :title="t('workbench.minimize')"
      @click="$emit('minimize')"
    >
      <Minus :size="14" />
    </button>
    <button
      class="window-btn maximize"
      :title="t('workbench.maximize')"
      @click="$emit('maximize')"
    >
      <Square v-if="!isMaximized" :size="12" />
      <Copy v-else :size="12" />
    </button>
    <button class="window-btn close" :title="t('workbench.close')" @click="$emit('close')">
      <X :size="14" />
    </button>
  </div>
</template>

<script setup lang="ts">
import { Copy, Minus, Square, X } from 'lucide-vue-next'
import { useI18n } from 'vue-i18n'

interface Props {
  isMaximized?: boolean
}

withDefaults(defineProps<Props>(), {
  isMaximized: false,
})

defineEmits<{
  minimize: []
  maximize: []
  close: []
}>()

const { t } = useI18n()
</script>

<style scoped>
@import './title-bar.css';
</style>
