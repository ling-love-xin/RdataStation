<template>
  <div class="editor-panel-factory">
    <QueryEditorPanel v-if="currentEditorType === 'query'" :params="props.params" />
    <AnalysisEditorPanel v-if="currentEditorType === 'analysis'" :params="props.params" />
    <CodeEditorPanel v-if="currentEditorType === 'code'" :params="props.params" />
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'

import { EditorModeResolver } from '@/extensions/builtin/workbench/manager/EditorModeResolver'
import type { EditorType } from '@/extensions/builtin/workbench/types/editor-types'

import AnalysisEditorPanel from './AnalysisEditorPanel.vue'
import CodeEditorPanel from './CodeEditorPanel.vue'
import QueryEditorPanel from './QueryEditorPanel.vue'

const props = defineProps<{
  params: Record<string, unknown>
}>()

const currentFilePath = computed(() => String(props.params.filePath || ''))
const currentLanguage = computed(() => String(props.params.language || 'sql'))

const currentEditorType = ref<EditorType>(
  EditorModeResolver.resolve(currentFilePath.value, currentLanguage.value)
)
</script>

<style scoped>
.editor-panel-factory {
  display: flex;
  flex-direction: column;
  height: 100%;
  overflow: hidden;
}
</style>