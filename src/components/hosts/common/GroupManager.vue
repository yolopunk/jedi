<template>
  <!-- 分组管理区域 -->
  <div>
    <div class="d-flex justify-end pa-3">
      <v-btn
        @click="$emit('add-group')"
        variant="flat"
        size="small"
        rounded="sm"
        class="jedi-btn jedi-hover-lift"
        style="background-color: var(--jedi-accent); color: white; box-shadow: 0 2px 4px rgba(52, 152, 219, 0.2); text-transform: uppercase; letter-spacing: 0.5px;"
      >
        <v-icon :icon="mdiPlus" size="small" class="mr-1"></v-icon>
        <span style="font-weight: 500; letter-spacing: 0.5px;">添加分组</span>
      </v-btn>
    </div>

    <!-- 分组选择区域 -->
    <div class="pa-2" style="background-color: #F8F9FA; border-top: 1px solid rgba(52, 152, 219, 0.1); border-radius: 0 0 8px 8px;">
      <div class="d-flex flex-wrap gap-2 px-2 py-1">
        <v-btn
          v-for="group in groups"
          :key="group.name"
          :value="group.name"
          :color="modelValue === group.name ? 'var(--jedi-accent)' : '#ECF0F1'"
          :variant="modelValue === group.name ? 'flat' : 'outlined'"
          class="group-btn"
          size="small"
          rounded="pill"
          :class="{'v-btn--active elevation-2': modelValue === group.name}"
          @click="$emit('update:modelValue', group.name)"
          :style="{
            minWidth: 'auto',
            borderColor: 'var(--jedi-accent)',
            fontWeight: 500,
            color: modelValue === group.name ? 'white !important' : 'var(--jedi-primary) !important'
          }"
        >
          <v-icon :icon="mdiDomain" size="x-small" class="mr-1" :style="{color: modelValue === group.name ? 'white !important' : 'var(--jedi-primary) !important'}"></v-icon>
          <span :style="{color: modelValue === group.name ? 'white !important' : 'var(--jedi-primary) !important', fontWeight: 500}">{{ group.name }}</span>
        </v-btn>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { mdiDomain, mdiPlus } from '@mdi/js'
import { Group } from '@/types/hosts'

// 定义组件属性
defineProps<{
  modelValue: string;
  groups: Group[];
}>()

// 定义组件事件
defineEmits<{
  (e: 'update:modelValue', value: string): void;
  (e: 'add-group'): void;
}>()
</script>
