<template>
  <NDropdown
    :show="show"
    :options="menuOptions"
    :x="x"
    :y="y"
    placement="bottom-start"
    @clickoutside="handleClickOutside"
    @select="handleSelect"
  />
</template>

<script setup lang="ts">
import {
  Eye,
  Table2,
  Code,
  Download,
  Trash2,
  Edit3,
  RefreshCw,
  Copy,
  Plus
} from 'lucide-vue-next'
import { NDropdown } from 'naive-ui'
import { computed, h } from 'vue'

import type { DropdownOption } from 'naive-ui'

interface Props {
  show: boolean
  x: number
  y: number
  nodeType: string
  nodeName: string
}

const props = defineProps<Props>()

const emit = defineEmits<{
  'update:show': [value: boolean]
  select: [key: string]
}>()

// 根据节点类型生成菜单选项
const menuOptions = computed<DropdownOption[]>(() => {
  const commonOptions: DropdownOption[] = [
    {
      key: 'refresh',
      label: '刷新',
      icon: () => h(RefreshCw, { size: 14 })
    },
    {
      key: 'copy-name',
      label: '复制名称',
      icon: () => h(Copy, { size: 14 })
    }
  ]

  switch (props.nodeType) {
    case 'table':
      return [
        {
          key: 'view-data',
          label: '查看数据',
          icon: () => h(Eye, { size: 14 })
        },
        {
          key: 'view-structure',
          label: '查看结构',
          icon: () => h(Table2, { size: 14 })
        },
        {
          type: 'divider',
          key: 'd1'
        },
        {
          key: 'generate-select',
          label: '生成 SELECT',
          icon: () => h(Code, { size: 14 })
        },
        {
          key: 'generate-insert',
          label: '生成 INSERT',
          icon: () => h(Code, { size: 14 })
        },
        {
          key: 'generate-update',
          label: '生成 UPDATE',
          icon: () => h(Code, { size: 14 })
        },
        {
          type: 'divider',
          key: 'd2'
        },
        {
          key: 'export-data',
          label: '导出数据',
          icon: () => h(Download, { size: 14 }),
          children: [
            { key: 'export-csv', label: '导出为 CSV' },
            { key: 'export-json', label: '导出为 JSON' },
            { key: 'export-sql', label: '导出为 SQL' }
          ]
        },
        {
          type: 'divider',
          key: 'd3'
        },
        {
          key: 'edit-table',
          label: '修改表结构',
          icon: () => h(Edit3, { size: 14 })
        },
        {
          key: 'delete-table',
          label: '删除表',
          icon: () => h(Trash2, { size: 14 }),
          props: {
            style: { color: '#ff4d4f' }
          }
        },
        ...commonOptions
      ]

    case 'view':
      return [
        {
          key: 'view-data',
          label: '查看数据',
          icon: () => h(Eye, { size: 14 })
        },
        {
          key: 'view-structure',
          label: '查看结构',
          icon: () => h(Table2, { size: 14 })
        },
        {
          type: 'divider',
          key: 'd1'
        },
        {
          key: 'generate-select',
          label: '生成 SELECT',
          icon: () => h(Code, { size: 14 })
        },
        {
          type: 'divider',
          key: 'd2'
        },
        {
          key: 'delete-view',
          label: '删除视图',
          icon: () => h(Trash2, { size: 14 }),
          props: {
            style: { color: '#ff4d4f' }
          }
        },
        ...commonOptions
      ]

    case 'connection':
      return [
        {
          key: 'new-query',
          label: '新建查询',
          icon: () => h(Plus, { size: 14 })
        },
        {
          key: 'refresh',
          label: '刷新',
          icon: () => h(RefreshCw, { size: 14 })
        },
        {
          type: 'divider',
          key: 'd1'
        },
        {
          key: 'disconnect',
          label: '断开连接',
          icon: () => h(Trash2, { size: 14 })
        }
      ]

    case 'database':
      return [
        {
          key: 'new-query',
          label: '新建查询',
          icon: () => h(Plus, { size: 14 })
        },
        {
          key: 'refresh',
          label: '刷新',
          icon: () => h(RefreshCw, { size: 14 })
        }
      ]

    default:
      return commonOptions
  }
})

const handleClickOutside = () => {
  emit('update:show', false)
}

const handleSelect = (key: string) => {
  emit('select', key)
  emit('update:show', false)
}
</script>
