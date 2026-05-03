<template>
  <div class="toolbar-container">
    <!-- 连接工具组 -->
    <div class="toolbar-group">
      <button class="toolbar-btn" title="新建连接" @click="$emit('new-connection')">
        <Plug :size="16" />
      </button>
      <button class="toolbar-btn" title="断开连接" @click="$emit('disconnect')">
        <Unplug :size="16" />
      </button>
      <div class="toolbar-divider" />
      <button class="toolbar-btn" title="刷新" @click="$emit('refresh')">
        <RefreshCw :size="16" />
      </button>
    </div>

    <!-- SQL 工具组 -->
    <div class="toolbar-group">
      <button class="toolbar-btn primary" title="执行 SQL (Ctrl+Enter)" @click="$emit('execute')">
        <Play :size="16" />
      </button>
      <button class="toolbar-btn" title="执行脚本" @click="$emit('execute-script')">
        <FileCode :size="16" />
      </button>
      <button class="toolbar-btn" title="停止执行" @click="$emit('stop')">
        <Square :size="14" />
      </button>
      <div class="toolbar-divider" />
      <button class="toolbar-btn" title="格式化 SQL" @click="$emit('format')">
        <AlignLeft :size="16" />
      </button>
      <button class="toolbar-btn" title="解释执行计划" @click="$emit('explain')">
        <GitBranch :size="16" />
      </button>
    </div>

    <!-- 事务工具组 -->
    <div class="toolbar-group">
      <button class="toolbar-btn" title="提交事务" @click="$emit('commit')">
        <Check :size="16" />
      </button>
      <button class="toolbar-btn" title="回滚事务" @click="$emit('rollback')">
        <RotateCcw :size="16" />
      </button>
      <div class="toolbar-divider" />
      <button class="toolbar-btn" title="自动提交">
        <ToggleLeft :size="16" />
      </button>
    </div>

    <!-- 数据工具组 -->
    <div class="toolbar-group">
      <button class="toolbar-btn" title="导出数据" @click="$emit('export')">
        <Download :size="16" />
      </button>
      <button class="toolbar-btn" title="导入数据" @click="$emit('import')">
        <Upload :size="16" />
      </button>
      <div class="toolbar-divider" />
      <button class="toolbar-btn" title="筛选">
        <Filter :size="16" />
      </button>
      <button class="toolbar-btn" title="搜索">
        <Search :size="16" />
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import {
  Plug, Unplug, RefreshCw, Play, FileCode, Square,
  AlignLeft, GitBranch, Check, RotateCcw, ToggleLeft,
  Download, Upload, Filter, Search
} from 'lucide-vue-next'

defineEmits<{
  'new-connection': []
  'disconnect': []
  'refresh': []
  'execute': []
  'execute-script': []
  'stop': []
  'format': []
  'explain': []
  'commit': []
  'rollback': []
  'export': []
  'import': []
}>()
</script>

<style scoped>
.toolbar-container {
  height: 40px;
  background: var(--bg-tertiary);
  border-bottom: 1px solid var(--border-color);
  display: flex;
  align-items: center;
  padding: 0 12px;
  gap: 8px;
  flex-shrink: 0;
}

.toolbar-group {
  display: flex;
  align-items: center;
  gap: 2px;
}

.toolbar-divider {
  width: 1px;
  height: 20px;
  background: var(--border-color);
  margin: 0 6px;
}

.toolbar-btn {
  width: 32px;
  height: 32px;
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

.toolbar-btn:hover {
  background: var(--bg-hover);
  color: var(--text-primary);
}

.toolbar-btn.primary {
  background: var(--success-color);
  color: white;
}

.toolbar-btn.primary:hover {
  background: var(--success-hover, #059669);
}
</style>
