<template>
  <NModal :show="modelValue" :mask-closable="true" @update:show="handleClose">
    <div class="add-ds-dialog" :style="dialogStyle">
      <!-- ====== titlebar ====== -->
      <div class="dialog-titlebar">
        <span class="titlebar-text">✦ {{ $t('navigator.addDataSource') }}</span>
        <div class="titlebar-actions">
          <span class="traffic-dot dot-min" />
          <span class="traffic-dot dot-max" />
          <span class="traffic-dot dot-cls" />
          <NButton quaternary circle size="tiny" @click="handleClose">
            <template #icon><X :size="14" /></template>
          </NButton>
        </div>
      </div>

      <!-- ====== body ====== -->
      <div class="dialog-body">
        <!-- 左侧栏占位 -->
        <aside class="sidebar">
          <div class="sidebar-placeholder">
            {{ $t('navigator.databaseTypes') }}
          </div>
        </aside>

        <!-- 右侧主面板 -->
        <div class="main-panel">
          <!-- panel header -->
          <div class="panel-header">
            <!-- 名称 + Scope -->
            <div class="header-row">
              <span class="header-label">{{ $t('navigator.formName') }}</span>
              <NInput
                v-model:value="formName"
                size="small"
                :placeholder="$t('navigator.dataSourceNamePlaceholder')"
                class="header-input-name"
              />
              <div class="scope-checkboxes">
                <NCheckbox :checked="saveGlobal" @update:checked="saveGlobal = $event">
                  {{ $t('navigator.scopeGlobal') }}
                </NCheckbox>
                <NCheckbox :checked="saveProject" @update:checked="saveProject = $event">
                  {{ $t('navigator.scopeProject') }}
                </NCheckbox>
              </div>
            </div>

            <!-- 描述 -->
            <div class="header-row">
              <span class="header-label">{{ $t('navigator.formDescription') }}</span>
              <NInput
                v-model:value="formDesc"
                type="textarea"
                size="small"
                :rows="2"
                :placeholder="$t('navigator.dataSourceDescPlaceholder')"
                class="header-input-full"
              />
            </div>

            <!-- 驱动 + URI -->
            <div class="header-row">
              <span class="header-label">{{ $t('navigator.formDriver') }}</span>
              <NSelect
                :value="selectedDriverId"
                :options="driverOptions"
                size="small"
                class="header-driver-select"
                @update:value="selectedDriverId = $event"
              />
              <span class="uri-label">URI</span>
              <div class="uri-display">mysql://root:****@localhost:3306/mydb</div>
              <NButton text size="tiny" class="uri-edit-btn">
                <template #icon><Edit3 :size="14" /></template>
              </NButton>
            </div>
          </div>

          <!-- Tab 区域占位 -->
          <div class="content-placeholder">
            <span class="placeholder-text">{{ $t('navigator.tabGeneral') }} · {{ $t('navigator.tabNetwork') }} · {{ $t('navigator.tabCapabilities') }} · {{ $t('navigator.tabDriverProps') }} · {{ $t('navigator.tabAdvanced') }}</span>
          </div>
        </div>
      </div>

      <!-- ====== footer ====== -->
      <div class="dialog-footer">
        <div class="test-result" :class="testResultClass">
          <span v-if="testResult">{{ testResult }}</span>
        </div>
        <NButton secondary size="small" @click="handleTest">
          <template #icon><PlugZap :size="14" /></template>
          {{ $t('navigator.testConnection') }}
        </NButton>
        <NButton size="small" @click="handleApply">
          {{ $t('navigator.apply') }}
        </NButton>
        <NButton size="small" @click="handleClose">{{ $t('navigator.cancel') }}</NButton>
        <NButton type="primary" size="small" :disabled="!canSave" @click="handleSave">
          {{ $t('navigator.saveDataSource') }}
        </NButton>
      </div>

      <!-- resize handle -->
      <div class="resize-handle" @mousedown="onResizeStart" />
    </div>
  </NModal>
</template>

<script setup lang="ts">
import { X, Edit3, PlugZap } from 'lucide-vue-next'
import { NButton, NCheckbox, NInput, NModal, NSelect } from 'naive-ui'
import { computed, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'

const { t } = useI18n()

// ====== props / emits ======
interface Props {
  modelValue: boolean
}

const props = defineProps<Props>()

interface Emits {
  (e: 'update:modelValue', v: boolean): void
  (e: 'save', data: Record<string, unknown>): void
}

const emit = defineEmits<Emits>()

// ====== resize ======
const MIN_W = 780
const MIN_H = 520
const MAX_W = 1300
const MAX_H = 950

const dialogWidth = ref(960)
const dialogHeight = ref(640)

const dialogStyle = computed(() => ({
  width: `${dialogWidth.value}px`,
  height: `${dialogHeight.value}px`,
}))

let isResizing = false
let startX = 0
let startY = 0
let startW = 0
let startH = 0

function clamp(v: number, min: number, max: number) {
  return Math.max(min, Math.min(max, v))
}

function onResizeStart(e: MouseEvent) {
  isResizing = true
  startX = e.clientX
  startY = e.clientY
  startW = dialogWidth.value
  startH = dialogHeight.value
  document.addEventListener('mousemove', onResizeMove)
  document.addEventListener('mouseup', onResizeEnd)
  e.preventDefault()
  e.stopPropagation()
}

function onResizeMove(e: MouseEvent) {
  if (!isResizing) return
  dialogWidth.value = clamp(startW + (e.clientX - startX), MIN_W, MAX_W)
  dialogHeight.value = clamp(startH + (e.clientY - startY), MIN_H, MAX_H)
}

function onResizeEnd() {
  isResizing = false
  document.removeEventListener('mousemove', onResizeMove)
  document.removeEventListener('mouseup', onResizeEnd)
}

// ====== state ======
const formName = ref('')
const formDesc = ref('')
const selectedDriverId = ref('')
const saveGlobal = ref(true)
const saveProject = ref(false)
const testResult = ref('')
const testResultClass = ref('')

// ====== placeholder driver options ======
const driverOptions = computed(() => [
  { label: t('navigator.noDriver'), value: '' },
])

// ====== canSave ======
const canSave = computed(
  () => !!formName.value && (saveGlobal.value || saveProject.value),
)

// ====== handlers ======
function handleTest() {
  testResult.value = t('navigator.testSuccessPlaceholder')
  testResultClass.value = 'success'
  setTimeout(() => {
    testResult.value = ''
    testResultClass.value = ''
  }, 8000)
}

function handleSave() {
  emit('save', {
    name: formName.value,
    description: formDesc.value,
    driverId: selectedDriverId.value,
    saveToGlobal: saveGlobal.value,
    saveToProject: saveProject.value,
  })
}

function handleApply() {
  emit('save', {
    name: formName.value,
    description: formDesc.value,
    driverId: selectedDriverId.value,
    saveToGlobal: saveGlobal.value,
    saveToProject: saveProject.value,
    applyOnly: true,
  })
}

function handleClose() {
  emit('update:modelValue', false)
}

// ====== 重置 ======
watch(
  () => props.modelValue,
  (val) => {
    if (val) {
      formName.value = ''
      formDesc.value = ''
      selectedDriverId.value = ''
      saveGlobal.value = true
      saveProject.value = false
      testResult.value = ''
      testResultClass.value = ''
    }
  },
)
</script>

<style scoped>
/* ====== dialog shell ====== */
.add-ds-dialog {
  display: flex;
  flex-direction: column;
  background: var(--color-bg-primary, #1e1e2e);
  border: 1px solid var(--color-border, rgba(255, 255, 255, 0.07));
  border-radius: 12px;
  overflow: hidden;
  position: relative;
  box-shadow: 0 24px 64px rgba(0, 0, 0, 0.5);
  max-height: 95vh;
}

/* ====== titlebar ====== */
.dialog-titlebar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 10px 16px;
  border-bottom: 1px solid var(--color-border, rgba(255, 255, 255, 0.07));
  flex-shrink: 0;
  user-select: none;
}

.titlebar-text {
  font-size: var(--font-size-md, 14px);
  font-weight: 600;
  color: var(--color-text-secondary, #a6adc8);
}

.titlebar-actions {
  display: flex;
  align-items: center;
  gap: 8px;
}

.traffic-dot {
  width: 12px;
  height: 12px;
  border-radius: 50%;
  flex-shrink: 0;
}

.traffic-dot.dot-min { background: #f9e2af; }
.traffic-dot.dot-max { background: #a6e3a1; }
.traffic-dot.dot-cls { background: #f38ba8; }

/* ====== body ====== */
.dialog-body {
  flex: 1;
  display: flex;
  overflow: hidden;
  min-height: 0;
}

/* ====== sidebar ====== */
.sidebar {
  width: 240px;
  min-width: 240px;
  border-right: 1px solid var(--color-border, rgba(255, 255, 255, 0.07));
  overflow-y: auto;
  flex-shrink: 0;
  background: var(--color-bg-secondary, #11111b);
}

.sidebar-placeholder {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 100%;
  font-size: var(--font-size-sm, 12px);
  color: var(--color-text-muted, #6c7086);
}

/* ====== main panel ====== */
.main-panel {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  min-width: 0;
}

/* ====== panel header ====== */
.panel-header {
  padding: 12px 20px 8px;
  border-bottom: 1px solid var(--color-border, rgba(255, 255, 255, 0.07));
  display: flex;
  flex-direction: column;
  gap: 10px;
  flex-shrink: 0;
}

.header-row {
  display: flex;
  align-items: center;
  gap: 8px;
}

.header-label {
  font-size: var(--font-size-sm, 12px);
  font-weight: 600;
  color: var(--color-text-muted, #6c7086);
  width: 40px;
  flex-shrink: 0;
  text-align: right;
}

.header-input-name {
  flex: 1;
  max-width: 260px;
}

.header-input-name :deep(.n-input__input-el) {
  text-align: left;
}

.header-input-full {
  flex: 1;
}

.header-input-full :deep(.n-input__textarea-el) {
  text-align: left;
}

.header-driver-select {
  width: 170px;
  flex-shrink: 0;
}

.scope-checkboxes {
  display: flex;
  align-items: center;
  gap: 12px;
  flex-shrink: 0;
  margin-left: auto;
}

.scope-checkboxes :deep(.n-checkbox) {
  font-size: var(--font-size-sm, 12px);
}

.uri-label {
  font-size: var(--font-size-sm, 12px);
  color: var(--color-text-muted, #6c7086);
  flex-shrink: 0;
  margin-left: 4px;
}

.uri-display {
  flex: 1;
  height: 30px;
  padding: 0 10px;
  background: var(--color-bg-elevated, #1a1b26);
  border: 1px solid var(--color-border-subtle, rgba(255, 255, 255, 0.06));
  border-radius: 6px;
  color: var(--brand-success, #a6e3a1);
  font-size: 11px;
  font-family: var(--font-mono, 'JetBrains Mono', 'Consolas', monospace);
  display: flex;
  align-items: center;
  overflow: hidden;
  white-space: nowrap;
  min-width: 0;
}

.uri-edit-btn {
  flex-shrink: 0;
  color: var(--color-text-muted, #6c7086);
}

.uri-edit-btn:hover {
  color: var(--color-text-primary, #cdd6f4);
}

/* ====== content placeholder ====== */
.content-placeholder {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--color-bg-secondary, #11111b);
}

.placeholder-text {
  font-size: var(--font-size-sm, 12px);
  color: var(--color-text-muted, #6c7086);
}

/* ====== footer ====== */
.dialog-footer {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 10px 20px;
  border-top: 1px solid var(--color-border, rgba(255, 255, 255, 0.07));
  background: var(--color-bg-secondary, #181825);
  flex-shrink: 0;
}

.test-result {
  flex: 1;
  font-size: var(--font-size-sm, 12px);
  display: flex;
  align-items: center;
  min-width: 0;
  overflow: hidden;
  white-space: nowrap;
  text-overflow: ellipsis;
}

.test-result.success { color: var(--brand-success, #a6e3a1); }
.test-result.error { color: var(--brand-danger, #f38ba8); }

/* ====== resize ====== */
.resize-handle {
  position: absolute;
  right: 0;
  bottom: 0;
  width: 16px;
  height: 16px;
  cursor: nwse-resize;
  z-index: 10;
}

.resize-handle::after {
  content: '';
  position: absolute;
  right: 3px;
  bottom: 3px;
  width: 8px;
  height: 8px;
  border-right: 2px solid var(--color-text-muted, #6c7086);
  border-bottom: 2px solid var(--color-text-muted, #6c7086);
  opacity: 0.4;
  transition: opacity 0.15s ease;
}

.resize-handle:hover::after { opacity: 0.8; }
</style>