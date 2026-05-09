<template>
  <NDrawer
    :show="visible"
    :width="340"
    placement="right"
    :on-update:show="onClose"
  >
    <NDrawerContent :title="`高级配置 — ${columnName}`" closable>
      <div class="drawer-body">
        <div class="param-section">
          <div class="param-header">
            <span class="param-label">生成器</span>
            <span class="param-value">{{ generatorLabel }}</span>
          </div>
        </div>

        <NDivider style="margin: 12px 0" />

        <div v-if="paramFields.length === 0" class="empty-params">
          此生成器无需额外配置参数
        </div>

        <div v-else class="params-form">
          <div
            v-for="field in paramFields"
            :key="field.name"
            class="param-field"
          >
            <label class="param-field-label">{{ field.label }}</label>

            <NInputNumber
              v-if="field.type === 'number'"
              :value="(localParams[field.name] as number) ?? (field.default as number)"
              size="small"
              :min="field.min"
              :max="field.max"
              @update:value="(v: number | null) => setParam(field.name, v ?? field.default)"
            />

            <NInput
              v-if="field.type === 'text'"
              :value="(localParams[field.name] as string) ?? ''"
              size="small"
              @update:value="(v: string) => setParam(field.name, v)"
            />

            <NSwitch
              v-if="field.type === 'boolean'"
              :value="(localParams[field.name] as boolean) ?? false"
              @update:value="(v: boolean) => setParam(field.name, v)"
            />

            <div v-if="field.type === 'string-array'" class="array-field">
              <div
                v-for="(item, i) in (localParams[field.name] as string[] | undefined) ?? []"
                :key="i"
                class="array-row"
              >
                <NInput
                  :value="item"
                  size="small"
                  @update:value="(v: string) => updateArrayItem(field.name, i, v)"
                />
                <NButton size="tiny" quaternary type="error" @click="removeArrayItem(field.name, i)">
                  <template #icon><Trash2 :size="12" /></template>
                </NButton>
              </div>
              <NButton size="small" dashed @click="addArrayItem(field.name)">
                <template #icon><Plus :size="14" /></template>
                添加值
              </NButton>
            </div>

            <div v-if="field.type === 'weighted-choices'" class="weighted-field">
              <div
                v-for="(item, i) in (localParams[field.name] as WeightedChoice[] | undefined) ?? []"
                :key="i"
                class="weighted-row"
              >
                <NInput
                  :value="item.value"
                  size="small"
                  placeholder="值"
                  @update:value="(v: string) => updateWeightedItem(field.name, i, 'value', v)"
                />
                <NInputNumber
                  :value="item.weight"
                  size="small"
                  :min="0"
                  :max="1"
                  :step="0.01"
                  placeholder="权重"
                  style="width: 80px"
                  @update:value="(v: number | null) => updateWeightedItem(field.name, i, 'weight', v ?? 0)"
                />
                <NButton size="tiny" quaternary type="error" @click="removeArrayItem(field.name, i)">
                  <template #icon><Trash2 :size="12" /></template>
                </NButton>
              </div>
              <NButton size="small" dashed @click="addWeightedItem(field.name)">
                <template #icon><Plus :size="14" /></template>
                添加选项
              </NButton>
            </div>
          </div>
        </div>
      </div>

      <template #footer>
        <div class="drawer-footer">
          <NButton size="small" @click="onClose">取消</NButton>
          <NButton type="primary" size="small" @click="onSave">保存配置</NButton>
        </div>
      </template>
    </NDrawerContent>
  </NDrawer>
</template>

<script setup lang="ts">
import { Trash2, Plus } from 'lucide-vue-next'
import {
  NDrawer, NDrawerContent, NDivider, NButton, NInput, NInputNumber, NSwitch,
} from 'naive-ui'
import { ref, computed, watch } from 'vue'

import type { GeneratorType } from '@/shared/api/mock-api'

interface ParamFieldDef {
  name: string
  label: string
  type: 'number' | 'text' | 'boolean' | 'string-array' | 'weighted-choices'
  default: unknown
  min?: number
  max?: number
}

interface WeightedChoice {
  value: string
  weight: number
}

const GENERATOR_PARAM_SCHEMA: Partial<Record<GeneratorType, ParamFieldDef[]>> = {
  auto_increment: [
    { name: 'start', label: '起始值', type: 'number', default: 1 },
    { name: 'step', label: '步长', type: 'number', default: 1, min: 1 },
  ],
  random_int: [
    { name: 'min', label: '最小值', type: 'number', default: 0 },
    { name: 'max', label: '最大值', type: 'number', default: 100 },
  ],
  random_float: [
    { name: 'min', label: '最小值', type: 'number', default: 0 },
    { name: 'max', label: '最大值', type: 'number', default: 1 },
    { name: 'precision', label: '小数位', type: 'number', default: 2, min: 0, max: 10 },
  ],
  random_decimal: [
    { name: 'min', label: '最小值', type: 'number', default: 0 },
    { name: 'max', label: '最大值', type: 'number', default: 1000 },
    { name: 'scale', label: '小数位', type: 'number', default: 2, min: 0, max: 10 },
  ],
  number_with_format: [
    { name: 'fmt', label: '格式模板', type: 'text', default: '###-###-####' },
  ],
  boolean: [
    { name: 'ratio', label: 'True 比例 (%)', type: 'number', default: 50, min: 0, max: 100 },
  ],
  constant: [
    { name: 'value', label: '常量值', type: 'text', default: '' },
  ],
  words: [
    { name: 'min', label: '最少词数', type: 'number', default: 3, min: 1 },
    { name: 'max', label: '最多词数', type: 'number', default: 8, min: 1 },
  ],
  sentence: [
    { name: 'min', label: '最少词数', type: 'number', default: 3, min: 1 },
    { name: 'max', label: '最多词数', type: 'number', default: 12, min: 1 },
  ],
  sentences: [
    { name: 'min', label: '最少句数', type: 'number', default: 2, min: 1 },
    { name: 'max', label: '最多句数', type: 'number', default: 5, min: 1 },
  ],
  paragraph: [
    { name: 'count', label: '句子数', type: 'number', default: 4, min: 1 },
  ],
  paragraphs: [
    { name: 'count', label: '段落数', type: 'number', default: 2, min: 1 },
  ],
  regex: [
    { name: 'pattern', label: '正则表达式', type: 'text', default: '[a-z]{5,10}' },
  ],
  template: [
    { name: 'template', label: '模板字符串', type: 'text', default: '' },
  ],
  password: [
    { name: 'min', label: '最小长度', type: 'number', default: 8, min: 1 },
    { name: 'max', label: '最大长度', type: 'number', default: 16, min: 1 },
  ],
  geohash: [
    { name: 'precision', label: '精度', type: 'number', default: 5, min: 1, max: 12 },
  ],
  datetime: [
    { name: 'min', label: '起始日期', type: 'text', default: '2020-01-01' },
    { name: 'max', label: '结束日期', type: 'text', default: '2026-12-31' },
  ],
  datetime_before: [
    { name: 'before', label: '早于', type: 'text', default: '2026-12-31' },
  ],
  datetime_after: [
    { name: 'after', label: '晚于', type: 'text', default: '2020-01-01' },
  ],
  datetime_between: [
    { name: 'start', label: '开始', type: 'text', default: '2020-01-01' },
    { name: 'end', label: '结束', type: 'text', default: '2026-12-31' },
  ],
  date: [
    { name: 'min', label: '起始', type: 'text', default: '2020-01-01' },
    { name: 'max', label: '结束', type: 'text', default: '2026-12-31' },
  ],
  image_url: [
    { name: 'width', label: '宽度', type: 'number', default: 640, min: 1 },
    { name: 'height', label: '高度', type: 'number', default: 480, min: 1 },
  ],
  image_url_with_seed: [
    { name: 'width', label: '宽度', type: 'number', default: 640, min: 1 },
    { name: 'height', label: '高度', type: 'number', default: 480, min: 1 },
    { name: 'seed', label: '种子', type: 'number', default: 0 },
  ],
  image_url_grayscale: [
    { name: 'width', label: '宽度', type: 'number', default: 640, min: 1 },
    { name: 'height', label: '高度', type: 'number', default: 480, min: 1 },
  ],
  image_url_blur: [
    { name: 'width', label: '宽度', type: 'number', default: 640, min: 1 },
    { name: 'height', label: '高度', type: 'number', default: 480, min: 1 },
    { name: 'blur_amount', label: '模糊量', type: 'number', default: 5, min: 1, max: 10 },
  ],
  image_url_custom: [
    { name: 'width', label: '宽度', type: 'number', default: 640, min: 1 },
    { name: 'height', label: '高度', type: 'number', default: 480, min: 1 },
    { name: 'grayscale', label: '灰度', type: 'boolean', default: false },
    { name: 'blurAmount', label: '模糊量', type: 'number', default: 0, min: 0, max: 10 },
    { name: 'seed', label: '种子', type: 'number', default: 0 },
  ],
  foreign_key: [
    { name: 'values', label: '备选值', type: 'string-array', default: [] },
  ],
  sequence: [
    { name: 'values', label: '序列值', type: 'string-array', default: [] },
    { name: 'cycle', label: '循环', type: 'boolean', default: true },
  ],
  weighted: [
    { name: 'choices', label: '加权选项', type: 'weighted-choices', default: [] },
  ],
}

const props = defineProps<{
  show: boolean
  generatorType: GeneratorType
  currentParams: Record<string, unknown>
  columnName: string
  columnIndex: number
}>()

const emit = defineEmits<{
  'update:show': [value: boolean]
  save: [index: number, params: Record<string, unknown>]
}>()

const visible = ref(props.show)
const localParams = ref<Record<string, unknown>>({ ...props.currentParams })

watch(() => props.show, (val) => {
  visible.value = val
  if (val) {
    localParams.value = { ...props.currentParams }
  }
})

watch(visible, (val) => emit('update:show', val))

const paramFields = computed(() =>
  GENERATOR_PARAM_SCHEMA[props.generatorType] ?? []
)

const generatorLabel = computed(() => {
  const gen = props.generatorType
  const labels: Partial<Record<GeneratorType, string>> = {
    auto_increment: '自动递增', random_int: '随机整数', random_float: '随机浮点数',
    random_decimal: '随机小数', number_with_format: '格式化数字', boolean: '布尔值',
    constant: '常量', words: '单词组', sentence: '句子', sentences: '多句',
    paragraph: '段落', paragraphs: '多段落', regex: '正则', template: '模板',
    password: '密码', geohash: 'GeoHash',
    datetime: '日期时间', datetime_before: '过去时间', datetime_after: '将来时间',
    datetime_between: '时间区间', date: '日期',
    image_url: '图片 URL', image_url_with_seed: '图片(种子)',
    image_url_grayscale: '图片(灰度)', image_url_blur: '图片(模糊)',
    image_url_custom: '图片(自定义)',
    foreign_key: '外键约束', sequence: '序列循环', weighted: '加权随机',
  }
  return labels[gen] ?? gen
})

function setParam(name: string, value: unknown) {
  localParams.value = { ...localParams.value, [name]: value }
}

function updateArrayItem(fieldName: string, index: number, value: string) {
  const arr = [...((localParams.value[fieldName] as string[]) ?? [])]
  arr[index] = value
  localParams.value = { ...localParams.value, [fieldName]: arr }
}

function addArrayItem(fieldName: string) {
  const arr = [...((localParams.value[fieldName] as string[]) ?? []), '']
  localParams.value = { ...localParams.value, [fieldName]: arr }
}

function removeArrayItem(fieldName: string, index: number) {
  const arr = ((localParams.value[fieldName] as string[]) ?? []).filter((_, i) => i !== index)
  localParams.value = { ...localParams.value, [fieldName]: arr }
}

function updateWeightedItem(fieldName: string, index: number, key: 'value' | 'weight', val: string | number) {
  const choices = (localParams.value[fieldName] as WeightedChoice[] | undefined) ?? []
  const updated = choices.map((item, i) => (i === index ? { ...item, [key]: val } : item))
  localParams.value = { ...localParams.value, [fieldName]: updated }
}

function addWeightedItem(fieldName: string) {
  const choices = [...((localParams.value[fieldName] as WeightedChoice[]) ?? []), { value: '', weight: 0.5 }]
  localParams.value = { ...localParams.value, [fieldName]: choices }
}

function onClose() {
  visible.value = false
}

function onSave() {
  const params: Record<string, unknown> = {}
  for (const field of paramFields.value) {
    const val = localParams.value[field.name]
    if (val !== undefined && val !== null && val !== '') {
      params[field.name] = val
    }
  }
  emit('save', props.columnIndex, params)
  visible.value = false
}
</script>

<style scoped>
.drawer-body {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.param-section {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.param-label {
  font-size: 12px;
  color: var(--color-text-muted);
}

.param-value {
  font-size: 13px;
  font-weight: 500;
  color: var(--brand-accent);
}

.empty-params {
  text-align: center;
  padding: 24px 0;
  font-size: 13px;
  color: var(--color-text-muted);
}

.params-form {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.param-field {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.param-field-label {
  font-size: 12px;
  font-weight: 500;
  color: var(--color-text-secondary);
}

.array-field {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.array-row {
  display: flex;
  gap: 4px;
  align-items: center;
}

.weighted-field {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.weighted-row {
  display: flex;
  gap: 4px;
  align-items: center;
}

.drawer-footer {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
}
</style>