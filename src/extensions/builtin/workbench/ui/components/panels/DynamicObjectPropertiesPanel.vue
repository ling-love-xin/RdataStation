<template>
  <div class="dynamic-properties-panel">
    <NTabs v-if="node" type="line" size="small">
      <NTabPane name="general" :tab="t('workbench.general')">
        <NDescriptions :column="1" size="small" bordered>
          <NDescriptionsItem :label="t('navigator.column')">{{ node.name }}</NDescriptionsItem>
          <NDescriptionsItem :label="t('resultPanel.type')">{{ node.type }}</NDescriptionsItem>
        </NDescriptions>
      </NTabPane>
      <NTabPane name="properties" :tab="t('workbench.properties')">
        <NEmpty :description="t('workbench.properties')" />
      </NTabPane>
    </NTabs>
    <NEmpty v-else :description="t('workbench.selectObject')" />
  </div>
</template>

<script setup lang="ts">
import { NTabs, NTabPane, NDescriptions, NDescriptionsItem, NEmpty } from 'naive-ui'
import { useI18n } from 'vue-i18n'

import type { NavigatorNode } from '@/shared/types/databaseMeta'

const { t } = useI18n()

interface Props {
  node?: NavigatorNode
}

defineProps<Props>()
</script>

<style scoped>
.dynamic-properties-panel {
  padding: 16px;
  height: 100%;
  overflow: auto;
}
</style>
