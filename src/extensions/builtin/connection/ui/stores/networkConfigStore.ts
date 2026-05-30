/**
 * networkConfigStore — 网络配置 Pinia Store
 *
 * 管理 SSH/SSL/Proxy 网络配置文件的加载、缓存和 CRUD。
 */

import { defineStore } from 'pinia'
import { ref, computed } from 'vue'

export interface NetworkProfile {
  id: string
  name: string
  network_type: string
  config: string
  created_at: string
  updated_at: string
  origin?: string
  source_id?: string
  snapshot_at?: string
}

export type NetworkType = 'ssh' | 'ssl' | 'proxy'

export const useNetworkConfigStore = defineStore('networkConfig', () => {
  const profiles = ref<NetworkProfile[]>([])
  const loading = ref(false)

  const sshProfiles = computed(() => profiles.value.filter(p => p.network_type === 'ssh'))

  const sslProfiles = computed(() => profiles.value.filter(p => p.network_type === 'ssl'))

  const proxyProfiles = computed(() => profiles.value.filter(p => p.network_type === 'proxy'))

  async function fetchAll() {
    loading.value = true
    try {
      const { invoke } = await import('@tauri-apps/api/core')
      profiles.value = await invoke<NetworkProfile[]>('list_network_configs')
    } catch (e) {
      console.error('[networkConfigStore] fetchAll failed:', e)
    } finally {
      loading.value = false
    }
  }

  async function fetchByType(networkType: NetworkType) {
    try {
      const { invoke } = await import('@tauri-apps/api/core')
      const result = await invoke<NetworkProfile[]>('list_network_configs_by_type', {
        networkType,
      })
      // Merge: replace profiles of this type
      const others = profiles.value.filter(p => p.network_type !== networkType)
      profiles.value = [...others, ...result]
    } catch (e) {
      console.error('[networkConfigStore] fetchByType failed:', e)
    }
  }

  function getProfiles(protocol: string): NetworkProfile[] {
    if (protocol === 'ssh') return sshProfiles.value
    if (protocol === 'ssl') return sslProfiles.value
    return proxyProfiles.value
  }

  /** 按协议类型过滤 — getProfiles 的语义化别名 */
  function forProtocol(protocol: NetworkType): NetworkProfile[] {
    return getProfiles(protocol)
  }

  async function save(profile: Omit<NetworkProfile, 'id' | 'created_at' | 'updated_at'>) {
    try {
      const { invoke } = await import('@tauri-apps/api/core')
      await invoke('create_network_config', { nc: profile })
      await fetchAll()
    } catch (e) {
      console.error('[networkConfigStore] save failed:', e)
      throw e
    }
  }

  async function remove(id: string) {
    try {
      const { invoke } = await import('@tauri-apps/api/core')
      await invoke('delete_network_config', { id })
      profiles.value = profiles.value.filter(p => p.id !== id)
    } catch (e) {
      console.error('[networkConfigStore] remove failed:', e)
      throw e
    }
  }

  return {
    profiles,
    sshProfiles,
    sslProfiles,
    proxyProfiles,
    loading,
    fetchAll,
    fetchByType,
    getProfiles,
    forProtocol,
    save,
    remove,
  }
})
