<template>
  <div class="table-schema-panel">
    <NTabs type="line" size="small">
      <NTabPane name="columns" tab="列">
        <NDataTable
          :columns="columnColumns"
          :data="columns"
          :bordered="false"
          size="small"
        />
      </NTabPane>
      <NTabPane name="indexes" tab="索引">
        <NEmpty description="暂无索引" />
      </NTabPane>
      <NTabPane name="constraints" tab="约束">
        <NEmpty description="暂无约束" />
      </NTabPane>
    </NTabs>
  </div>
</template>

<script setup lang="ts">
import { NTabs, NTabPane, NDataTable, NEmpty } from 'naive-ui'

import type { DataTableColumns } from 'naive-ui'

interface ColumnInfo {
  name: string
  type: string
  nullable: boolean
  defaultValue?: string
}

interface Props {
  columns?: ColumnInfo[]
}

const props = withDefaults(defineProps<Props>(), {
  columns: () => []
})

const columnColumns: DataTableColumns<ColumnInfo> = [
  { title: '名称', key: 'name' },
  { title: '类型', key: 'type' },
  { title: '可空', key: 'nullable', render: (row) => row.nullable ? '是' : '否' },
  { title: '默认值', key: 'defaultValue' }
]
</script>

<style scoped>
.table-schema-panel {
  padding: 16px;
  height: 100%;
  overflow: auto;
}
</style>
