<template>
  <div v-if="!empty" class="topo-preview">
    <div class="topo-title">📡 数据路径预览</div>
    <div class="topo-path">
      <template v-for="(node, idx) in nodes" :key="idx">
        <!-- 箭头分隔符 -->
        <div
          v-if="idx > 0"
          class="topo-arrow"
          :class="{ 'tls-arrow': node.kind === 'ssl' }"
        >
          <span v-if="node.kind !== 'ssl'" class="topo-arrow-label">
            {{ getArrowLabel(node) }}
          </span>
          <span v-else class="topo-arrow-label">TLS 🔐</span>
        </div>

        <!-- 节点 -->
        <div class="topo-node" :class="getNodeClass(node.kind)">
          <span class="topo-node-icon">{{ getNodeIcon(node.kind) }}</span>
          <span class="topo-node-label">{{ node.label }}</span>
          <span v-if="node.detail" class="topo-node-detail">{{ node.detail }}</span>
        </div>
      </template>

      <!-- 目标数据库 -->
      <div class="topo-arrow">
        ────▶
      </div>
      <div class="topo-node db-target">
        <span class="topo-node-icon">🗄</span>
        <span class="topo-node-label">{{ targetLabel }}</span>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import type { TopologyNode, TopologyNodeKind } from '../../types/network-chain'

defineProps<{
  nodes: TopologyNode[]
  targetLabel: string
  empty: boolean
}>()

function getNodeIcon(kind: TopologyNodeKind): string {
  const icons: Record<TopologyNodeKind, string> = {
    self: '🏠',
    ssh: '🔒',
    proxy: '🌐',
    ssl: '🛡',
    target: '🗄',
  }
  return icons[kind] || '●'
}

function getNodeClass(kind: TopologyNodeKind): string {
  const classes: Record<TopologyNodeKind, string> = {
    self: 'self',
    ssh: 'ssh-jump',
    proxy: 'proxy-node',
    ssl: 'ssl-node',
    target: 'db-target',
  }
  return classes[kind] || ''
}

function getArrowLabel(_node: TopologyNode): string {
  return ''
}
</script>

<style scoped>
.topo-preview {
  margin-top: var(--spacing-lg);
  padding: var(--spacing-lg);
  background: var(--color-bg-raised, #11111b);
  border: 1px solid var(--color-border);
  border-radius: var(--border-radius-lg);
}

.topo-title {
  font-size: var(--font-size-xs);
  font-weight: 600;
  color: var(--color-text-muted);
  text-transform: uppercase;
  letter-spacing: 0.5px;
  margin-bottom: var(--spacing-md);
}

.topo-path {
  display: flex;
  align-items: center;
  gap: 0;
  flex-wrap: wrap;
  font-size: var(--font-size-sm);
}

.topo-node {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 6px 12px;
  background: var(--color-bg-active, #2a2a3c);
  border-radius: var(--border-radius-md);
  color: var(--color-text-secondary);
  font-weight: 500;
  border: 1px solid var(--color-border);
}

.topo-node.self {
  background: rgba(137, 180, 250, 0.06);
  border-color: rgba(137, 180, 250, 0.15);
  color: #89b4fa;
}

.topo-node.ssh-jump {
  background: rgba(166, 227, 161, 0.06);
  border-color: rgba(166, 227, 161, 0.15);
  color: var(--brand-success);
}

.topo-node.proxy-node {
  background: rgba(203, 166, 247, 0.06);
  border-color: rgba(203, 166, 247, 0.15);
  color: #cba6f7;
}

.topo-node.ssl-node {
  background: rgba(137, 180, 250, 0.06);
  border-color: rgba(137, 180, 250, 0.15);
  color: #89b4fa;
}

.topo-node.db-target {
  background: rgba(250, 179, 135, 0.06);
  border-color: rgba(250, 179, 135, 0.15);
  color: #fab387;
}

.topo-node-icon {
  flex-shrink: 0;
}

.topo-node-label {
  white-space: nowrap;
}

.topo-node-detail {
  font-size: var(--font-size-xxs);
  color: var(--color-text-muted);
  font-weight: 400;
}

.topo-arrow {
  display: flex;
  align-items: center;
  color: var(--color-text-muted);
  font-size: var(--font-size-xxs);
  padding: 0 6px;
  flex-shrink: 0;
}

.topo-arrow.tls-arrow {
  background: rgba(137, 180, 250, 0.06);
  border: 1px dashed rgba(137, 180, 250, 0.2);
  border-radius: var(--border-radius-sm);
  padding: 2px 8px;
}

.topo-arrow.tls-arrow .topo-arrow-label {
  color: #89b4fa;
  font-weight: 600;
}

.topo-arrow-label {
  font-size: var(--font-size-xxs);
  color: var(--color-text-muted);
  padding: 2px 6px;
  background: rgba(255, 255, 255, 0.03);
  border-radius: var(--border-radius-sm);
  white-space: nowrap;
}
</style>