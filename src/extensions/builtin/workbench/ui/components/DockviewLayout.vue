<template>
    <div :class="['dockview-fullscreen', uiStore.isDark ? 'dockview-theme-dark' : 'dockview-theme-light']">
        <DockviewVue ref="dockviewRef" @ready="onReady" />
    </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { DockviewVue } from 'dockview-vue'
import type { DockviewReadyEvent } from 'dockview-core'
import { useUiStore } from '@/shared/stores/ui'
import { registerGlobalComponent } from '@/core/vue-app-manager'

import TestCenterPanel from './TestCenterPanel.vue'
import TestLeftPanel from './TestLeftPanel.vue'
import TestBottomPanel from './TestBottomPanel.vue'

const uiStore = useUiStore()
const dockviewRef = ref<InstanceType<typeof DockviewVue> | null>(null)

registerGlobalComponent('testCenter', TestCenterPanel)
registerGlobalComponent('testLeft', TestLeftPanel)
registerGlobalComponent('testBottom', TestBottomPanel)

function onReady(event: DockviewReadyEvent) {
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
}
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
