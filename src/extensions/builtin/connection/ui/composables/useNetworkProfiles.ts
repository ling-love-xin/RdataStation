/**
 * useNetworkProfiles — 网络配置文件列表 Composable
 *
 * 核心职责：从后端获取 SSH/SSL/Proxy 三类网络配置，解析 config JSON 并生成可读摘要。
 */
import { invoke } from '@tauri-apps/api/core'
import { ref, computed, readonly } from 'vue'

// ==================== 类型 ====================

export interface NetworkProfile {
  id: string
  name: string
  type: 'ssh' | 'ssl' | 'proxy'
  config: unknown
  detail: string
  origin?: string
}

interface ConfigRaw {
  id: string
  name?: string
  network_type: string
  config: string
  origin?: string
  created_at: string
  updated_at: string
}

// ==================== 类型守卫 ====================

function isRecord(v: unknown): v is Record<string, unknown> {
  return typeof v === 'object' && v !== null
}

function isSshConfig(v: unknown): v is { host?: string; port?: number } {
  return isRecord(v) && !('mode' in v) && !('type' in v)
}

function isSslConfig(v: unknown): v is { mode?: string } {
  return isRecord(v) && 'mode' in v
}

function isProxyConfig(v: unknown): v is { host?: string; port?: number } {
  return isRecord(v) && 'type' in v
}

// ==================== 状态 ====================

const sshProfiles = ref<NetworkProfile[]>([])
const sslProfiles = ref<NetworkProfile[]>([])
const proxyProfiles = ref<NetworkProfile[]>([])
const loading = ref(false)
const error = ref<string | null>(null)

// ==================== 工具函数 ====================

export function parseConfig<T>(configJson: string): T | null {
  try { return JSON.parse(configJson) as T } catch { return null }
}

function buildDetail(type: string, cfg: unknown): string {
  if (isSslConfig(cfg)) return cfg.mode ?? 'verify-full'
  if (isSshConfig(cfg)) return `${cfg.host ?? 'localhost'}:${cfg.port ?? 22}`
  if (isProxyConfig(cfg)) return `${cfg.host ?? 'proxy.corp.com'}:${cfg.port ?? 1080}`
  return type === 'ssh' ? 'localhost:22' : type === 'ssl' ? 'verify-full' : 'proxy.corp.com:1080'
}

function toProfile(raw: ConfigRaw): NetworkProfile | null {
  const config = parseConfig<unknown>(raw.config)
  if (config === null) return null
  const type = raw.network_type as NetworkProfile['type']
  return { id: raw.id, name: raw.name ?? raw.id, type, config, detail: buildDetail(type, config), origin: raw.origin }
}

// ==================== API ====================

async function loadByType(type: 'ssh' | 'ssl' | 'proxy'): Promise<void> {
  try {
    const raws = await invoke<ConfigRaw[]>('list_network_configs', { networkType: type })
    const profiles = raws.map(toProfile).filter((p): p is NetworkProfile => p !== null)
    if (type === 'ssh') sshProfiles.value = profiles
    else if (type === 'ssl') sslProfiles.value = profiles
    else proxyProfiles.value = profiles
  } catch (e) {
    console.error(`[useNetworkProfiles] Failed to load ${type}:`, e)
    error.value = e instanceof Error ? e.message : String(e)
  }
}

async function loadAll(): Promise<void> {
  loading.value = true; error.value = null
  try { await Promise.all([loadByType('ssh'), loadByType('ssl'), loadByType('proxy')]) }
  finally { loading.value = false }
}

// ==================== Composable ====================

export function useNetworkProfiles() {
  return {
    sshProfiles: computed(() => sshProfiles.value),
    sslProfiles: computed(() => sslProfiles.value),
    proxyProfiles: computed(() => proxyProfiles.value),
    loading: readonly(loading),
    error: readonly(error),
    loadAll,
    loadByType,
    parseConfig,
  }
}