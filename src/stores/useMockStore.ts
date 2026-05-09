import { defineStore } from 'pinia'
import { ref, computed } from 'vue'

import {
  mockApi,
  type MockConfig,
  type MockGenerateResult,
  type ColumnDef,
  type GeneratorType,
  type MockExportFormat,
} from '@/shared/api/mock-api'

export interface MockHistorySummary {
  id: string
  tableName: string
  rowCount: number
  seed: number | null
  status: string
  generatedAt: string
}

export const useMockStore = defineStore('mock', () => {
  const tableName = ref('mock_users')
  const rowCount = ref(100)
  const seed = ref<number | null>(null)
  const locale = ref<string>('ZH_CN')
  const columns = ref<ColumnDef[]>([
    { name: 'id', dataType: 'integer', generator: { type: 'auto_increment' }, nullableRatio: 0, unique: true },
    { name: 'username', dataType: 'varchar', generator: { type: 'username' }, nullableRatio: 0, unique: false },
    { name: 'email', dataType: 'varchar', generator: { type: 'email' }, nullableRatio: 0.1, unique: false },
    { name: 'created_at', dataType: 'datetime', generator: { type: 'datetime' }, nullableRatio: 0, unique: false },
  ])

  const generatedTableName = ref('')
  const previewData = ref<Array<Array<unknown>>>([])
  const generatedColumns = ref<string[]>([])
  const previewLoading = ref(false)
  const generateLoading = ref(false)
  const lastResult = ref<MockGenerateResult | null>(null)
  const lastHistories = ref<MockHistorySummary[]>([])

  const mockConfig = computed((): MockConfig => ({
    tableName: tableName.value,
    rowCount: rowCount.value,
    seed: seed.value ?? null,
    locale: locale.value as MockConfig['locale'],
    columns: columns.value,
  }))

  function addColumn() {
    const idx = columns.value.length + 1
    columns.value.push({
      name: `column_${idx}`,
      dataType: 'varchar',
      generator: { type: 'words' },
      nullableRatio: 0,
      unique: false,
    })
  }

  function removeColumn(index: number) {
    if (columns.value.length <= 1) return
    columns.value.splice(index, 1)
  }

  function updateColumn(index: number, patch: Partial<ColumnDef>) {
    const col = columns.value[index]
    if (col) {
      Object.assign(col, patch)
    }
  }

  function setColumnType(index: number, type: GeneratorType) {
    const col = columns.value[index]
    if (col) {
      col.generator = { type, params: {} }
    }
  }

  async function generate() {
    generateLoading.value = true
    try {
      const result = await mockApi.generate(mockConfig.value)
      lastResult.value = result
      generatedTableName.value = result.tempTableName
      previewData.value = result.preview
      generatedColumns.value = result.columns
      return result
    } finally {
      generateLoading.value = false
    }
  }

  async function preview(tableName: string, limit = 50) {
    previewLoading.value = true
    try {
      const data = await mockApi.preview(tableName, limit)
      previewData.value = data
      return data
    } finally {
      previewLoading.value = false
    }
  }

  async function doExport(format: MockExportFormat, outputPath?: string) {
    if (!generatedTableName.value) return ''
    return await mockApi.exportData({
      tempTableName: generatedTableName.value,
      format,
      outputPath,
    })
  }

  async function saveToScratchpad(format: MockExportFormat) {
    if (!generatedTableName.value) return ''
    return await mockApi.saveToScratchpad({
      tempTableName: generatedTableName.value,
      format,
    })
  }

  async function autoMapColumn(idx: number) {
    const col = columns.value[idx]
    if (!col) return
    const mapped = await mockApi.mapColumn(col.name, col.dataType)
    columns.value[idx] = {
      ...col,
      generator: { ...mapped.generator },
    }
  }

  async function persistAsAsset(name: string) {
    if (!generatedTableName.value) return null
    return await mockApi.persistAsAsset({
      tempTableName: generatedTableName.value,
      name,
    })
  }

  async function loadHistory(limit = 20) {
    const result = await mockApi.getHistory(limit)
    lastHistories.value = result.map(r => ({
      id: r.id,
      tableName: r.tableName,
      rowCount: r.rowCount,
      seed: r.seed,
      status: r.status,
      generatedAt: r.generatedAt,
    }))
  }

  async function clearHistory() {
    await mockApi.clearHistory()
    lastHistories.value = []
  }

  async function reGenerate(historyId: string) {
    return await mockApi.reGenerate(historyId)
  }

  function reset() {
    generatedTableName.value = ''
    previewData.value = []
    generatedColumns.value = []
    lastResult.value = null
    generateLoading.value = false
    previewLoading.value = false
  }

  return {
    tableName,
    rowCount,
    seed,
    locale,
    columns,
    generatedTableName,
    previewData,
    generatedColumns,
    previewLoading,
    generateLoading,
    lastResult,
    lastHistories,
    mockConfig,
    addColumn,
    removeColumn,
    updateColumn,
    setColumnType,
    generate,
    preview,
    doExport,
    saveToScratchpad,
    persistAsAsset,
    autoMapColumn,
    loadHistory,
    clearHistory,
    reGenerate,
    reset,
  }
})