<template>
  <div v-if="qualityScore" class="quality-score-section">
    <div class="quality-badge" :class="scoreLevelClass">
      <span class="quality-score-num">{{ Math.round(qualityScore.overall_score) }}</span>
      <span class="quality-level">{{ qualityScore.level }}</span>
    </div>
    <div class="quality-summary">{{ qualityScore.summary }}</div>
    <div class="quality-dimensions">
      <div v-for="dim in qualityScore.dimensions" :key="dim.name" class="quality-dim">
        <div class="dim-header">
          <span class="dim-name">{{ dim.name }}</span>
          <span class="dim-score">{{ Math.round(dim.score) }}</span>
        </div>
        <div class="dim-bar-track">
          <div
            class="dim-bar-fill"
            :style="{ width: dim.score + '%' }"
            :class="dimScoreBarClass(dim.score)"
          />
        </div>
        <span class="dim-detail">{{ dim.detail }}</span>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'

import type { QualityScore } from '../../../services/result-analysis'

const props = defineProps<{
  qualityScore: QualityScore | null
}>()

const scoreLevelClass = computed(() => {
  if (!props.qualityScore) return ''
  const s = props.qualityScore.overall_score
  if (s >= 85) return 'score-excellent'
  if (s >= 70) return 'score-good'
  if (s >= 50) return 'score-fair'
  if (s >= 30) return 'score-poor'
  return 'score-bad'
})

function dimScoreBarClass(score: number): string {
  if (score >= 80) return 'bar-good'
  if (score >= 50) return 'bar-fair'
  return 'bar-poor'
}
</script>

<style scoped>
.quality-score-section {
  padding: 10px 12px;
  margin: 6px 0;
  background: var(--bg-elevated, #2a2a2a);
  border-radius: 6px;
  border: 1px solid var(--border-color, #333);
}
.quality-badge {
  display: inline-flex;
  align-items: baseline;
  gap: 6px;
  padding: 4px 10px;
  border-radius: 5px;
  margin-bottom: 6px;
}
.quality-badge.score-excellent { background: rgba(0, 184, 148, 0.15); color: var(--brand-success); }
.quality-badge.score-good     { background: rgba(0, 206, 201, 0.15); color: #00cec9; }
.quality-badge.score-fair     { background: rgba(253, 203, 110, 0.15); color: var(--brand-warning); }
.quality-badge.score-poor     { background: rgba(214, 48, 49, 0.15); color: var(--brand-danger); }
.quality-badge.score-bad      { background: rgba(214, 48, 49, 0.25); color: var(--brand-danger); }

.quality-score-num { font-size: 20px; font-weight: 700; line-height: 1; }
.quality-level { font-size: 11px; opacity: 0.8; }
.quality-summary { font-size: 11px; color: var(--text-secondary, #aaa); margin-bottom: 8px; line-height: 1.4; }

.quality-dimensions { display: flex; flex-direction: column; gap: 6px; }
.quality-dim { display: flex; flex-direction: column; gap: 3px; }
.dim-header { display: flex; justify-content: space-between; align-items: center; }
.dim-name { font-size: 11px; font-weight: 500; }
.dim-score { font-size: 12px; font-weight: 600; }

.dim-bar-track {
  height: 4px;
  background: var(--bg-secondary, #333);
  border-radius: 2px;
  overflow: hidden;
}
.dim-bar-fill { height: 100%; border-radius: 2px; transition: width 0.4s ease; }
.dim-bar-fill.bar-good { background: var(--brand-success); }
.dim-bar-fill.bar-fair { background: var(--brand-warning); }
.dim-bar-fill.bar-poor { background: var(--brand-danger); }

.dim-detail { font-size: 10px; color: var(--text-tertiary, #888); line-height: 1.3; }
</style>