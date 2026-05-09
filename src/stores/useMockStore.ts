import { defineStore } from 'pinia'
import { ref, computed } from 'vue'

import {
  mockApi,
  type MockConfig,
  type MockGenerateResult,
  type ColumnDef,
  type GeneratorType,
  type MockExportFormat,
  type MockGenerationTask,
  type MockColumnInput,
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
  const persistenceHistory = ref<MockGenerationTask[]>([])
  const persistenceLoading = ref(false)

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

  async function generateAndSave(projectPath: string) {
    const result = await generate()
    if (result) {
      try {
        await saveTask(projectPath)
      } catch {
        // 持久化失败不影响生成流程
      }
    }
    return result
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

  function buildTaskInput(): MockGenerationTask {
    const now = new Date().toISOString()
    return {
      id: crypto.randomUUID(),
      tableName: tableName.value,
      tableAlias: generatedTableName.value || null,
      rowCount: rowCount.value,
      seed: seed.value ?? null,
      locale: locale.value,
      sceneId: null,
      saveFormat: 'table',
      status: 'success',
      errorMessage: null,
      generatedRows: rowCount.value,
      generationTimeMs: null,
      createdAt: now,
      updatedAt: now,
    }
  }

  function buildColumnInputs(): MockColumnInput[] {
    return columns.value.map((col, idx) => ({
      id: crypto.randomUUID(),
      columnName: col.name,
      columnType: col.dataType,
      generator: buildGeneratorString(col.generator),
      generatorParams: col.generator.params
        ? JSON.stringify(col.generator.params)
        : null,
      nullRatio: col.nullableRatio,
      isUnique: col.unique ?? false,
      isPrimaryKey: col.name === 'id',
      isForeignKey: false,
      refTable: null,
      refColumn: null,
      comment: null,
      confidence: 'manual',
      sortOrder: idx,
    }))
  }

  function buildGeneratorString(generator: { type: string; params?: Record<string, unknown> }): string {
    const localeSuffix = locale.value ? `(${locale.value})` : ''
    return `${generator.type}${localeSuffix}`
  }

  async function saveTask(projectPath: string) {
    const task = buildTaskInput()
    const cols = buildColumnInputs()
    return await mockApi.saveTask(projectPath, task, cols)
  }

  async function loadHistoryV2(projectPath: string, limit = 20) {
    persistenceLoading.value = true
    try {
      persistenceHistory.value = await mockApi.getHistoryV2(projectPath, limit)
    } finally {
      persistenceLoading.value = false
    }
  }

  async function loadDetail(projectPath: string, taskId: string) {
    const detail = await mockApi.getDetail(projectPath, taskId)
    tableName.value = detail.task.tableName
    rowCount.value = detail.task.rowCount
    seed.value = detail.task.seed ?? null
    locale.value = detail.task.locale
    columns.value = detail.columns.map<ColumnDef>((col: Record<string, unknown>) => ({
      name: col.columnName as string,
      dataType: col.columnType as ColumnDef['dataType'],
      generator: {
        type: (col.generator as string).replace(/\\(.*\\)$/, '') as GeneratorType,
        params: col.generatorParams
          ? JSON.parse(col.generatorParams as string)
          : undefined,
      },
      nullableRatio: col.nullRatio as number,
      unique: col.isUnique as boolean,
    }))
    return detail
  }

  async function deletePersistenceTask(projectPath: string, taskId: string) {
    await mockApi.deleteTask(projectPath, taskId)
    persistenceHistory.value = persistenceHistory.value.filter(t => t.id !== taskId)
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
    persistenceHistory,
    persistenceLoading,
    mockConfig,
    addColumn,
    removeColumn,
    updateColumn,
    setColumnType,
    generate,
    generateAndSave,
    preview,
    doExport,
    saveToScratchpad,
    persistAsAsset,
    autoMapColumn,
    loadHistory,
    clearHistory,
    reGenerate,
    saveTask,
    buildTaskInput,
    buildColumnInputs,
    loadHistoryV2,
    loadDetail,
    deletePersistenceTask,
    reset,
  }
})