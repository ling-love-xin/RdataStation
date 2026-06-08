<template>
  <div class="adv-sec env-sec">
    <EnvironmentSelector
      :model-value="environmentId"
      :options="envSelectOpts"
      :label="$t('navigator.advancedEnv')"
      :manage-label="$t('navigator.manage')"
      @update:model-value="onEnvChange"
      @manage="showEnvMgr = true"
    />
    <div v-if="envPolicyTags.length" class="env-tags">
      <span v-for="tag in envPolicyTags" :key="tag.key" :class="['epi-tag', tag.kind]">{{
        tag.label
      }}</span>
    </div>
    <div v-if="