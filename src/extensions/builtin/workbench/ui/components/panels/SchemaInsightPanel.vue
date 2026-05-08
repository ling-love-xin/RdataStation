<template>
  <div class="schema-insight-panel">
    <div
      v-if="insightStore.isSchemaInsightLoading && !insightStore.schemaInsight"
      class="loading-section"
    >
      <NSkeleton text :repeat="6" />
      <NSkeleton text style="width: 60%" />
    </div>

    <div v-else-if="localError" class="error-section">
      <NAlert type="error" :bordered="false">
        <template #header>{{ localError }}</template>
      </NAlert>
      <NButton size="small" quaternary @click="retry">{{ t('schemaInsight.retry') }}</NButton>
    </div>

    <div v-else-if="insightStore.schemaInsight" class="report-content">
      <div class="insight-header">
        <div class="title-row">
          <BarChart3 :size="16" class="title-icon" />
          <span class="title">{{ insightStore.schemaInsight.schema_name }}</span>
          <span class="meta-info"
            >{{ t('schemaInsight.tableCount', { count: insightStore.schemaInsight.table_count }) }}
            /
            {{
              t('schemaInsight.columnCount', { count: insightStore.schemaInsight.total_columns })
            }}</span
          >
        </div>
        <div class="header-actions">
          <NButton
            size="tiny"
            quaternary
            :title="t('schemaInsight.exportJson')"
            @click="exportJSON"
          >
            <Download :size="12" />
          </NButton>
          <NButton
            size="tiny"
            quaternary
            :title="t('schemaInsight.exportMarkdown')"
            @click="exportMarkdown"
          >
            <FileText :size="12" />
          </NButton>
          <NButton size="tiny" quaternary @click="retry">
            <RefreshCw :size="12" />
          </NButton>
        </div>
      </div>

      <div class="health-bar">
        <div class="health-score">
          <span class="score-num" :style="{ color: healthColor }">{{
            Math.round(insightStore.schemaInsight.health_score)
          }}</span>
          <span class="score-level" :style="{ color: healthColor }">{{
            insightStore.schemaInsight.health_level
          }}</span>
        </div>
        <div class="health-desc">{{ insightStore.schemaInsight.summary }}</div>
      </div>

      <NCollapse :default-expanded-names="['fk', 'types']">
        <!-- FK Candidates -->
        <NCollapseItem name="fk" :title="t('schemaInsight.fkCandidates')">
          <template #header-extra>
            <NTag size="tiny" :bordered="false">{{
              insightStore.schemaInsight.fk_candidates.length
            }}</NTag>
          </template>
          <div v-if="insightStore.schemaInsight.fk_candidates.length === 0" class="empty-note">{{
            t('schemaInsight.noFkDetected')
          }}</div>
          <div v-else class="fk-list">
            <div
              v-for="(fk, idx) in insightStore.schemaInsight.fk_candidates"
              :key="idx"
              class="fk-row"
            >
              <div class="fk-tables">
                <NTag
                  size="tiny"
                  type="primary"
                  :bordered="false"
                  class="clickable-tag"
                  @click="openDrillTable(fk.source_table)"
                  >{{ fk.source_table }}</NTag
                >
                <span class="fk-arrow">→</span>
                <NTag
                  size="tiny"
                  type="info"
                  :bordered="false"
                  class="clickable-tag"
                  @click="openDrillTable(fk.target_table)"
                  >{{ fk.target_table }}</NTag
                >
              </div>
              <div class="fk-detail">
                <span class="fk-col">{{ fk.source_column }} → {{ fk.target_column }}</span>
                <NTag
                  size="tiny"
                  :bordered="false"
                  :type="
                    fk.confidence === 'high'
                      ? 'success'
                      : fk.confidence === 'medium'
                        ? 'warning'
                        : 'default'
                  "
                  >{{ fk.confidence }}</NTag
                >
              </div>
              <div class="fk-pattern">{{ fk.naming_pattern }}</div>
            </div>
          </div>
        </NCollapseItem>

        <!-- Type Mismatches -->
        <NCollapseItem name="types" :title="t('schemaInsight.typeMismatches')">
          <template #header-extra>
            <NTag
              size="tiny"
              :bordered="false"
              :type="insightStore.schemaInsight.type_mismatches.length > 0 ? 'warning' : 'success'"
              >{{ insightStore.schemaInsight.type_mismatches.length }}</NTag
            >
          </template>
          <div v-if="insightStore.schemaInsight.type_mismatches.length === 0" class="empty-note">{{
            t('schemaInsight.noTypeMismatch')
          }}</div>
          <div v-else class="mismatch-list">
            <div
              v-for="(m, idx) in insightStore.schemaInsight.type_mismatches"
              :key="idx"
              class="mismatch-row"
            >
              <div class="mismatch-header">
                <span class="mismatch-col">{{ m.column_name }}</span>
                <NTag
                  size="tiny"
                  :bordered="false"
                  :type="
                    m.severity === 'critical'
                      ? 'error'
                      : m.severity === 'warning'
                        ? 'warning'
                        : 'default'
                  "
                  >{{ m.severity }}</NTag
                >
              </div>
              <div class="mismatch-tables">
                <button
                  v-for="tbl in m.tables"
                  :key="tbl.table_name"
                  class="mismatch-table link-table"
                  @click="openDrillTable(tbl.table_name)"
                >
                  {{ tbl.table_name }} <code>{{ tbl.data_type }}</code>
                </button>
              </div>
            </div>
          </div>
        </NCollapseItem>

        <!-- Orphan Tables -->
        <NCollapseItem name="orphan" :title="t('schemaInsight.orphanTables')">
          <template #header-extra>
            <NTag
              size="tiny"
              :bordered="false"
              :type="insightStore.schemaInsight.orphan_tables.length > 3 ? 'warning' : 'default'"
              >{{ insightStore.schemaInsight.orphan_tables.length }}</NTag
            >
          </template>
          <div v-if="insightStore.schemaInsight.orphan_tables.length === 0" class="empty-note">{{
            t('schemaInsight.noOrphanTables')
          }}</div>
          <div v-else class="orphan-list">
            <div
              v-for="o in insightStore.schemaInsight.orphan_tables"
              :key="o.table_name"
              class="orphan-row"
            >
              <NTag
                size="tiny"
                :bordered="false"
                class="clickable-tag"
                @click="openDrillTable(o.table_name)"
                >{{ o.table_name }}</NTag
              >
              <span class="orphan-col-count">{{
                t('schemaInsight.columnCount', { count: o.column_count })
              }}</span>
              <span class="orphan-reason">{{ o.reason }}</span>
            </div>
          </div>
        </NCollapseItem>

        <!-- Redundant Columns -->
        <NCollapseItem name="redundant" :title="t('schemaInsight.redundantColumns')">
          <template #header-extra>
            <NTag size="tiny" :bordered="false">{{
              insightStore.schemaInsight.redundant_columns.length
            }}</NTag>
          </template>
          <div
            v-if="insightStore.schemaInsight.redundant_columns.length === 0"
            class="empty-note"
            >{{ t('schemaInsight.noRedundantColumns') }}</div
          >
          <div v-else class="redundant-list">
            <div
              v-for="r in insightStore.schemaInsight.redundant_columns"
              :key="r.column_name"
              class="redundant-row"
            >
              <span class="redundant-col">{{ r.column_name }}</span>
              <NTag size="tiny" :bordered="false">{{
                t('schemaInsight.tableCount', { count: r.table_count })
              }}</NTag>
              <div class="redundant-suggest">{{ r.suggestion }}</div>
            </div>
          </div>
        </NCollapseItem>
      </NCollapse>
    </div>
  </div>
</template>

<script setup lang="ts">
import { BarChart3, RefreshCw, Download, FileText } from 'lucide-vue-next'
import { NSkeleton, NAlert, NButton, NTag, NCollapse, NCollapseItem } from 'naive-ui'
import { computed, onMounted, ref } from 'vue'
import { useI18n } from 'vue-i18n'

import { useInsightStore } from '../../stores/insight-store'

const { t } = useI18n()
const insightStore = useInsightStore()

interface Props {
  connId: string
  dbType?: string
  database: string
  schema: string
}

const props = defineProps<Props>()

const localError = ref<string | null>(null)

const healthColor = computed(() => {
  const s = insightStore.schemaInsight?.health_score ?? 0
  if (s >= 85) return '#1a7a1a'
  if (s >= 70) return '#1a6db5'
  if (s >= 50) return '#b57a1a'
  if (s >= 30) return '#b54a1a'
  return '#b51a1a'
})

function openDrillTable(tableName: string): void {
  insightStore.requestTableProfile({
    connId: props.connId,
    dbType: props.dbType ?? 'unknown',
    database: props.database,
    schema: props.schema,
    table: tableName,
  })
}

async function retry(): Promise<void> {
  localError.value = null
  try {
    await insightStore.loadSchemaInsight({
      connId: props.connId,
      database: props.database,
      schema: props.schema,
    })
    if (!insightStore.schemaInsight) {
      localError.value = t('schemaInsight.loadFailed')
    }
  } catch {
    localError.value = t('schemaInsight.loadFailed')
  }
}

function exportJSON(): void {
  const r = insightStore.schemaInsight
  if (!r) return
  const blob = new Blob([JSON.stringify(r, null, 2)], { type: 'application/json' })
  const url = URL.createObjectURL(blob)
  const a = document.createElement('a')
  a.href = url
  a.download = `schema-insight-${r.schema_name}-${new Date().toISOString().slice(0, 10)}.json`
  a.click()
  URL.revokeObjectURL(url)
}

function exportMarkdown(): void {
  const r = insightStore.schemaInsight
  if (!r) return
  const lines: string[] = [
    `# ${t('schemaInsight.schemaInsightReport')}：${r.schema_name}`,
    '',
    `| 指标 | 值 |`,
    `|:--|:--|`,
    `| 表数 | ${r.table_count} |`,
    `| 列总数 | ${r.total_columns} |`,
    `| 健康评分 | ${Math.round(r.health_score)} / 100 (${r.health_level}) |`,
    '',
    `> ${r.summary}`,
    '',
    '## 外键候选',
    ...r.fk_candidates.map(
      fk =>
        `- ${fk.source_table}.\`${fk.source_column}\` → ${fk.target_table}.\`${fk.target_column}\` (${fk.confidence}, ${fk.naming_pattern})`
    ),
    r.fk_candidates.length === 0 ? '- *未检测到*' : '',
    '',
    '## 类型不一致',
    ...r.type_mismatches.map(
      m =>
        `- **${m.column_name}** (${m.severity}): ${m.tables.map(t => `${t.table_name}=${t.data_type}`).join(', ')}`
    ),
    r.type_mismatches.length === 0 ? '- *类型一致*' : '',
    '',
    '## 孤立表',
    ...r.orphan_tables.map(o => `- ${o.table_name} (${o.column_count} 列): ${o.reason}`),
    r.orphan_tables.length === 0 ? '- *无*' : '',
    '',
    '## 冗余列',
    ...r.redundant_columns.map(c => `- ${c.column_name} (${c.table_count} 表): ${c.suggestion}`),
    r.redundant_columns.length === 0 ? '- *无*' : '',
  ]
  const blob = new Blob([lines.join('\n')], { type: 'text/markdown' })
  const url = URL.createObjectURL(blob)
  const a = document.createElement('a')
  a.href = url
  a.download = `schema-insight-${r.schema_name}-${new Date().toISOString().slice(0, 10)}.md`
  a.click()
  URL.revokeObjectURL(url)
}

onMounted(() => {
  void retry()
})
</script>

<style scoped>
.schema-insight-panel {
  height: 100%;
  overflow-y: auto;
  padding: 0;
  display: flex;
  flex-direction: column;
}
.loading-section {
  padding: 12px;
}
.error-section {
  padding: 12px;
  display: flex;
  flex-direction: column;
  gap: 8px;
  align-items: flex-start;
}

.report-content {
  flex: 1;
  overflow-y: auto;
}

.insight-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 12px;
  border-bottom: 1px solid var(--border-color);
  flex-shrink: 0;
}
.title-row {
  display: flex;
  align-items: center;
  gap: 6px;
}
.title-icon {
  color: var(--primary-color);
}
.title {
  font-size: 13px;
  font-weight: 600;
}
.meta-info {
  font-size: 10px;
  color: var(--text-tertiary);
  margin-left: 4px;
}
.header-actions {
  display: flex;
  align-items: center;
  gap: 2px;
}

.health-bar {
  padding: 8px 12px;
  border-bottom: 1px solid var(--border-color);
  background: var(--bg-tertiary);
  flex-shrink: 0;
}
.health-score {
  display: flex;
  align-items: center;
  gap: 6px;
  margin-bottom: 4px;
}
.score-num {
  font-size: 24px;
  font-weight: 700;
  line-height: 1;
}
.score-level {
  font-size: 13px;
  font-weight: 600;
}
.health-desc {
  font-size: 10px;
  color: var(--text-secondary);
  line-height: 1.4;
}

.empty-note {
  font-size: 11px;
  color: var(--text-tertiary);
  padding: 8px 0;
}

.fk-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}
.fk-row {
  padding: 6px 8px;
  background: var(--bg-secondary);
  border-radius: 4px;
}
.fk-tables {
  display: flex;
  align-items: center;
  gap: 6px;
  margin-bottom: 3px;
}
.fk-arrow {
  color: var(--text-tertiary);
  font-size: 11px;
}
.fk-detail {
  display: flex;
  align-items: center;
  gap: 6px;
  margin-bottom: 2px;
}
.fk-col {
  font-size: 10px;
  color: var(--text-secondary);
}
.fk-pattern {
  font-size: 9px;
  color: var(--text-tertiary);
}

.mismatch-list {
  display: flex;
  flex-direction: column;
  gap: 6px;
}
.mismatch-row {
  padding: 4px 0;
}
.mismatch-header {
  display: flex;
  align-items: center;
  gap: 6px;
  margin-bottom: 2px;
}
.mismatch-col {
  font-size: 12px;
  font-weight: 600;
}
.mismatch-tables {
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
}
.mismatch-table {
  font-size: 10px;
  color: var(--text-secondary);
}
.mismatch-table code {
  font-size: 9px;
  background: var(--bg-hover);
  padding: 1px 3px;
  border-radius: 2px;
}

.orphan-list {
  display: flex;
  flex-direction: column;
  gap: 4px;
}
.orphan-row {
  display: flex;
  align-items: center;
  gap: 6px;
}
.orphan-col-count {
  font-size: 10px;
  color: var(--text-tertiary);
}
.orphan-reason {
  font-size: 10px;
  color: var(--text-secondary);
}

.redundant-list {
  display: flex;
  flex-direction: column;
  gap: 6px;
}
.redundant-row {
  padding: 4px 0;
}
.redundant-col {
  font-size: 12px;
  font-weight: 600;
}
.redundant-suggest {
  font-size: 10px;
  color: var(--text-secondary);
  margin-top: 2px;
}

.clickable-tag {
  cursor: pointer;
}

.clickable-tag:hover {
  opacity: 0.8;
  text-decoration: underline;
}

.link-table {
  background: none;
  border: none;
  cursor: pointer;
  padding: 0;
  font-size: inherit;
}

.link-table:hover code {
  text-decoration: underline;
}
</style>
