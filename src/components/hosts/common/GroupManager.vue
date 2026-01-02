<template>
  <!-- 分组管理区域 -->
  <div class="group-tabs-container sticky-header">

    <!-- 分组标签页 -->
    <v-tabs
      v-model="selectedTab"
      bg-color="transparent"
      color="var(--jedi-accent)"
      align-tabs="start"
      show-arrows
      slider-color="var(--jedi-accent)"
      class="group-tabs"
    >
      <v-tab
        v-for="group in groups"
        :key="group.name"
        :value="group.name"
        class="group-tab"
        @click="$emit('update:modelValue', group.name)"
      >
        <v-icon :icon="mdiDomain" size="small" class="mr-1"></v-icon>
        {{ group.name }}
      </v-tab>

      <!-- 添加分组按钮作为最后一个标签 -->
      <v-tab
        value="add-group"
        class="add-group-tab"
        @click="$emit('add-group')"
      >
        <v-icon :icon="mdiPlus" size="small" class="mr-1"></v-icon>
        {{ $t('hosts.groups.add') }}
      </v-tab>
    </v-tabs>
  </div>
</template>

<script setup lang="ts">
import { mdiDomain, mdiPlus } from '@mdi/js'
import { Group } from '@/types/hosts'
import { computed } from 'vue'

// 定义组件属性
const props = defineProps<{
  modelValue: string;
  groups: Group[];
}>()

// 定义组件事件
const emit = defineEmits<{
  (e: 'update:modelValue', value: string): void;
  (e: 'add-group'): void;
}>()

// 计算属性：当前选中的标签
const selectedTab = computed({
  get: () => props.modelValue,
  set: (value) => {
    if (value !== 'add-group') {
      emit('update:modelValue', value)
    }
  }
})
</script>

<style scoped>
.sticky-header {
  position: sticky;
  top: 0;
  z-index: 10;
  background-color: var(--jedi-bg-surface);
  border-bottom: 1px solid var(--jedi-border);
  transition: background-color 0.3s ease;
}

.group-tabs-container {
  /* Border handled by sticky-header */
}

.group-tabs {
  border-radius: 0;
}

.group-tab {
  min-width: 100px;
  font-weight: 500;
  text-transform: none;
  letter-spacing: 0.3px;
  transition: all 0.3s ease;
  color: var(--jedi-text-secondary) !important;
}

.group-tab.v-tab--selected {
  color: var(--jedi-text-primary) !important;
  font-weight: 600;
}

.add-group-tab {
  min-width: 120px;
  font-weight: 500;
  text-transform: none;
  letter-spacing: 0.3px;
  color: var(--jedi-success) !important;
  transition: all 0.3s ease;
}

.add-group-tab:hover {
  background-color: var(--jedi-bg-surface-hover) !important;
}
</style>
