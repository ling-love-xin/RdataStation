<template>
  <div class="empty-state">
    <div class="empty-icon">
      <Database :size="48" />
    </div>
    
    <div class="empty-content">
      <h3>{{ title }}</h3>
      <p>{{ description }}</p>
      
      <div class="empty-actions">
        <button 
          v-for="action in actions" 
          :key="action.id"
          class="btn-action"
          :class="{ primary: action.primary }"
          @click="$emit('action', action.id)"
        >
          <component :is="action.icon" :size="14" />
          {{ action.label }}
        </button>
      </div>
    </div>

    <div class="empty-decoration">
      <div class="decoration-circle"></div>
      <div class="decoration-circle small"></div>
      <div class="decoration-circle tiny"></div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { Database } from 'lucide-vue-next'

import type { Component } from 'vue'

interface Action {
  id: string
  label: string
  icon: Component
  primary?: boolean
}

defineProps<{
  title?: string
  description?: string
  actions?: Action[]
}>()

defineEmits<{
  action: [id: string]
}>()
</script>

<style scoped>
.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  min-height: 200px;
  padding: 40px;
  position: relative;
  overflow: hidden;
}

.empty-icon {
  width: 96px;
  height: 96px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: linear-gradient(135deg, rgba(0, 180, 100, 0.1), rgba(100, 100, 255, 0.1));
  border-radius: 24px;
  margin-bottom: 20px;
  color: var(--primary-color);
}

.empty-content {
  text-align: center;
  z-index: 1;
}

.empty-content h3 {
  margin: 0 0 8px 0;
  font-size: 16px;
  font-weight: 600;
  color: var(--text-primary);
}

.empty-content p {
  margin: 0 0 2