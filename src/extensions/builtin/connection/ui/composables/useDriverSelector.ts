import { invoke } from '@tauri-apps/api/core'
import { Database, Globe, Server, Settings, Shield, Zap } from 'lucide-vue-next'
import { ref, computed, type Component } from 'vue'

import { loadDriverSchema, generateDefaultFormData } from '../utils/schema-loader'

import type { DriverDescriptor } from '../types/connection'
import type { DriverFormSchema, FormSectionConfig } from '../types/form-schema'

interface TabConfig {
  key: string
  label: string
  icon: Component
  disabled?: boolean
}

const FILE_BASED_DRIVERS = new Set(['sqlite', 'duckdb'])

export function useDriverSelector() {
  const selectedDriver = ref<DriverDescriptor | null>(null)
  const showDriverTree = ref(true)
  const activeTab = ref('general')

  const driversCache = ref<DriverDescriptor[] | null>(null)
  const drivers = ref<DriverDescriptor[]>([])

  const currentSchema = ref<DriverFormSchema | null>(null)
  const dynamicFormData = ref<Record<string, unknown>>({})
  const dynamicFormSections = ref<FormSectionConfig[]>([])

  const isFileBased = computed(() => {
    if (!selectedDriver.value) return true
    return FILE_BASED_DRIVERS.has(selectedDriver.value.id)
  })

  const visibleTabs = computed<TabConfig[]>(() => {
    if (!selectedDriver.value) return []

    const tabs: TabConfig[] = [
      { key: 'general', label: '常规', icon: Settings },
    ]

    tabs.push({ key: 'ssh', label: 'SSH 隧道', icon: Server, disabled: isFileBased.value })
    tabs.push({ key: 'ssl', label: 'SSL/TLS', icon: Shield, disabled: isFileBased.value })
    tabs.push({ key: 'proxy', label: '代理', icon: Globe, disabled: isFileBased.value })

    const isDuckdbDriver = selectedDriver.value.id === 'duckdb'
    tabs.push({ key: 'duckdb', label: '本地加速', icon: Zap, disabled: isDuckdbDriver })

    tabs.push({ key: 'driver', label: '驱动', icon: Database })

    return tabs
  })

  async function loadDrivers(force = false) {
    if (!force && driversCache.value) {
      drivers.value = driversCache.value
      return
    }

    try {
      drivers.value = await invoke<DriverDescriptor[]>('get_drivers')
      driversCache.value = drivers.value
    } catch (e) {
      console.error('加载驱动列表失败:', e)
    }
  }

  async function loadDynamicForm(driverId: string) {
    const schema = await loadDriverSchema(driverId)

    if (schema) {
      currentSchema.value = schema
      dynamicFormSections.value = schema.sections
      dynamicFormData.value = generateDefaultFormData(schema)

      return {
        dynamicData: dynamicFormData.value,
        defaultPort: schema.metadata?.defaultPort,
      }
    } else {
      currentSchema.value = null
      dynamicFormSections.value = []
      dynamicFormData.value = {}
      return null
    }
  }

  function findDriverById(driverId: string): DriverDescriptor | null {
    return drivers.value.find((d: DriverDescriptor) => d.id === driverId) || null
  }

  return {
    selectedDriver,
    showDriverTree,
    activeTab,
    drivers,
    driversCache,
    visibleTabs,
    isFileBased,
    dynamicFormSections,
    dynamicFormData,
    currentSchema,
    loadDrivers,
    loadDynamicForm,
    findDriverById,
  }
}