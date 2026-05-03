<template>
  <div class="sql-filter-input">
    <div class="filter-row">
      <NInput
        :value="expression"
        size="tiny"
        type="textarea"
        :autosize="{ minRows: 1, maxRows: 3 }"
        placeholder="输入 WHERE 条件，可包含子查询&#10;示例: ArtistId IN (SELECT id FROM artists)"
        @update:value="emit('update:expression', $event || '')"
        @keydown.enter.ctrl="onExecute"
      />
      <NButton size="tiny" type="primary" :loading="loading" @click="onExecute">
        <template #icon><Play :size="12" /></template>
        执行
      </NButton>
    </div>
    <div class="hint">Ctrl+Enter 执行，将拼接为 SELECT ... FROM (原始SQL) WHERE 条件</div>
  </div>
</template>

<script setup lang="ts">
import { Play } from 'lucide-vue-next'
import { NInput, NButton } from 'naive-ui'

defineProps<{
  expression: string
  loading: boolean
}>()

const emit = defineEmits<{
  'update:expression': [string]
  execute: []
}>()

function onExecute() { emit('execute') }
</script>

<style scoped>
.sql-filter-input { padding: 4px 8px; border-bottom: 1px solid var(--border-color, #333); flex-shrink: 0; }
.filter-row { display: flex; align-items: flex-start; gap: 4px; }
.hint { font-size: 10px; color: var(--text-tertiary, #666); margin-top: 2px; }
</style>
