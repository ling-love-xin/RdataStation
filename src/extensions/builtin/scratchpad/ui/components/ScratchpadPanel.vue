<template>
  <div
class="scratchpad-panel"
    :class="{ 'dragover-active': isDragOver }"
    :data-drop-hint="t('scratchpad.dragToImport')"
    @click="handleGlobalClick"
    @dragover.prevent="handleDragOver"
    @dragleave="handleDragLeave"
    @drop.prevent="handleDrop"
  >
    <div class="explorer-titlebar" @dblclick="handleCollapseAll">
      <span class="explorer-titlebar-label">{{ t('scratchpad.title') }}</span>
      <span class="explorer-titlebar-actions">
        <button class="toolbar-btn" :disabled="isLoading" :title="t('scratchpad.import')" @click="handleImportFile">
          <Upload :size="14" />
        </button>
        <button class="toolbar-btn" :disabled="isLoading" :title="t('scratchpad.reference')" @click="handleAddReference">
          <FolderSymlink :size="14" />
        </button>
        <button class="toolbar-btn" :class="{ 'toolbar-btn-active': showExplorerFilter }" :title="t('scratchpad.search')" @click="toggleExplorerFilter">
          <Search :size="14" />
        </button>
        <NDropdown trigger="click" :options="moreMenuOptions" @select="handleMoreMenuSelect">
          <button class="toolbar-btn" :title="t('scratchpad.more')">
            <MoreVertical :size="14" />
          </button>
        </NDropdown>
      </span>
    </div>

    <div class="scratchpad-explorer-body">
      <div v-if="isLoading" class="loading-state">
        <NSpin size="small" />
      </div>

      <div v-else-if="notInitialized" class="empty-state">
        <div class="empty-icon-wrapper">
          <FolderOpen :size="32" />
        </div>
        <div class="empty-title">{{ t('scratchpad.noProjectTitle') }}</div>
        <div class="empty-hint">{{ t('scratchpad.noProjectHint') }}</div>
      </div>

      <div v-else-if="error" class="error-state">
        <span class="error-text">{{ error }}</span>
        <NButton size="tiny" @click="loadFiles">{{ t('scratchpad.retry') }}</NButton>
      </div>

      <div v-else-if="showExplorerFilter" class="search-panel">
        <div class="search-panel-header">
          <div class="search-input-wrapper">
            <NInput v-model:value="searchQuery" size="small" :placeholder="t('scratchpad.search')" class="search-panel-input" />
            <div class="search-toggle-buttons">
              <button class="toggle-btn" :class="{ 'toggle-btn-active': caseSensitive }" :title="t('scratchpad.matchCase')" @click="caseSensitive = !caseSensitive">Aa</button>
              <button class="toggle-btn" :class="{ 'toggle-btn-active': wholeWord }" :title="t('scratchpad.matchWholeWord')" @click="wholeWord = !wholeWord"><WholeWord :size="13" /></button>
              <button class="toggle-btn" :class="{ 'toggle-btn-active': isRegex }" :title="t('scratchpad.useRegex')" @click="toggleRegex">.*</button>
            </div>
          </div>
          <button class="toolbar-btn" :title="t('scratchpad.closeSearch')" @click="toggleExplorerFilter"><X :size="14" /></button>
        </div>

        <div v-if="regexError" class="search-notice search-notice-error">
          <NIcon size="14"><Info /></NIcon>
          <span>{{ regexError }}</span>
        </div>

        <div class="search-filter-toggle" @click="showSearchFilters = !showSearchFilters">
          <NIcon size="12"><component :is="showSearchFilters ? ChevronDown : ChevronRight" /></NIcon>
          <span>{{ t('scratchpad.filesToInclude') }}</span>
        </div>
        <div v-show="showSearchFilters" class="search-filters">
          <NInput v-model:value="searchInclude" size="small" :placeholder="t('scratchpad.includePattern')" class="search-filter-input" />
          <NInput v-model:value="searchExclude" size="small" :placeholder="t('scratchpad.excludePattern')" class="search-filter-input" />
        </div>

        <div v-if="showReplaceBar" class="search-replace-bar">
          <NInput v-model:value="replaceWith" size="small" :placeholder="t('scratchpad.replaceWith')" class="search-replace-input" />
          <NButton size="small" :loading="replaceInProgress" @click="handleReplaceAll">{{ t('scratchpad.replaceAll') }}</NButton>
        </div>

        <div v-if="searchQuery.trim()" class="search-results">
          <div v-if="searchResult && searchResult.matches.length > 0">
            <div v-for="[file, matches] in searchResultsByFile" :key="file" class="search-result-group">
              <div class="search-result-file-header" @click="toggleSearchFileExpand(file)">
                <NIcon size="12"><component :is="expandedSearchFiles.has(file) ? ChevronDown : ChevronRight" /></NIcon>
                <NIcon size="14"><FileText /></NIcon>
                <span class="search-result-file-name">{{ file }}</span>
                <span class="search-result-match-count">{{ matches.length }}</span>
              </div>
              <div v-show="expandedSearchFiles.has(file)" class="search-result-matches">
                <div
                  v-for="match in matches"
                  :key="`${match.file}:${match.line_number}`"
                  class="search-result-line"
                  @click="handleSearchMatchClick(match)"
                >
                  <span class="search-result-line-number">{{ match.line_number }}</span>
                  <span class="search-result-line-content" v-html="highlightSearchMatch(match)"></span>
                </div>
              </div>
            </div>
          </div>
          <div v-else-if="searchQuery.trim()" class="search-no-results">
            <span>{{ t('scratchpad.noResults') }}</span>
          </div>
          <div v-if="resultsSummaryText" class="search-results-footer">
            {{ resultsSummaryText }}
          </div>
        </div>
        <div v-else class="search-no-results">
          <span>{{ t('scratchpad.typeToSearch') }}</span>
        </div>
      </div>

      <div v-else ref="treeContainerRef" class="tree-container" @contextmenu.prevent="showBlankMenu($event)">
        <div v-if="recentFileEntries.length > 0" class="pane-section">
          <div class="pane-section-header" @click="showRecent = !showRecent" @contextmenu.stop>
            <NIcon size="14">
              <component :is="showRecent ? ChevronDown : ChevronRight" />
            </NIcon>
            <NIcon size="14"><History /></NIcon>
            <span class="pane-section-title">{{ t('scratchpad.recentFiles') }}</span>
            <span class="pane-section-actions">
              <button class="toolbar-btn" :title="t('scratchpad.newFile')" @click.stop="handleCreateFile"><FilePlus :size="14" /></button>
              <button class="toolbar-btn" :title="t('scratchpad.saveAll')" @click.stop="handleSaveAll"><FileText :size="14" /></button>
              <button class="toolbar-btn" :title="t('scratchpad.closeRecent')" @click.stop="handleCloseRecent"><X :size="14" /></button>
            </span>
          </div>
          <div v-show="showRecent" class="pane-section-body">
            <div v-for="entry in recentFileEntries" :key="entry.path" class="recent-entry" @click="handleOpen(entry)">
              <NIcon size="14"><FileText /></NIcon>
              <span class="recent-name">{{ entry.name }}</span>
            </div>
          </div>
        </div>

        <div class="pane-section pane-section-main" :style="getSectionFlexStyle('local')">
          <div class="pane-section-header" @click="toggleGroup('local')" @contextmenu.stop>
            <NIcon size="14">
              <component :is="groupExpanded.local ? ChevronDown : ChevronRight" />
            </NIcon>
            <NIcon size="14"><Folder /></NIcon>
            <span class="pane-section-title">{{ t('scratchpad.localDrafts') }}</span>
            <span class="pane-section-actions">
              <button class="toolbar-btn" :title="t('scratchpad.newFile')" @click.stop="handleCreateFile"><FilePlus :size="14" /></button>
              <button class="toolbar-btn" :title="t('scratchpad.newFolder')" @click.stop="handleCreateFolder"><FolderPlus :size="14" /></button>
              <button class="toolbar-btn" :title="t('scratchpad.import')" @click.stop="handleImportFile"><Upload :size="14" /></button>
              <button class="toolbar-btn" :title="t('scratchpad.collapseAll')" @click.stop="handleCollapseAll"><ChevronsUpDown :size="14" /></button>
            </span>
          </div>
        <div v-show="groupExpanded.local" class="pane-section-body">
          <div v-if="isInlineCreateActive && inlineCreateParentPath === null" class="inline-tree-entry">
            <NIcon size="14" class="node-icon">
              <component :is="inlineCreateIsFolder ? Folder : FileText" />
            </NIcon>
            <input
              ref="rootInlineInputRef"
              v-model="rootInlineCreateName"
              class="rename-input"
              :placeholder="inlineCreateIsFolder ? t('scratchpad.newFolder') : t('scratchpad.newFile')"
              @keyup.enter="commitRootInlineCreate"
              @keyup.escape="cancelRootInlineCreate"
              @blur="commitRootInlineCreate"
              @click.stop
            />
          </div>
          <div v-if="filteredLocalEntries.length === 0 && inlineCreateParentPath !== null" class="empty-state">
            <div class="empty-icon-wrapper">
              <NIcon size="32"><FolderOpen /></NIcon>
            </div>
            <span class="empty-title">{{ t('scratchpad.emptyScratchpad') }}</span>
            <span class="empty-hint">{{ t('scratchpad.emptyScratchpadHint') }}</span>
            <div class="empty-actions">
              <NButton size="small" type="primary" @click="handleCreateFile">
                <template #icon>
                  <NIcon size="14"><FilePlus /></NIcon>
                </template>
                {{ t('scratchpad.newFile') }}
              </NButton>
              <NButton size="small" @click="handleImportFile">
                <template #icon>
                  <NIcon size="14"><Upload /></NIcon>
                </template>
                {{ t('scratchpad.import') }}
              </NButton>
            </div>
          </div>
          <div
            v-if="useVirtualScrollEnabled"
            class="virtual-scroll-viewport"
            :style="{ height: virtualScrollTotalHeight + 'px' }"
          >
            <div class="virtual-scroll-spacer" :style="{ height: virtualScrollPaddingTop + 'px' }" />
            <ScratchpadTreeNode
              v-for="item in visibleTreeEntries"
              :key="item.entry.path"
              :entry="item.entry"
              :depth="item.depth"
              :expanded-keys="expandedKeys"
              :selected-key="selectedKey"
              :selected-keys="selectedKeys"
              :renaming-key="renamingKey"
              :inline-create-parent-path="inlineCreateParentPath"
              :inline-create-is-folder="inlineCreateIsFolder"
              :dirty-files="dirtyFiles"
              @select="handleSelect"
              @open="handleOpen"
              @contextmenu="showEntryMenu"
              @toggle-expand="handleToggleExpand"
              @start-rename="startRename"
              @finish-rename="finishRename"
              @cancel-rename="cancelRename"
              @drag-start="handleTreeNodeDragStart"
              @create-inline="confirmInlineCreate"
            />
          </div>
          <template v-else>
            <ScratchpadTreeNode
              v-for="item in flattenedTree"
              :key="item.entry.path"
              :entry="item.entry"
              :depth="item.depth"
              :expanded-keys="expandedKeys"
              :selected-key="selectedKey"
              :selected-keys="selectedKeys"
              :renaming-key="renamingKey"
              :inline-create-parent-path="inlineCreateParentPath"
              :inline-create-is-folder="inlineCreateIsFolder"
              :dirty-files="dirtyFiles"
              @select="handleSelect"
              @open="handleOpen"
              @contextmenu="showEntryMenu"
              @toggle-expand="handleToggleExpand"
              @start-rename="startRename"
              @finish-rename="finishRename"
              @cancel-rename="cancelRename"
              @drag-start="handleTreeNodeDragStart"
              @create-inline="confirmInlineCreate"
            />
          </template>
        </div>
      </div>

      <div
        v-if="groupExpanded.local && groupExpanded.external"
        class="section-resize-handle"
        :class="{ 'section-resize-active': resizing === 'local' }"
        @mousedown="startResize('local', $event)"
      ></div>

      <div class="pane-section" :style="getSectionFlexStyle('external')">
        <div class="pane-section-header" @click="toggleGroup('external')" @contextmenu.stop>
          <NIcon size="14">
            <component :is="groupExpanded.external ? ChevronDown : ChevronRight" />
          </NIcon>
          <NIcon size="14"><FolderSymlink /></NIcon>
          <span class="pane-section-title">{{ t('scratchpad.externalReferences') }}</span>
          <span class="pane-section-actions">
            <button class="toolbar-btn" :title="t('scratchpad.reference')" @click.stop="handleAddReference"><FolderSymlink :size="14" /></button>
            <button class="toolbar-btn" :title="t('scratchpad.collapseAll')" @click.stop="toggleGroup('external')"><ChevronsUpDown :size="14" /></button>
          </span>
        </div>
        <div v-show="groupExpanded.external" class="pane-section-body">
          <div v-if="externalReferences.length === 0" class="empty-hint">
            {{ t('scratchpad.noReferences') }}
          </div>
          <div
            v-for="ref in externalReferences"
            :key="ref.alias"
            class="ref-row"
            @click="handleRefClick(ref)"
            @contextmenu.prevent="showRefMenu($event, ref)"
          >
            <NIcon size="14">
              <FolderSymlink :class="{ 'ref-invalid': isRefInvalid(ref) }" />
            </NIcon>
            <span class="ref-name">{{ ref.alias }}</span>
            <span
              :class="['ref-path', { 'ref-path-invalid': isRefInvalid(ref) }]"
              :title="ref.path"
              >{{ ref.path }}</span
            >
            <span v-if="isRefInvalid(ref)" class="ref-badge">{{ t('scratchpad.invalid') }}</span>
          </div>
        </div>
      </div>

      <div
        v-if="groupExpanded.external && showTrash"
        class="section-resize-handle"
        :class="{ 'section-resize-active': resizing === 'external' }"
        @mousedown="startResize('external', $event)"
      ></div>

      <div class="pane-section" :style="getSectionFlexStyle('trash')">
        <div class="pane-section-header" @click="toggleTrash" @contextmenu.stop>
          <NIcon size="14">
            <component :is="showTrash ? ChevronDown : ChevronRight" />
          </NIcon>
          <NIcon size="14"><Trash2 /></NIcon>
          <span class="pane-section-title">{{ t('scratchpad.trash') }}</span>
          <span class="pane-section-actions">
            <button v-if="trashEntries.length > 0" class="toolbar-btn" :title="t('scratchpad.emptyTrash')" @click.stop="handleEmptyTrash"><X :size="14" /></button>
            <button class="toolbar-btn" :title="t('scratchpad.collapseAll')" @click.stop="toggleTrash"><ChevronsUpDown :size="14" /></button>
          </span>
        </div>
        <div v-show="showTrash" class="pane-section-body">
          <div v-if="trashEntries.length === 0" class="empty-hint">
            {{ t('scratchpad.noTrashItems') }}
          </div>
          <div
            v-for="item in trashEntries"
            :key="item.path"
            class="ref-row"
          >
            <NIcon size="14"><FileText /></NIcon>
            <span class="ref-name">{{ item.name }}</span>
            <span class="ref-path" :title="item.path">{{ item.path }}</span>
            <NButton
              size="tiny"
              quaternary
              type="primary"
              @click.stop="handleRestore(item.name)"
            >
              {{ t('scratchpad.restoreFromTrash') }}
            </NButton>
          </div>
        </div>
      </div>
    </div>

    </div>

    <NModal v-model:show="showRefModal" :title="t('scratchpad.addReference')">
      <div class="modal-body">
        <NInput
          v-model:value="newRefAlias"
          :placeholder="t('scratchpad.aliasPlaceholder')"
          class="modal-input"
        />
        <NInput
          v-model:value="newRefPath"
          :placeholder="t('scratchpad.pathPlaceholder')"
          class="modal-input"
        />
        <NButton size="small" @click="browseRefPath">
          <template #icon>
            <NIcon><FolderOpen /></NIcon>
          </template>
          {{ t('scratchpad.browse') }}
        </NButton>
        <div class="modal-actions">
          <NButton size="small" @click="showRefModal = false">{{ t('common.cancel') }}</NButton>
          <NButton
            size="small"
            type="primary"
            :disabled="!newRefAlias.trim() || !newRefPath.trim()"
            @click="confirmAddReference"
            >{{ t('common.confirm') }}</NButton
          >
        </div>
      </div>
    </NModal>

    <NModal
      v-model:show="showPromoteConfirm"
      :title="t('scratchpad.promoteToResource')"
      preset="dialog"
      :positive-text="t('scratchpad.promoteKeepDraft')"
      :negative-text="t('scratchpad.promoteDeleteDraft')"
      type="info"
      @positive-click="handlePromoteConfirm(false)"
      @negative-click="handlePromoteConfirm(true)"
    >
      <template v-if="promoteTarget">
        {{ t('scratchpad.promoteToResourceConfirm', { name: promoteTarget.name }) }}
      </template>
    </NModal>

    <NModal
      v-model:show="showConflictDialog"
      title="⚠️ 检测到外部修改"
      preset="dialog"
      type="warning"
      positive-text="重新加载"
      negative-text="忽略"
      @positive-click="handleConflictReload"
      @negative-click="handleConflictIgnore"
    >
      <template v-if="conflictFilePath">
        <div class="conflict-message">
          <p>文件 <strong>{{ conflictFilePath }}</strong> 被外部程序修改。</p>
          <p>当前编辑器中有未保存的更改。重新加载将丢失未保存的内容。</p>
        </div>
        <div class="conflict-actions">
          <NButton size="small" type="info" @click="handleConflictDiff">
            {{ t('scratchpad.viewDiff') }}
          </NButton>
        </div>
      </template>
    </NModal>

    <NModal
      v-model:show="showDiffDialog"
      :title="t('scratchpad.diffView')"
      preset="card"
      style="width: 800px; max-width: 90vw;"
      :mask-closable="false"
    >
      <template #header-extra>
        <NButton size="small" @click="handleDiffClose">{{ t('common.back') }}</NButton>
      </template>
      <div v-if="diffResult" class="diff-container">
        <div class="diff-labels">
          <span class="diff-label-left">{{ diffLeftLabel }}</span>
          <span class="diff-label-right">{{ diffRightLabel }}</span>
        </div>
        <div class="diff-lines">
          <div
            v-for="(line, idx) in diffResult.lines"
            :key="idx"
            :class="['diff-line', `diff-${line.kind}`]"
          >
            <span class="diff-line-num diff-num-left">{{ line.line_number_left || '' }}</span>
            <span class="diff-line-num diff-num-right">{{ line.line_number_right || '' }}</span>
            <span class="diff-line-content">{{ line.content }}</span>
          </div>
        </div>
      </div>
      <template #footer>
        <NButton size="small" @click="handleDiffClose">{{ t('common.close') }}</NButton>
        <NButton size="small" type="info" @click="handleDiffAcceptRight">
          {{ t('scratchpad.acceptRight') }}
        </NButton>
      </template>
    </NModal>

    <div v-if="moveUndoVisible" class="undo-bar move-undo-bar">
      <span class="undo-text">{{ t('scratchpad.undoMoveHint', { name: moveUndoName }) }}</span>
      <NButton size="tiny" type="primary" quaternary @click="handleMoveUndo">
        {{ t('scratchpad.undo') }}
      </NButton>
      <NButton size="tiny" quaternary @click="dismissMoveUndo">
        <template #icon>
          <NIcon size="12"><X /></NIcon>
        </template>
      </NButton>
    </div>

    <div
      v-if="contextMenu.visible"
      class="scratchpad-context-menu"
      :style="{ left: `${contextMenu.x}px`, top: `${contextMenu.y}px` }"
    >
      <div
        v-for="item in contextMenu.items"
        :key="item.key"
        :class="['menu-item', { 'menu-item-danger': item.danger }]"
        @click="handleMenuAction(item.key)"
      >
        <NIcon v-if="item.icon" size="14"><component :is="item.icon" /></NIcon>
        <span>{{ item.label }}</span>
        <span v-if="item.shortcut" class="menu-shortcut">{{ item.shortcut }}</span>
      </div>
    </div>

    <div v-if="undoState.visible" class="undo-bar">
      <span class="undo-text">{{ t('scratchpad.undoDeleteHint', { name: undoState.name }) }}</span>
      <NButton size="tiny" type="primary" quaternary @click="handleUndoDelete">
        {{ t('scratchpad.undo') }}
      </NButton>
      <NButton size="tiny" quaternary @click="dismissUndo">
        <template #icon>
          <NIcon size="12"><X /></NIcon>
        </template>
      </NButton>
    </div>
  </div>
</template>

<script setup lang="ts">
import { listen } from '@tauri-apps/api/event'
import {
  FilePlus,
  FolderPlus,
  Folder,
  FolderOpen,
  Upload,
  Search,
  ChevronDown,
  ChevronRight,
  ChevronsUpDown,
  FolderSymlink,
  FileText,
  Trash2,
  Pencil,
  Copy,
  ExternalLink,
  X,
  BarChart3,
  GitBranch,
  ArrowUpDown,
  ArrowUp,
  ArrowDown,
  Info,
  History,
  MoreVertical,
  WholeWord,
} from 'lucide-vue-next'
import { NButton, NIcon, NInput, NSpin, NModal, NDropdown, createDiscreteApi } from 'naive-ui'
import { ref, reactive, computed, watch, onMounted, onUnmounted, nextTick } from 'vue'
import { useI18n } from 'vue-i18n'

import ScratchpadTreeNode from './ScratchpadTreeNode.vue'
import { useScratchpad } from '../composables/use-scratchpad'

import type { ScratchpadChangeEvent, ScratchpadEntry, ExternalReference, SearchMatch, SearchResult, DiffResult } from '../../types'

const { t } = useI18n()
const { message } = createDiscreteApi(['message'])

const {
  response,
  isLoading,
  notInitialized,
  error,
  searchQuery,
  localEntries,
  externalReferences,
  scratchpadPath,
  loadFiles,
  createEntry,
  deleteEntry,
  renameEntry,
  loadFileContent,
  saveFile,
  importFile,
  addReference,
  removeReference,
  isRefInvalid,
  openInExplorerAction,
  searchContent,
  trashEntries,
  loadTrashEntries,
  restoreTrashEntry,
  emptyTrashBin,
  analyzableFiles,
  loadAnalyzableFiles,
  startWatching,
  stopWatching,
  promoteToResource,
  applyFileChanges,
  flattenVisibleEntries,
  dirtyFiles,
  markClean,
  externalConflicts,
  dismissConflict,
  loadChildEntries,
  hasChildrenLoaded,
  clipboardMode,
  moveEntry,
  replaceInFile,
  diffWithContent,
  validateRegex,
  setContentSnapshot,
  getContentSnapshot,
  normalizePathForCompare,
} = useScratchpad()

const fileMeta = computed(() => response.value?.file_meta ?? {})

const contentSearchMode = ref(false)
const searchResult = ref<SearchResult | null>(null)
const showPromoteConfirm = ref(false)
const promoteTarget = ref<ScratchpadEntry | null>(null)
const showConflictDialog = ref(false)
const conflictFilePath = ref<string | null>(null)
const showTrash = ref(false)
const sortBy = ref<'name' | 'size' | 'modified'>('name')
const sortOrder = ref<'asc' | 'desc'>('asc')
const showExplorerFilter = ref(false)
const caseSensitive = ref(false)
const isRegex = ref(false)
const regexError = ref<string | null>(null)
const wholeWord = ref(false)
const expandedSearchFiles = ref<Set<string>>(new Set())
const showSearchFilters = ref(false)
const searchInclude = ref('')
const searchExclude = ref('')
const showRecent = ref(true)
const recentFiles = ref<string[]>([])
const sectionFlex = reactive<Record<string, number>>({ local: 60, external: 20, trash: 20 })
const resizing = ref<string | null>(null)
const resizeStartY = ref(0)
const resizeStartFlex = ref<[number, number]>([0, 0])
const MAX_RECENT = 5
let contentSearchTimer: ReturnType<typeof setTimeout> | null = null

const showReplaceBar = ref(false)
const replaceWith = ref('')
const replacePreviewCount = ref(0)
const replaceInProgress = ref(false)

const showDiffDialog = ref(false)
const diffResult = ref<DiffResult | null>(null)
const diffLeftLabel = ref('')
const diffRightLabel = ref('')
const diffFilePath = ref('')

const moveUndoVisible = ref(false)
const moveUndoFromPath = ref('')
const moveUndoToParent = ref('')
const moveUndoName = ref('')
let moveUndoTimer: ReturnType<typeof setTimeout> | null = null
const MOVE_UNDO_TIMEOUT = 5000

function toggleExplorerFilter(): void {
  showExplorerFilter.value = !showExplorerFilter.value
  if (showExplorerFilter.value) {
    contentSearchMode.value = true
    searchQuery.value = ''
    searchResult.value = null
    showReplaceBar.value = false
    expandedSearchFiles.value = new Set()
    nextTick(() => {
      const input = document.querySelector('.search-panel-input input') as HTMLInputElement | null
      input?.focus()
    })
  } else {
    searchQuery.value = ''
    contentSearchMode.value = false
    searchResult.value = null
    showReplaceBar.value = false
  }
}

const moreMenuOptions = computed(() => [
  { label: t('scratchpad.newFile'), key: 'newFile' },
  { label: t('scratchpad.newFolder'), key: 'newFolder' },
  { label: t('scratchpad.refresh'), key: 'refresh' },
  { type: 'divider' as const, key: 'd1' },
  { label: t('scratchpad.sortByName'), key: 'sortName' },
  { label: t('scratchpad.sortBySize'), key: 'sortSize' },
  { label: t('scratchpad.sortByModified'), key: 'sortModified' },
  { type: 'divider' as const, key: 'd2' },
  { label: t('scratchpad.expandAll'), key: 'expandAll' },
  { label: t('scratchpad.collapseAll'), key: 'collapseAll' },
])

function handleMoreMenuSelect(key: string): void {
  switch (key) {
    case 'newFile': handleCreateFile(); break
    case 'newFolder': handleCreateFolder(); break
    case 'refresh': loadFiles(); break
    case 'sortName': toggleSort('name'); break
    case 'sortSize': toggleSort('size'); break
    case 'sortModified': toggleSort('modified'); break
    case 'expandAll': handleExpandAll(); break
    case 'collapseAll': handleCollapseAll(); break
  }
}

const searchResultsByFile = computed(() => {
  const sr = searchResult.value
  if (!sr || sr.matches.length === 0) return new Map<string, SearchMatch[]>()
  const grouped = new Map<string, SearchMatch[]>()
  for (const m of sr.matches) {
    const list = grouped.get(m.file) || []
    list.push(m)
    grouped.set(m.file, list)
  }
  return grouped
})

const resultsSummaryText = computed(() => {
  const sr = searchResult.value
  if (!sr) return ''
  const fileCount = searchResultsByFile.value.size
  const matchCount = sr.matches.length
  if (matchCount === 0) return t('scratchpad.noResults')
  return t('scratchpad.searchSummary', { files: fileCount, count: matchCount })
})

function toggleSearchFileExpand(file: string): void {
  const next = new Set(expandedSearchFiles.value)
  if (next.has(file)) {
    next.delete(file)
  } else {
    next.add(file)
  }
  expandedSearchFiles.value = next
}

function handleSearchMatchClick(match: SearchMatch): void {
  openFileAtLine(match.file, match.line_number)
}

function highlightSearchMatch(match: SearchMatch): string {
  const line = escapeHtml(match.line_content)
  const q = searchQuery.value.trim()
  if (!q || isRegex.value) return line
  const lowerLine = caseSensitive.value ? line : line.toLowerCase()
  const lowerQ = caseSensitive.value ? q : q.toLowerCase()
  const idx = lowerLine.indexOf(lowerQ)
  if (idx === -1) return line
  const before = line.slice(0, idx)
  const matched = line.slice(idx, idx + q.length)
  const after = line.slice(idx + q.length)
  return `${escapeHtml(before)}<mark class="search-hl">${escapeHtml(matched)}</mark>${escapeHtml(after)}`
}

function toggleRegex(): void {
  isRegex.value = !isRegex.value
  regexError.value = null
  if (isRegex.value) {
    const result = validateRegex(searchQuery.value)
    regexError.value = result.error
  }
}

function toggleSort(field: 'name' | 'size' | 'modified'): void {
  if (sortBy.value === field) {
    sortOrder.value = sortOrder.value === 'asc' ? 'desc' : 'asc'
  } else {
    sortBy.value = field
    sortOrder.value = 'asc'
  }
}

function sortIcon(field: 'name' | 'size' | 'modified'): typeof ArrowUpDown {
  if (sortBy.value !== field) return ArrowUpDown
  return sortOrder.value === 'asc' ? ArrowUp : ArrowDown
}

const contentResults = computed(() => {
  const sr = searchResult.value
  if (!sr) return new Map<string, SearchMatch[]>()
  const grouped = new Map<string, SearchMatch[]>()
  for (const m of sr.matches) {
    const list = grouped.get(m.file) || []
    list.push(m)
    grouped.set(m.file, list)
  }
  return grouped
})

const contentAllMatches = computed(() => searchResult.value?.matches ?? [])

const searchNotice = computed(() => {
  const sr = searchResult.value
  if (!sr) return null
  const parts: string[] = []
  if (sr.truncated) {
    parts.push(t('scratchpad.searchTruncated', { max: 500 }))
  }
  if (sr.total_files_skipped > 0) {
    parts.push(t('scratchpad.searchSkippedFiles', {
      count: sr.total_files_skipped,
      size: '10MB'
    }))
  }
  if (parts.length === 0 && sr.matches.length === 0) {
    return t('scratchpad.noResults')
  }
  if (parts.length === 0) return null
  return parts.join('；')
})

watch(searchQuery, async (val) => {
  if (!contentSearchMode.value) return
  if (contentSearchTimer) clearTimeout(contentSearchTimer)
  if (!val.trim()) {
    searchResult.value = null
    return
  }
  contentSearchTimer = setTimeout(async () => {
    const result = await searchContent(val.trim(), caseSensitive.value)
    searchResult.value = result
  }, 300)
})

watch(searchResult, (sr) => {
  if (sr && sr.matches.length > 0) {
    const files = new Set(sr.matches.map(m => m.file))
    expandedSearchFiles.value = new Set(files)
  }
})

watch(caseSensitive, async () => {
  const q = searchQuery.value.trim()
  if (!contentSearchMode.value || !q) return
  const result = await searchContent(q, caseSensitive.value)
  searchResult.value = result
})

const recentFileEntries = computed(() => {
  if (recentFiles.value.length === 0) return []
  const scratchpadBase = scratchpadPath.value || ''
  return recentFiles.value
    .map(rp => localEntries.value.find(e => {
      const relPath = scratchpadBase
        ? e.path.replace(scratchpadBase.replace(/\\/g, '/'), '').replace(/^\//, '')
        : e.path
      return relPath === rp
    }))
    .filter((e): e is ScratchpadEntry => !!e)
})

function addRecentFile(relativePath: string): void {
  const list = recentFiles.value.filter(p => p !== relativePath)
  list.unshift(relativePath)
  recentFiles.value = list.slice(0, MAX_RECENT)
}

watch(searchQuery, async (val) => {
  if (isRegex.value) {
    const result = validateRegex(val || '')
    regexError.value = result.error
  }
})

function toggleReplaceBar(): void {
  showReplaceBar.value = !showReplaceBar.value
  replacePreviewCount.value = 0
}

async function computeReplacePreview(): Promise<void> {
  const q = searchQuery.value.trim()
  const rw = replaceWith.value
  if (!q || !rw) {
    replacePreviewCount.value = 0
    return
  }
  const files = contentResults.value
  let totalReplacements = 0
  for (const [file] of files) {
    try {
      const content = await loadFileContent(file)
      if (content === null) continue
      if (isRegex.value) {
        try {
          const re = new RegExp(q, caseSensitive.value ? 'g' : 'gi')
          const matches = content.match(re)
          if (matches) totalReplacements += matches.length
        } catch {
          continue
        }
      } else {
        let count = 0
        let idx = 0
        const search = q
        const searchLower = caseSensitive.value ? search : search.toLowerCase()
        const contentLower = caseSensitive.value ? content : content.toLowerCase()
        while ((idx = contentLower.indexOf(searchLower, idx)) !== -1) {
          count++
          idx += search.length
        }
        totalReplacements += count
      }
    } catch {
      continue
    }
  }
  replacePreviewCount.value = totalReplacements
}

async function handleReplaceAll(): Promise<void> {
  const q = searchQuery.value.trim()
  const rw = replaceWith.value
  if (!q || !rw) return
  replaceInProgress.value = true
  const files = contentResults.value
  let totalFiles = 0
  let totalReplaced = 0
  for (const [file] of files) {
    const result = await replaceInFile(file, q, rw, isRegex.value)
    if (result && result.replaced > 0) {
      totalFiles++
      totalReplaced += result.replaced
    }
  }
  replaceInProgress.value = false
  if (totalReplaced > 0) {
    message.success(
      t('scratchpad.replaceSuccess', { files: totalFiles, count: totalReplaced })
    )
    const result = await searchContent(q, caseSensitive.value)
    searchResult.value = result
  } else {
    message.info(t('scratchpad.replaceNoMatches'))
  }
}

const filteredLocalEntries = computed(() => {
  let entries = localEntries.value
  if (contentSearchMode.value) {
    const results = contentResults.value
    const base = scratchpadPath.value || ''
    entries = entries.filter(e => {
      let rel = e.path
      if (base) {
        rel = e.path.replace(base.replace(/\\/g, '/'), '').replace(/^\//, '')
      }
      return results.has(rel)
    })
  }
  return [...entries].sort((a, b) => {
    let cmp = 0
    if (sortBy.value === 'name') {
      cmp = a.name.localeCompare(b.name)
    } else if (sortBy.value === 'size') {
      cmp = a.size - b.size
    } else if (sortBy.value === 'modified') {
      const da = a.modified_at || ''
      const db = b.modified_at || ''
      cmp = da.localeCompare(db)
    }
    return sortOrder.value === 'asc' ? cmp : -cmp
  })
})

const multiSelected = computed(() => selectedKeys.value.size)

const expandedKeys = ref<Set<string>>(new Set())
const selectedKey = ref<string | null>(null)
const selectedKeys = ref<Set<string>>(new Set())
const lastSelectPath = ref<string | null>(null)
const clipboardEntry = ref<ScratchpadEntry | null>(null)
const renamingKey = ref<string | null>(null)

const undoState = reactive<{
  visible: boolean
  name: string
  relativePath: string
  timer: ReturnType<typeof setTimeout> | null
}>({
  visible: false,
  name: '',
  relativePath: '',
  timer: null,
})

const UNDO_TIMEOUT = 5000

const ROW_HEIGHT = 28
const OVERSCAN = 8
const VIRTUAL_SCROLL_THRESHOLD = 50

const treeContainerRef = ref<HTMLElement | null>(null)
const treeScrollTop = ref(0)
const treeContainerHeight = ref(0)

const flattenedTree = computed(() =>
  flattenVisibleEntries(filteredLocalEntries.value, expandedKeys.value)
)

const useVirtualScrollEnabled = computed(
  () => flattenedTree.value.length > VIRTUAL_SCROLL_THRESHOLD
)

const visibleTreeEntries = computed(() => {
  if (!useVirtualScrollEnabled.value) {
    return flattenedTree.value
  }
  const total = flattenedTree.value.length
  const start = Math.floor(treeScrollTop.value / ROW_HEIGHT)
  const visibleCount = Math.ceil(treeContainerHeight.value / ROW_HEIGHT)
  const from = Math.max(0, start - OVERSCAN)
  const to = Math.min(total, start + visibleCount + OVERSCAN)
  return flattenedTree.value.slice(from, to)
})

const virtualScrollPaddingTop = computed(() => {
  if (!useVirtualScrollEnabled.value || flattenedTree.value.length === 0) return 0
  const total = flattenedTree.value.length
  const start = Math.floor(treeScrollTop.value / ROW_HEIGHT)
  const visibleCount = Math.ceil(treeContainerHeight.value / ROW_HEIGHT)
  const from = Math.max(0, start - OVERSCAN)
  return from * ROW_HEIGHT
})

const virtualScrollTotalHeight = computed(() => {
  if (!useVirtualScrollEnabled.value) return undefined
  return flattenedTree.value.length * ROW_HEIGHT
})

function handleTreeScroll(): void {
  const el = treeContainerRef.value
  if (!el) return
  treeScrollTop.value = el.scrollTop
  treeContainerHeight.value = el.clientHeight
}

function updateTreeContainerSize(): void {
  const el = treeContainerRef.value
  if (!el) return
  treeContainerHeight.value = el.clientHeight
}

function showUndo(name: string, relativePath: string): void {
  if (undoState.timer) clearTimeout(undoState.timer)
  undoState.visible = true
  undoState.name = name
  undoState.relativePath = relativePath
  undoState.timer = setTimeout(() => {
    dismissUndo()
  }, UNDO_TIMEOUT)
}

function dismissUndo(): void {
  undoState.visible = false
  if (undoState.timer) {
    clearTimeout(undoState.timer)
    undoState.timer = null
  }
}

async function handleUndoDelete(): Promise<void> {
  const name = undoState.relativePath
  dismissUndo()
  const ok = await restoreTrashEntry(name)
  if (ok) message.success(t('scratchpad.restoredSuccess', { name }))
}

const groupExpanded = reactive({ external: false, local: true })

const showRefModal = ref(false)
const newRefAlias = ref('')
const newRefPath = ref('')
const inlineCreateParentPath = ref<string | null>(null)
const inlineCreateIsFolder = ref(false)
const isInlineCreateActive = ref(false)
const rootInlineInputRef = ref<HTMLInputElement | null>(null)
const rootInlineCreateName = ref('')

interface ContextMenuItem {
  key: string
  label: string
  icon?: typeof FileText
  danger?: boolean
  shortcut?: string
}

const contextMenu = reactive<{
  visible: boolean
  x: number
  y: number
  target: ScratchpadEntry | ExternalReference | null
  isRefTarget: boolean
  isBlankTarget: boolean
  items: ContextMenuItem[]
}>({
  visible: false,
  x: 0,
  y: 0,
  target: null,
  isRefTarget: false,
  isBlankTarget: false,
  items: [],
})

let openDialog: typeof import('@tauri-apps/plugin-dialog').open | null = null
if (typeof window !== 'undefined' && (window as any).__TAURI__) {
  import('@tauri-apps/plugin-dialog')
    .then(m => {
      openDialog = m.open
    })
    .catch(() => {
      openDialog = null
    })
}



function handleGlobalClick(): void {
  if (renamingKey.value) {
    cancelRename()
  }
}

function toggleGroup(group: 'external' | 'local'): void {
  groupExpanded[group] = !groupExpanded[group]
}

function toggleTrash(): void {
  showTrash.value = !showTrash.value
  if (showTrash.value) {
    loadTrashEntries()
  }
}

const expandedSections = computed(() => {
  const sections: string[] = []
  if (groupExpanded.local) sections.push('local')
  if (groupExpanded.external) sections.push('external')
  if (showTrash.value) sections.push('trash')
  return sections
})

function getSectionFlexStyle(section: string): Record<string, string> {
  const expanded = expandedSections.value
  if (!expanded.includes(section)) {
    return { flex: '0 0 auto' }
  }
  if (expanded.length === 1) {
    return { flex: '1 1 0', minHeight: '0' }
  }
  const flex = sectionFlex[section] || 1
  return { flex: `${flex} 1 0`, minHeight: '40px' }
}

function startResize(section: string, event: MouseEvent): void {
  event.preventDefault()
  const expanded = expandedSections.value
  const idx = expanded.indexOf(section)
  if (idx < 0 || idx >= expanded.length - 1) return
  const current = expanded[idx]
  const next = expanded[idx + 1]
  resizing.value = section
  resizeStartY.value = event.clientY
  resizeStartFlex.value = [sectionFlex[current] || 1, sectionFlex[next] || 1]
  document.addEventListener('mousemove', handleResizeMove)
  document.addEventListener('mouseup', handleResizeEnd)
}

function handleResizeMove(event: MouseEvent): void {
  if (!resizing.value) return
  const expanded = expandedSections.value
  const idx = expanded.indexOf(resizing.value)
  if (idx < 0 || idx >= expanded.length - 1) return
  const current = expanded[idx]
  const next = expanded[idx + 1]
  const delta = event.clientY - resizeStartY.value
  const total = resizeStartFlex.value[0] + resizeStartFlex.value[1]
  if (total === 0) return
  const container = document.querySelector('.tree-container')
  const containerHeight = container ? container.clientHeight : 400
  if (containerHeight === 0) return
  const deltaRatio = (delta / containerHeight) * 100
  let newFlex0 = resizeStartFlex.value[0] + deltaRatio
  let newFlex1 = resizeStartFlex.value[1] - deltaRatio
  newFlex0 = Math.max(10, Math.min(90, newFlex0))
  newFlex1 = Math.max(10, Math.min(90, newFlex1))
  sectionFlex[current] = newFlex0
  sectionFlex[next] = newFlex1
}

function handleResizeEnd(): void {
  resizing.value = null
  document.removeEventListener('mousemove', handleResizeMove)
  document.removeEventListener('mouseup', handleResizeEnd)
}

function handleSelect(entry: ScratchpadEntry, event?: MouseEvent): void {
  const ctrl = event?.ctrlKey || event?.metaKey
  const shift = event?.shiftKey

  if (ctrl) {
    const next = new Set(selectedKeys.value)
    if (next.has(entry.path)) {
      next.delete(entry.path)
    } else {
      next.add(entry.path)
    }
    selectedKeys.value = next
    selectedKey.value = entry.path
    lastSelectPath.value = entry.path
    return
  }

  if (shift && lastSelectPath.value) {
    const flatEntries = flattenedTree.value.map(item => item.entry)
    const startIdx = flatEntries.findIndex(e => e.path === lastSelectPath.value)
    const endIdx = flatEntries.findIndex(e => e.path === entry.path)
    if (startIdx !== -1 && endIdx !== -1) {
      const from = Math.min(startIdx, endIdx)
      const to = Math.max(startIdx, endIdx)
      const next = new Set<string>()
      for (let i = from; i <= to; i++) {
        next.add(flatEntries[i].path)
      }
      selectedKeys.value = next
      selectedKey.value = entry.path
    }
    return
  }

  selectedKey.value = entry.path
  selectedKeys.value = new Set<string>()
  lastSelectPath.value = entry.path
}

async function handleToggleExpand(entry: ScratchpadEntry): Promise<void> {
  const normalizedPath = normalizePathForCompare(entry.path)
  const next = new Set(expandedKeys.value)
  if (next.has(normalizedPath)) {
    next.delete(normalizedPath)
    expandedKeys.value = next
    return
  }
  if (entry.kind === 'folder' && !hasChildrenLoaded(entry)) {
    await loadChildEntries(entry.path)
  }
  next.add(normalizedPath)
  expandedKeys.value = next
}

const dragEntry = ref<ScratchpadEntry | null>(null)

function handleTreeNodeDragStart(event: DragEvent, entry: ScratchpadEntry): void {
  dragEntry.value = entry
  const scratchpadBase = scratchpadPath.value || ''
  const relativePath = scratchpadBase
    ? entry.path.replace(scratchpadBase.replace(/\\/g, '/'), '').replace(/^\//, '')
    : entry.path
  if (event.dataTransfer) {
    event.dataTransfer.setData('application/x-scratchpad-file', relativePath)
    event.dataTransfer.setData('text/plain', relativePath)
    event.dataTransfer.effectAllowed = 'copy'
  }
}

function collectFolderPaths(entries: ScratchpadEntry[]): string[] {
  const result: string[] = []
  for (const e of entries) {
    if (e.kind === 'folder') result.push(e.path)
    if (e.children) {
      result.push(...collectFolderPaths(e.children))
    }
  }
  return result
}

async function handleExpandAll(): Promise<void> {
  const allFolders = collectFolderPaths(filteredLocalEntries.value)
  expandedKeys.value = new Set(allFolders)
  for (const path of allFolders) {
    const entry = findEntryInTree(localEntries.value, path)
    if (entry && entry.kind === 'folder' && !hasChildrenLoaded(entry)) {
      await loadChildEntries(path)
    }
  }
}

function handleCollapseAll(): void {
  expandedKeys.value = new Set()
}

function handleOpen(entry: ScratchpadEntry): void {
  closeContextMenu()
  if (entry.kind === 'folder') {
    handleToggleExpand(entry)
    return
  }
  openFileInEditor(entry)
}

async function openFileInEditor(entry: ScratchpadEntry): Promise<void> {
  const ext = entry.name.includes('.') ? '.' + entry.name.split('.').pop()?.toLowerCase() : ''
  const langMap: Record<string, string> = {
    '.sql': 'sql',
    '.py': 'python',
    '.json': 'json',
    '.txt': 'plaintext',
    '.md': 'markdown',
  }
  const language = langMap[ext] || 'plaintext'

  const scratchpadBase = scratchpadPath.value || ''
  const relativePath = scratchpadBase
    ? entry.path.replace(scratchpadBase.replace(/\\/g, '/'), '').replace(/^\//, '')
    : entry.path

  const content = await loadFileContent(relativePath)
  if (content === null) {
    return
  }

  setContentSnapshot(relativePath, content)

  const metaForFile = fileMeta.value[relativePath]
  const lastConnectionId = metaForFile?.last_connection_id || ''

  window.dispatchEvent(
    new CustomEvent('open-sql-editor', {
      detail: {
        connectionId: lastConnectionId || '',
        databaseName: '',
        sql: content,
        scratchpadRelativePath: relativePath,
        scratchpadFileName: entry.name,
        language,
      },
    })
  )
  addRecentFile(relativePath)
}

async function openFileAtLine(relativePath: string, line: number): Promise<void> {
  const entry = localEntries.value.find(e => {
    const scratchpadBase = scratchpadPath.value || ''
    const relPath = scratchpadBase
      ? e.path.replace(scratchpadBase.replace(/\\/g, '/'), '').replace(/^\//, '')
      : e.path
    return relPath === relativePath
  })
  if (!entry) return

  const ext = entry.name.includes('.')
    ? '.' + entry.name.split('.').pop()?.toLowerCase()
    : ''
  const langMap: Record<string, string> = {
    '.sql': 'sql',
    '.py': 'python',
    '.json': 'json',
    '.txt': 'plaintext',
    '.md': 'markdown',
  }
  const language = langMap[ext] || 'plaintext'

  const content = await loadFileContent(relativePath)
  if (content === null) return

  setContentSnapshot(relativePath, content)

  const metaForFile = fileMeta.value[relativePath]
  const lastConnectionId = metaForFile?.last_connection_id || ''

  window.dispatchEvent(
    new CustomEvent('open-sql-editor', {
      detail: {
        connectionId: lastConnectionId || '',
        databaseName: '',
        sql: content,
        scratchpadRelativePath: relativePath,
        scratchpadFileName: entry.name,
        language,
        initialLine: line,
      },
    })
  )
  addRecentFile(relativePath)
}

async function handleFileClick(file: string): Promise<void> {
  openFileAtLine(file, 0)
}

function handleLineClick(file: string, line: number): void {
  openFileAtLine(file, line)
}

function highlightMatch(line: string, query: string): string {
  if (!query) return escapeHtml(line)
  const q = query.trim()
  if (!q) return escapeHtml(line)
  const idx = line.toLowerCase().indexOf(q.toLowerCase())
  if (idx === -1) return escapeHtml(line)
  const before = escapeHtml(line.slice(0, idx))
  const match = escapeHtml(line.slice(idx, idx + q.length))
  const after = escapeHtml(line.slice(idx + q.length))
  return `${before}<mark class="search-hl">${match}</mark>${after}`
}

function escapeHtml(str: string): string {
  return str
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&#39;')
}

function startInlineCreate(parentPath: string | null, isFolder: boolean): void {
  groupExpanded.local = true
  isInlineCreateActive.value = true
  inlineCreateParentPath.value = parentPath
  inlineCreateIsFolder.value = isFolder
  rootInlineCreateName.value = ''
  if (parentPath === null) {
    nextTick(() => {
      rootInlineInputRef.value?.focus()
    })
  } else {
    expandedKeys.value = new Set([...expandedKeys.value, parentPath])
    const entry = findEntryInTree(localEntries.value, parentPath)
    if (entry && entry.kind === 'folder' && !hasChildrenLoaded(entry)) {
      loadChildEntries(parentPath)
    }
  }
}

function cancelInlineCreate(): void {
  isInlineCreateActive.value = false
  inlineCreateParentPath.value = null
  inlineCreateIsFolder.value = false
  rootInlineCreateName.value = ''
}

function commitRootInlineCreate(): void {
  const name = rootInlineCreateName.value.trim()
  if (!name) {
    cancelInlineCreate()
    return
  }
  confirmInlineCreate(name)
}

function cancelRootInlineCreate(): void {
  cancelInlineCreate()
}

async function confirmInlineCreate(name: string): Promise<void> {
  const parentPath = inlineCreateParentPath.value
  const isFolder = inlineCreateIsFolder.value
  if (!name) {
    cancelInlineCreate()
    return
  }
  const entry = await createEntry(name, isFolder, parentPath || undefined)
  if (entry) {
    cancelInlineCreate()
    if (parentPath) {
      expandedKeys.value = new Set([...expandedKeys.value, parentPath])
    }
    message.success(t('scratchpad.createdSuccess', { name }))
    if (!isFolder) {
      openFileInEditor(entry)
    }
  }
}

async function handleCreateFile(): Promise<void> {
  const selectedFolder = findSelectedFolder()
  startInlineCreate(selectedFolder, false)
}

async function handleCreateFolder(): Promise<void> {
  const selectedFolder = findSelectedFolder()
  startInlineCreate(selectedFolder, true)
}

function findSelectedFolder(): string | null {
  const sel = selectedKey.value
  if (!sel) return null
  const entry = findEntryInTree(localEntries.value, sel)
  if (entry && entry.kind === 'folder') return sel
  if (entry) {
    return getParentPathOfEntry(sel)
  }
  return null
}

function findEntryInTree(entries: ScratchpadEntry[], path: string): ScratchpadEntry | undefined {
  for (const e of entries) {
    if (e.path === path) return e
    if (e.children) {
      const found = findEntryInTree(e.children, path)
      if (found) return found
    }
  }
  return undefined
}

function getParentPathOfEntry(path: string): string | null {
  const normalized = path.replace(/\\/g, '/')
  const lastSlash = normalized.lastIndexOf('/')
  return lastSlash > 0 ? normalized.substring(0, lastSlash) : null
}

async function handleImportFile(): Promise<void> {
  if (!openDialog) {
    console.warn('[Scratchpad] Dialog plugin not available')
    message.warning(t('scratchpad.dialogNotAvailable'))
    return
  }
  try {
    const selected = await openDialog({
      multiple: false,
      title: t('scratchpad.selectFileToImport'),
      filters: [{ name: t('scratchpad.allFiles'), extensions: ['*'] }],
    })
    if (selected && typeof selected === 'string') {
      const result = await importFile(selected)
      if (result) message.success(t('scratchpad.importedSuccess', { name: result.name }))
    }
  } catch (e) {
    console.error('[Scratchpad] Import dialog error:', e)
  }
}

async function handleRestore(name: string): Promise<void> {
  const ok = await restoreTrashEntry(name)
  if (ok) message.success(t('scratchpad.restoredSuccess', { name }))
}

const isDragOver = ref(false)
let dragLeaveTimer: ReturnType<typeof setTimeout> | null = null

function handleDragOver(event: DragEvent): void {
  if (dragLeaveTimer) {
    clearTimeout(dragLeaveTimer)
    dragLeaveTimer = null
  }
  isDragOver.value = true
}

function handleDragLeave(event: DragEvent): void {
  dragLeaveTimer = setTimeout(() => {
    isDragOver.value = false
  }, 100)
}

async function handleDrop(event: DragEvent): Promise<void> {
  isDragOver.value = false
  const files = event.dataTransfer?.files
  if (!files || files.length === 0) return

  for (let i = 0; i < files.length; i++) {
    const filePath = (files[i] as unknown as { path?: string }).path
    if (filePath) {
      const result = await importFile(filePath)
      if (result) message.success(t('scratchpad.importedSuccess', { name: result.name }))
    }
  }
}

async function handleAddReference(): Promise<void> {
  newRefAlias.value = ''
  newRefPath.value = ''
  showRefModal.value = true
}

async function handleSaveAll(): Promise<void> {
  message.info(t('scratchpad.saveAllHint'))
}

async function handleCloseRecent(): Promise<void> {
  recentFiles.value = []
  message.success(t('scratchpad.recentCleared'))
}

async function handleEmptyTrash(): Promise<void> {
  try {
    const ok = await emptyTrashBin()
    if (ok) message.success(t('scratchpad.trashEmptied'))
  } catch (e) {
    message.error(e instanceof Error ? e.message : String(e))
  }
}

async function browseRefPath(): Promise<void> {
  if (!openDialog) return
  try {
    const selected = await openDialog({
      directory: true,
      multiple: false,
      title: t('scratchpad.selectRefDirectory'),
    })
    if (selected && typeof selected === 'string') {
      newRefPath.value = selected
    }
  } catch (e) {
    console.error('[Scratchpad] Browse error:', e)
  }
}

async function confirmAddReference(): Promise<void> {
  const alias = newRefAlias.value.trim()
  const path = newRefPath.value.trim()
  if (!alias || !path) return
  showRefModal.value = false
  await addReference(alias, path)
}

function handleRefClick(ref: ExternalReference): void {
  selectedKey.value = ref.alias
}

function closeContextMenu(): void {
  contextMenu.visible = false
  contextMenu.target = null
}

const ANALYZABLE_EXTENSIONS = ['.csv', '.parquet', '.json', '.xlsx', '.duckdb']

function isAnalyzableFile(entry: ScratchpadEntry): boolean {
  const ext = entry.name.includes('.') ? '.' + entry.name.split('.').pop()?.toLowerCase() : ''
  return ANALYZABLE_EXTENSIONS.includes(ext)
}

async function handleAnalyzeDuckDB(entry: ScratchpadEntry): Promise<void> {
  await loadAnalyzableFiles()
  const match = analyzableFiles.value.find(f => f.relative_path === entry.path)
  if (!match) return

  const content = match.duckdb_query_hint

  window.dispatchEvent(
    new CustomEvent('open-sql-editor', {
      detail: {
        connectionId: '',
        databaseName: '',
        sql: content,
        scratchpadRelativePath: '',
        scratchpadFileName: `${entry.name} (DuckDB Preview)`,
        language: 'sql',
      },
    })
  )
}

function showBlankMenu(event: MouseEvent): void {
  event.preventDefault()
  event.stopPropagation()
  const pos = clampToViewport(event.clientX, event.clientY, 180, 100)
  contextMenu.x = pos.x
  contextMenu.y = pos.y
  contextMenu.isRefTarget = false
  contextMenu.isBlankTarget = true
  contextMenu.target = null
  contextMenu.items = [
    { key: 'new-file', label: t('scratchpad.newFile'), icon: FilePlus },
    { key: 'new-folder', label: t('scratchpad.newFolder'), icon: FolderPlus },
  ]
  contextMenu.visible = true
}

function showEntryMenu(event: MouseEvent, entry: ScratchpadEntry): void {
  event.preventDefault()
  event.stopPropagation()

  if (!selectedKeys.value.has(entry.path)) {
    selectedKey.value = entry.path
    selectedKeys.value = new Set<string>([entry.path])
    lastSelectPath.value = entry.path
  } else {
    selectedKey.value = entry.path
  }

  const multi = multiSelected.value > 1
  const pos = clampToViewport(event.clientX, event.clientY, 180, 240)
  contextMenu.x = pos.x
  contextMenu.y = pos.y
  contextMenu.isRefTarget = false
  contextMenu.isBlankTarget = false
  contextMenu.target = entry
  contextMenu.items = multi
    ? [
        { key: 'batch-delete', label: t('scratchpad.batchDelete', { n: multiSelected.value }), icon: Trash2, danger: true },
      ]
    : [
        { key: 'new-file', label: t('scratchpad.newFile'), icon: FilePlus },
        { key: 'new-folder', label: t('scratchpad.newFolder'), icon: FolderPlus },
        { key: 'open', label: t('scratchpad.open'), icon: FileText },
        ...(isAnalyzableFile(entry)
          ? [
              {
                key: 'analyze-duckdb',
                label: t('scratchpad.analyzeWithDuckDB'),
                icon: BarChart3,
              },
            ]
          : []),
        ...(entry.kind === 'folder'
          ? [
              {
                key: 'toggle-folder',
                label: expandedKeys.value.has(entry.path)
                  ? t('scratchpad.collapse')
                  : t('scratchpad.expand'),
                icon: ChevronRight,
              },
            ]
          : []),
        { key: 'copy-file', label: t('scratchpad.copyFile'), icon: Copy },
        { key: 'cut-file', label: t('scratchpad.cutFile'), icon: Copy },
        { key: 'rename', label: t('scratchpad.rename'), icon: Pencil, shortcut: 'F2' },
        { key: 'copy-path', label: t('scratchpad.copyPath'), icon: Copy },
        { key: 'promote', label: t('scratchpad.promoteToResource'), icon: GitBranch },
        { key: 'delete', label: t('scratchpad.delete'), icon: Trash2, danger: true, shortcut: 'Del' },
      ]
  if (clipboardEntry.value) {
    contextMenu.items.push({ key: 'paste-file', label: t('scratchpad.pasteFile'), icon: FilePlus })
  }
  contextMenu.visible = true
}

function showRefMenu(event: MouseEvent, ref: ExternalReference): void {
  event.preventDefault()
  event.stopPropagation()
  const pos = clampToViewport(event.clientX, event.clientY, 180, 100)
  contextMenu.x = pos.x
  contextMenu.y = pos.y
  contextMenu.isRefTarget = true
  contextMenu.isBlankTarget = false
  contextMenu.target = ref
  contextMenu.items = [
    { key: 'open-ref-location', label: t('scratchpad.openLocation'), icon: ExternalLink },
    { key: 'remove-ref', label: t('scratchpad.removeReference'), icon: X, danger: true },
  ]
  contextMenu.visible = true
}

function clampToViewport(
  x: number,
  y: number,
  menuWidth: number,
  menuHeight: number
): { x: number; y: number } {
  const w = window.innerWidth
  const h = window.innerHeight
  return {
    x: Math.min(x, w - menuWidth),
    y: Math.min(y, h - menuHeight),
  }
}

async function handleMenuAction(key: string): Promise<void> {
  const isRefTarget = contextMenu.isRefTarget
  const isBlankTarget = contextMenu.isBlankTarget
  const target = contextMenu.target
  closeContextMenu()

  if (key === 'new-file') {
    let parentPath: string | null = null
    if (!isBlankTarget && !isRefTarget && target) {
      const entry = target as ScratchpadEntry
      parentPath = entry.kind === 'folder'
        ? entry.path
        : getParentPathOfEntry(entry.path)
    }
    startInlineCreate(parentPath, false)
    return
  }

  if (key === 'new-folder') {
    let parentPath: string | null = null
    if (!isBlankTarget && !isRefTarget && target) {
      const entry = target as ScratchpadEntry
      parentPath = entry.kind === 'folder'
        ? entry.path
        : getParentPathOfEntry(entry.path)
    }
    startInlineCreate(parentPath, true)
    return
  }

  if (isRefTarget) {
    const ref = target as ExternalReference
    if (ref) {
      if (key === 'remove-ref') {
        await removeReference(ref.alias)
      } else if (key === 'open-ref-location') {
        await openInExplorerAction(ref.path)
      }
    }
    return
  }

  const entry = target as ScratchpadEntry
  if (!entry) return
  switch (key) {
    case 'open':
      openFileInEditor(entry)
      break
    case 'analyze-duckdb':
      handleAnalyzeDuckDB(entry)
      break
    case 'toggle-folder':
      handleToggleExpand(entry)
      break
    case 'rename':
      startRename(entry)
      break
    case 'copy-path':
      await navigator.clipboard.writeText(entry.path)
      message.success(t('scratchpad.pathCopied'))
      break
    case 'delete':
      await deleteEntry(entry.path)
      showUndo(entry.name, entry.name)
      break
    case 'promote':
      promoteTarget.value = entry
      showPromoteConfirm.value = true
      break
    case 'copy-file':
      clipboardEntry.value = entry
      clipboardMode.value = 'copy'
      break
    case 'cut-file':
      clipboardEntry.value = entry
      clipboardMode.value = 'cut'
      message.info(t('scratchpad.cutFileHint'))
      break
    case 'paste-file':
      await handlePaste()
      break
    case 'batch-delete': {
      const paths = [...selectedKeys.value]
      const confirmed = window.confirm(t('scratchpad.batchDeleteConfirm', { n: paths.length }))
      if (!confirmed) break
      for (const p of paths) {
        await deleteEntry(p)
      }
      message.success(t('scratchpad.batchDeletedSuccess', { n: paths.length }))
      selectedKeys.value = new Set<string>()
      break
    }
  }
}

async function handlePaste(): Promise<void> {
  const src = clipboardEntry.value
  if (!src) return

  if (clipboardMode.value === 'cut') {
    const fromRelPath = src.path.replace(scratchpadPath.value || '', '').replace(/^\//, '').replace(/\\/g, '/')
    const selectedFolder = findSelectedFolder()
    const toParent = selectedFolder || ''
    const entry = await moveEntry(fromRelPath, toParent)
    if (entry) {
      clipboardEntry.value = null
      clipboardMode.value = 'copy'
      showMoveUndo(fromRelPath, toParent, entry.name)
      message.success(t('scratchpad.movedSuccess', { name: entry.name }))
    }
    return
  }

  const content = await loadFileContent(src.path.replace(scratchpadPath.value || '', '').replace(/^\//, ''))
  if (content === null) return
  const baseName = src.name.replace(/\.([^.]+)$/, '')
  const ext = src.name.includes('.') ? '.' + src.name.split('.').pop() : ''
  const copyName = `${baseName}_copy${ext}`
  const entry = await createEntry(copyName, false)
  if (entry) {
    const relPath = entry.path.replace(scratchpadPath.value || '', '').replace(/^\//, '')
    await saveFile(relPath, content)
    message.success(t('scratchpad.pasteCopied', { name: copyName }))
  }
}

function showMoveUndo(fromPath: string, toParent: string, name: string): void {
  dismissMoveUndo()
  moveUndoVisible.value = true
  moveUndoFromPath.value = fromPath
  moveUndoToParent.value = toParent
  moveUndoName.value = name
  moveUndoTimer = setTimeout(() => {
    dismissMoveUndo()
  }, MOVE_UNDO_TIMEOUT)
}

function dismissMoveUndo(): void {
  moveUndoVisible.value = false
  if (moveUndoTimer) {
    clearTimeout(moveUndoTimer)
    moveUndoTimer = null
  }
}

async function handleMoveUndo(): Promise<void> {
  const fromPath = moveUndoFromPath.value
  const toParent = moveUndoToParent.value
  dismissMoveUndo()
  const entry = await moveEntry(
    `${toParent ? toParent + '/' : ''}${moveUndoName.value}`,
    getParentPathOfEntry(fromPath) || ''
  )
  if (entry) {
    message.success(t('scratchpad.undoMoveSuccess', { name: entry.name }))
  }
}

async function handlePromoteConfirm(removeAfter: boolean): Promise<void> {
  showPromoteConfirm.value = false
  const entry = promoteTarget.value
  if (!entry) return
  const scratchpadBase = scratchpadPath.value || ''
  const relativePath = scratchpadBase
    ? entry.path.replace(scratchpadBase.replace(/\\/g, '/'), '').replace(/^\//, '')
    : entry.path
  const result = await promoteToResource(relativePath, removeAfter)
  promoteTarget.value = null
  if (result) message.success(t('scratchpad.promotedSuccess', { name: entry.name }))
}

function handleConflictReload(): void {
  const path = conflictFilePath.value
  showConflictDialog.value = false
  conflictFilePath.value = null
  if (path) {
    markClean(path)
    dismissConflict(path)
    handleOpenByPath(path)
  }
}

function handleConflictIgnore(): void {
  const path = conflictFilePath.value
  showConflictDialog.value = false
  conflictFilePath.value = null
  if (path) dismissConflict(path)
}

async function handleConflictDiff(): Promise<void> {
  const path = conflictFilePath.value
  if (!path) return
  showConflictDialog.value = false
  try {
    const entry = findEntryInTree(localEntries.value, path)
    if (!entry) return
    const editorSnapshot = getContentSnapshot(path)
    if (editorSnapshot === null) {
      handleConflictReload()
      return
    }
    const result = await diffWithContent(
      path,
      editorSnapshot,
      t('scratchpad.diffCurrentFile'),
      t('scratchpad.diffEditedContent')
    )
    if (result) {
      diffResult.value = result
      diffFilePath.value = path
      diffLeftLabel.value = t('scratchpad.diffCurrentFile')
      diffRightLabel.value = t('scratchpad.diffEditedContent')
      showDiffDialog.value = true
    }
  } catch {
    handleConflictReload()
  }
}

async function handleDiffAcceptRight(): Promise<void> {
  const path = diffFilePath.value
  showDiffDialog.value = false
  if (!path) return
  markClean(path)
  dismissConflict(path)
  handleOpenByPath(path)
}

function handleDiffClose(): void {
  showDiffDialog.value = false
  if (diffFilePath.value) {
    showConflictDialog.value = true
  }
}

async function handleOpenByPath(path: string): Promise<void> {
  const entry = findEntryInTree(localEntries.value, path)
  if (entry && entry.kind === 'file') {
    handleOpen(entry)
  }
}

watch(externalConflicts, (newConflicts) => {
  if (newConflicts.length > 0 && !showConflictDialog.value) {
    conflictFilePath.value = newConflicts[0]
    showConflictDialog.value = true
  }
})

function startRename(entry: ScratchpadEntry): void {
  renamingKey.value = entry.path
}

async function finishRename(entry: ScratchpadEntry, newName: string): Promise<void> {
  renamingKey.value = null
  if (newName && newName !== entry.name) {
    const result = await renameEntry(entry.path, newName)
    if (result) message.success(t('scratchpad.renamedSuccess', { name: newName }))
  }
}

function cancelRename(): void {
  renamingKey.value = null
}

async function handleKeydown(event: KeyboardEvent): Promise<void> {
  if ((event.ctrlKey || event.metaKey) && event.key === 'f') {
    event.preventDefault()
    if (!showExplorerFilter.value) {
      toggleExplorerFilter()
    }
    return
  }
  if ((event.ctrlKey || event.metaKey) && event.key === 'n') {
    event.preventDefault()
    handleCreateFile()
    return
  }
  if ((event.ctrlKey || event.metaKey) && event.key === 'a') {
    event.preventDefault()
    const all = new Set<string>()
    for (const item of flattenedTree.value) {
      all.add(item.entry.path)
    }
    selectedKeys.value = all
    return
  }
  if (!selectedKey.value) {
    if (event.key === 'ArrowDown' || event.key === 'ArrowUp') {
      event.preventDefault()
      const entries = flattenedTree.value.map(item => item.entry)
      if (entries.length > 0) {
        selectedKey.value = event.key === 'ArrowDown' ? entries[0].path : entries[entries.length - 1].path
        scrollToSelected()
      }
    }
    return
  }

  if (event.key === 'ArrowDown' || event.key === 'ArrowUp') {
    event.preventDefault()
    const entries = flattenedTree.value.map(item => item.entry)
    const idx = entries.findIndex(e => e.path === selectedKey.value)
    if (idx === -1) return
    const nextIdx = event.key === 'ArrowDown'
      ? Math.min(idx + 1, entries.length - 1)
      : Math.max(idx - 1, 0)
    selectedKey.value = entries[nextIdx].path
    scrollToSelected()
    return
  }

  if (event.key === 'Enter') {
    event.preventDefault()
    const entry = localEntries.value.find(e => e.path === selectedKey.value)
    if (entry) handleOpen(entry)
    return
  }

  if (event.key === 'F2') {
    event.preventDefault()
    const entry = localEntries.value.find(e => e.path === selectedKey.value)
    if (entry) startRename(entry)
  } else if (event.key === 'Delete') {
    event.preventDefault()
    if (multiSelected.value > 1) {
      const paths = [...selectedKeys.value]
      const confirmed = window.confirm(t('scratchpad.batchDeleteConfirm', { n: paths.length }))
      if (!confirmed) return
      for (const p of paths) {
        await deleteEntry(p)
      }
      message.success(t('scratchpad.batchDeletedSuccess', { n: paths.length }))
      selectedKeys.value = new Set<string>()
    } else {
      const entry = localEntries.value.find(e => e.path === selectedKey.value)
      if (entry) {
        await deleteEntry(entry.path)
        showUndo(entry.name, entry.name)
      }
    }
  }
}

function scrollToSelected(): void {
  nextTick(() => {
    const el = document.querySelector('.node-row.selected')
    el?.scrollIntoView({ block: 'nearest' })
  })
}

let unlisten: (() => void) | null = null
let resizeObserver: ResizeObserver | null = null

onMounted(async () => {
  await loadFiles()
  await nextTick()
  updateTreeContainerSize()
  resizeObserver = new ResizeObserver(() => updateTreeContainerSize())
  if (treeContainerRef.value) {
    resizeObserver.observe(treeContainerRef.value)
  }
  document.addEventListener('click', closeContextMenu)
  document.addEventListener('keydown', handleKeydown)
  window.addEventListener('project-switched', loadFiles)
  await startWatching()
  try {
    const unlistenFn = await listen<ScratchpadChangeEvent>('scratchpad-changed', (event) => {
      const payload = event.payload
      if (!payload.changes || payload.changes.length === 0) {
        return
      }
      const savedExpanded = new Set(expandedKeys.value)
      const savedSelected = selectedKey.value
      applyFileChanges(payload).then(() => {
        expandedKeys.value = savedExpanded
        selectedKey.value = savedSelected
      })
    })
    unlisten = unlistenFn
  } catch {
    // event listener setup failed, watcher still works for in-app changes
  }
})

onUnmounted(() => {
  resizeObserver?.disconnect()
  document.removeEventListener('click', closeContextMenu)
  document.removeEventListener('keydown', handleKeydown)
  window.removeEventListener('project-switched', loadFiles)
  if (unlisten) {
    unlisten()
    unlisten = null
  }
  stopWatching()
})
</script>

<style scoped>
.scratchpad-panel {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: var(--dv-background-color, #1e1f22);
  overflow: hidden;
  position: relative;
}

.scratchpad-panel.dragover-active::after {
  content: attr(data-drop-hint);
  position: absolute;
  inset: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--brand-accent-soft);
  border: 2px dashed var(--brand-accent);
  color: var(--brand-accent);
  font-size: var(--font-size-lg);
  font-weight: 600;
  z-index: 100;
  pointer-events: none;
}

.explorer-titlebar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  height: 22px;
  padding: 0 var(--spacing-sm) 0 var(--spacing-md);
  flex-shrink: 0;
  border-bottom: 1px solid var(--color-border-subtle);
}

.explorer-titlebar-label {
  font-size: var(--font-size-xs);
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  color: var(--color-text-muted);
  user-select: none;
}

.explorer-titlebar-actions {
  display: flex;
  align-items: center;
  gap: 1px;
}

.scratchpad-toolbar {
  display: flex;
  align-items: center;
  padding: 0 var(--spacing-xs);
  background: var(--color-bg-secondary);
  border-bottom: 1px solid var(--color-border-subtle);
  gap: 1px;
  min-height: 28px;
  flex-shrink: 0;
}

.scratchpad-explorer-body {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  background: var(--dv-background-color, #1e1f22);
}

.toolbar-btn {
  width: 22px;
  height: 22px;
  border: none;
  border-radius: var(--border-radius-sm);
  background: transparent;
  color: var(--color-text-secondary);
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition:
    background 0.1s ease,
    color 0.1s ease;
  padding: 0;
  outline: none;
}

.toolbar-btn:focus-visible {
  box-shadow: 0 0 0 1px var(--primary-color);
  z-index: 1;
}

.toolbar-btn:hover:not(:disabled) {
  background: var(--color-hover);
  color: var(--color-text-primary);
}

.toolbar-btn:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

.toolbar-btn-primary {
  color: var(--color-text-primary);
}

.toolbar-btn-active {
  background: var(--color-hover);
  color: var(--color-text-primary);
}

.toolbar-separator {
  width: 1px;
  height: 16px;
  background: var(--color-border-subtle);
  margin: 0 var(--spacing-xs);
}

.icon-spin {
  animation: toolbar-spin 0.8s linear infinite;
}

@keyframes toolbar-spin {
  from {
    transform: rotate(0deg);
  }
  to {
    transform: rotate(360deg);
  }
}

.search-panel {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  background: var(--color-bg-primary);
}

.search-panel-header {
  display: flex;
  align-items: center;
  gap: var(--spacing-xs);
  padding: var(--spacing-xs) var(--spacing-sm);
  border-bottom: 1px solid var(--color-border-subtle);
  flex-shrink: 0;
}

.search-input-wrapper {
  flex: 1;
  display: flex;
  align-items: center;
  gap: var(--spacing-xs);
}

.search-panel-input {
  flex: 1;
}

.search-panel-input :deep(.n-input) {
  --n-border: var(--color-border-subtle);
  --n-border-hover: var(--primary-color);
  --n-border-focus: var(--primary-color);
  --n-color: var(--color-bg-tertiary);
  --n-color-focus: var(--color-bg-primary);
  --n-text-color: var(--color-text-primary);
  --n-placeholder-color: var(--color-text-muted);
}

.search-toggle-buttons {
  display: flex;
  align-items: center;
  gap: 1px;
}

.toggle-btn {
  height: 22px;
  padding: 0 var(--spacing-xs);
  border: none;
  background: transparent;
  color: var(--color-text-muted);
  font-size: var(--font-size-xs);
  font-weight: 600;
  cursor: pointer;
  border-radius: var(--border-radius-sm);
  display: flex;
  align-items: center;
  justify-content: center;
  transition: background 0.15s, color 0.15s;
}

.toggle-btn:hover {
  background: var(--color-hover);
  color: var(--color-text-secondary);
}

.toggle-btn-active {
  background: var(--brand-accent-soft);
  color: var(--brand-accent);
}

.search-filter-toggle {
  display: flex;
  align-items: center;
  gap: var(--spacing-xs);
  padding: var(--spacing-xs) var(--spacing-sm);
  cursor: pointer;
  font-size: var(--font-size-xs);
  color: var(--color-text-muted);
  user-select: none;
  transition: color 0.15s;
}

.search-filter-toggle:hover {
  color: var(--color-text-secondary);
}

.search-filters {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-xs);
  padding: 0 var(--spacing-sm) var(--spacing-xs);
}

.search-filter-input {
  width: 100%;
}

.search-filter-input :deep(.n-input) {
  --n-border: var(--color-border-subtle);
  --n-border-hover: var(--primary-color);
  --n-border-focus: var(--primary-color);
  --n-color: var(--color-bg-tertiary);
  --n-color-focus: var(--color-bg-primary);
  --n-text-color: var(--color-text-primary);
  --n-placeholder-color: var(--color-text-muted);
}

.search-replace-bar {
  display: flex;
  align-items: center;
  gap: var(--spacing-xs);
  padding: var(--spacing-xs) var(--spacing-sm);
  border-top: 1px solid var(--color-border-subtle);
}

.search-replace-input {
  flex: 1;
}

.search-replace-input :deep(.n-input) {
  --n-border: var(--color-border-subtle);
  --n-border-hover: var(--primary-color);
  --n-border-focus: var(--primary-color);
  --n-color: var(--color-bg-tertiary);
  --n-color-focus: var(--color-bg-primary);
  --n-text-color: var(--color-text-primary);
  --n-placeholder-color: var(--color-text-muted);
}

.search-results {
  flex: 1;
  overflow-y: auto;
  padding: var(--spacing-xs) 0;
}

.search-result-group {
  margin-bottom: 1px;
}

.search-result-file-header {
  display: flex;
  align-items: center;
  gap: var(--spacing-xs);
  height: 22px;
  padding: 0 var(--spacing-sm);
  cursor: pointer;
  user-select: none;
  transition: background-color 0.15s;
}

.search-result-file-header:hover {
  background-color: var(--color-hover);
}

.search-result-file-name {
  flex: 1;
  font-size: var(--font-size-md);
  color: var(--color-text-primary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.search-result-match-count {
  font-size: var(--font-size-xs);
  color: var(--color-text-muted);
  background: var(--color-bg-tertiary);
  padding: 0 var(--spacing-xs);
  border-radius: var(--border-radius-sm);
}

.search-result-matches {
  padding: 0;
}

.search-result-line {
  display: flex;
  align-items: center;
  gap: var(--spacing-sm);
  height: 22px;
  padding: 0 var(--spacing-sm) 0 var(--spacing-xl);
  cursor: pointer;
  transition: background-color 0.15s;
}

.search-result-line:hover {
  background-color: var(--color-hover);
}

.search-result-line-number {
  font-size: var(--font-size-xs);
  color: var(--color-text-muted);
  min-width: 28px;
  text-align: right;
  font-family: var(--font-monospace);
  flex-shrink: 0;
}

.search-result-line-content {
  font-size: var(--font-size-md);
  font-family: var(--font-monospace);
  color: var(--color-text-secondary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  flex: 1;
}

.search-results-footer {
  padding: var(--spacing-xs) var(--spacing-sm);
  font-size: var(--font-size-xs);
  color: var(--color-text-muted);
  border-top: 1px solid var(--color-border-subtle);
}

.search-hl {
  background: var(--brand-accent-soft);
  color: var(--brand-accent);
  border-radius: 1px;
}

.search-notice {
  display: flex;
  align-items: center;
  gap: var(--spacing-xs);
  padding: var(--spacing-xs) var(--spacing-sm);
  font-size: var(--font-size-xs);
}

.search-notice-error {
  color: var(--brand-danger);
  background: color-mix(in srgb, var(--brand-danger) 10%, transparent);
}

.search-no-results {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: var(--spacing-lg);
  font-size: var(--font-size-md);
  color: var(--color-text-muted);
}

.inline-tree-entry {
  display: flex;
  align-items: center;
  gap: var(--spacing-xs);
  height: 22px;
  padding: 0 var(--spacing-sm) 0 var(--spacing-lg);
}

.inline-tree-entry .rename-input {
  flex: 1;
  height: 18px;
  padding: 0 3px;
  font-size: var(--font-size-md);
  font-family: var(--font-family);
  border: 1px solid var(--primary-color);
  border-radius: 1px;
  outline: none;
  background: var(--color-bg-primary);
  color: var(--color-text-primary);
}

.tree-container {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  min-height: 0;
}

.virtual-scroll-viewport {
  position: relative;
  width: 100%;
}

.virtual-scroll-spacer {
  width: 100%;
}

.loading-state,
.error-state {
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 24px;
  gap: var(--spacing-sm);
}

.error-text {
  color: var(--brand-danger);
  font-size: var(--font-size-md);
}

.pane-section {
  display: flex;
  flex-direction: column;
  border-top: 1px solid var(--color-border-subtle);
  overflow: hidden;
}

.pane-section:first-child {
  border-top: none;
}

.pane-section-main {
  min-height: 0;
}

.pane-section-main .pane-section-body {
  flex: 1;
  overflow-y: auto;
  min-height: 0;
}

.pane-section-header {
  display: flex;
  align-items: center;
  gap: var(--spacing-xs);
  height: 26px;
  padding: 0 var(--spacing-sm) 0 var(--spacing-sm);
  cursor: pointer;
  font-size: var(--font-size-md);
  font-weight: 600;
  color: var(--color-text-secondary);
  transition: background-color 0.15s, color 0.15s;
  user-select: none;
}

.pane-section-header:hover {
  background-color: var(--color-hover);
}

.pane-section-title {
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.pane-section-actions {
  display: flex;
  gap: 1px;
  margin-left: auto;
  font-size: var(--font-size-xs);
  opacity: 0;
  transition: opacity 0.15s;
}

.pane-section-header:hover .pane-section-actions {
  opacity: 1;
}

.pane-section-actions .n-button {
  font-size: var(--font-size-xs);
  padding: 0 var(--spacing-xs);
  height: 18px;
}

.pane-section-body {
  padding: 0;
}

.section-resize-handle {
  height: 4px;
  cursor: row-resize;
  background: transparent;
  transition: background 0.15s;
  flex-shrink: 0;
  position: relative;
}

.section-resize-handle:hover,
.section-resize-active {
  background: var(--primary-color);
}

.ref-row {
  display: flex;
  align-items: center;
  gap: var(--spacing-xs);
  height: 22px;
  padding: 0 var(--spacing-sm) 0 var(--spacing-lg);
  font-size: var(--font-size-md);
  cursor: pointer;
  transition: background-color 0.15s;
}

.ref-row:hover {
  background-color: var(--color-hover);
}

.ref-name {
  color: var(--color-text-primary);
  font-weight: 400;
}

.ref-path {
  font-size: var(--font-size-xs);
  color: var(--color-text-muted);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  flex: 1;
}

.ref-path-invalid {
  color: var(--brand-danger);
  text-decoration: line-through;
}

.ref-badge {
  font-size: var(--font-size-xxs);
  color: var(--brand-danger);
  background: var(--brand-danger-soft);
  padding: 0 var(--spacing-xs);
  border-radius: var(--border-radius-sm);
  flex-shrink: 0;
}

.ref-invalid {
  color: var(--brand-danger);
}

.empty-hint {
  font-size: var(--font-size-md);
  color: var(--color-text-muted);
  padding: var(--spacing-sm) var(--spacing-lg);
}

.recent-section {
  border-bottom: 1px solid var(--color-border-subtle);
}

.recent-header {
  display: flex;
  align-items: center;
  gap: var(--spacing-xs);
  height: 22px;
  padding: 0 var(--spacing-md) 0 var(--spacing-lg);
  cursor: pointer;
  user-select: none;
}

.recent-header:hover {
  background-color: var(--color-hover);
}

.recent-title {
  font-size: var(--font-size-md);
  font-weight: 400;
  color: var(--color-text-secondary);
}

.recent-count {
  font-size: var(--font-size-xs);
  color: var(--color-text-muted);
}

.recent-list {
  padding-bottom: var(--spacing-xs);
}

.recent-entry {
  display: flex;
  align-items: center;
  gap: var(--spacing-xs);
  height: 22px;
  padding: 0 var(--spacing-sm) 0 var(--spacing-xl);
  cursor: pointer;
  font-size: var(--font-size-md);
  color: var(--color-text-primary);
  transition: background-color 0.15s;
}

.recent-entry:hover {
  background-color: var(--color-hover);
}

.recent-name {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  font-size: var(--font-size-md);
}

.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: var(--spacing-xl) var(--spacing-lg);
  gap: var(--spacing-sm);
}

.empty-icon-wrapper {
  color: var(--color-text-muted);
  opacity: 0.3;
  margin-bottom: var(--spacing-xs);
}

.empty-title {
  font-size: var(--font-size-md);
  font-weight: 400;
  color: var(--color-text-muted);
}

.empty-hint {
  font-size: var(--font-size-sm);
  color: var(--color-text-muted);
  padding: 0;
  text-align: center;
}

.empty-actions {
  display: flex;
  gap: var(--spacing-sm);
  margin-top: var(--spacing-sm);
}

.modal-body {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-sm);
  padding: var(--spacing-lg);
  min-width: 360px;
}

.template-picker {
  display: flex;
  align-items: center;
  gap: var(--spacing-sm);
}

.template-label {
  font-size: var(--font-size-md);
  color: var(--color-text-secondary);
  white-space: nowrap;
}

.template-options {
  display: flex;
  gap: var(--spacing-xs);
}

.modal-input {
  width: 100%;
}

.modal-actions {
  display: flex;
  justify-content: flex-end;
  gap: var(--spacing-sm);
}

.scratchpad-context-menu {
  position: fixed;
  z-index: 1000;
  background: var(--color-bg-elevated);
  border: 1px solid var(--color-border);
  border-radius: var(--border-radius-md);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.2);
  min-width: 180px;
  padding: var(--spacing-xs) 0;
}

.menu-item {
  display: flex;
  align-items: center;
  gap: var(--spacing-sm);
  height: 24px;
  padding: 0 var(--spacing-md);
  font-size: var(--font-size-md);
  cursor: pointer;
  transition: background-color 0.15s;
  color: var(--color-text-primary);
}

.menu-item:hover {
  background-color: var(--color-hover);
}

.menu-item-danger {
  color: var(--brand-danger);
}

.menu-item-danger:hover {
  background-color: var(--brand-accent-soft);
}

.menu-shortcut {
  margin-left: auto;
  font-size: var(--font-size-xs);
  color: var(--color-text-muted);
}

.menu-item-danger .menu-shortcut {
  opacity: 0.6;
}

.undo-bar {
  position: absolute;
  bottom: 0;
  left: 0;
  right: 0;
  display: flex;
  align-items: center;
  gap: var(--spacing-sm);
  padding: var(--spacing-sm) var(--spacing-md);
  background: var(--color-bg-elevated);
  border-top: 1px solid var(--color-border);
  font-size: var(--font-size-md);
  z-index: 10;
  animation: undo-bar-in 0.2s ease-out;
}

@keyframes undo-bar-in {
  from {
    transform: translateY(100%);
    opacity: 0;
  }
  to {
    transform: translateY(0);
    opacity: 1;
  }
}

.undo-text {
  flex: 1;
  color: var(--color-text-secondary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.move-undo-bar {
  background: var(--color-bg-info-soft);
  border-top: 1px solid var(--color-border-info);
}

.conflict-actions {
  display: flex;
  gap: var(--spacing-sm);
  justify-content: center;
  margin-top: var(--spacing-md);
}

.replace-bar {
  display: flex;
  align-items: center;
  gap: var(--spacing-sm);
  padding: var(--spacing-sm) var(--spacing-lg);
  border-bottom: 1px solid var(--color-border-subtle);
  background: var(--color-bg-secondary);
}

.replace-input {
  flex: 1;
}

.replace-preview {
  font-size: var(--font-size-md);
  color: var(--color-text-muted);
  white-space: nowrap;
}

.diff-container {
  max-height: 60vh;
  overflow: auto;
  font-family: 'Consolas', 'Courier New', monospace;
  font-size: 13px;
  line-height: 1.5;
}

.diff-labels {
  display: flex;
  border-bottom: 1px solid var(--color-border);
  background: var(--color-bg-secondary);
  position: sticky;
  top: 0;
  z-index: 1;
}

.diff-label-left,
.diff-label-right {
  flex: 1;
  padding: var(--spacing-xs) var(--spacing-sm);
  font-weight: 600;
  color: var(--color-text-secondary);
  border-right: 1px solid var(--color-border-subtle);
}

.diff-label-right {
  border-right: none;
}

.diff-lines {
  border: 1px solid var(--color-border-subtle);
  border-top: none;
}

.diff-line {
  display: flex;
  min-height: 22px;
}

.diff-line-num {
  width: 44px;
  padding: 0 var(--spacing-xs);
  text-align: right;
  color: var(--color-text-muted);
  background: var(--color-bg-tertiary);
  border-right: 1px solid var(--color-border-subtle);
  user-select: none;
  flex-shrink: 0;
}

.diff-num-right {
  border-right: none;
}

.diff-line-content {
  flex: 1;
  padding: 0 var(--spacing-sm);
  white-space: pre;
  overflow: hidden;
  text-overflow: ellipsis;
}

.diff-unchanged {
  background: transparent;
}

.diff-added {
  background: var(--color-bg-success-soft, rgba(34, 197, 94, 0.1));
}

.diff-added .diff-line-content::before {
  content: '+ ';
  color: var(--brand-success, #22c55e);
  font-weight: bold;
}

.diff-removed {
  background: var(--color-bg-danger-soft, rgba(239, 68, 68, 0.1));
}

.diff-removed .diff-line-content::before {
  content: '- ';
  color: var(--brand-danger, #ef4444);
  font-weight: bold;
}

.diff-unchanged .diff-line-content::before {
  content: '  ';
}
</style>
