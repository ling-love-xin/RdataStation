<template>
  <div ref="selectorRef" class="project-selector">
    <button class="project-btn" @click="toggleProjectMenu">
      <Database :size="14" class="project-icon" />
      <span class="project-name">{{ currentProject }}</span>
      <ChevronDown :size="12" class="project-chevron" :class="{ open: showProjectMenu }" />
    </button>

    <!-- 项目下拉菜单 -->
    <Transition name="dropdown">
      <div v-if="showProjectMenu" class="dropdown-panel project-dropdown">
        <div class="dropdown-section">
          <div class="dropdown-label">{{ t('workbench.recentProjects') }}</div>
          <div
            v-for="project in recentProjects"
            :key="project.id"
            class="dropdown-item"
            :class="{ active: project.id === currentProjectId }"
            @click="handleSwitchProject(project)"
          >
            <Database :size="14" />
            <span>{{ project.name }}</span>
          </div>
        </div>
        <div class="dropdown-divider" />
        <div class="dropdown-section">
          <div class="dropdown-item" @click="handleNewProject">
            <Plus :size="14" />
            <span>{{ t('workbench.newProject') }}</span>
          </div>
          <div class="dropdown-item" @click="handleOpenProject">
            <FolderOpen :size="14" />
            <span>{{ t('workbench.openProject') }}</span>
          </div>
        </div>
      </div>
    </Transition>
  </div>
</template>

<script setup lang="ts">
import { Database, ChevronDown, Plus, FolderOpen } from 'lucide-vue-next'
import { onMounted, onUnmounted, ref } from 'vue'
import { useI18n } from 'vue-i18n'

interface Project {
  id: string
  name: string
  path: string
}

interface Props {
  currentProject: string
  currentProjectId?: string | null
  recentProjects: Project[]
}

defineProps<Props>()

const emit = defineEmits<{
  'switch-project': [project: Project]
  'new-project': []
  'open-project': []
}>()

const { t } = useI18n()

const showProjectMenu = ref(false)
const selectorRef = ref<HTMLElement | null>(null)

function toggleProjectMenu() {
  showProjectMenu.value = !showProjectMenu.value
}

function handleClickOutside(event: MouseEvent) {
  const target = event.target as HTMLElement
  if (selectorRef.value && !selectorRef.value.contains(target)) {
    showProjectMenu.value = false
  }
}

function handleKeyDown(event: KeyboardEvent) {
  if (event.key === 'Escape' && showProjectMenu.value) {
    showProjectMenu.value = false
  }
}

function handleSwitchProject(project: Project) {
  emit('switch-project', project)
  showProjectMenu.value = false
}

function handleNewProject() {
  emit('new-project')
  showProjectMenu.value = false
}

function handleOpenProject() {
  emit('open-project')
  showProjectMenu.value = false
}

onMounted(() => {
  document.addEventListener('click', handleClickOutside)
  document.addEventListener('keydown', handleKeyDown)
})

onUnmounted(() => {
  document.removeEventListener('click', handleClickOutside)
  document.removeEventListener('keydown', handleKeyDown)
})
</script>

<style scoped>
@import './title-bar.css';

.project-dropdown {
  left: 0;
  min-width: 220px;
}
</style>
