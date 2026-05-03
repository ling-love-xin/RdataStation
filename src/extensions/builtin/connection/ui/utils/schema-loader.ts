
import type { DriverFormSchema } from '../types/form-schema'

export { generateDefaultFormData, validateFormData } from '../types/form-schema'

const schemaCache = new Map<string, DriverFormSchema>()

export async function loadDriverSchema(driverId: string): Promise<DriverFormSchema | null> {
  if (schemaCache.has(driverId)) {
    return schemaCache.get(driverId)!
  }

  try {
    const schemaModule = await import(`../schemas/${driverId}.json`)
    const schema = schemaModule.default as DriverFormSchema
    schemaCache.set(driverId, schema)
    return schema
  } catch (e) {
    console.warn(`未找到驱动 ${driverId} 的表单配置，使用默认配置`)
    return null
  }
}

export async function loadAllSchemas(): Promise<DriverFormSchema[]> {
  const schemaFiles = import.meta.glob('../schemas/*.json')
  const schemas: DriverFormSchema[] = []

  for (const path of Object.keys(schemaFiles)) {
    try {
      const module = await schemaFiles[path]()
      const schema = (module as any).default as DriverFormSchema
      schemas.push(schema)
      schemaCache.set(schema.driverId, schema)
    } catch (e) {
      console.error(`加载表单配置失败: ${path}`, e)
    }
  }

  return schemas
}

export function clearSchemaCache(): void {
  schemaCache.clear()
}
