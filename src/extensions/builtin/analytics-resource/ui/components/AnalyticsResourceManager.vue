<template>
  <div class="analytics-resource-manager">
    <div class="header">
      <h2>分析资源管理器</h2>
      <div class="header-actions">
        <button class="btn" @click="refresh">
          <span class="icon">🔄</span>
          刷新
        </button>
        <button class="btn" @click="showSettings = true">
          <span class="icon">⚙️</span>
          设置
        </button>
      </div>
    </div>

    <SearchBar
      v-model="searchQuery"
      :debounce-ms="300"
      @search="handleSearch"
      @clear="handleClearSearch"
    />

    <FilterBar
      v-model:selected-scope="selectedScope"
      v-model:selected-type="selectedType"
      v-model:selected-sort="sortBy"
      v-model:sort-order="sortOrder"
      :selected-count="selectedResources.length"
      @clear-selection="clearSelection"
      @batch-delete="handleBatchDelete"
    />

    <FolderList
      :folders="folders"
      :selected-folder-id="selectedFolderId"
      @select-folder="selectFolder"
      @create-folder="showCreateFolder = true"
      @drop-resource="handleDropResource"
    />

    <ResourceList
      :items="filteredResources"
      :selected-ids="selectedResources"
      :display-settings="store.settings.display"
      empty-icon="📭"
      empty-text="暂无资源"
      @select="handleSelectResource"
      @open="handleOpenResource"
      @delete="handleDeleteResource"
      @edit="handleEditResource"
      @copy="handleCopyResource"
      @dragstart="handleDragStart"
      @dragend="handleDragEnd"
    />

    <Pagination
      v-if="total > 0"
      :page="page"
      :page-size="pageSize"
      :total="total"
      :total-pages="totalPages"
      @update:page="handlePageChange"
      @update:page-size="handlePageSizeChange"
      @prev="handlePrevPage"
      @next="handleNextPage"
    />

    <div class="footer">
      <button class="btn" @click="showCreateResource = true">
        <span class="icon">+</span>
        添加资源
      </button>
      <button class="btn" @click="showRecycleBin = true">
        <span class="icon">🗑️</span>
        回收站
      </button>
    </div>

    <CreateResourceModal
      v-if="showCreateResource || showEditResource"
      :edit-resource="editingResource"
      @close="closeEditModal"
      @create="handleCreateResource"
      @update="handleUpdateResource"
    />

    <CreateFolderModal
      v-if="showCreateFolder"
      @close="showCreateFolder = false"
      @create="handleCreateFolder"
    />

    <RecycleBinModal
      v-if="showRecycleBin"
      @close="showRecycleBin = false"
    />

    <SettingsModal
      v-if="showSettings"
      :settings="store.settings"
      @close="showSettings = false"
      @save="handleSaveSettings"
      @clear-cache="handleClearCache"
    />

    <ToastContainer />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted } from 'vue'


import CreateFolderModal from './CreateFolderModal.vue'
import CreateResourceModal from './CreateResourceModal.vue'
import FilterBar from './FilterBar.vue'
import FolderList from './FolderList.vue'
import Pagination from './Pagination.vue'
import RecycleBinModal from './RecycleBinModal.vue'
import ResourceList from './ResourceList.vue'
import SearchBar from './SearchBar.vue'
import SettingsModal from './SettingsModal.vue'
import ToastContainer from './ToastContainer.vue'
import { useToast } from '../composables/use-toast'
import { useAnalyticsResourceStore } from '../stores/analytics-resource-store'

import type { CreateResourceRequest, CreateFolderRequest, AnalyticsResource , AnalyticsResourceSettings } from '../../types'

const store = useAnalyticsResourceStore()
const toast = useToast()

const searchQuery = ref('')
const showCreateResource = ref(false)
const showCreateFolder = ref(false)
const showRecycleBin = ref(false)
const showEditResource = ref(false)
const showSettings = ref(false)
const editingResource = ref<AnalyticsResource>()

const resources = computed(() => store.resources)
const folders = computed(() => store.folders)
const selectedResources = computed(() => store.selectedResources)
const selectedFolderId = computed(() => store.selectedFolderId)
const selectedScope = computed({
  get: () => store.selectedScope,
  set: (v) => store.selectScope(v),
})
const selectedType = computed({
  get: () => store.selectedType,
  set: (v) => store.selectType(v),
})
const sortBy = computed({
  get: () => store.sortBy,
  set: (v) => store.setSort(v),
})
const sortOrder = computed({
  get: () => store.sortOrder,
  set: (v) => { store.sortOrder = v },
})
const page = computed(() => store.page)
const pageSize = computed(() => store.pageSize)
const total = computed(() => store.total)
const totalPages = computed(() => store.totalPages)

const filteredResources = computed(() => {
  let result = store.filteredResources

  if (searchQuery.value) {
    const query = searchQuery.value.toLowerCase()
    result = result.filter(r =>
      r.name.toLowerCase().includes(query) ||
      (r.alias && r.alias.toLowerCase().includes(query))
    )
  }

  if (sortBy.value) {
    result = [...result].sort((a, b) => {
      const aVal = a[sortBy.value as keyof AnalyticsResource]
      const bVal = b[sortBy.value as keyof AnalyticsResource]

      if (aVal === null || aVal === undefined) return 1
      if (bVal === null || bVal === undefined) return -1

      let comparison = 0
      if (typeof aVal === 'string' && typeof bVal === 'string') {
        comparison = aVal.localeCompare(bVal)
      } else if (typeof aVal === 'number' && typeof bVal === 'number') {
        comparison = aVal - bVal
      }

      return sortOrder.value === 'asc' ? comparison : -comparison
    })
  }

  return result
})

function handleSearch(query: string) {
  // Search is handled by computed filtering
}

function handleClearSearch() {
  searchQuery.value = ''
}

function handleSelectResource(id: string, multiSelect: boolean) {
  store.selectResource(id, multiSelect)
}

function handleOpenResource(resource: AnalyticsResource) {
  toast.info(`打开资源: ${resource.name}`)
}

function handleDeleteResource(id: string) {
  if (confirm('确定删除该资源吗？')) {
    deleteResource(id)
  }
}

function handleBatchDelete() {
  if (selectedResources.value.length === 0) return
  if (confirm(`确定删除选中的 ${selectedResources.value.length} 个资源吗？`)) {
    batchDeleteResources(selectedResources.value)
  }
}

function handleEditResource(resource: AnalyticsResource) {
  editingResource.value = resource
  showEditResource.value = true
  showCreateResource.value = false
}

function handleCopyResource(resource: AnalyticsResource) {
  cloneResource(resource.id)
}

function selectFolder(id: string | null) {
  store.selectFolder(id)
}

function clearSelection() {
  store.clearSelection()
}

function handlePageChange(newPage: number) {
  store.setPage(newPage)
  loadData()
}

function handlePageSizeChange(newSize: number) {
  store.setPageSize(newSize)
  loadData()
}

function handlePrevPage() {
  store.prevPage()
  loadData()
}

function handleNextPage() {
  store.nextPage()
  loadData()
}

async function refresh() {
  try {
    await Promise.all([
      loadData(),
      store.loadFolders(),
      store.loadTags(),
    ])
    toast.success('刷新成功')
  } catch (error) {
    toast.error('刷新失败')
  }
}

async function loadData() {
  try {
    await store.loadResourcesPaginated({
      scope: selectedScope.value || undefined,
      resource_type: selectedType.value || undefined,
      folder_id: selectedFolderId.value || undefined,
      search: searchQuery.value || undefined,
    })
  } catch (error) {
    console.error('Failed to load data:', error)
  }
}

async function deleteResource(id: string) {
  try {
    await store.deleteResource(id)
    toast.success('删除成功')
  } catch (error) {
    toast.error('删除失败')
  }
}

async function batchDeleteResources(ids: string[]) {
  try {
    await store.batchDeleteResources(ids)
    toast.success(`成功删除 ${ids.length} 个资源`)
  } catch (error) {
    toast.error('批量删除失败')
  }
}

async function cloneResource(id: string) {
  try {
    const cloned = await store.cloneResource(id)
    toast.success(`已克隆: ${cloned.name}`)
  } catch (error) {
    toast.error('克隆失败')
  }
}

async function handleCreateResource(input: CreateResourceRequest) {
  try {
    await store.createResource(input)
    showCreateResource.value = false
    toast.success('创建成功')
  } catch (error) {
    toast.error('创建失败')
  }
}

function closeEditModal() {
  showEditResource.value = false
  showCreateResource.value = false
  editingResource.value = undefined
}

async function handleUpdateResource(id: string, input: CreateResourceRequest) {
  try {
    await store.updateResource(id, input)
    closeEditModal()
    toast.success('更新成功')
  } catch (error) {
    toast.error('更新失败')
  }
}

async function handleCreateFolder(input: CreateFolderRequest) {
  try {
    await store.createFolder(input)
    showCreateFolder.value = false
    toast.success('创建成功')
  } catch (error) {
    toast.error('创建失败')
  }
}

function handleDragStart(resources: AnalyticsResource[]) {
  console.log('Dragging resources:', resources.map(r => r.name))
}

function handleDragEnd() {
  console.log('Drag ended')
}

async function handleDropResource(folderId: string | null, resources: AnalyticsResource[]) {
  try {
    if (folderId) {
      for (const resource of resources) {
        await store.addResourceToFolder(resource.id, folderId)
      }
      toast.success(`已将 ${resources.length} 个资源添加到文件夹`)
    } else {
      for (const resource of resources) {
        const existingFolders = store.getResourceFolders(resource.id)
        for (const fId of existingFolders) {
          await store.removeResourceFromFolder(resource.id, fId)
        }
      }
      toast.success(`已将 ${resources.length} 个资源移出文件夹`)
    }
  } catch (error) {
    toast.error('移动资源失败')
  }
}



function handleSaveSettings(settings: AnalyticsResourceSettings) {
  store.saveSettings(settings)
  showSettings.value = false
  toast.success('设置已保存')
}

function handleClearCache() {
  store.clearCache()
  toast.success('缓存已清除')
}

function handleKeyDown(e: KeyboardEvent) {
  if (e.key === 'Delete' && selectedResources.value.length > 0) {
    e.preventDefault()
    handleBatchDelete()
    return
  }
  if ((e.ctrlKey || e.metaKey) && e.key === 'n') {
    e.preventDefault()
    showCreateResource.value = true
    return
  }
  if ((e.ctrlKey || e.metaKey) && e.key === 'e') {
    e.preventDefault()
    const selected = store.resources.find(r => selectedResources.value.includes(r.id))
    if (selected) {
      handleEditResource(selected)
    }
    return
  }
  if ((e.ctrlKey || e.metaKey) && e.key === 'd') {
    e.preventDefault()
    if (selectedResources.value.length > 0) {
      handleBatchDelete()
    }
    return
  }
  if ((e.ctrlKey || e.metaKey) && e.shiftKey && e.key === 'c') {
    e.preventDefault()
    const selected = store.resources.find(r => selectedResources.value.includes(r.id))
    if (selected) {
      handleCopyResource(selected)
    }
    return
  }
  if ((e.ctrlKey || e.metaKey) && e.key === 'f') {
    e.preventDefault()
    return
  }
  if ((e.ctrlKey || e.metaKey) && e.key === 'a') {
    e.preventDefault()
    return
  }
}

onMounted(async () => {
  try {
    await store.initStore()
    await loadData()
  } catch (error) {
    toast.error('初始化失败')
  }
  document.addEventListener('keydown', handleKeyDown)
})

onUnmounted(() => {
  document.removeEventListener('keydown', handleKeyDown)
})

// Watch for filter changes to reload data
watch([selectedScope, selectedType], () => {
  store.setPage(1)
  loadData()
})
</script>

<style scoped>
.analytics-resource-manager {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: var(--bg-primary);
  color: var(--text-primary);
  font-family: var(--font-sans);
}

.header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: var(--size-lg);
  border-bottom: 1px solid var(--border-color);
}

.header h2 {
  margin: 0;
  font-size: 16px;
  font-weight: 600;
  color: var(--text-primary);
}

.header-actions {
  display: flex;
  gap: var(--size-sm);
}

.footer {
  display: flex;
  gap: var(--size-md);
  padding: var(--size-lg);
  border-top: 1px solid var(--border-color);
}

.btn {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 6px 12px;
  border: 1px solid var(--border-color);
  border-radius: var(--radius-md);
  background: var(--bg-primary);
  color: var(--text-secondary);
  font-size: 13px;
  cursor: pointer;
  transition: all 0.2s;
  height: var(--height-btn);
}

.btn:hover {
  border-color: var(--primary-color);
  color: var(--primary-color);
}

.icon {
  font-size: 14px;
}
</style>
