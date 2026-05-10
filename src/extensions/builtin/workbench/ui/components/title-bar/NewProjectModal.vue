<template>
  <Teleport to="body">
    <Transition name="modal">
      <div v-if="visible" class="modal-overlay" @click.self="handleCancel">
        <div class="modal-container">
          <header class="modal-header">
            <h2>{{ t('workbench.newProject') }}</h2>
            <button class="icon-btn btn-close" @click="handleCancel">
              <X :size="20" />
            </button>
          </header>
          <div class="modal-body">
            <FormField
              v-model="form.name"
              :label="t('workbench.projectName')"
              required
              :placeholder="t('workbench.projectName')"
              @keyup.enter="handleConfirm"
            />
            <FormField
              v-model="form.description"
              type="textarea"
              :label="t('workbench.projectDescription')"
              :placeholder="t('workbench.projectDescription')"
              :rows="3"
            />
            <PathField
              v-model="form.path"
              :label="t('workbench.projectPath')"
              required
              :placeholder="t('workbench.selectProjectPath')"
              @browse="handleBrowse"
            />
          </div>
          <footer class="modal-footer">
            <button class="btn-secondary" @click="handleCancel">
              {{ t('common.cancel') }}
            </button>
            <button class="btn-primary" :disabled="!canSubmit || isSubmitting" @click="handleConfirm">
              <span v-if="isSubmitting">{{ t('workbench.creating') }}</span>
              <span v-else>{{ t('workbench.create') }}</span>
            </button>
          </footer>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<script setup lang="ts">
import { X } from 'lucide-vue-next'
import { useI18n } from 'vue-i18n'

import FormField from './FormField.vue'
import PathField from './PathField.vue'
import { useNewProject } from '../../composables/useNewProject'

interface Props {
  visible: boolean
}

const props = defineProps<Props>()

const emit = defineEmits<{
  confirm: [name: string, path: string, description?: string]
  cancel: []
}>()

const { t } = useI18n()

const { form, isSubmitting, canSubmit, browsePath, submit } = useNewProject(
  () => props.visible
)

function handleBrowse() {
  browsePath()
}

function handleConfirm() {
  submit((name, path, description) => {
    emit('confirm', name, path, description)
  })
}

function handleCancel() {
  emit('cancel')
}
</script>

<style scoped>
@import './modal.css';
</style>
