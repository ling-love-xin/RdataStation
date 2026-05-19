/**
 * 驱动适配器 — 将 DB 层的 Driver / config_schema 转换为前端需要的 DriverDescriptor / DriverFormSchema
 *
 * 桥梁：Backend `get_available_drivers` → `Driver` struct → 前端 `DriverDescriptor` + `DriverFormSchema`
 */

import type { DriverDescriptor } from '../types/connection'
import type { DriverFormSchema, FormSectionConfig, FormFieldConfig } from '../types/form-schema'

// ==================== 后端 Driver 结构体映射 ====================

/** 后端 Driver 表的行结构（从 Rust serde Serialize 输出） */
export interface BackendDriver {
  id: string
  type_id: string
  name: string
  driver_kind: string
  is_file: boolean
  default_port: number | null
  url_template: string | null
  download_url: string | null
  version: string | null
  config_schema: string // JSON string
  supported_auth_types: string | null // JSON array string
  capabilities: string | null // JSON array string
  enabled: boolean
}

export interface BackendDataSourceType {
  id: string
  name: string
  category: string
  icon: string | null
  enabled: boolean
}

// ==================== config_schema JSON 结构 ====================

interface DbConfigSchema {
  fields?: DbFieldDef[]
  options?: DbFieldDef[]
}

interface DbFieldDef {
  key: string
  label: string
  type: string
  required?: boolean
  default?: string
  placeholder?: string
  values?: string[]
}

// ==================== 转换函数 ====================

/**
 * 将后端 Driver 转为前端 DriverDescriptor
 * 从 config_schema JSON 中提取 fields/options 构建 fields 和 extraOptions
 */
export function backendDriverToDescriptor(driver: BackendDriver): DriverDescriptor {
  const schema = parseConfigSchema(driver.config_schema)

  // 从 config_schema fields 构建 DriverField[] 列表
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const fields = (schema.fields || []).map(f => ({
    name: f.key,
    label: f.label,
    type: mapFieldType(f.type),
    required: f.required ?? false,
    default: f.default ?? '',
    placeholder: f.placeholder,
  })) as any

  // 从 config_schema options 构建 extraOptions
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const extraOptions = (schema.options || []).map(o => ({
    name: o.key,
    label: o.label,
    type: mapOptionType(o.type),
  })) as any

  // 解析 capabilities
  let capabilitiesStr = driver.capabilities
  if (capabilitiesStr) {
    try {
      capabilitiesStr = JSON.parse(capabilitiesStr) as string
    } catch {
      // keep as-is
    }
  }

  const features = tryParseJsonArray(driver.capabilities)

  const descriptor: DriverDescriptor = {
    id: driver.id,
    name: driver.name,
    icon: driver.id,
    version: driver.version || undefined,
    features,
    category: driver.type_id,
    defaultPort: driver.default_port ?? undefined,
    default_port: driver.default_port ?? undefined,
    description: driver.name,
    driverKind: driver.driver_kind,
    urlTemplate: driver.url_template || undefined,
    fields: fields.length > 0 ? fields : undefined,
    extraOptions: extraOptions.length > 0 ? extraOptions : undefined,
    extra_options: extraOptions.length > 0 ? extraOptions : undefined,
    requireFile: driver.is_file,
    require_file: driver.is_file,
    requireDatabase: !driver.is_file,
    require_database: !driver.is_file,
    supportsSsl: driver.supported_auth_types?.includes('ssl') ?? false,
    supportsSshTunnel: false,
    supportsHttpProxy: false,
    supportsSocksProxy: false,
  }

  return descriptor
}

/**
 * 将 config_schema JSON 字符串转为 DriverFormSchema
 * 用于 DynamicFormRenderer 渲染表单
 */
export function configSchemaToFormSchema(
  driver: BackendDriver
): DriverFormSchema {
  const schema = parseConfigSchema(driver.config_schema)

  const sections: FormSectionConfig[] = []

  // Section 1: Connection 字段
  const connectionFields: FormFieldConfig[] = (schema.fields || []).map(mapDbFieldToFormField)

  if (connectionFields.length > 0) {
    sections.push({
      key: 'connection',
      title: '连接参数',
      description: '数据库连接必需参数',
      fields: connectionFields,
    })
  }

  // Section 2: 选项字段
  const optionFields: FormFieldConfig[] = (schema.options || []).map(mapDbFieldToFormField)

  if (optionFields.length > 0) {
    sections.push({
      key: 'options',
      title: '高级选项',
      icon: 'settings',
      collapsible: true,
      collapsed: false,
      fields: optionFields,
    })
  }

  return {
    driverId: driver.id,
    driverName: driver.name,
    version: driver.version || undefined,
    sections,
    metadata: {
      category: driver.type_id,
      description: driver.name,
      features: tryParseJsonArray(driver.capabilities),
      defaultPort: driver.default_port ?? undefined,
      driverKind: driver.driver_kind,
      urlTemplate: driver.url_template || undefined,
      requireFile: driver.is_file,
      requireDatabase: !driver.is_file,
      supportsSsl: driver.supported_auth_types?.includes('ssl') ?? false,
      supportsSshTunnel: false,
      supportsHttpProxy: false,
      supportsSocksProxy: false,
    },
  }
}

/**
 * 解析 config_schema JSON 为结构化对象
 */
export function parseConfigSchema(raw: string): DbConfigSchema {
  try {
    return JSON.parse(raw) as DbConfigSchema
  } catch {
    return {}
  }
}

/**
 * 判断是否为文件数据库
 */
export function isFileDatabase(driver: BackendDriver): boolean {
  return driver.is_file
}

// ==================== 内部辅助 ====================

function mapDbFieldToFormField(f: DbFieldDef): FormFieldConfig {
  const fieldType = mapFieldType(f.type)
  const formField: FormFieldConfig = {
    key: f.key,
    label: f.label,
    type: fieldType as FormFieldConfig['type'],
    required: f.required ?? false,
    default: f.default,
    placeholder: f.placeholder,
  }

  // select 类型需要 options
  if (fieldType === 'select' && f.values) {
    formField.options = f.values.map(v => ({ label: v, value: v }))
  }

  return formField
}

function mapFieldType(dbType: string): string {
  const typeMap: Record<string, string> = {
    text: 'text',
    password: 'password',
    number: 'number',
    select: 'select',
    checkbox: 'checkbox',
    file: 'file',
    textarea: 'textarea',
  }
  return typeMap[dbType] || 'text'
}

function mapOptionType(dbType: string): 'string' | 'number' | 'boolean' | 'select' {
  const typeMap: Record<string, 'string' | 'number' | 'boolean' | 'select'> = {
    text: 'string',
    password: 'string',
    number: 'number',
    select: 'select',
    checkbox: 'boolean',
  }
  return typeMap[dbType] || 'string'
}

function tryParseJsonArray(raw: string | null | undefined): string[] {
  if (!raw) return []
  try {
    const parsed = JSON.parse(raw)
    if (Array.isArray(parsed)) {
      return parsed.filter(
        (item: unknown): item is string => typeof item === 'string'
      )
    }
    return []
  } catch {
    return raw.split(',').map(s => s.trim()).filter(Boolean)
  }
}