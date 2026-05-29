/**
 * useUrlBuilder — URI 构建 Composable
 *
 * 从 AddDataSourceDialog.vue 提取，处理：
 * - URI 预览（uriPreview computed）
 * - 连接 URL 构建（buildUrl，用于连接测试和保存）
 */
import { computed, type ComputedRef, type Ref } from 'vue'

export interface DriverInfo {
  id: string
  name: string
  type_id: string
  is_file?: boolean
  default_port?: number | string
  url_template?: string
}

export interface UseUrlBuilderOptions {
  /** 当前选中的驱动（可为 null） */
  selectedDriver: ComputedRef<DriverInfo | null>
  /** 表单数据 */
  formData: Ref<Record<string, unknown>>
  /** 是否处于手动 URI 编辑模式 */
  uriEditing: Ref<boolean>
  /** 手动编辑的 URI 值 */
  manualUri: Ref<string>
}

export function useUrlBuilder(opts: UseUrlBuilderOptions) {
  const { selectedDriver, formData, uriEditing, manualUri } = opts

  /** 使用 url_template 构建 URL */
  function applyTemplate(template: string, fd: Record<string, unknown>): string {
    return template
      .replace('{host}', String(fd.host || 'localhost'))
      .replace('{port}', String(fd.port || ''))
      .replace('{database}', String(fd.database || ''))
      .replace('{username}', String(fd.username || ''))
      .replace('{password}', String(fd.password || ''))
      .replace('{file_path}', String(fd.file_path || fd.database || ''))
      .replace('{driver}', String(fd.driver || ''))
  }

  /** URI 预览（展示用，密码用 **** 遮蔽） */
  const uriPreview = computed(() => {
    const d = selectedDriver.value
    if (!d) return ''
    const fd = formData.value
    if (d.url_template) {
      const masked = { ...fd, password: fd.password ? '****' : '' }
      return applyTemplate(d.url_template, masked)
    }
    if (d.is_file) return `${d.name.toLowerCase()}://${fd.file_path || fd.database || './data.db'}`
    const usr = fd.username || 'user'
    const pw = fd.password ? '****' : ''
    const h = fd.host || 'localhost'
    const p = fd.port || d.default_port || ''
    const db = fd.database || ''
    if (pw) return `${d.name.toLowerCase()}://${usr}:${pw}@${h}${p ? ':' + p : ''}/${db}`
    return `${d.name.toLowerCase()}://${usr}@${h}${p ? ':' + p : ''}/${db}`
  })

  /** 构建实际连接 URL（用于测试/保存） */
  function buildUrl(): string {
    if (uriEditing.value && manualUri.value) return manualUri.value
    const d = selectedDriver.value
    if (!d) return ''
    const fd = formData.value
    if (d.url_template) {
      return applyTemplate(d.url_template, fd)
    }
    const proto = d.type_id.toLowerCase()
    if (d.is_file) return `${proto}://${fd.file_path || fd.database || './data.db'}`
    const h = String(fd.host || 'localhost')
    const po = String(fd.port || d.default_port || '')
    const db = String(fd.database || '')
    const u = String(fd.username || '')
    const pw = String(fd.password || '')
    if (u && pw) return `${proto}://${u}:${pw}@${h}${po ? ':' + po : ''}/${db}`
    return `${proto}://${u}@${h}${po ? ':' + po : ''}/${db}`
  }

  return { uriPreview, buildUrl }
}