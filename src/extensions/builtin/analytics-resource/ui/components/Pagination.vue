<template>
  <div class="pagination">
    <div class="pagination-info">
      <span>共 {{ total }} 项</span>
      <select v-model="localPageSize" class="page-size-select" @change="handlePageSizeChange">
        <option :value="10">10/页</option>
        <option :value="20">20/页</option>
        <option :value="50">50/页</option>
        <option :value="100">100/页</option>
      </select>
    </div>

    <div class="pagination-controls">
      <button class="page-btn" :disabled="page <= 1" @click="emit('prev')">
        ‹
      </button>

      <template v-for="p in visiblePages" :key="p">
        <span v-if="p === '...'" class="page-ellipsis">...</span>
        <button
          v-else
          :class="['page-btn', { active: p === page }]"
          @click="goToPage(p as number)"
        >
          {{ p }}
        </button>
      </template>

      <button class="page-btn" :disabled="page >= totalPages" @click="emit('next')">
        ›
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue'

const props = withDefaults(defineProps<{
  page: number
  pageSize: number
  total: number
  totalPages: number
}>(), {
  page: 1,
  pageSize: 20,
  total: 0,
  totalPages: 1,
})

const emit = defineEmits<{
  'update:page': [page: number]
  'update:pageSize': [size: number]
  'prev': []
  'next': []
}>()

const localPageSize = ref(props.pageSize)

watch(localPageSize, (newSize) => {
  emit('update:pageSize', newSize)
})

function handlePageSizeChange() {
  emit('update:page', 1)
}

function goToPage(p: number) {
  if (p >= 1 && p <= props.totalPages) {
    emit('update:page', p)
  }
}

const visiblePages = computed<(number | string)[]>(() => {
  const pages: (number | string)[] = []
  const total = props.totalPages
  const current = props.page

  if (total <= 7) {
    for (let i = 1; i <= total; i++) {
      pages.push(i)
    }
  } else {
    pages.push(1)

    if (current > 3) {
      pages.push('...')
    }

    const start = Math.max(2, current - 1)
    const end = Math.min(total - 1, current + 1)

    for (let i = start; i <= end; i++) {
      pages.push(i)
    }

    if (current < total - 2) {
      pages.push('...')
    }

    pages.push(total)
  }

  return pages
})
</script>

<style scoped>
.pagination {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px 16px;
  border-top: 1px solid var(--color-border, #d9d9d9);
  background: var(--color-background-elevated, #f5f5f5);
}

.pagination-info {
  display: flex;
  align-items: center;
  gap: 12px;
  font-size: 13px;
  color: var(--text-secondary, #666);
}

.page-size-select {
  padding: 4px 8px;
  border: 1px solid var(--color-border, #d9d9d9);
  border-radius: 4px;
  background: var(--color-background, #fff);
  font-size: 13px;
  cursor: pointer;
}

.pagination-controls {
  display: flex;
  align-items: center;
  gap: 4px;
}

.page-btn {
  min-width: 32px;
  height: 32px;
  display: flex;
  align-items: center;
  justify-content: center;
  border: 1px solid var(--color-border, #d9d9d9);
  border-radius: 4px;
  background: var(--color-background, #fff);
  font-size: 13px;
  cursor: pointer;
  transition: all 0.15s;
}

.page-btn:hover:not(:disabled) {
  border-color: var(--color-primary, #165dff);
}

.page-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.page-btn.active {
  background: var(--color-primary, #165dff);
  border-color: var(--color-primary, #165dff);
  color: white;
}

.page-ellipsis {
  padding: 0 4px;
  color: var(--text-tertiary, #999);
}
</style>
