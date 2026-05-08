<template>
  <div class="mock-panel">
    <div class="panel-header">
      <h3 class="panel-title">Mock 数据生成器</h3>
      <div class="header-actions">
        <NButton size="small" quaternary @click="loadHistory">
          <template #icon>
            <Clock :size="14" />
          </template>
        </NButton>
      </div>
    </div>

    <div class="panel-body">
      <div class="config-section">
        <div class="config-row">
          <div class="config-item">
            <label class="config-label">表名</label>
            <NInput v-model:value="store.tableName" size="small" placeholder="table_name" />
          </div>
          <div class="config-item">
            <label class="config-label">行数</label>
            <NInputNumber v-model:value="store.rowCount" size="small" :min="1" :max="1000000" />
          </div>
          <div class="config-item">
            <label class="config-label">种子</label>
            <NInput
              v-model:value="seedInput"
              size="small"
              placeholder="可选"
              @update:value="onSeedChange"
            />
          </div>
          <div class="config-item">
            <label class="config-label">地区</label>
            <NSelect
              v-model:value="store.locale"
              size="small"
              :options="localeOptions"
              style="width: 120px"
            />
          </div>
        </div>
      </div>

      <div class="columns-section">
        <div class="section-header">
          <span class="section-title">列配置</span>
          <NButton size="small" quaternary @click="store.addColumn()">
            <template #icon>
              <Plus :size="14" />
            </template>
            添加列
          </NButton>
        </div>

        <div class="columns-list">
          <div
            v-for="(col, idx) in store.columns"
            :key="idx"
            class="column-row"
          >
            <NInput
              :value="col.name"
              size="small"
              placeholder="列名"
              style="flex: 0 0 130px"
              @update:value="(v: string) => store.updateColumn(idx, { name: v })"
            />
            <NSelect
              :value="col.dataType"
              size="small"
              :options="dataTypeOptions"
              style="width: 110px"
              @update:value="(v: ColumnDataType) => store.updateColumn(idx, { dataType: v })"
            />
            <NSelect
              :value="col.generator.type"
              size="small"
              :options="generatorOptions"
              filterable
              style="flex: 1; min-width: 140px"
              @update:value="(v: GeneratorType) => store.setColumnType(idx, v)"
            />
            <NButton
              size="small"
              quaternary
              type="error"
              :disabled="store.columns.length <= 1"
              @click="store.removeColumn(idx)"
            >
              <template #icon>
                <Trash2 :size="14" />
              </template>
            </NButton>
          </div>
        </div>
      </div>

      <div class="generate-section">
        <NButton
          type="primary"
          :loading="store.generateLoading"
          :disabled="!store.tableName || store.columns.length === 0"
          @click="onGenerate"
        >
          <template #icon>
            <Play :size="16" />
          </template>
          生成 {{ store.rowCount }} 行
        </NButton>

        <span v-if="store.lastResult" class="generate-info">
          已生成 {{ store.lastResult.rowCount }} 行，
          耗时 {{ store.lastResult.elapsedMs }}ms
        </span>
      </div>

      <div v-if="store.previewData.length > 0" class="preview-section">
        <div class="section-header">
          <span class="section-title">预览 (前 {{ store.previewData.length }} 行)</span>
          <div class="preview-actions">
            <NDropdown trigger="click" :options="exportDropdownOptions" @select="onExportAction">
              <NButton size="small" quaternary>
                <template #icon>
                  <Download :size="14" />
                </template>
                导出/保存
              </NButton>
            </NDropdown>
          </div>
        </div>

        <div class="preview-table-wrap">
          <NDataTable
            :columns="previewTableColumns"
            :data="previewTableData"
            size="small"
            :bordered="true"
            :single-line="false"
            :max-height="320"
            virtual-scroll
          />
        </div>
      </div>

      <div v-if="store.lastHistories.length > 0" class="history-section">
        <div class="section-header">
          <span class="section-title">生成历史</span>
          <NButton size="small" quaternary @click="store.clearHistory()">
            <template #icon>
              <Trash2 :size="14" />
            </template>
            清空
          </NButton>
        </div>

        <div class="history-list">
          <div
            v-for="item in store.lastHistories"
            :key="item.id"
            class="history-item"
            @click="onReGenerate(item.id)"
          >
            <div class="history-item-left">
              <span class="history-table">{{ item.tableName }}</span>
              <span class="history-rows">{{ item.rowCount }} 行</span>
              <span class="history-status">{{ item.status }}</span>
            </div>
            <div class="history-item-right">
              <span class="history-time">{{ formatTimeStr(item.generatedAt) }}</span>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import {
  Clock, Plus, Trash2, Play, Download,
} from 'lucide-vue-next'
import {
  NButton, NInput, NInputNumber, NSelect, NDataTable, NDropdown,
  createDiscreteApi,
} from 'naive-ui'
import { ref, computed, onMounted } from 'vue'

import type {
  GeneratorType, ColumnDataType, MockExportFormat,
} from '@/shared/api/mock-api'
import { useMockStore } from '@/stores/useMockStore'

const store = useMockStore()
const { message } = createDiscreteApi(['message'])

const seedInput = ref(store.seed !== null ? String(store.seed) : '')

const localeOptions = [
  { label: '🇨🇳 中文 (ZH_CN)', value: 'ZH_CN' },
  { label: '🇺🇸 English (EN)', value: 'EN' },
  { label: '🇯🇵 日本語 (JA_JP)', value: 'JA_JP' },
  { label: '🇫🇷 Français (FR_FR)', value: 'FR_FR' },
  { label: '🇩🇪 Deutsch (DE_DE)', value: 'DE_DE' },
]

const dataTypeOptions = [
  { label: 'INTEGER', value: 'integer' },
  { label: 'BIGINT', value: 'bigint' },
  { label: 'FLOAT', value: 'float' },
  { label: 'DOUBLE', value: 'double' },
  { label: 'DECIMAL', value: 'decimal' },
  { label: 'BOOLEAN', value: 'boolean' },
  { label: 'VARCHAR', value: 'varchar' },
  { label: 'TEXT', value: 'text' },
  { label: 'DATE', value: 'date' },
  { label: 'DATETIME', value: 'datetime' },
  { label: 'TIMESTAMP', value: 'timestamp' },
  { label: 'UUID', value: 'uuid' },
]

const generatorOptions = [
  { label: '自动递增', value: 'auto_increment', type: 'number' },
  { label: '随机整数', value: 'random_int', type: 'number' },
  { label: '随机浮点数', value: 'random_float', type: 'number' },
  { label: '布尔值', value: 'boolean', type: 'general' },
  { label: '用户名', value: 'username', type: 'person' },
  { label: '邮箱', value: 'email', type: 'person' },
  { label: '安全邮箱', value: 'safe_email', type: 'person' },
  { label: '姓名', value: 'name', type: 'person' },
  { label: '名', value: 'first_name', type: 'person' },
  { label: '姓', value: 'last_name', type: 'person' },
  { label: '手机号', value: 'phone_number', type: 'person' },
  { label: '密码', value: 'password', type: 'person' },
  { label: '句子', value: 'sentence', type: 'text' },
  { label: '段落', value: 'paragraph', type: 'text' },
  { label: '单词', value: 'words', type: 'text' },
  { label: '国家', value: 'country', type: 'address' },
  { label: '城市', value: 'city', type: 'address' },
  { label: '街道名', value: 'street_name', type: 'address' },
  { label: '邮编', value: 'zip_code', type: 'address' },
  { label: '日期时间', value: 'datetime', type: 'datetime' },
  { label: '日期', value: 'date', type: 'datetime' },
  { label: '时间', value: 'time', type: 'datetime' },
  { label: '公司名', value: 'company_name', type: 'business' },
  { label: '职位', value: 'job_title', type: 'business' },
  { label: '行业', value: 'industry', type: 'business' },
  { label: 'UUID v4', value: 'uuid_v4', type: 'special' },
  { label: 'URL', value: 'url', type: 'special' },
  { label: 'IP 地址', value: 'ip_address', type: 'special' },
  { label: 'MAC 地址', value: 'mac_address', type: 'special' },
  { label: '十六进制颜色', value: 'hex_color', type: 'special' },
  { label: '用户代理', value: 'user_agent', type: 'special' },
  { label: '币种代码', value: 'currency_code', type: 'business' },
  { label: '信用卡号', value: 'credit_card_number', type: 'business' },
  { label: 'ISBN', value: 'isbn', type: 'special' },
  { label: '文件名', value: 'file_name', type: 'special' },
  { label: '文件扩展名', value: 'file_extension', type: 'special' },
  { label: 'Markdown', value: 'md_blockquote_multi', type: 'text' },
  { label: '经纬度', value: 'latitude', type: 'address' },
  { label: '图片URL', value: 'image_url', type: 'special' },
  { label: 'IPv4', value: 'ipv4', type: 'special' },
  { label: 'IPv6', value: 'ipv6', type: 'special' },
  { label: 'SemVer', value: 'semver', type: 'special' },
  { label: '常量', value: 'constant', type: 'general' },
]

const exportDropdownOptions = [
  { label: '📄 导出 CSV', key: 'Csv' },
  { label: '📊 导出 Parquet', key: 'Parquet' },
  { label: '📈 导出 XLSX', key: 'Xlsx' },
  { label: '📝 导出 SQL INSERT', key: 'SqlInsert' },
  { label: '---', key: '_divider', type: 'divider' },
  { label: '💾 保存到草稿箱 (CSV)', key: 'scratchpad_Csv' },
  { label: '💾 保存到草稿箱 (Parquet)', key: 'scratchpad_Parquet' },
  { label: '💾 保存到草稿箱 (XLSX)', key: 'scratchpad_Xlsx' },
  { label: '💾 保存到草稿箱 (SQL)', key: 'scratchpad_SqlInsert' },
  { label: '---', key: '_divider2', type: 'divider' },
  { label: '🗄️ 持久化为分析资产', key: 'persist' },
]

const previewTableColumns = computed(() => {
  if (store.generatedColumns.length > 0) {
    return store.generatedColumns.map(name => ({
      title: name,
      key: name,
      width: 160,
      ellipsis: { tooltip: true },
    }))
  }
  if (store.previewData.length > 0 && store.previewData[0]) {
    return store.previewData[0].map((_, i) => ({
      title: store.columns[i]?.name ?? `列${i + 1}`,
      key: `col_${i}`,
      width: 160,
      ellipsis: { tooltip: true },
    }))
  }
  return []
})

const previewTableData = computed(() => {
  if (store.generatedColumns.length > 0) {
    return store.previewData.map(row => {
      const record: Record<string, unknown> = {}
      store.generatedColumns.forEach((col, i) => {
        record[col] = row[i]
      })
      return record
    })
  }
  return store.previewData.map(row => {
    const record: Record<string, unknown> = {}
    row.forEach((val, i) => {
      record[`col_${i}`] = val
    })
    return record
  })
})

function onSeedChange(value: string) {
  seedInput.value = value
  if (value === '' || value === null) {
    store.seed = null
  } else {
    const parsed = parseInt(value, 10)
    store.seed = isNaN(parsed) ? null : parsed
  }
}

async function onGenerate() {
  try {
    await store.generate()
    message.success(`成功生成 ${store.lastResult?.rowCount ?? 0} 行数据`)
  } catch (e) {
    message.error(`生成失败: ${String(e)}`)
  }
}

async function onExportAction(key: string) {
  try {
    if ((key as string).startsWith('scratchpad_')) {
      const format = (key as string).replace('scratchpad_', '') as MockExportFormat
      const path = await store.saveToScratchpad(format)
      message.success(`已保存到草稿箱: ${path}`)
    } else if (key === 'persist') {
      const result = await store.persistAsAsset(store.tableName)
      if (result) {
        message.success(`已持久化为分析资产: ${result.tableName}`)
      }
    } else {
      const path = await store.doExport(key as MockExportFormat)
      message.success(`已导出: ${path}`)
    }
  } catch (e) {
    message.error(`操作失败: ${String(e)}`)
  }
}

async function onReGenerate(historyId: string) {
  try {
    const result = await store.reGenerate(historyId)
    store.generatedTableName = result.tempTableName
    store.previewData = result.preview
    store.generatedColumns = result.columns
    message.success(`已重新生成 ${result.rowCount} 行`)
  } catch (e) {
    message.error(`重新生成失败: ${String(e)}`)
  }
}

function formatTimeStr(timestamp: string): string {
  try {
    const d = new Date(timestamp)
    const pad = (n: number) => String(n).padStart(2, '0')
    return `${d.getMonth() + 1}/${d.getDate()} ${pad(d.getHours())}:${pad(d.getMinutes())}`
  } catch {
    return timestamp
  }
}

async function loadHistory() {
  try {
    await store.loadHistory(20)
  } catch {
    message.error('加载历史失败')
  }
}

onMounted(() => {
  loadHistory()
})
</script>

<style scoped>
.mock-panel {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: var(--color-bg-primary, #1e1f22);
}

.panel-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 16px;
  border-bottom: 1px solid var(--color-border, #4a5458);
}

.panel-title {
  margin: 0;
  font-size: 14px;
  font-weight: 600;
  color: var(--color-text-primary, #e5e7eb);
}

.header-actions {
  display: flex;
  gap: 4px;
}

.panel-body {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow-y: auto;
  padding: 12px 16px;
  gap: 12px;
}

.config-section {
  padding: 12px;
  background: var(--color-bg-secondary, #2b2d30);
  border-radius: var(--border-radius-md, 6px);
  border: 1px solid var(--color-border-subtle, #3c3f41);
}

.config-row {
  display: flex;
  gap: 12px;
  align-items: flex-end;
  flex-wrap: wrap;
}

.config-item {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.config-label {
  font-size: 12px;
  font-weight: 500;
  color: var(--color-text-secondary, #9ca3af);
}

.columns-section {
  border: 1px solid var(--color-border-subtle, #3c3f41);
  border-radius: var(--border-radius-md, 6px);
  overflow: hidden;
}

.section-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 12px;
  background: var(--color-bg-secondary, #2b2d30);
  border-bottom: 1px solid var(--color-border-subtle, #3c3f41);
}

.section-title {
  font-size: 12px;
  font-weight: 600;
  color: var(--color-text-secondary, #9ca3af);
  text-transform: uppercase;
}

.columns-list {
  padding: 8px;
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.column-row {
  display: flex;
  gap: 6px;
  align-items: center;
}

.generate-section {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 8px 0;
}

.generate-info {
  font-size: 12px;
  color: var(--color-text-secondary, #9ca3af);
}

.preview-section {
  border: 1px solid var(--color-border-subtle, #3c3f41);
  border-radius: var(--border-radius-md, 6px);
  overflow: hidden;
  min-height: 0;
}

.preview-actions {
  display: flex;
  gap: 4px;
}

.preview-table-wrap {
  overflow: auto;
  max-height: 360px;
}

.history-section {
  border: 1px solid var(--color-border-subtle, #3c3f41);
  border-radius: var(--border-radius-md, 6px);
  overflow: hidden;
}

.history-list {
  padding: 4px 8px;
  max-height: 160px;
  overflow-y: auto;
}

.history-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 6px 4px;
  cursor: pointer;
  border-radius: 4px;
  transition: background 0.15s;
}

.history-item:hover {
  background: var(--color-hover, #454545);
}

.history-item-left {
  display: flex;
  gap: 8px;
  align-items: center;
  font-size: 12px;
}

.history-table {
  color: var(--color-text-primary, #e5e7eb);
  font-weight: 500;
}

.history-rows {
  color: var(--color-text-secondary, #9ca3af);
}

.history-status {
  color: var(--brand-success, #00b894);
  font-size: 11px;
}

.history-item-right {
  font-size: 11px;
  color: var(--color-text-muted, #6b7280);
}
</style>