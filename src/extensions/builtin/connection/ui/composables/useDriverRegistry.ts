/**
 * 驱动注册表面 — 获取后端可用的数据库驱动列表
 *
 * TODO: 后续接入 Tauri backend `get_available_drivers` 命令
 */
import { ref } from 'vue'

import type { DriverDescriptor } from '../types/connection'

const allDrivers = ref<DriverDescriptor[]>([])

export function useDriverRegistry() {
  // TODO: fetch from Tauri backend
  return { allDrivers }
}