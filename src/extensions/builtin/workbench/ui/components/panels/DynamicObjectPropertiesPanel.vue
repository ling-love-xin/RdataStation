<template>
  <div class="dynamic-properties-panel">
    <NTabs v-if="node" type="line" size="small">
      <NTabPane name="general" tab="通用">
        <NDescriptions :column="1" size="small" bordered>
          <NDescriptionsItem label="名称">{{ node.name }}</NDescriptionsItem>
          <NDescriptionsItem label="类型">{{ node.type }}</NDescriptionsItem>
        </NDescriptions>
      </NTabPane>
      <NTabPane name="properties" tab="属性">
        <NEmpty description="暂无属性" />
      </NTabPane>
    </NTabs>
    <NEmpty v-else description="请选择一个对象" />
  </div>
</template>

<script setup lang="ts">
import { NTabs, NTabPane, NDescriptions, NDescriptionsItem, NEmpty } from 'naive-ui'

import type { NavigatorNode } from '@/shared/types/databaseMeta'

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
