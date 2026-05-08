import { ref, type ComputedRef } from 'vue'

import type { ResultTab } from '../types/result'

export function useResultExport(activeTab: ComputedRef<ResultTab | null>) {
  const isExporting = ref(false)

  function generateCsv(): string {
    const tab = activeTab.value
    if (!tab) return ''
    const header = tab.columns.map(c => `"${c.replace(/"/g, '""')}"`).join(',')
    const body = tab.rows
      .map(r => r.map(v => (v === null ? '' : `"${String(v).replace(/"/g, '""')}"`)).join(','))
      .join('\n')
    return header + '\n' + body
  }

  function generateJson(): string {
    const tab = activeTab.value
    if (!tab) return ''
    const data = tab.rows.map(r => {
      const obj: Record<string, unknown> = {}
      tab.columns.forEach((c, i) => {
        obj[c] = r[i]
      })
      return obj
    })
    return JSON.stringify(data, null, 2)
  }

  function generateInsert(): string {
    const tab = activeTab.value
    if (!tab) return ''
    const tableName = 'table_name'
    const cols = tab.columns.join(', ')
    const values = tab.rows
      .map(
        r =>
          `(${r
            .map(v => {
              if (v === null) return 'NULL'
              if (typeof v === 'number') return String(v)
              return `'${String(v).replace(/'/g, "''")}'`
            })
            .join(', ')})`
      )
      .join(',\n')
    return `INSERT INTO ${tableName} (${cols}) VALUES\n${values};`
  }

  async function exportAs(format: string): Promise<void> {
    const tab = activeTab.value
    if (!tab || tab.rows.length === 0) return

    isExporting.value = true
    try {
      let content = ''
      switch (format) {
        case 'csv':
          content = generateCsv()
          break
        case 'json':
          content = generateJson()
          break
        case 'insert':
          content = generateInsert()
          break
        default:
          content = generateCsv()
      }
      await navigator.clipboard.writeText(content)

      // Message handled by parent
    } finally {
      isExporting.value = false
    }
  }

  function copyCellValue(value: unknown): void {
    const text = value === null ? 'NULL' : String(value)
    navigator.clipboard.writeText(text)
  }

  return {
    isExporting,
    exportAs,
    copyCellValue,
    generateCsv,
    generateJson,
    generateInsert,
  }
}
