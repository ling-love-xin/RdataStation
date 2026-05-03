<template>
  <div class="project-select-view" :class="{ dark: uiStore.isDark }">
    <div class="main-content">
      <!-- 左侧介绍区 -->
      <div class="left-panel">
        <div class="brand">
          <div class="logo">
            <Database :size="48" />
          </div>
          <h1 class="brand-title">RdataStation</h1>
          <p class="brand-subtitle">新一代数据库管理工具</p>
        </div>

        <div class="features">
          <div class="feature-item">
            <div class="feature-icon">
              <Zap :size="24" />
            </div>
            <div class="feature-text">
              <h3>高性能</h3>
              <p>基于 DuckDB 的本地加速引擎</p>
            </div>
          </div>
          <div class="feature-item">
            <div class="feature-icon">
              <Shield :size="24" />
            </div>
            <div class="feature-text">
              <h3>安全可靠</h3>
              <p>本地存储，数据完全掌控</p>
            </div>
          </div>
          <div class="feature-item">
            <div class="feature-icon">
              <Layers :size="24" />
            </div>
            <div class="feature-text">
              <h3>多数据库支持</h3>
              <p>MySQL、PostgreSQL、SQLite、DuckDB</p>
            </div>
          </div>
        </div>
      </div>

      <!-- 右侧操作区 -->
      <div class="right-panel">
        <div class="action-cards">
          <div class="action-card primary" @click="createNewProject">
            <div class="action-icon">
              <FolderPlus :size="32" />
            </div>
            <div class="action-content">
              <h2>新建项目</h2>
              <p>创建一个新的工作空间</p>
            </div>
            <ChevronRight :size="20" />
          </div>

          <div class="action-card" @click="openExistingProject">
            <div class="action-icon">
              <FolderOpen :size="32" />
            </div>
            <div class="action-content">
              <h2>打开项目</h2>
              <p>浏览并打开已有项目</p>
            </div>
            <ChevronRight :size="20" />
          </div>
        </div>

        <!-- 最近项目 -->
        <div class="recent-section">
          <div class="recent-header">
            <h3>最近打开</h3>
            <button v-if="recentProjects.length > 0" class="clear-btn" @click="clearRecentProjects">
              清除历史
            </button>
          </div>

          <div v-if="recentProjects.length === 0" class="recent-empty">
            <FolderX :size="32" />
            <p>暂无最近项目</p>
          </div>

          <div v-else class="recent-list">
            <div
              v-for="project in recentProjects"
              :key="project.id"
              class="recent-item"
              @click="openProject(project)"
            >
              <div class="recent-icon">
                <Database :size="18" />
              </div>
              <div class="recent-info">
                <span class="recent-name">{{ project.name }}</span>
                <span class="recent-path">{{ project.path }}</span>
              </div>
              <span class="recent-time">{{ formatTime(project.lastOpened) }}</span>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- 新建项目对话框 -->
    <Teleport to="body">
      <Transition name="modal">
        <div v-if="showNewProjectModal" class="modal-overlay" @click.self="closeModal">
          <div class="modal-container">
            <header class="modal-header">
              <h2>新建项目</h2>
              <button class="btn-close" @click="closeModal">
                <X :size="20" />
              </button>
            </header>
            <div class="modal-body">
              <div class="form-section">
                <label class="form-label">
                  项目名称
                  <span class="required">*</span>
                </label>
                <input
                  v-model="newProjectName"
                  type="text"
                  class="form-input"
                  placeholder="输入项目名称"
                  @keyup.enter="confirmCreateProject"
                />
              </div>
              <div class="form-section">
                <label class="form-label">
                  项目描述
                </label>
                <textarea
                  v-model="newProjectDescription"
                  class="form-input"
                  placeholder="输入项目描述（可选）"
                  rows="3"
                />
              </div>
              <div class="form-section">
                <label class="form-label">
                  项目路径
                  <span class="required">*</span>
                </label>
                <div class="path-input-wrapper">
                  <input
                    v-model="newProjectPath"
                    type="text"
                    class="form-input"
                    placeholder="选择项目保存路径"
                    readonly
                  />
                  <button class="btn-browse" @click="browseProjectPath">
                    浏览
                  </button>
                </div>
              </div>
            </div>
            <footer class="modal-footer">
              <button class="btn-secondary" @click="closeModal">取消</button>
              <button
                class="btn-primary"
                :disabled="!canCreateProject || isCreating"
                @click="confirmCreateProject"
              >
                <span v-if="isCreating">创建中...</span>
                <span v-else>创建</span>
              </button>
            </footer>
          </div>
        </div>
      </Transition>
    </Teleport>
  </div>
</template>

<script setup lang="ts">
import {
  Database,
  FolderPlus,
  FolderOpen,
  FolderX,
  ChevronRight,
  Zap,
  Shield,
  Layers,
  X
} from 'lucide-vue-next'
import { ref, computed, onMounted } from 'vue'
import { useRouter } from 'vue-router'

import { useProjectStore } from '@/core/project/stores/project'
import { useUiStore } from '@/shared/stores/ui'

interface Project {
  id: string
  name: string
  description?: string
  path: string
  lastOpened: number
}

const router = useRouter()
const uiStore = useUiStore()
const projectStore = useProjectStore()

// 新建项目对话框状态
const showNewProjectModal = ref(false)
const newProjectName = ref('')
const newProjectDescription = ref('')
const newProjectPath = ref('')
const isCreating = ref(false)

// 计算属性
const canCreateProject = computed(() => {
  return newProjectName.value.trim() && newProjectPath.value.trim()
})

// 从 ProjectStore 获取最近项目列表
const recentProjects = computed(() => {
  return projectStore.recentProjects.map(p => ({
    id: p.id,
    name: p.name,
    description: p.description,
    path: p.path,
    lastOpened: new Date(p.updatedAt).getTime()
  }))
})

// 格式化时间
const formatTime = (timestamp: number): string => {
  const date = new Date(timestamp)
  const now = new Date()
  const diff = now.getTime() - date.getTime()
  const days = Math.floor(diff / (1000 * 60 * 60 * 24))

  if (days === 0) {
    const hours = Math.floor(diff / (1000 * 60 * 60))
    if (hours === 0) {
      const minutes = Math.floor(diff / (1000 * 60))
      return minutes === 0 ? '刚刚' : `${minutes} 分钟前`
    }
    return `${hours} 小时前`
  } else if (days === 1) {
    return '昨天'
  } else if (days < 7) {
    return `${days} 天前`
  } else {
    return date.toLocaleDateString('zh-CN')
  }
}

// 创建新项目
const createNewProject = () => {
  showNewProjectModal.value = true
  newProjectName.value = ''
  newProjectDescription.value = ''
  newProjectPath.value = ''
}

// 打开已有项目
const openExistingProject = async () => {
  try {
    const { open } = await import('@tauri-apps/plugin-dialog')
    const selected = await open({
      directory: true,
      multiple: false,
      title: '选择项目文件夹'
    })

    if (selected && typeof selected === 'string') {
      const projectName = selected.split(/[/\\]/).pop() || '未命名项目'
      const project: Project = {
        id: Date.now().toString(),
        name: projectName,
        path: selected,
        lastOpened: Date.now()
      }

      addToRecentProjects(project)
      await enterWorkbench(project)
    }
  } catch (error) {
    console.error('打开项目失败:', error)
  }
}

// 打开项目
const openProject = async (project: Project) => {
  project.lastOpened = Date.now()
  addToRecentProjects(project)
  await enterWorkbench(project)
}

// 进入工作台
const enterWorkbench = async (project: Project) => {
  console.log('enterWorkbench 被调用:', project)
  
  // 保存到 localStorage
  localStorage.setItem('currentProject', JSON.stringify(project))

  // 同时更新 project store
  const projectStore = useProjectStore()
  await projectStore.setCurrentProject({
    id: project.id,
    name: project.name,
    description: project.description,
    path: project.path,
    createdAt: new Date(project.lastOpened).toISOString(),
    updatedAt: new Date().toISOString()
  })

  console.log('准备跳转到工作台')
  router.push('/workbench')
}

// 添加到最近项目
const addToRecentProjects = (project: Project) => {
  // 最近项目列表由 ProjectStore 管理，通过 loadRecentProjects 从后端加载
  // 这里只需要刷新列表即可
  projectStore.loadRecentProjects(true)
}

// 清除最近项目
const clearRecentProjects = () => {
  // 清除操作需要调用后端 API（暂未实现）
  // 暂时只刷新列表
  projectStore.loadRecentProjects(true)
}

// 关闭对话框
const closeModal = () => {
  showNewProjectModal.value = false
}

// 浏览项目路径
const browseProjectPath = async () => {
  try {
    const { open } = await import('@tauri-apps/plugin-dialog')
    const selected = await open({
      directory: true,
      multiple: false,
      title: '选择项目保存位置'
    })

    if (selected && typeof selected === 'string') {
      newProjectPath.value = selected
    }
  } catch (error) {
    console.error('选择路径失败:', error)
  }
}

// 确认创建项目
const confirmCreateProject = async () => {
  if (!canCreateProject.value) {
    return
  }

  isCreating.value = true

  try {
    const projectStore = useProjectStore()
    const result = await projectStore.createProject(
      newProjectName.value.trim(),
      newProjectPath.value.trim(),
      newProjectDescription.value.trim() || undefined
    )

    console.log('createProject 返回结果:', result)

    if (result) {
      // 处理 path 字段，后端返回的是 { type: 'Local', path: '...' } 格式
      const projectPath = typeof result.path === 'string' 
        ? result.path 
        : (result.path?.path || newProjectPath.value.trim())

      const project: Project = {
        id: result.id,
        name: result.name,
        description: result.description,
        path: projectPath,
        lastOpened: Date.now()
      }

      console.log('准备进入工作台:', project)
      addToRecentProjects(project)
      closeModal()
      await enterWorkbench(project)
    } else {
      console.error('创建项目失败：返回结果为 null')
    }
  } catch (error) {
    console.error('创建项目失败:', error)
  } finally {
    isCreating.value = false
  }
}

// 生命周期
onMounted(async () => {
  await projectStore.loadRecentProjects()
})
</script>

<style scoped>
.project-select-view {
  width: 100vw;
  height: 100vh;
  background: var(--bg-primary, #f8fafc);
  display: flex;
  font-family: var(--font-sans);
}

.project-select-view.dark {
  background: var(--bg-primary, #0f172a);
}

.main-content {
  flex: 1;
  display: flex;
  max-width: 1400px;
  margin: 0 auto;
  padding: 60px;
  gap: 80px;
}

/* 左侧介绍区 */
.left-panel {
  flex: 1;
  display: flex;
  flex-direction: column;
  justify-content: center;
  padding-right: 40px;
}

.brand {
  margin-bottom: 60px;
}

.logo {
  width: 80px;
  height: 80px;
  border-radius: 20px;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  display: flex;
  align-items: center;
  justify-content: center;
  color: white;
  margin-bottom: 24px;
}

.brand-title {
  font-size: 42px;
  font-weight: 700;
  color: var(--text-primary, #1e293b);
  margin: 0 0 8px 0;
}

.brand-subtitle {
  font-size: 18px;
  color: var(--text-secondary, #64748b);
  margin: 0;
}

.features {
  display: flex;
  flex-direction: column;
  gap: 24px;
}

.feature-item {
  display: flex;
  align-items: flex-start;
  gap: 16px;
}

.feature-icon {
  width: 48px;
  height: 48px;
  border-radius: 12px;
  background: var(--bg-secondary, white);
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--primary-color, #667eea);
  flex-shrink: 0;
}

.feature-text h3 {
  font-size: 16px;
  font-weight: 600;
  color: var(--text-primary, #1e293b);
  margin: 0 0 4px 0;
}

.feature-text p {
  font-size: 14px;
  color: var(--text-secondary, #64748b);
  margin: 0;
}

/* 右侧操作区 */
.right-panel {
  flex: 1;
  display: flex;
  flex-direction: column;
  justify-content: center;
  max-width: 480px;
}

.action-cards {
  display: flex;
  flex-direction: column;
  gap: 16px;
  margin-bottom: 40px;
}

.action-card {
  display: flex;
  align-items: center;
  gap: 20px;
  padding: 24px;
  background: var(--bg-secondary, white);
  border: 2px solid var(--border-color, #e2e8f0);
  border-radius: 16px;
  cursor: pointer;
  transition: all 0.3s;
  flex-direction: row;
}

.action-card:hover {
  border-color: var(--primary-color, #667eea);
  box-shadow: 0 4px 20px rgba(102, 126, 234, 0.15);
  transform: translateY(-2px);
}

.action-card.primary {
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  border-color: transparent;
  color: white;
}

.action-card.primary:hover {
  box-shadow: 0 8px 30px rgba(102, 126, 234, 0.3);
}

.action-icon {
  width: 56px;
  height: 56px;
  border-radius: 14px;
  background: var(--bg-tertiary, #f1f5f9);
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--primary-color, #667eea);
  flex-shrink: 0;
}

.action-card.primary .action-icon {
  background: rgba(255, 255, 255, 0.2);
  color: white;
}

.action-content {
  flex: 1;
  text-align: left;
}

.action-content h2 {
  font-size: 18px;
  font-weight: 600;
  margin: 0 0 4px 0;
  color: var(--text-primary, #1e293b);
  writing-mode: horizontal-tb;
  text-orientation: mixed;
  white-space: nowrap;
}

.action-card.primary .action-content h2 {
  color: white;
}

.action-content p {
  font-size: 14px;
  margin: 0;
  color: var(--text-secondary, #64748b);
  writing-mode: horizontal-tb;
  text-orientation: mixed;
  white-space: nowrap;
}

.action-card.primary .action-content p {
  color: rgba(255, 255, 255, 0.8);
}

/* 最近项目 */
.recent-section {
  background: var(--bg-secondary, white);
  border-radius: 16px;
  padding: 24px;
}

.recent-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 16px;
}

.recent-header h3 {
  font-size: 14px;
  font-weight: 600;
  color: var(--text-secondary, #64748b);
  text-transform: uppercase;
  letter-spacing: 0.5px;
  margin: 0;
}

.clear-btn {
  padding: 4px 12px;
  border-radius: 6px;
  border: none;
  background: transparent;
  color: var(--text-tertiary, #94a3b8);
  font-size: 13px;
  cursor: pointer;
  transition: all 0.2s;
}

.clear-btn:hover {
  background: var(--bg-hover, #f1f5f9);
  color: var(--text-secondary, #64748b);
}

.recent-empty {
  text-align: center;
  padding: 32px;
  color: var(--text-tertiary, #94a3b8);
}

.recent-empty svg {
  margin-bottom: 12px;
  opacity: 0.5;
}

.recent-empty p {
  margin: 0;
  font-size: 14px;
}

.recent-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.recent-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px;
  border-radius: 10px;
  cursor: pointer;
  transition: all 0.2s;
}

.recent-item:hover {
  background: var(--bg-hover, #f8fafc);
}

.recent-icon {
  width: 36px;
  height: 36px;
  border-radius: 8px;
  background: var(--bg-tertiary, #f1f5f9);
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--primary-color, #667eea);
  flex-shrink: 0;
}

.recent-info {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.recent-name {
  font-size: 14px;
  font-weight: 500;
  color: var(--text-primary, #1e293b);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.recent-path {
  font-size: 12px;
  color: var(--text-tertiary, #94a3b8);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.recent-time {
  font-size: 12px;
  color: var(--text-tertiary, #94a3b8);
  flex-shrink: 0;
}

/* 对话框 - 使用主题变量 */
.modal-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.6);
  backdrop-filter: blur(4px);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
}

.modal-container {
  background: var(--bg-secondary, #ffffff);
  border-radius: 16px;
  width: 100%;
  max-width: 480px;
  overflow: hidden;
  border: 1px solid var(--border-color, #e0e0e0);
}

.modal-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 20px 24px;
  border-bottom: 1px solid var(--border-color, #e0e0e0);
}

.modal-header h2 {
  font-size: 18px;
  font-weight: 600;
  color: var(--text-primary, #1e293b);
  margin: 0;
}

.btn-close {
  width: 32px;
  height: 32px;
  border-radius: 8px;
  border: none;
  background: var(--bg-tertiary, #f1f5f9);
  color: var(--text-secondary, #64748b);
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.2s;
}

.btn-close:hover {
  background: var(--bg-hover, #e2e8f0);
  color: var(--text-primary, #475569);
}

.modal-body {
  padding: 24px;
}

.form-section {
  margin-bottom: 20px;
}

.form-section:last-child {
  margin-bottom: 0;
}

.form-label {
  display: block;
  font-size: 14px;
  font-weight: 500;
  color: var(--text-secondary, #374151);
  margin-bottom: 8px;
}

.form-label .required {
  color: var(--danger-color, #ef4444);
  margin-left: 4px;
}

.form-input {
  width: 100%;
  padding: 10px 14px;
  border: 1px solid var(--border-color, #d1d5db);
  border-radius: 8px;
  font-size: 14px;
  color: var(--text-primary, #1e293b);
  background: var(--bg-primary, #ffffff);
  transition: all 0.2s;
  resize: vertical;
}

.form-input:focus {
  outline: none;
  border-color: var(--primary-color, #667eea);
  box-shadow: 0 0 0 3px var(--primary-light, rgba(102, 126, 234, 0.1));
}

.form-input::placeholder {
  color: var(--text-tertiary, #94a3b8);
}

.path-input-wrapper {
  display: flex;
  gap: 8px;
}

.path-input-wrapper .form-input {
  flex: 1;
}

.btn-browse {
  padding: 10px 16px;
  border: 1px solid var(--border-color, #d1d5db);
  border-radius: 8px;
  background: var(--bg-tertiary, #f1f5f9);
  color: var(--text-secondary, #374151);
  font-size: 14px;
  cursor: pointer;
  transition: all 0.2s;
  white-space: nowrap;
}

.btn-browse:hover {
  background: var(--bg-hover, #e2e8f0);
  border-color: var(--border-color, #9ca3af);
}

.modal-footer {
  display: flex;
  justify-content: flex-end;
  gap: 12px;
  padding: 16px 24px;
  border-top: 1px solid var(--border-color, #e0e0e0);
  background: var(--bg-tertiary, #f8fafc);
}

.btn-secondary {
  padding: 10px 20px;
  border: 1px solid var(--border-color, #d1d5db);
  border-radius: 8px;
  background: var(--bg-secondary, #ffffff);
  color: var(--text-secondary, #374151);
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s;
}

.btn-secondary:hover {
  background: var(--bg-hover, #f9fafb);
  border-color: var(--border-color, #9ca3af);
}

.btn-primary {
  padding: 10px 20px;
  border: none;
  border-radius: 8px;
  background: var(--primary-color, #667eea);
  color: white;
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s;
}

.btn-primary:hover:not(:disabled) {
  background: var(--primary-dark, #5a67d8);
}

.btn-primary:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

/* 过渡动画 */
.modal-enter-active,
.modal-leave-active {
  transition: all 0.3s ease;
}

.modal-enter-from,
.modal-leave-to {
  opacity: 0;
}

.modal-enter-from .modal-container,
.modal-leave-to .modal-container {
  transform: scale(0.95);
}
</style>
