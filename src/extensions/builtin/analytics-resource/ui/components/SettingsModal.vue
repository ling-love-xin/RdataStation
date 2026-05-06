<template>
  <div class="modal-overlay" @click.self="$emit('close')">
    <div class="modal settings-modal">
      <div class="modal-header">
        <h3>⚙️ 设置</h3>
        <button class="close-btn" @click="$emit('close')">✕</button>
      </div>

      <div class="modal-body">
        <div class="settings-tabs">
          <button
            v-for="tab in tabs"
            :key="tab.id"
            :class="['tab-btn', { active: activeTab === tab.id }]"
            @click="activeTab = tab.id"
          >
            {{ tab.icon }} {{ tab.label }}
          </button>
        </div>

        <div class="settings-content">
          <!-- 通用设置 -->
          <div v-if="activeTab === 'general'" class="settings-section">
            <h4>通用设置</h4>
            
            <div class="setting-item">
              <label class="setting-label">
                <span>默认作用域</span>
                <span class="setting-description">新资源的默认作用域</span>
              </label>
              <select v-model="settings.general.defaultScope" class="form-input">
                <option value="project">📂 项目</option>
                <option value="global">🌍 全局</option>
                <option value="session">📌 会话</option>
              </select>
            </div>

            <div class="setting-item">
              <label class="setting-label">
                <span>每页显示数量</span>
                <span class="setting-description">资源列表每页显示的数量</span>
              </label>
              <select v-model.number="settings.general.defaultPageSize" class="form-input">
                <option :value="10">10</option>
                <option :value="20">20</option>
                <option :value="50">50</option>
                <option :value="100">100</option>
              </select>
            </div>

            <div class="setting-item">
              <label class="setting-label">
                <span>默认排序字段</span>
                <span class="setting-description">资源列表默认排序字段</span>
              </label>
              <select v-model="settings.general.defaultSortField" class="form-input">
                <option value="name">名称</option>
                <option value="created_at">创建时间</option>
                <option value="updated_at">更新时间</option>
                <option value="row_count">行数</option>
              </select>
            </div>

            <div class="setting-item">
              <label class="setting-label">
                <span>默认排序方向</span>
              </label>
              <select v-model="settings.general.defaultSortOrder" class="form-input">
                <option value="asc">升序</option>
                <option value="desc">降序</option>
              </select>
            </div>
          </div>

          <!-- 显示设置 -->
          <div v-if="activeTab === 'display'" class="settings-section">
            <h4>显示设置</h4>
            
            <div class="setting-item">
              <label class="setting-label">
                <span>显示资源图标</span>
                <span class="setting-description">在资源列表中显示图标</span>
              </label>
              <label class="toggle-switch">
                <input v-model="settings.display.showIcons" type="checkbox" />
                <span class="slider"></span>
              </label>
            </div>

            <div class="setting-item">
              <label class="setting-label">
                <span>显示作用域标签</span>
                <span class="setting-description">在资源列表中显示作用域标签</span>
              </label>
              <label class="toggle-switch">
                <input v-model="settings.display.showScopeTags" type="checkbox" />
                <span class="slider"></span>
              </label>
            </div>

            <div class="setting-item">
              <label class="setting-label">
                <span>显示资源元数据</span>
                <span class="setting-description">显示行数、列数、文件大小等信息</span>
              </label>
              <label class="toggle-switch">
                <input v-model="settings.display.showMetadata" type="checkbox" />
                <span class="slider"></span>
              </label>
            </div>

            <div class="setting-item">
              <label class="setting-label">
                <span>启用虚拟滚动</span>
                <span class="setting-description">大数据量时启用虚拟滚动优化性能</span>
              </label>
              <label class="toggle-switch">
                <input v-model="settings.display.enableVirtualScroll" type="checkbox" />
                <span class="slider"></span>
              </label>
            </div>
          </div>

          <!-- 缓存设置 -->
          <div v-if="activeTab === 'cache'" class="settings-section">
            <h4>缓存设置</h4>
            
            <div class="setting-item">
              <label class="setting-label">
                <span>启用查询缓存</span>
                <span class="setting-description">缓存查询结果以提升响应速度</span>
              </label>
              <label class="toggle-switch">
                <input v-model="settings.cache.enabled" type="checkbox" />
                <span class="slider"></span>
              </label>
            </div>

            <div class="setting-item">
              <label class="setting-label">
                <span>缓存过期时间（秒）</span>
                <span class="setting-description">缓存自动过期时间</span>
              </label>
              <input
                v-model.number="settings.cache.ttlSeconds"
                type="number"
                min="10"
                max="3600"
                class="form-input"
              />
            </div>

            <div class="setting-item">
              <label class="setting-label">
                <span>最大缓存数量</span>
                <span class="setting-description">缓存的最大条目数</span>
              </label>
              <input
                v-model.number="settings.cache.maxSize"
                type="number"
                min="5"
                max="200"
                class="form-input"
              />
            </div>

            <button class="btn btn-secondary" @click="clearCache">
              🗑️ 清除缓存
            </button>
          </div>

          <!-- 快捷键设置 -->
          <div v-if="activeTab === 'shortcuts'" class="settings-section">
            <h4>快捷键</h4>
            
            <div class="shortcuts-list">
              <div v-for="shortcut in shortcuts" :key="shortcut.key" class="shortcut-item">
                <span class="shortcut-action">{{ shortcut.label }}</span>
                <kbd class="shortcut-key">{{ shortcut.key }}</kbd>
              </div>
            </div>
          </div>
        </div>
      </div>

      <div class="modal-footer">
        <button class="btn btn-secondary" @click="resetSettings">
          重置为默认
        </button>
        <button class="btn btn-primary" @click="handleSave">
          保存设置
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">import { ref, reactive, watch } from 'vue';

import type { AnalyticsResourceSettings } from '../../types';
const props = defineProps<{
 settings: AnalyticsResourceSettings;
}>();
const emit = defineEmits<{
  close: [];
  save: [settings: AnalyticsResourceSettings];
  clearCache: [];
}>();
const tabs = [
 { id: 'general', label: '通用', icon: '⚙️' },
 { id: 'display', label: '显示', icon: '🎨' },
 { id: 'cache', label: '缓存', icon: '💾' },
 { id: 'shortcuts', label: '快捷键', icon: '⌨️' },
];
const activeTab = ref('general');
const shortcuts = [
 { key: 'Ctrl+N', label: '新建资源' },
 { key: 'Ctrl+E', label: '编辑资源' },
 { key: 'Ctrl+D', label: '删除资源' },
 { key: 'Ctrl+Shift+C', label: '克隆资源' },
 { key: 'Ctrl+F', label: '搜索' },
 { key: 'Ctrl+A', label: '全选' },
 { key: 'Delete', label: '删除选中' },
];
const localSettings = reactive<AnalyticsResourceSettings>(JSON.parse(JSON.stringify(props.settings)));
watch(() => props.settings, (newSettings) => {
 Object.assign(localSettings, newSettings);
}, { deep: true });
function handleSave() {
 emit('save', JSON.parse(JSON.stringify(localSettings)));
}
function resetSettings() {
 Object.assign(localSettings, {
 general: {
 defaultScope: 'project',
 defaultPageSize: 20,
 defaultSortField: 'created_at',
 defaultSortOrder: 'desc',
 },
 display: {
 showIcons: true,
 showScopeTags: true,
 showMetadata: true,
 enableVirtualScroll: true,
 },
 cache: {
 enabled: true,
 ttlSeconds: 30,
 maxSize: 50,
 },
 });
}
function clearCache() {
  emit('clearCache');
}
</script>

<style scoped>
.settings-modal {
  width: 600px;
}

.settings-tabs {
  display: flex;
  gap: var(--size-sm);
  padding-bottom: var(--size-md);
  border-bottom: 1px solid var(--border-color);
  margin-bottom: var(--size-lg);
}

.tab-btn {
  padding: var(--size-sm) var(--size-md);
  border: none;
  border-radius: var(--radius-md);
  background: transparent;
  color: var(--text-secondary);
  font-size: 13px;
  cursor: pointer;
  transition: all 0.2s;
}

.tab-btn:hover {
  background: var(--bg-secondary);
}

.tab-btn.active {
  background: var(--primary-light);
  color: var(--primary-color);
}

.settings-content {
  max-height: 400px;
  overflow-y: auto;
}

.settings-section h4 {
  font-size: 14px;
  font-weight: 600;
  color: var(--text-primary);
  margin-bottom: var(--size-md);
  padding-bottom: var(--size-sm);
  border-bottom: 1px solid var(--border-color-light);
}

.setting-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: var(--size-md) 0;
  border-bottom: 1px solid var(--border-color-light);
}

.setting-label {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.setting-label span:first-child {
  font-size: 13px;
  color: var(--text-primary);
  font-weight: 500;
}

.setting-description {
  font-size: 11px;
  color: var(--text-tertiary);
}

.toggle-switch {
  position: relative;
  display: inline-block;
  width: 48px;
  height: 26px;
  cursor: pointer;
}

.toggle-switch input {
  opacity: 0;
  width: 0;
  height: 0;
}

.toggle-switch .slider {
  position: absolute;
  cursor: pointer;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background-color: var(--border-color);
  transition: 0.3s;
  border-radius: 26px;
}

.toggle-switch .slider:before {
  position: absolute;
  content: "";
  height: 20px;
  width: 20px;
  left: 3px;
  bottom: 3px;
  background-color: white;
  transition: 0.3s;
  border-radius: 50%;
  box-shadow: var(--shadow-sm);
}

.toggle-switch input:checked + .slider {
  background-color: var(--primary-color);
}

.toggle-switch input:checked + .slider:before {
  transform: translateX(22px);
}

.shortcuts-list {
  display: flex;
  flex-direction: column;
  gap: var(--size-sm);
}

.shortcut-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: var(--size-sm) var(--size-md);
  background: var(--bg-secondary);
  border-radius: var(--radius-md);
}

.shortcut-action {
  font-size: 13px;
  color: var(--text-primary);
}

.shortcut-key {
  padding: 4px 8px;
  background: var(--bg-primary);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-sm);
  font-family: var(--font-mono);
  font-size: 12px;
  color: var(--text-secondary);
}
</style>
