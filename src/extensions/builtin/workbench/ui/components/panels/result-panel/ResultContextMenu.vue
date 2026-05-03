<template>
  <teleport to="body">
    <div
      v-if="visible"
      class="result-context-menu"
      :style="{ left: x + 'px', top: y + 'px' }"
      @click.stop="close"
      @contextmenu.prevent="close"
    >
      <div v-if="type === 'cell'" class="menu-group">
        <div class="menu-item" @click.stop="emit('action', { action: 'copyCell', value, column })">
          <Copy :size="12" /><span>复制单元格值</span>
        </div>
        <div class="menu-item" @click.stop="emit('action', { action: 'copyRow', value, column })">
          <ClipboardList :size="12" /><span>复制行(TSV)</span>
        </div>
        <div class="menu-item" @click.stop="emit('action', { action: 'copyRowJson', value, column })">
          <FileJson :size="12" /><span>复制为 JSON</span>
        </div>
        <div class="menu-item" @click.stop="emit('action', { action: 'copyRowInsert', value, column })">
          <FileCode :size="12" /><span>复制为 INSERT</span>
        </div>
        <div class="menu-divider" />
        <div class="menu-item" @click.stop="emit('action', { action: 'filterByValue', value, column })">
          <Filter :size="12" /><span>以此值过滤 (即时过滤)</span>
        </div>
        <div class="menu-item" @click.stop="emit('action', { action: 'sqlFilterByValue', value, column })">
          <Database :size="12" /><span>以此值过滤 (SQL过滤)</span>
        </div>
        <div class="menu-divider" />
        <div class="menu-item" @click.stop="emit('action', { action: 'openColumnInsights', value, column })">
          <BarChart3 :size="12" /><span>列洞察</span>
        </div>
      </div>

      <div v-if="type === 'header'" class="menu-group">
        <div class="menu-item" @click.stop="emit('action', { action: 'sortAsc', column })">
          <ArrowUp :size="12" /><span>升序排序</span>
        </div>
        <div class="menu-item" @click.stop="emit('action', { action: 'sortDesc', column })">
          <ArrowDown :size="12" /><span>降序排序</span>
        </div>
        <div class="menu-divider" />
        <div class="menu-item" @click.stop="emit('action', { action: 'sendSortToSql', column, sortDir })">
          <Database :size="12" /><span>发送排序到 SQL 过滤模式</span>
        </div>
        <div class="menu-item" @click.stop="emit('action', { action: 'sendSortToDuckdb', column, sortDir })">
          <Brain :size="12" /><span>发送排序到 DuckDB 分析</span>
        </div>
        <div class="menu-divider" />
        <div class="menu-item" @click.stop="emit('action', { action: 'hideColumn', column })">
          <EyeOff :size="12" /><span>隐藏列</span>
        </div>
        <div class="menu-item" @click.stop="emit('action', { action: 'autoSizeColumn', column })">
          <Maximize2 :size="12" /><span>自适应列宽</span>
        </div>
        <div class="menu-item" @click.stop="emit('action', { action: 'autoSizeAll', column })">
          <Maximize2 :size="12" /><span>自适应所有列宽</span>
        </div>
        <div class="menu-divider" />
        <div class="menu-item" @click.stop="emit('action', { action: 'columnSummary', column })">
          <Calculator :size="12" /><span>列汇总 (发送到 DuckDB)</span>
        </div>
        <div class="menu-item" @click.stop="emit('action', { action: 'openColumnInsights', column })">
          <BarChart3 :size="12" /><span>列洞察面板</span>
        </div>
      </div>
    </div>
  </teleport>
</template>

<script setup lang="ts">
import {
  Copy, ClipboardList, FileJson, FileCode, Filter, Database,
  BarChart3, ArrowUp, ArrowDown, EyeOff, Maximize2, Calculator, Brain
} from 'lucide-vue-next'

defineProps<{
  visible: boolean
  x: number
  y: number
  type: 'cell' | 'header'
  value?: any
  column?: string
  sortDir?: string
}>()

const emit = defineEmits<{
  action: [payload: Record<string, any>]
  close: []
}>()

function close() { emit('close') }
</script>

<style scoped>
.result-context-menu {
  position: fixed;
  z-index: 10000;
  min-width: 200px;
  background: var(--bg-primary, #2d2d30);
  border: 1px solid var(--border-color, #3e3e42);
  border-radius: 6px;
  box-shadow: 0 4px 12px rgba(0,0,0,0.4);
  padding: 4px 0;
  font-size: 12px;
}
.menu-group {}
.menu-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 6px 12px;
  cursor: pointer;
  color: var(--text-primary, #ccc);
  transition: background 0.1s;
}
.menu-item:hover { background: var(--primary-color, #0078d4); color: #fff; }
.menu-divider { height: 1px; background: var(--border-color, #3e3e42); margin: 4px 0; }
</style>
