<template>
    <div :class="['dockview-fullscreen', uiStore.isDark ? 'dockview-theme-dark' : 'dockview-theme-light']">
        <DockviewVue ref="dockviewRef" @ready="onReady" />
    </div>
</template>

<script setup lang="ts">
import { ref, markRaw } from 'vue';
import { DockviewVue } from 'dockview-vue';
import type { DockviewReadyEvent } from 'dockview-core';
import { useUiStore } from '@/shared/stores/ui';
import { registerGlobalComponent } from '@/core/vue-app-manager';

// 导入我们之前创建的测试组件！
import TestCenterPanel from './TestCenterPanel.vue';
import TestLeftPanel from './TestLeftPanel.vue';
import TestBottomPanel from './TestBottomPanel.vue';

const uiStore = useUiStore();
const dockviewRef = ref<InstanceType<typeof DockviewVue> | null>(null);

// 立即全局注册组件！
console.log('[DockviewLayout] Registering global components');
registerGlobalComponent('testCenter', TestCenterPanel);
registerGlobalComponent('testLeft', TestLeftPanel);
registerGlobalComponent('testBottom', TestBottomPanel);
console.log('[DockviewLayout] Global components registered');

function onReady(event: DockviewReadyEvent) {
    console.log('[DockviewLayout] Dockview ready!');
    
    // 方法 1：直接传组件对象！试试这个！
    console.log('[DockviewLayout] Adding panels with direct component objects');
    
    // 添加中心面板
    event.api.addPanel({
        id: 'test-center',
        component: markRaw(TestCenterPanel),
        title: 'Center',
        minimumWidth: 400
    });

    // 添加左侧面板
    event.api.addPanel({
        id: 'test-left',
        component: markRaw(TestLeftPanel),
        title: 'Left',
        position: {
            referencePanel: 'test-center',
            direction: 'left'
        },
        minimumWidth: 280
    });

    // 添加底部面板
    event.api.addPanel({
        id: 'test-bottom',
        component: markRaw(TestBottomPanel),
        title: 'Bottom',
        position: {
            referencePanel: 'test-center',
            direction: 'below'
        },
        minimumHeight: 150
    });

    console.log('[DockviewLayout] All test panels added!');
}
</script>

<script lang="ts">
// 同时通过 export default 的 components 选项也注册！双重保险！
import TestCenterPanel from './TestCenterPanel.vue';
import TestLeftPanel from './TestLeftPanel.vue';
import TestBottomPanel from './TestBottomPanel.vue';

export default {
  components: {
    'testCenter': TestCenterPanel,
    'testLeft': TestLeftPanel,
    'testBottom': TestBottomPanel
  }
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
