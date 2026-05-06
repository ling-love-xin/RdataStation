<template>
  <div class="settings-panel">
    <div class="settings-header">
      <h2>设置</h2>
    </div>

    <div class="settings-nav">
      <button
        v-for="tab in tabs"
        :key="tab.id"
        class="nav-item"
        :class="{ active: activeTab === tab.id }"
        @click="activeTab = tab.id"
      >
        <component :is="tab.icon" :size="16" />
        <span>{{ tab.label }}</span>
      </button>
    </div>

    <div class="settings-content">
      <!-- 连接池设置 -->
      <div v-if="activeTab === 'connection-pool'" class="settings-section">
        <h3>连接池设置</h3>
        
        <div class="setting-item">
          <label>最大连接数</label>
          <input
            type="number"
            v-model.number="settings.connectionPool.maxConnections"
            min="1"
            max="100"
          />
          <span class="hint">控制同时打开的最大数据库连接数</span>
        </div>

        <div class="setting-item">
          <label>最小空闲连接数</label>
          <input
            type="number"
            v-model.number="settings.connectionPool.minIdleConnections"
            min="0"
            max="50"
          />
          <span class="hint">保持的最小空闲连接数</span>
        </div>

        <div class="setting-item">
          <label>连接超时时间（秒）</label>
          <input
            type="number"
            v-model.number="settings.connectionPool.connectionTimeout"
            min="1"
            max="300"
          />
          <span class="hint">建立连接的超时时间</span>
        </div>

        <div class="setting-item">
          <label>连接空闲超时（秒）</label>
          <input
            type="number"
            v-model.number="settings.connectionPool.idleTimeout"
            min="10"
            max="3600"
          />
          <span class="hint">连接空闲后自动释放的时间</span>
        </div>

        <div class="setting-item">
          <label>
            <input type="checkbox" v-model="settings.connectionPool.autoReconnect" />
            自动重连
          </label>
          <span class="hint">连接断开时自动尝试重新连接</span>
        </div>

        <div class="setting-item">
          <label>
            <input type="checkbox" v-model="settings.connectionPool.healthCheck" />
            健康检查
          </label>
          <span class="hint">定期检查连接健康状态</span>
        </div>

        <div class="setting-item" v-if="settings.connectionPool.healthCheck">
          <label>健康检查间隔（秒）</label>
          <input
            type="number"
            v-model.number="settings.connectionPool.healthCheckInterval"
            min="10"
            max="300"
          />
          <span class="hint">健康检查的时间间隔</span>
        </div>
      </div>

      <!-- 操作历史设置 -->
      <div v-if="activeTab === 'history'" class="settings-section">
        <h3>操作历史设置</h3>
        
        <div class="setting-item">
          <label>保留历史记录数量</label>
          <input
            type="number"
            v-model.number="settings.history.maxHistoryItems"
            min="10"
            max="1000"
          />
          <span class="hint">最大保留的历史记录数量</span>
        </div>

        <div class="setting-item">
          <label>历史记录保留天数</label>
          <input
            type="number"
            v-model.number="settings.history.retentionDays"
            min="1"
            max="365"
          />
          <span class="hint">历史记录保留的天数</span>
        </div>

        <div class="setting-item">
          <label>
            <input type="checkbox" v-model="settings.history.enableHistory" />
            启用操作历史
          </label>
          <span class="hint">记录用户的操作历史</span>
        </div>

        <div class="setting-item">
          <label>
            <input type="checkbox" v-model="settings.history.includeSQL" />
            记录 SQL 语句
          </label>
          <span class="hint">在历史记录中包含执行的 SQL 语句</span>
        </div>

        <div class="setting-item">
          <label>
            <input type="checkbox" v-model="settings.history.enableUndo" />
            启用撤销/重做
          </label>
          <span class="hint">支持操作的撤销和重做</span>
        </div>

        <button class="btn-clear-history" @click="clearHistory">
          <Trash2 :size="14" />
          清除所有历史记录
        </button>
      </div>

      <!-- 健康监控设置 -->
      <div v-if="activeTab === 'monitoring'" class="settings-section">
        <h3>健康监控设置</h3>
        
        <div class="setting-item">
          <label>
            <input type="checkbox" v-model="settings.monitoring.enableMonitoring" />
            启用健康监控
          </label>
          <span class="hint">监控数据库连接和系统状态</span>
        </div>

        <div class="setting-item" v-if="settings.monitoring.enableMonitoring">
          <label>监控更新间隔（秒）</label>
          <input
            type="number"
            v-model.number="settings.monitoring.updateInterval"
            min="1"
            max="60"
          />
          <span class="hint">监控数据更新的时间间隔</span>
        </div>

        <div class="setting-item">
          <label>
            <input type="checkbox" v-model="settings.monitoring.enableAlerts" />
            启用告警通知
          </label>
          <span class="hint">当连接状态发生变化时发送通知</span>
        </div>

        <div class="setting-item" v-if="settings.monitoring.enableAlerts">
          <label>
            <input type="checkbox" v-model="settings.monitoring.alertOnDisconnect" />
            连接断开时告警
          </label>
        </div>

        <div class="setting-item" v-if="settings.monitoring.enableAlerts">
          <label>
            <input type="checkbox" v-model="settings.monitoring.alertOnSlowQuery" />
            慢查询告警
          </label>
          <span class="hint">查询执行时间超过阈值时告警</span>
        </div>

        <div class="setting-item" v-if="settings.monitoring.alertOnSlowQuery">
          <label>慢查询阈值（毫秒）</label>
          <input
            type="number"
            v-model.number="settings.monitoring.slowQueryThreshold"
            min="100"
            max="30000"
          />
          <span class="hint">超过此时间的查询被视为慢查询</span>
        </div>
      </div>

      <!-- 性能设置 -->
      <div v-if="activeTab === 'performance'" class="settings-section">
        <h3>性能设置</h3>
        
        <div class="setting-item">
          <label>虚拟滚动缓冲区大小</label>
          <input
            type="number"
            v-model.number="settings.performance.virtualScrollBuffer"
            min="1"
            max="20"
          />
          <span class="hint">虚拟滚动的缓冲区大小（项数）</span>
        </div>

        <div class="setting-item">
          <label>缓存最大大小（MB）</label>
          <input
            type="number"
            v-model.number="settings.performance.maxCacheSize"
            min="10"
            max="500"
          />
          <span class="hint">内存缓存的最大大小</span>
        </div>

        <div class="setting-item">
          <label>缓存过期时间（分钟）</label>
          <input
            type="number"
            v-model.number="settings.performance.cacheExpireMinutes"
            min="5"
            max="1440"
          />
          <span class="hint">缓存数据的过期时间</span>
        </div>

        <div class="setting-item">
          <label>
            <input type="checkbox" v-model="settings.performance.enableLazyLoad" />
            启用懒加载
          </label>
          <span class="hint">只在需要时加载数据</span>
        </div>

        <div class="setting-item">
          <label>
            <input type="checkbox" v-model="settings.performance.enablePreload" />
            启用预加载
          </label>
          <span class="hint">提前加载相邻节点的数据</span>
        </div>
      </div>

      <!-- 快捷键设置 -->
      <div v-if="activeTab === 'shortcuts'" class="settings-section">
        <h3>快捷键设置</h3>
        
        <div class="shortcuts-list">
          <div v-for="shortcut in shortcuts" :key="shortcut.key" class="shortcut-item">
            <span class="shortcut-name">{{ shortcut.name }}</span>
            <input
              type="text"
              v-model="shortcut.value"
              class="shortcut-input"
              readonly
            />
            <button class="btn-edit-shortcut">修改</button>
          </div>
        </div>

        <button class="btn-reset-shortcuts">
          <RotateCcw :size="14" />
          重置为默认
        </button>
      </div>

      <!-- 外观设置 -->
      <div v-if="activeTab === 'appearance'" class="settings-section">
        <h3>外观设置</h3>
        
        <div class="setting-item">
          <label>主题</label>
          <div class="theme-options">
            <button
              v-for="theme in themes"
              :key="theme.id"
              class="theme-option"
              :class="{ active: settings.appearance.theme === theme.id }"
              @click="settings.appearance.theme = theme.id"
            >
              <div class="theme-preview" :class="theme.id"></div>
              <span>{{ theme.name }}</span>
            </button>
          </div>
        </div>

        <div class="setting-item">
          <label>字体大小</label>
          <select v-model="settings.appearance.fontSize">
            <option :value="12">12px</option>
            <option :value="13">13px</option>
            <option :value="14">14px</option>
            <option :value="15">15px</option>
            <option :value="16">16px</option>
          </select>
        </div>

        <div class="setting-item">
          <label>
            <input type="checkbox" v-model="settings.appearance.compactMode" />
            紧凑模式
          </label>
          <span class="hint">减少组件间距，更紧凑的布局</span>
        </div>
      </div>
    </div>

    <div class="settings-footer">
      <button class="btn-cancel" @click="resetSettings">取消</button>
      <button class="btn-save" @click="saveSettings">保存设置</button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive } from 'vue'
import {
  Database,
  History,
  Activity,
  Zap,
  Keyboard,
  Palette,
  Trash2,
  RotateCcw
} from 'lucide-vue-next'

const tabs = [
  { id: 'connection-pool', label: '连接池', icon: Database },
  { id: 'history', label: '操作历史', icon: History },
  { id: 'monitoring', label: '健康监控', icon: Activity },
  { id: 'performance', label: '性能', icon: Zap },
  { id: 'shortcuts', label: '快捷键', icon: Keyboard },
  { id: 'appearance', label: '外观', icon: Palette }
]

const activeTab = ref('connection-pool')

const themes = [
  { id: 'light', name: '浅色' },
  { id: 'dark', name: '深色' },
  { id: 'system', name: '跟随系统' }
]

const shortcuts = reactive([
  { key: 'newConnection', name: '新建连接', value: 'Ctrl+N' },
  { key: 'disconnect', name: '断开连接', value: 'Ctrl+D' },
  { key: 'refresh', name: '刷新', value: 'Ctrl+R' },
  { key: 'search', name: '搜索', value: 'Ctrl+F' },
  { key: 'beginTransaction', name: '开始事务', value: 'Ctrl+B' },
  { key: 'commitTransaction', name: '提交事务', value: 'Ctrl+Shift+B' },
  { key: 'rollbackTransaction', name: '回滚事务', value: 'Ctrl+Shift+R' }
])

const settings = reactive({
  connectionPool: {
    maxConnections: 10,
    minIdleConnections: 2,
    connectionTimeout: 30,
    idleTimeout: 300,
    autoReconnect: true,
    healthCheck: true,
    healthCheckInterval: 60
  },
  history: {
    maxHistoryItems: 100,
    retentionDays: 30,
    enableHistory: true,
    includeSQL: true,
    enableUndo: true
  },
  monitoring: {
    enableMonitoring: true,
    updateInterval: 5,
    enableAlerts: true,
    alertOnDisconnect: true,
    alertOnSlowQuery: true,
    slowQueryThreshold: 1000
  },
  performance: {
    virtualScrollBuffer: 5,
    maxCacheSize: 100,
    cacheExpireMinutes: 60,
    enableLazyLoad: true,
    enablePreload: true
  },
  appearance: {
    theme: 'system',
    fontSize: 13,
    compactMode: false
  }
})

function clearHistory() {
  if (confirm('确定要清除所有历史记录吗？')) {
    console.log('清除历史记录')
  }
}

function resetSettings() {
  Object.assign(settings.connectionPool, {
    maxConnections: 10,
    minIdleConnections: 2,
    connectionTimeout: 30,
    idleTimeout: 300,
    autoReconnect: true,
    healthCheck: true,
    healthCheckInterval: 60
  })
  Object.assign(settings.history, {
    maxHistoryItems: 100,
    retentionDays: 30,
    enableHistory: true,
    includeSQL: true,
    enableUndo: true
  })
  Object.assign(settings.monitoring, {
    enableMonitoring: true,
    updateInterval: 5,
    enableAlerts: true,
    alertOnDisconnect: true,
    alertOnSlowQuery: true,
    slowQueryThreshold: 1000
  })
  Object.assign(settings.performance, {
    virtualScrollBuffer: 5,
    maxCacheSize: 100,
    cacheExpireMinutes: 60,
    enableLazyLoad: true,
    enablePreload: true
  })
  Object.assign(settings.appearance, {
    theme: 'system',
    fontSize: 13,
    compactMode: false
  })
}

function saveSettings() {
  localStorage.setItem('rdata-station-settings', JSON.stringify(settings))
  console.log('设置已保存')
}

function loadSettings() {
  const saved = localStorage.getItem('rdata-station-settings')
  if (saved) {
    try {
      const savedSettings = JSON.parse(saved)
      Object.assign(settings, savedSettings)
    } catch {
      console.error('加载设置失败')
    }
  }
}

loadSettings()
</script>

<style scoped>
.settings-panel {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: var(--bg-primary);
}

.settings-header {
  padding: 16px;
  border-bottom: 1px solid var(--border-color);
}

.settings-header h2 {
  margin: 0;
  font-size: 16px;
  font-weight: 600;
  color: var(--text-primary);
}

.settings-nav {
  display: flex;
  gap: 4px;
  padding: 8px;
  border-bottom: 1px solid var(--border-color);
  overflow-x: auto;
}

.nav-item {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 6px 12px;
  background: transparent;
  border: none;
  border-radius: 4px;
  font-size: 12px;
  color: var(--text-secondary);
  cursor: pointer;
  transition: all 0.15s;
  white-space: nowrap;
}

.nav-item:hover {
  background: var(--bg-tertiary);
  color: var(--text-primary);
}

.nav-item.active {
  background: var(--primary-color);
  color: white;
}

.settings-content {
  flex: 1;
  overflow-y: auto;
  padding: 16px;
}

.settings-section {
  animation: fadeIn 0.2s ease;
}

@keyframes fadeIn {
  from {
    opacity: 0;
  }
  to {
    opacity: 1;
  }
}

.settings-section h3 {
  font-size: 14px;
  font-weight: 600;
  color: var(--text-primary);
  margin: 0 0 16px 0;
  padding-bottom: 8px;
  border-bottom: 1px solid var(--border-color);
}

.setting-item {
  margin-bottom: 16px;
}

.setting-item label {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 13px;
  color: var(--text-primary);
  margin-bottom: 6px;
  cursor: pointer;
}

.setting-item input[type="number"],
.setting-item select {
  width: 150px;
  padding: 6px 10px;
  border: 1px solid var(--border-color);
  border-radius: 4px;
  font-size: 13px;
  background: var(--bg-secondary);
  color: var(--text-primary);
  outline: none;
}

.setting-item input[type="number"]:focus,
.setting-item select:focus {
  border-color: var(--primary-color);
}

.setting-item input[type="checkbox"] {
  width: 16px;
  height: 16px;
}

.setting-item .hint {
  display: block;
  font-size: 12px;
  color: var(--text-tertiary);
  margin-top: 4px;
}

.btn-clear-history,
.btn-reset-shortcuts {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 6px 12px;
  background: var(--error-color);
  border: none;
  border-radius: 4px;
  font-size: 12px;
  color: white;
  cursor: pointer;
  transition: background-color 0.15s;
}

.btn-clear-history:hover,
.btn-reset-shortcuts:hover {
  background: var(--error-color-dark);
}

.shortcuts-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.shortcut-item {
  display: flex;
  align-items: center;
  gap: 12px;
}

.shortcut-name {
  flex: 1;
  font-size: 13px;
  color: var(--text-primary);
}

.shortcut-input {
  width: 100px;
  padding: 4px 8px;
  border: 1px solid var(--border-color);
  border-radius: 4px;
  font-size: 12px;
  background: var(--bg-secondary);
  color: var(--text-primary);
  font-family: monospace;
  text-align: center;
}

.btn-edit-shortcut {
  padding: 4px 10px;
  background: var(--bg-tertiary);
  border: 1px solid var(--border-color);
  border-radius: 4px;
  font-size: 12px;
  color: var(--text-secondary);
  cursor: pointer;
  transition: all 0.15s;
}

.btn-edit-shortcut:hover {
  background: var(--bg-secondary);
  color: var(--text-primary);
}

.theme-options {
  display: flex;
  gap: 12px;
}

.theme-option {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 6px;
  padding: 10px 16px;
  background: var(--bg-secondary);
  border: 2px solid transparent;
  border-radius: 8px;
  cursor: pointer;
  transition: all 0.15s;
}

.theme-option:hover {
  border-color: var(--border-color);
}

.theme-option.active {
  border-color: var(--primary-color);
}

.theme-preview {
  width: 32px;
  height: 32px;
  border-radius: 4px;
}

.theme-preview.light {
  background: linear-gradient(135deg, #ffffff 50%, #f0f0f0 50%);
  border: 1px solid #e0e0e0;
}

.theme-preview.dark {
  background: linear-gradient(135deg, #2d2d2d 50%, #1a1a1a 50%);
  border: 1px solid #3d3d3d;
}

.theme-preview.system {
  background: linear-gradient(135deg, #ffffff 50%, #2d2d2d 50%);
  border: 1px solid #e0e0e0;
}

.settings-footer {
  display: flex;
  gap: 8px;
  justify-content: flex-end;
  padding: 12px 16px;
  border-top: 1px solid var(--border-color);
}

.btn-cancel,
.btn-save {
  padding: 6px 16px;
  border-radius: 4px;
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.15s;
}

.btn-cancel {
  background: transparent;
  border: 1px solid var(--border-color);
  color: var(--text-secondary);
}

.btn-cancel:hover {
  background: var(--bg-tertiary);
}

.btn-save {
  background: var(--primary-color);
  border: none;
  color: white;
}

.btn-save:hover {
  background: var(--primary-dark);
}
</style>