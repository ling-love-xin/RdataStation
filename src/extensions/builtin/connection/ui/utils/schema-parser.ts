/**
 * JSON Schema 表单生成器
 *
 * 根据后端定义的 config_schema 动态生成前端表单字段配置
 */

export interface SchemaProperty {
  type: 'string' | 'number' | 'boolean' | 'integer' | 'array' | 'object'
  title?: string
  description?: string
  default?: unknown
  minimum?: number
  maximum?: number
  maxLength?: number
  minLength?: number
  pattern?: string
  enum?: unknown[]
  required?: boolean
  format?: 'password' | 'uri' | 'email' | 'hostname' | 'ipv4' | 'ipv6'
  items?: SchemaProperty
  properties?: Record<string, SchemaProperty>
}

export interface FormFieldConfig {
  key: string
  label: string
  type: 'input' | 'input-number' | 'select' | 'switch' | 'textarea'
  placeholder?: string
  defaultValue?: unknown
  rules?: Array<{ required?: boolean; message?: string; pattern?: string }>
  options?: Array<{ label: string; value: unknown }>
  min?: number
  max?: number
  step?: number
  rows?: number
  disabled?: boolean
  helpText?: string
}

export interface FormSchema {
  type: 'object'
  properties: Record<string, SchemaProperty>
  required?: string[]
}

/**
 * 解析 JSON Schema 并生成表单字段配置列表
 *
 * @param schema - JSON Schema 字符串
 * @returns 表单字段配置数组
 */
export function parseSchemaToFormFields(schema: string): FormFieldConfig[] {
  if (!schema) return []

  let parsed: FormSchema
  try {
    parsed = JSON.parse(schema) as FormSchema
  } catch {
    console.warn('[SchemaParser] 解析 config_schema 失败:', schema)
    return []
  }

  if (parsed.type !== 'object' || !parsed.properties) {
    return []
  }

  const requiredFields = new Set(parsed.required || [])
  const fields: FormFieldConfig[] = []

  for (const [key, prop] of Object.entries(parsed.properties)) {
    const field = convertPropertyToField(key, prop, requiredFields.has(key))
    if (field) {
      fields.push(field)
    }
  }

  return fields
}

/**
 * 将单个 JSON Schema 属性转换为表单字段配置
 */
function convertPropertyToField(
  key: string,
  prop: SchemaProperty,
  isRequired: boolean
): FormFieldConfig | null {
  const label = prop.title || formatLabel(key)

  const rules: FormFieldConfig['rules'] = []

  if (isRequired) {
    rules.push({ required: true, message: `${label}不能为空` })
  }

  if (prop.minLength !== undefined) {
    rules.push({ pattern: `.{${prop.minLength},}`, message: `${label}至少${prop.minLength}个字符` })
  }

  if (prop.maxLength !== undefined) {
    rules.push({
      pattern: `^.{0,${prop.maxLength}}$`,
      message: `${label}最多${prop.maxLength}个字符`,
    })
  }

  if (prop.pattern) {
    rules.push({ pattern: prop.pattern, message: `${label}格式不正确` })
  }

  const rules_obj = rules.length > 0 ? rules : undefined

  switch (prop.type) {
    case 'string':
      return {
        key,
        label,
        type: determineStringInputType(prop),
        placeholder: prop.description || `请输入${label}`,
        defaultValue: prop.default,
        rules: rules_obj,
        helpText: prop.description,
      }

    case 'number':
    case 'integer':
      return {
        key,
        label,
        type: 'input-number',
        placeholder: prop.description || `请输入${label}`,
        defaultValue: prop.default ?? prop.minimum ?? 0,
        rules: rules_obj,
        min: prop.minimum,
        max: prop.maximum,
        helpText: prop.description,
      }

    case 'boolean':
      return {
        key,
        label,
        type: 'switch',
        defaultValue: prop.default ?? false,
        rules: rules_obj,
        helpText: prop.description,
      }

    case 'array':
      if (prop.items?.enum) {
        return {
          key,
          label,
          type: 'select',
          placeholder: prop.description || `请选择${label}`,
          defaultValue: prop.default,
          rules: rules_obj,
          options: prop.items.enum.map(v => ({ label: String(v), value: v })),
          helpText: prop.description,
        }
      }
      return null

    case 'object':
      if (prop.enum) {
        return {
          key,
          label,
          type: 'select',
          placeholder: prop.description || `请选择${label}`,
          defaultValue: prop.default,
          rules: rules_obj,
          options: prop.enum.map(v => ({ label: String(v), value: v })),
          helpText: prop.description,
        }
      }
      return null

    default:
      return null
  }
}

/**
 * 根据属性格式和类型决定输入框类型
 */
function determineStringInputType(prop: SchemaProperty): FormFieldConfig['type'] {
  if (prop.format === 'password') {
    return 'input'
  }
  if (prop.format === 'uri' || prop.format === 'email' || prop.format === 'hostname') {
    return 'input'
  }
  if (prop.description && prop.description.includes('密码')) {
    return 'input'
  }
  return 'input'
}

/**
 * 将驼峰命名转换为中文标签
 */
function formatLabel(key: string): string {
  const labelMap: Record<string, string> = {
    host: '主机',
    port: '端口',
    database: '数据库',
    username: '用户名',
    password: '密码',
    ssl: '启用 SSL',
    sslMode: 'SSL 模式',
    sslCert: 'SSL 证书',
    sslKey: 'SSL 密钥',
    sslCa: 'CA 证书',
    timeout: '连接超时',
    charset: '字符集',
    socket: 'Unix Socket',
    compression: '启用压缩',
    autoReconnect: '自动重连',
    useUnicode: '使用 Unicode',
    connectionTimeout: '连接超时时间',
    socketTimeout: 'Socket 超时时间',
    serverTimezone: '服务器时区',
    sslTls: '启用 SSL/TLS',
    sshTunnel: '启用 SSH 隧道',
    proxy: '启用代理',
  }

  return (
    labelMap[key] ||
    key
      .replace(/([A-Z])/g, '_$1')
      .replace(/_+/g, ' ')
      .trim()
  )
}
