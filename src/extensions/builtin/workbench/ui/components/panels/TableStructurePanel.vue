<template>
  <div class="table-structure-panel">
    <!-- 表信息头部 -->
    <div class="table-header">
      <div class="table-title">
        <Table2 :size="20" />
        <span class="table-name">{{ tableName }}</span>
        <NTag v-if="schemaName" size="small" type="info">{{ schemaName }}</NTag>
        <NTag v-if="databaseName" size="small">{{ databaseName }}</NTag>
      </div>
      <div class="table-actions">
        <NButtonGroup size="small">
          <NButton title="查看数据" @click="handleViewData">
            <template #icon>
              <Eye :size="14" />
            </template>
            查看数据
          </NButton>
          <NButton title="生成查询" @click="handleGenerateSelect">
            <template #icon>
              <Code :size="14" />
            </template>
            生成查询
          </NButton>
        </NButtonGroup>
      </div>
    </div>

    <!-- 标签页 -->
    <NTabs v-model:value="activeTab" type="line" size="small" class="structure-tabs">
      <NTabPane name="columns" tab="列">
        <div class="tab-content">
          <NDataTable
            :columns="columnColumns"
            :data="columns"
            :loading="loading"
            size="small"
            :pagination="false"
            :bordered="false"
          />
        </div>
      </NTabPane>

      <NTabPane name="indexes" tab="索引">
        <div class="tab-content">
          <NEmpty v-if="indexes.length === 0" description="暂无索引信息" />
          <NDataTable
            v-else
            :columns="indexColumns"
            :data="indexes"
            size="small"
            :pagination="false"
            :bordered="false"
          />
        </div>
      </NTabPane>

      <NTabPane name="constraints" tab="约束">
        <div class="tab-content">
          <NEmpty v-if="constraints.length === 0" description="暂无约束信息" />
          <NDataTable
            v-else
            :columns="constraintColumns"
            :data="constraints"
            size="small"
            :pagination="false"
            :bordered="false"
          />
        </div>
      </NTabPane>

      <NTabPane name="ddl" tab="DDL">
        <div class="tab-content">
          <NCode :code="ddl" language="sql" show-line-numbers />
        </div>
      </NTabPane>
    </NTabs>
  </div>
</template>

<script setup lang="ts">
import { Table2, Eye, Code } from 'lucide-vue-next'
import { NTag, NTabs, NTabPane, NDataTable, NButton, NButtonGroup, NEmpty, NCode } from 'naive-ui'
import { ref, onMounted } from 'vue'

import * as metadataApi from '@/extensions/builtin/navigator/infrastructure/api/metadataApi'

interface Props {
  connectionId: string
  databaseName: string
  schemaName?: string
  tableName: string
}

const props = defineProps<Props>()

const emit = defineEmits<{
  viewData: [tableName: string]
  generateSelect: [tableName: string, columns: string[]]
}>()

// 状态
const loading = ref(false)
const activeTab = ref('columns')
const columns = ref<any[]>([])
const indexes = ref<any[]>([])
const constraints = ref<any[]>([])
const ddl = ref('')

// 列定义
const columnColumns = [
  { title: '列名', key: 'name', width: 150 },
  { title: '数据类型', key: 'dataType', width: 120 },
  { title: '可空', key: 'nullable', width: 60 },
  { title: '默认值', key: 'defaultValue', width: 120 },
  { title: '注释', key: 'comment', ellipsis: { tooltip: true } }
]

const indexColumns = [
  { title: '索引名', key: 'name', width: 150 },
  { title: '类型', key: 'type', width: 100 },
  { title: '列', key: 'columns', ellipsis: { tooltip: true } },
  { title: '唯一', key: 'unique', width: 60 }
]

const constraintColumns = [
  { title: '约束名', key: 'name', width: 150 },
  { title: '类型', key: 'type', width: 100 },
  { title: '列', key: 'columns', ellipsis: { tooltip: true } }
]

// 加载表结构
async function loadTableStructure() {
  loading.value = true
  try {
    const schema = props.schemaName || 'public'
    
    // 获取列信息
    const columnNodes = await metadataApi.getColumns(
      props.connectionId,
      props.databaseName,
      schema,
      props.tableName
    )
    
    columns.value = columnNodes.map(col => ({
      name: col.name,
      dataType: col.metadata?.dataType || 'unknown',
      nullable: col.metadata?.nullable ? '是' : '否',
      defaultValue: col.metadata?.defaultValue || '',
      comment: col.metadata?.comment || ''
    }))
    
    // 生成 DDL
    generateDDL()
  } catch (error) {
    console.error('Failed to load table structure:', error)
  } finally {
    loading.value = false
  }
}

// 生成 DDL
function generateDDL() {
  const columnDefs = columns.value.map(col => {
    let def = `  ${col.name} ${col.dataType}`
    if (!col.nullable) def += ' NOT NULL'
    if (col.defaultValue) def += ` DEFAULT ${col.defaultValue}`
    return def
  }).join(',\n')
  
  ddl.value = `CREATE TABLE ${props.tableName} (\n${columnDefs}\n);`
}

// 查看数据
function handleViewData() {
  emit('viewData', props.tableName)
}

// 生成查询
function handleGenerateSelect() {
  const columnNames = columns.value.map(col => col.name)
  emit('generateSelect', props.tableName, columnNames)
}

onMounted(() => {
  loadTableStructure()
})
</script>

<style scoped>
.table-structure-panel {
  height: 100%;
  display: flex;
  flex-direction: column;
  background-color: var(--bg-primary);
}

.table-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 16px;
  border-bottom: 1px solid var(--border-color);
  background-color: var(--bg-secondary);
}

.table-title {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 16px;
  font-weight: 500;
}

.table-name {
  color: var(--text-primary);
}

.table-actions {
  display: flex;
  gap: 8px;
}

.structure-tabs {
  flex: 1;
  overflow: hidden;
}

.structure-tabs :deep(.n-tabs-nav) {
  padding: 0 16px;
  background-color: var(--bg-secondary);
  border-bottom: 1px solid var(--border-color);
}

.structure-tabs :deep(.n-tabs-pane-wrapper) {
  height: calc(100% - 41px);
}

.tab-content {
  height: 100%;
  padding: 16px;
  overflow: auto;
}
</style>
