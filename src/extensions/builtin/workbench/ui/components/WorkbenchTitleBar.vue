<template>
  <div class="title-bar" data-tauri-drag-region>
    <div class="title-bar-left">
      <!-- 汉堡菜单按钮 - 展开/隐藏菜单 -->
      <button
        class="hamburger-btn"
        :class="{ active: showMenuBar }"
        :title="t('workbench.menu')"
        @click="toggleMenuBar"
      >
        <Menu :size="16" />
      </button>

      <!-- 可展开的菜单栏 -->
      <Transition name="menu-slide">
        <div v-if="showMenuBar" class="menu-bar">
          <div class="menu-item">{{ t('workbench.fileMenu') }}</div>
          <div class="menu-item">{{ t('workbench.editMenu') }}</div>
          <div class="menu-item">{{ t('workbench.viewMenu') }}</div>
          <div class="menu-item">{{ t('workbench.connectionMenu') }}</div>
          <div class="menu-item">{{ t('workbench.runMenu') }}</div>
          <div class="menu-item">{{ t('workbench.toolsMenu') }}</div>
          <div class="menu-item">{{ t('workbench.helpMenu') }}</div>
        </div>
      </Transition>

      <!-- 项目名称区域 -->
      <div class="project-section">
        <button class="project-btn" @click="toggleProjectMenu">
          <Database :size="14" class="project-icon" />
          <span class="project-name">{{ currentProject }}</span>
          <ChevronDown :size="12" class="project-chevron" :class="{ open: showProjectMenu }" />
        </button>

        <!-- 项目下拉菜单 -->
        <div v-if="showProjectMenu" class="project-dropdown">
          <div class="dropdown-section">
            <div class="dropdown-label">{{ t('workbench.recentProjects') }}</div>
            <div
              v-for="project in recentProjects"
              :key="project.id"
              class="dropdown-item"
              :class="{ active: project.id === projectStore.currentProject?.id }"
              @click="switchProject(project)"
            >
              <Database :size="14" />
              <span>{{ project.name }}</span>
            </div>
          </div>
          <div class="dropdown-divider" />
          <div class="dropdown-section">
            <div class="dropdown-item" @click="newProject">
              <Plus :size="14" />
              <span>{{ t('workbench.newProject') }}</span>
            </div>
            <div class="dropdown-item" @click="openProject">
              <FolderOpen :size="14" />
              <span>{{ t('workbench.openProject') }}</span>
            </div>
          </div>
        </div>
      </div>
    </div>

    <div class="title-bar-center">
      <!-- 搜索/命令面板 -->
      <div class="command-center">
        <button class="command-btn" :title="t('workbench.search') + ' (' + t('workbench.searchShortcut') + ')'">
          <Search :size="14" />
          <span class="command-text">{{ t('workbench.search') }}</span>
          <span class="shortcut">{{ t('workbench.searchShortcut') }}</span>
        </button>
      </div>
    </div>

    <div class="title-bar-right">
      <!-- 自定义工具栏按钮 (三点菜单) -->
      <div class="custom-toolbar-section">
        <button
          class="more-btn"
          title="自定义工具栏"
          @click="toggleCustomToolbar"
        >
          <MoreHorizontal :size="16" />
        </button>

        <!-- 自定义工具栏下拉菜单 -->
        <div v-if="showCustomToolbar" class="custom-toolbar-dropdown">
          <div class="dropdown-header">{{ t('workbench.customizeToolbar') }}</div>
          <div class="dropdown-divider" />
          <div class="toolbar-options">
            <label
              v-for="tool in availableTools"
              :key="tool.id"
              class="toolbar-option"
            >
              <input
                v-model="tool.enabled"
                type="checkbox"
                @change="saveToolbarConfig"
              >
              <component :is="tool.icon" :size="14" />
              <span>{{ tool.name }}</span>
            </label>
          </div>
          <div class="dropdown-divider" />
          <div class="dropdown-item" @click="resetToolbar">
            <RotateCcw :size="14" />
            <span>{{ t('workbench.resetToDefault') }}</span>
          </div>
        </div>
      </div>

      <!-- 主题切换按钮 -->
      <button
        class="theme-toggle-btn"
        :title="uiStore.isDark ? t('workbench.switchToLight') : t('workbench.switchToDark')"
        @click="uiStore.toggleTheme"
      >
        <Sun v-if="uiStore.isDark" :size="14" />
        <Moon v-else :size="14" />
      </button>

      <!-- 布局控制按钮 -->
      <div class="layout-controls">
        <button class="layout-btn" :title="t('workbench.customizeLayout')" @click="toggleCustomizeLayout">
          <LayoutTemplate :size="14" />
        </button>
        <button class="layout-btn" :title="t('workbench.maximize')">
          <PanelTop :size="14" />
        </button>
      </div>

      <!-- 窗口控制按钮 -->
      <div class="window-controls">
        <button class="window-btn minimize" :title="t('workbench.minimize')" @click="$emit('minimize')">
          <Minus :size="14" />
        </button>
        <button class="window-btn maximize" :title="t('workbench.maximize')" @click="$emit('maximize')">
          <Square v-if="!isMaximized" :size="12" />
          <Copy v-else :size="12" />
        </button>
        <button class="window-btn close" :title="t('workbench.close')" @click="$emit('close')">
          <X :size="14" />
        </button>
      </div>
    </div>
  </div>

  <!-- 新建项目对话框 -->
  <Teleport to="body">
    <Transition name="modal">
      <div v-if="showNewProjectModal" class="modal-overlay" @click.self="closeNewProjectModal">
        <div class="modal-container">
          <header class="modal-header">
            <h2>{{ t('workbench.newProject') }}</h2>
            <button class="btn-close" @click="closeNewProjectModal">
              <X :size="20" />
            </button>
          </header>
          <div class="modal-body">
            <div class="form-section">
              <label class="form-label">
                {{ t('workbench.projectName') }}
                <span class="required">*</span>
              </label>
              <input
                v-model="newProjectName"
                type="text"
                class="form-input"
                :placeholder="t('workbench.projectName')"
                @keyup.enter="confirmCreateProject"
              />
            </div>
            <div class="form-section">
              <label class="form-label">
                {{ t('workbench.projectDescription') }}
              </label>
              <textarea
                v-model="newProjectDescription"
                class="form-input"
                :placeholder="t('workbench.projectDescription')"
                rows="3"
              />
            </div>
            <div class="form-section">
              <label class="form-label">
                {{ t('workbench.projectPath') }}
                <span class="required">*</span>
              </label>
              <div class="path-input-wrapper">
                <input
                  v-model="newProjectPath"
                  type="text"
                  class="form-input"
                  :placeholder="t('workbench.selectProjectPath')"
                  readonly
                />
                <button class="btn-browse" @click="browseProjectPath">
                  {{ t('workbench.browse') }}
                </button>
              </div>
            </div>
          </div>
          <footer class="modal-footer">
            <button class="btn-secondary" @click="closeNewProjectModal">{{ t('common.cancel') }}</button>
            <button
              class="btn-primary"
              :disabled="!canCreateProject || isCreating"
              @click="confirmCreateProject"
            >
              <span v-if="isCreating">{{ t('workbench.creating') }}</span>
              <span v-else>{{ t('workbench.create') }}</span>
            </button>
          </footer>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<script setup lang="ts">
import {
  Menu, Database, ChevronDown, Plus, FolderOpen, Search,
  MoreHorizontal, RotateCcw, Sun, Moon, LayoutTemplate, PanelTop,
  Minus, Square, X, Copy, Settings, History, BookOpen, Keyboard,
  Terminal, Zap
} from 'lucide-vue-next'
import { ref, computed, markRaw, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { useRouter } from 'vue-router'

import { useProjectStore } from '@/core/project/stores/project'
import { useLayoutStore } from '@/extensions/builtin/workbench/ui/stores/layout-store'
import { useUiStore } from '@/shared/stores/ui'

interface Props {
  isMaximized?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  isMaximized: false
})

const emit = defineEmits<{
  minimize: []
  maximize: []
  close: []
}>()

const { t } = useI18n()
const uiStore = useUiStore()
const projectStore = useProjectStore()
const layoutStore = useLayoutStore()
const router = useRouter()

// 菜单栏显示状态
const showMenuBar = ref(false)

// 项目菜单状态
const showProjectMenu = ref(false)
const currentProject = computed(() => projectStore.currentProject?.name || t('workbench.defaultProject'))

// 最近项目列表（从 ProjectStore 获取）
const recentProjects = computed(() => projectStore.recentProjects)

// 自定义工具栏状态
const showCustomToolbar = ref(false)

// 可用工具列表
const availableTools = ref([
  { id: 'settings', name: t('workbench.settings'), icon: markRaw(Settings), enabled: false, action: () => console.log('open settings') },
  { id: 'history', name: t('workbench.history'), icon: markRaw(History), enabled: false, action: () => console.log('open history') },
  { id: 'docs', name: t('workbench.docs'), icon: markRaw(BookOpen), enabled: false, action: () => console.log('open docs') },
  { id: 'shortcuts', name: t('workbench.shortcuts'), icon: markRaw(Keyboard), enabled: false, action: () => console.log('open shortcuts') },
  { id: 'terminal', name: t('workbench.terminal'), icon: markRaw(Terminal), enabled: false, action: () => console.log('open terminal') },
  { id: 'quick', name: t('workbench.quickActions'), icon: markRaw(Zap), enabled: false, action: () => console.log('quick actions') },
])

// 启用的工具
const enabledTools = computed(() => availableTools.value.filter(t => t.enabled))

// 切换菜单栏显示
const toggleMenuBar = () => {
  showMenuBar.value = !showMenuBar.value
  showProjectMenu.value = false
  showCustomToolbar.value = false
}

// 切换项目菜单
const toggleProjectMenu = () => {
  showProjectMenu.value = !showProjectMenu.value
  showCustomToolbar.value = false
}

// 切换项目
const switchProject = async (project: { id: string; name: string; path: string }) => {
  try {
    // 使用 ProjectStore 的 switchProject 方法（乐观更新 + 后台同步）
    await projectStore.switchProject(project.id)
    showProjectMenu.value = false
  } catch (error) {
    console.error('切换项目失败:', error)
  }
}

// 新建项目对话框状态
const showNewProjectModal = ref(false)
const newProjectName = ref('')
const newProjectDescription = ref('')
const newProjectPath = ref('')
const isCreating = ref(false)

const canCreateProject = computed(() => {
  return newProjectName.value.trim() && newProjectPath.value.trim()
})

// 新建项目
const newProject = () => {
  showProjectMenu.value = false
  showNewProjectModal.value = true
  newProjectName.value = ''
  newProjectDescription.value = ''
  newProjectPath.value = ''
}

// 关闭新建项目对话框
const closeNewProjectModal = () => {
  showNewProjectModal.value = false
}

// 浏览项目路径
const browseProjectPath = async () => {
  try {
    const { open } = await import('@tauri-apps/plugin-dialog')
    const selected = await open({
      directory: true,
      multiple: false,
      title: t('workbench.selectProjectPath')
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
  console.log('[WorkbenchTitleBar] confirmCreateProject 被调用')
  console.log('[WorkbenchTitleBar] 项目名称:', newProjectName.value)
  console.log('[WorkbenchTitleBar] 项目路径:', newProjectPath.value)
  console.log('[WorkbenchTitleBar] 项目描述:', newProjectDescription.value)
  console.log('[WorkbenchTitleBar] canCreateProject:', canCreateProject.value)
  
  if (!canCreateProject.value) {
    console.warn('[WorkbenchTitleBar] 项目创建条件不满足')
    return
  }

  isCreating.value = true

  try {
    console.log('[WorkbenchTitleBar] 开始调用 projectStore.createProject...')
    // 使用 ProjectStore 的 createProject 方法
    const project = await projectStore.createProject(
      newProjectName.value.trim(),
      newProjectPath.value.trim(),
      newProjectDescription.value.trim() || undefined
    )

    console.log('[WorkbenchTitleBar] createProject 返回结果:', project)

    if (project) {
      console.log('[WorkbenchTitleBar] 项目创建成功，关闭对话框')
      closeNewProjectModal()
    } else {
      console.error('[WorkbenchTitleBar] 创建项目失败：返回结果为 null')
    }
  } catch (error) {
    console.error('[WorkbenchTitleBar] 创建项目失败:', error)
  } finally {
    isCreating.value = false
  }
}

// 打开项目
const openProject = async () => {
  showProjectMenu.value = false
  try {
    const { open } = await import('@tauri-apps/plugin-dialog')
    const selected = await open({
      directory: true,
      multiple: false,
      title: t('workbench.selectProjectFolder')
    })

    if (selected && typeof selected === 'string') {
      // 使用 ProjectStore 的 openProject 方法
      await projectStore.openProject(selected)
    }
  } catch (error) {
    console.error('打开项目失败:', error)
  }
}

// 切换自定义工具栏菜单
const toggleCustomToolbar = () => {
  showCustomToolbar.value = !showCustomToolbar.value
  showProjectMenu.value = false
}

// 切换自定义布局对话框
const toggleCustomizeLayout = () => {
  window.dispatchEvent(new CustomEvent('open-customize-layout-dialog'))
}

// 保存工具栏配置
const saveToolbarConfig = () => {
  localStorage.setItem('customToolbar', JSON.stringify(availableTools.value.map(t => ({ id: t.id, enabled: t.enabled }))))
}

// 重置工具栏
const resetToolbar = () => {
  availableTools.value.forEach(t => t.enabled = false)
  saveToolbarConfig()
  showCustomToolbar.value = false
}

// 加载工具栏配置
const loadToolbarConfig = () => {
  const saved = localStorage.getItem('customToolbar')
  if (saved) {
    const config = JSON.parse(saved)
    availableTools.value.forEach(tool => {
      const savedTool = config.find((c: { id: string }) => c.id === tool.id)
      if (savedTool) {
        tool.enabled = savedTool.enabled
      }
    })
  }
}

// 暴露给父组件
defineExpose({
  enabledTools,
  loadToolbarConfig
})

// 生命周期
onMounted(async () => {
  // 加载最近项目列表（从 SQLite）
  await projectStore.loadRecentProjects()
  loadToolbarConfig()
})
</script>

<style scoped>
.title-bar {
  height: 36px;
  background: var(--bg-secondary);
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0;
  flex-shrink: 0;
  user-select: none;
  border-bottom: 1px solid var(--border-color);
}

.title-bar-left {
  display: flex;
  align-items: center;
  height: 100%;
  gap: 4px;
  padding-left: 4px;
}

/* 汉堡菜单按钮 */
.hamburger-btn {
  width: 32px;
  height: 28px;
  border: none;
  background: transparent;
  color: var(--text-secondary);
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 4px;
  transition: all 0.2s;
}

.hamburger-btn:hover,
.hamburger-btn.active {
  background: var(--bg-hover);
  color: var(--text-primary);
}

/* 菜单栏 */
.menu-bar {
  display: flex;
  align-items: center;
  height: 100%;
  gap: 2px;
  overflow: hidden;
}

.menu-slide-enter-active,
.menu-slide-leave-active {
  transition: all 0.2s ease;
}

.menu-slide-enter-from,
.menu-slide-leave-to {
  opacity: 0;
  width: 0;
  transform: translateX(-10px);
}

.menu-item {
  padding: 0 10px;
  font-size: 13px;
  color: var(--text-secondary);
  cursor: pointer;
  height: 28px;
  display: flex;
  align-items: center;
  transition: all 0.2s;
  border-radius: 4px;
  white-space: nowrap;
}

.menu-item:hover {
  background: var(--bg-hover);
  color: var(--text-primary);
}

/* 项目名称区域 */
.project-section {
  position: relative;
  margin-left: 8px;
}

.project-btn {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 4px 10px;
  background: var(--bg-tertiary);
  border: 1px solid var(--border-color);
  border-radius: 4px;
  color: var(--text-secondary);
  font-size: 13px;
  cursor: pointer;
  transition: all 0.2s;
  height: 28px;
}

.project-btn:hover {
  background: var(--bg-hover);
  border-color: var(--border-color);
}

.project-icon {
  color: var(--primary-color);
}

.project-name {
  max-width: 120px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.project-chevron {
  transition: transform 0.2s;
}

.project-chevron.open {
  transform: rotate(180deg);
}

/* 项目下拉菜单 */
.project-dropdown {
  position: absolute;
  top: calc(100% + 4px);
  left: 0;
  min-width: 200px;
  background: var(--bg-primary);
  border: 1px solid var(--border-color);
  border-radius: 6px;
  padding: 6px 0;
  z-index: 1000;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
}

.dropdown-section {
  padding: 4px 0;
}

.dropdown-label {
  padding: 4px 12px;
  font-size: 11px;
  color: var(--text-tertiary);
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.dropdown-item {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 8px 12px;
  cursor: pointer;
  font-size: 13px;
  color: var(--text-secondary);
  transition: background 0.15s;
}

.dropdown-item:hover {
  background: var(--primary-light);
  color: var(--text-primary);
}

.dropdown-item.active {
  background: var(--primary-light);
  color: var(--text-primary);
}

.dropdown-divider {
  height: 1px;
  background: var(--border-color);
  margin: 4px 0;
}

.title-bar-center {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
}

/* 搜索框 */

.command-btn {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 5px 14px;
  background: var(--bg-tertiary);
  border: 1px solid var(--border-color);
  border-radius: 4px;
  color: var(--text-secondary);
  font-size: 13px;
  cursor: pointer;
  transition: all 0.2s;
  min-width: 200px;
  max-width: 300px;
  justify-content: space-between;
}

.command-btn:hover {
  background: var(--bg-hover);
  border-color: var(--border-color);
}

.command-text {
  flex: 1;
  text-align: left;
}

.command-btn .shortcut {
  color: var(--text-tertiary);
  font-size: 11px;
  padding: 2px 6px;
  background: var(--bg-tertiary);
  border-radius: 3px;
}

.title-bar-right {
  display: flex;
  align-items: center;
  height: 100%;
  gap: 4px;
  padding-right: 4px;
}

/* 自定义工具栏区域 */
.custom-toolbar-section {
  position: relative;
}

.more-btn {
  width: 28px;
  height: 28px;
  border: none;
  background: transparent;
  color: var(--text-secondary);
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 4px;
  transition: all 0.2s;
}

.more-btn:hover {
  background: var(--bg-hover);
  color: var(--text-primary);
}

/* 自定义工具栏下拉菜单 */
.custom-toolbar-dropdown {
  position: absolute;
  top: calc(100% + 4px);
  right: 0;
  min-width: 180px;
  background: var(--bg-primary);
  border: 1px solid var(--border-color);
  border-radius: 6px;
  padding: 6px 0;
  z-index: 1000;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
}

.dropdown-header {
  padding: 6px 12px;
  font-size: 12px;
  font-weight: 600;
  color: var(--text-primary);
}

.toolbar-options {
  padding: 4px 0;
}

.toolbar-option {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 8px 12px;
  cursor: pointer;
  font-size: 13px;
  color: var(--text-secondary);
  transition: background 0.15s;
}

.toolbar-option:hover {
  background: var(--primary-light);
  color: var(--text-primary);
}

.toolbar-option input[type="checkbox"] {
  width: 14px;
  height: 14px;
  accent-color: var(--primary-color);
}

/* 主题切换按钮 */
.theme-toggle-btn {
  width: 28px;
  height: 28px;
  border: none;
  background: transparent;
  color: var(--text-secondary);
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 4px;
  transition: all 0.2s;
  margin-right: 4px;
}

.theme-toggle-btn:hover {
  background: var(--bg-hover);
  color: var(--text-primary);
}

/* 布局控制按钮 */
.layout-controls {
  display: flex;
  align-items: center;
  height: 100%;
  padding: 0 4px;
  border-right: 1px solid var(--border-color);
  border-left: 1px solid var(--border-color);
}

.layout-btn {
  width: 28px;
  height: 28px;
  border: none;
  background: transparent;
  color: var(--text-secondary);
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 4px;
  transition: all 0.2s;
}

.layout-btn:hover {
  background: var(--bg-hover);
  color: var(--text-primary);
}

.window-controls {
  display: flex;
  align-items: center;
  height: 100%;
}

.window-btn {
  width: 40px;
  height: 100%;
  border: none;
  background: transparent;
  color: var(--text-secondary);
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.2s;
}

.window-btn:hover {
  background: var(--bg-hover);
}

.window-btn.close:hover {
  background: var(--danger-color);
  color: var(--bg-primary);
}

/* 新建项目对话框样式 */
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
  background: var(--bg-secondary, #252526);
  border-radius: 16px;
  width: 100%;
  max-width: 480px;
  overflow: hidden;
  border: 1px solid var(--border-color, #3e3e42);
}

.modal-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 20px 24px;
  border-bottom: 1px solid var(--border-color, #3e3e42);
}

.modal-header h2 {
  font-size: 18px;
  font-weight: 600;
  color: var(--text-primary, #cccccc);
  margin: 0;
}

.btn-close {
  width: 32px;
  height: 32px;
  border-radius: 8px;
  border: none;
  background: var(--bg-tertiary, #2d2d30);
  color: var(--text-secondary, #858585);
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.2s;
}

.btn-close:hover {
  background: var(--bg-hover, #3c3c3c);
  color: var(--text-primary, #cccccc);
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
  color: var(--text-secondary, #858585);
  margin-bottom: 8px;
}

.form-label .required {
  color: var(--danger-color, #F53F3F);
  margin-left: 4px;
}

.form-input {
  width: 100%;
  padding: 10px 14px;
  border: 1px solid var(--border-color, #3e3e42);
  border-radius: 8px;
  font-size: 14px;
  color: var(--text-primary, #cccccc);
  background: var(--bg-primary, #1e1e1e);
  transition: all 0.2s;
  resize: vertical;
}

.form-input:focus {
  outline: none;
  border-color: var(--primary-color, #165DFF);
  box-shadow: 0 0 0 3px var(--primary-light, rgba(22, 93, 255, 0.1));
}

.form-input::placeholder {
  color: var(--text-tertiary, #666666);
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
  border: 1px solid var(--border-color, #3e3e42);
  border-radius: 8px;
  background: var(--bg-tertiary, #2d2d30);
  color: var(--text-secondary, #858585);
  font-size: 14px;
  cursor: pointer;
  transition: all 0.2s;
  white-space: nowrap;
}

.btn-browse:hover {
  background: var(--bg-hover, #3c3c3c);
  border-color: var(--border-color-hover, #4e4e52);
}

.modal-footer {
  display: flex;
  justify-content: flex-end;
  gap: 12px;
  padding: 16px 24px;
  border-top: 1px solid var(--border-color, #3e3e42);
  background: var(--bg-tertiary, #2d2d30);
}

.btn-secondary {
  padding: 10px 20px;
  border: 1px solid var(--border-color, #3e3e42);
  border-radius: 8px;
  background: var(--bg-secondary, #252526);
  color: var(--text-secondary, #858585);
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s;
}

.btn-secondary:hover {
  background: var(--bg-hover, #3c3c3c);
  border-color: var(--border-color-hover, #4e4e52);
}

.btn-primary {
  padding: 10px 20px;
  border: none;
  border-radius: 8px;
  background: var(--primary-color, #165DFF);
  color: white;
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s;
}

.btn-primary:hover:not(:disabled) {
  background: var(--primary-dark, #0E42D2);
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
