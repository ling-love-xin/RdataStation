/**
 * 驱动类型定义
 */

import type { DriverConfig, DriverField } from '@/shared/types/databaseMeta'

export type { DriverConfig, DriverField }

// 驱动类型
export type DriverType = 'mysql' | 'postgresql' | 'sqlite' | 'duckdb' | string

// 驱动配置选项
export interface DriverOption {
  label: string
  value: string
}

// 驱动字段值
export type DriverFieldValue = string | number | boolean | undefined

// 驱动配置表单数据
export interface DriverFormData {
  [key: string]: DriverFieldValue
}
