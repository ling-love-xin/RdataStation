<template>
  <div class="multi-tab-results">
    <!-- Tab 栏 -->
    <div v-if="results.length > 0" class="tab-bar">
      <NTabs
        v-model:value="activeTab"
        type="card"
        size="small"
        animated
        closable
        @close="handleCloseTab"
      >
        <NTab
          v-for="(result, index) in results"
          :key="result.id"
          :name="result.id"
          :tab="result.tabName"
        >
          <template #label>
            <div class="tab-label">
              <span class="tab-index">#{{ index + 1 }}</span>
              <span class="tab-name">{{ result.tabName }}</span>
              <NTag
                v-if="result.error"
                size="small"
                type="error"
                class="tab-status"
              >
                错误
              </NTag>
              <NTag
                v-else-if="result.success"
                size="small"
                type="success"
                class="tab-status"
              >
                {{ result.rowCount }} 行
              </NTag>
            </div>
          </template>
        </NTab>
      </NTabs>
    </div>

    <!-- 结果内容区 -->
    <div class="tab-content">
      <template v-if="results.length === 0">
        <div class="empty-state">
          <NEmpty description="执行 SQL 查看结果" />
        </div>
      </template>
      
      <template v-else>
        <div
          v-for="result in results"
          v-show="result.id === activeTab"
          :key="result.id"
          class="result-panel-wrapper"
        >
          <!-- 成功结果 -->
          <QueryResultPanel
            v-if="result.success && result.data"
            :result="result.data"
          />
          
          <!-- 错误结果 -->
          <div v-else-if="result.error" class="error-state">
            <NAlert type="error" :title="result.errorTitle || '执行错误'">
              <pre class="error-message">{{ result.error }}</pre>
            </NAlert>
          </div>
          
          <!-- 执行中 -->
          <div v-else class="loading-state">
            <NSpin size="large">
              <template #description>
                正在执行...
              </template>
            </NSpin>
          </div>
        </div>
      </template>
    </div>

    <!-- 底部状态栏 -->
    <div v-if="results.length > 0" class="status-bar">
      <span class="status-item">
        共 {{ results.length }} 个语句
      </span>
      <span class="status-item">
        成功: {{ successCount }}
      </span>
      <span v-if="errorCount > 0" class="status-item error">
        失败: {{ errorCount }}
      </span>
      <span class="status-item">
        总耗时: {{ totalExecutionTime }}ms
      </span>
    </div>
  </div>
</template>

<script setup lang="ts">
import { NTabs, NTab, NTag, NEmpty, NAlert, NSpin } from 'naive-ui'
import { ref, computed, watch } from 'vue'

import QueryResultPanel from './QueryResultPanel.vue'

interface QueryResultData {
  columns: string[]
  rows: unknown[][]
  totalRows?: number
  executionTime?: number
  rowCount?: number
  affectedRows?: number
}

interface TabResult {
  id: string
  tabName: string
  statementIndex: number
  success: boolean
  error?: string
  errorTitle?: string
  data?: QueryResultData | null
  rowCount: number
  executionTime: number
}

interface Props {
  results?: Array<{
    index: number
    result: QueryResultData | null
    error: string | null
  }>
  isExecuting?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  results: () => [],
  isExecuting: false
})

const activeTab = ref<string>('')
const tabResults = ref<TabResult[]>([])

// 监听全局多 Tab 结果更新事件
const _handleMultiTabResultUpdate = (event: CustomEvent) => {
  if (event.detail?.results) {
    // 转换为 TabResult 格式
    tabResults.value = event.detail.results.map((item: { index: number; result: QueryResultData | null; error: string | null }, _index: number) => {
      const id = `result-${item.index}`
      const rowCount = item.result?.rowCount || item.result?.rows?.length || 0
      const executionTime = item.result?.executionTime || 0
      
      return {
        id,
        tabName: `语句 ${item.index + 1}`,
        statementIndex: item.index,
        success: !item.error,
        error: item.error || undefined,
        errorTitle: item.error ? `语句 ${item.index + 1} 执行失败` : undefined,
        data: item.result,
        rowCount,
        executionTime
      }
    })

    // 设置默认激活的 tab
    if (tabResults.value.length > 0) {
      const firstError = tabResults.value.find(r => r.error)
      activeTab.value = firstError?.id || tabResults.value[0].id
    }
  }
}

// 计算属性
const successCount = computed(() => tabResults.value.filter(r => r.success).length)
const errorCount = computed(() => tabResults.value.filter(r => r.error).length)
const totalExecutionTime = computed(() => 
  tabResults.value.reduce((sum, r) => sum + r.executionTime, 0)
)

// 监听 props.results 变化
watch(() => props.results, (newResults) => {
  if (!newResults || newResults.length === 0) {
    tabResults.value = []
    activeTab.value = ''
    return
  }

  // 转换为 TabResult 格式
  tabResults.value = newResults.map((item, _index) => {
    const id = `result-${item.index}`
    const rowCount = item.result?.rowCount || item.result?.rows?.length || 0
    const executionTime = item.result?.executionTime || 0
    
    return {
      id,
      tabName: `语句 ${item.index + 1}`,
      statementIndex: item.index,
      success: !item.error,
      error: item.error || undefined,
      errorTitle: item.error ? `语句 ${item.index + 1} 执行失败` : undefined,
      data: item.result,
      rowCount,
      executionTime
    }
  })

  // 设置默认激活的 tab
  if (tabResults.value.length > 0) {
    // 优先显示第一个错误的 tab
    const firstError = tabResults.value.find(r => r.error)
    activeTab.value = firstError?.id || tabResults.value[0].id
  }
}, { deep: true, immediate: true })

// 关闭 tab
const handleCloseTab = (name: string) => {
  const index = tabResults.value.findIndex(r => r.id === name)
  if (index === -1) return

  tabResults.value.splice(index, 1)

  // 如果关闭的是当前激活的 tab，切换到相邻的 tab
  if (activeTab.value === name) {
    if (tabResults.value.length > 0) {
      const newIndex = Math.min(index, tabResults.value.length - 1)
      activeTab.value = tabResults.value[newIndex].id
    } else {
      activeTab.value = ''
    }
  }
}

// 暴露方法供外部调用
defineExpose({
  setActiveTab(index: number) {
    if (index >= 0 && index < tabResults.value.length) {
      activeTab.value = tabResults.value[index].id
    }
  }
})
</script>

<style scoped>
.multi-tab-results {
  display: flex;
  flex-direction: column;
  height: 100%;
  overflow: hidden;
}

.tab-bar {
  border-bottom: 1px solid var(--n-border-color);
  background: var(--n-color);
}

.tab-label {
  display: flex;
  align-items: center;
  gap: 6px;
}

.tab-index {
  color: var(--n-text-color-3);
  font-size: 12px;
}

.tab-name {
  font-weight: 500;
}

.tab-status {
  margin-left: 4px;
}

.tab-content {
  flex: 1;
  overflow: hidden;
  position: relative;
}

.result-panel-wrapper {
  height: 100%;
  overflow: hidden;
}

.empty-state {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 100%;
}

.error-state {
  padding: 16px;
}

.error-message {
  margin: 0;
  white-space: pre-wrap;
  word-break: break-all;
  font-family: 'Consolas', 'Monaco', monospace;
  font-size: 12px;
  line-height: 1.5;
}

.loading-state {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 100%;
}

.status-bar {
  display: flex;
  align-items: center;
  gap: 16px;
  padding: 4px 12px;
  background: var(--n-color);
  border-top: 1px solid var(--n-border-color);
  font-size: 12px;
  color: var(--n-text-color-3);
}

.status-item.error {
  color: var(--n-error-color);
}
</style>
