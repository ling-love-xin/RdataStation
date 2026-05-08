import { tauriInvoke } from './index'

// ==================== Mock 类型定义 ====================

export type ColumnDataType =
  | 'integer'
  | 'bigint'
  | 'float'
  | 'double'
  | 'decimal'
  | 'boolean'
  | 'varchar'
  | 'text'
  | 'date'
  | 'datetime'
  | 'timestamp'
  | 'uuid'
  | 'blob'

export type GeneratorType =
  | 'auto_increment' | 'random_int' | 'random_float' | 'random_decimal'
  | 'digit' | 'number_with_format' | 'boolean'
  | 'constant' | 'words' | 'word' | 'sentence' | 'sentences'
  | 'paragraph' | 'paragraphs' | 'regex' | 'template'
  | 'md_italic' | 'md_bold' | 'md_link' | 'md_bullet'
  | 'md_list' | 'md_blockquote_single' | 'md_blockquote_multi' | 'md_code'
  | 'name' | 'name_with_title' | 'first_name' | 'last_name'
  | 'title' | 'suffix' | 'email' | 'safe_email'
  | 'free_email_provider' | 'domain_suffix' | 'free_email'
  | 'username' | 'password' | 'phone_number' | 'cell_number'
  | 'country' | 'country_code' | 'country_name' | 'city'
  | 'city_prefix' | 'city_suffix' | 'state_name' | 'state_abbr'
  | 'street_name' | 'street_suffix' | 'zip_code' | 'post_code'
  | 'building_number' | 'secondary_address' | 'secondary_address_type'
  | 'latitude' | 'longitude' | 'geohash' | 'timezone'
  | 'ip_address' | 'ipv4' | 'ipv6' | 'ip' | 'mac_address'
  | 'datetime' | 'datetime_before' | 'datetime_after'
  | 'datetime_between' | 'date' | 'time' | 'duration'
  | 'company_name' | 'company_suffix' | 'job_title'
  | 'profession' | 'industry' | 'seniority' | 'field'
  | 'position' | 'buzzword' | 'buzzword_middle' | 'buzzword_tail'
  | 'catch_phrase' | 'bs_verb' | 'bs_adj' | 'bs_noun' | 'bs'
  | 'currency_code' | 'currency_name' | 'currency_symbol'
  | 'bic' | 'isin' | 'credit_card_number'
  | 'uuid_v1' | 'uuid_v3' | 'uuid_v4' | 'uuid_v5'
  | 'url' | 'user_agent' | 'mime_type'
  | 'semver' | 'semver_stable' | 'semver_unstable'
  | 'file_path' | 'file_name' | 'file_extension' | 'dir_path'
  | 'image_url' | 'image_url_with_seed' | 'image_url_grayscale'
  | 'image_url_blur' | 'image_url_custom'
  | 'hex_color' | 'rgb_color' | 'rgba_color'
  | 'hsl_color' | 'hsla_color' | 'color'
  | 'ferroid_ulid' | 'ferroid_twitter_id' | 'ferroid_instagram_id'
  | 'ferroid_mastodon_id' | 'ferroid_discord_id'
  | 'isbn' | 'isbn10' | 'isbn13' | 'rfc_status' | 'valid_status'
  | 'licence_plate' | 'health_insurance'
  | 'foreign_key' | 'sequence' | 'weighted'

export interface GeneratorConfig {
  type: GeneratorType
  params?: Record<string, unknown>
}

export interface ColumnDef {
  name: string
  dataType: ColumnDataType
  generator: GeneratorConfig
  nullableRatio: number
  unique: boolean
}

export type Locale =
  | 'ZH_CN' | 'EN' | 'JA_JP' | 'ZH_TW'
  | 'FR_FR' | 'DE_DE' | 'IT_IT' | 'PT_BR' | 'PT_PT'
  | 'NL_NL' | 'AR_SA' | 'TR_TR' | 'FA_IR'

export interface MockConfig {
  tableName: string
  rowCount: number
  seed: number | null
  locale: Locale
  columns: ColumnDef[]
}

export interface MockGenerateResult {
  tableName: string
  tempTableName: string
  rowCount: number
  preview: Array<Array<unknown>>
  columns: string[]
  elapsedMs: number
}

export interface ColumnMappingResponse {
  columnName: string
  generator: GeneratorConfig
  confidence: string
  sampleValue: string
}

export type MockExportFormat = 'Csv' | 'Parquet' | 'Xlsx' | 'Table' | 'SqlInsert'

export interface MockExportInput {
  tempTableName: string
  format: MockExportFormat
  outputPath?: string
  tableName?: string
}

export interface ScenarioTemplate {
  id: string
  name: string
  description: string
  category: string
  locale: string
  tables: unknown[]
}

export interface MockHistoryRecord {
  id: string
  tableName: string
  rowCount: number
  seed: number | null
  configJson: string
  generatedAt: string
  status: string
}

export interface MockPersistAssetResult {
  tableName: string
  rowCount: number
  columnCount: number
}

// ==================== Mock 相关 API ====================

export const mockApi = {
  generate(config: MockConfig) {
    return tauriInvoke<MockGenerateResult>('mock_generate', { config })
  },

  preview(tableName: string, limit: number) {
    return tauriInvoke<Array<Array<unknown>>>('mock_preview', { tableName, limit })
  },

  exportData(input: MockExportInput) {
    return tauriInvoke<string>('mock_export', { input })
  },

  mapColumn(columnName: string, dataType: string) {
    return tauriInvoke<ColumnMappingResponse>('mock_map_column', { columnName, dataType })
  },

  mapColumnsBatch(columns: Array<[string, string]>) {
    return tauriInvoke<ColumnMappingResponse[]>('mock_map_columns_batch', { columns })
  },

  listTemplates() {
    return tauriInvoke<ScenarioTemplate[]>('mock_list_templates')
  },

  applyTemplate(templateId: string) {
    return tauriInvoke<ScenarioTemplate>('mock_apply_template', { templateId })
  },

  importSchema(input: {
    connId: string
    database: string
    schema?: string
    tables: string[]
    connectionType?: string
    projectPath?: string
  }) {
    return tauriInvoke<ColumnDef[]>('mock_import_schema', { input })
  },

  saveToScratchpad(input: { tempTableName: string; format: MockExportFormat }) {
    return tauriInvoke<string>('mock_save_to_scratchpad', { input })
  },

  persistAsAsset(input: { tempTableName: string; name: string }) {
    return tauriInvoke<MockPersistAssetResult>('mock_persist_as_asset', { input })
  },

  getHistory(limit = 50) {
    return tauriInvoke<MockHistoryRecord[]>('mock_get_history', { limit })
  },

  clearHistory() {
    return tauriInvoke<number>('mock_clear_history')
  },

  reGenerate(historyId: string) {
    return tauriInvoke<MockGenerateResult>('mock_re_generate', { historyId })
  },
}