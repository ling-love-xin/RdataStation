<template>
    <div :class="['dockview-fullscreen', uiStore.isDark ? 'dockview-theme-dark' : 'dockview-theme-light']">
        <DockviewVue ref="dockviewRef" @ready="onReady" />
    </div>
</template>

<script setup lang="ts">
import { DockviewVue } from 'dockview-vue'
import { ref, onMounted, onUnmounted } from 'vue'
import { useI18n } from 'vue-i18n'

import { registerGlobalComponent } from '@/core/vue-app-manager'
import { useUiStore } from '@/shared/stores/ui'

import ColumnInsightPanel from './panels/ColumnInsightPanel.vue'
import TableProfileView from './panels/TableProfileView.vue'
import TestBottomPanel from './TestBottomPanel.vue'
import TestCenterPanel from './TestCenterPanel.vue'
import TestLeftPanel from './TestLeftPanel.vue'

import type { DockviewApi } from 'dockview-core'
import type { DockviewReadyEvent } from 'dockview-vue'

const { t } = useI18n()
const uiStore = useUiStore()
const dockviewRef = ref<InstanceType<typeof DockviewVue> | null>(null)
let dockviewApi: DockviewApi | null = null

registerGlobalComponent('testCenter', TestCenterPanel)
registerGlobalComponent('testLeft', TestLeftPanel)
registerGlobalComponent('testBottom', TestBottomPanel)
registerGlobalComponent('columnInsight', ColumnInsightPanel)
registerGlobalComponent('tableProfile', TableProfileView)

function onReady(event: DockviewReadyEvent) {
    dockviewApi = event.api

    event.api.addPanel({
        id: 'test-center',
        component: 'testCenter',
        title: 'Center',
        minimumWidth: 400,
    })

    event.api.addPanel({
        id: 'test-left',
        component: 'testLeft',
        title: 'Left',
        position: {
            referencePanel: 'test-center',
            direction: 'left',
        },
        minimumWidth: 280,
    })

    event.api.addPanel({
        id: 'test-bottom',
        component: 'testBottom',
        title: 'Bottom',
        position: {
            referencePanel: 'test-center',
            direction: 'below',
        },
        minimumHeight: 150,
    })

    event.api.addPanel({
        id: 'right-insight',
        component: 'columnInsight',
        title: t('workbench.insight'),
        position: {
            referencePanel: 'test-center',
            direction: 'right',
        },
        minimumWidth: 260,
        initialWidth: 300,
    })
}

function handleTableProfile(event: Event): void {
    if (!dockviewApi) return
    const detail = (event as CustomEvent).detail as {
        connId: string; dbType: string; database: string; schema: string; table: string
    }
    if (!detail?.table) return

    const panelId = `table-profile-${detail.database}-${detail.schema}-${detail.table}`

    const existing = dockviewApi.getPanel(panelId)
    if (existing) {
        existing.focus()
        return
    }

    dockviewApi.addPanel({
        id: panelId,
        component: 'tableProfile',
        title: `📊 ${detail.table}`,
        position: {
            referencePanel: 'test-center',
            direction: 'within',
        },
        params: {
            connId: detail.connId,
            dbType: detail.dbType,
            database: detail.database,
            schema: detail.schema,
            table: detail.table
        }
    })
}

onMounted(() => {
    window.addEventListener('open-table-profile', handleTableProfile)
})

onUnmounted(() => {
    window.removeEventListener('open-table-profile', handleTableProfile)
})
</script>

<style scoped>
.dockview-fullscreen {
    width: 100%;
    height: 100%;
    min-height: 0;
    flex: 1;
    overflow: hidden;
    position: relative;
}

.dockview-fullscreen :deep(.dockview) {
    width: 100% !important;
    height: 100% !important;
}
</style>
