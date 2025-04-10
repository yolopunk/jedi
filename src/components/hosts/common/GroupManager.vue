<template>
  <!-- 分组管理区域 -->
  <div>
    <div class="d-flex justify-end pa-3">
      <v-btn
        @click="$emit('add-group')"
        color="#1976D2"
        variant="flat"
        size="small"
        rounded="pill"
        class="jedi-btn jedi-hover-lift"
        style="box-shadow: 0 2px 4px rgba(25, 118, 210, 0.2);"
      >
        <v-icon :icon="mdiPlus" size="small" class="mr-1"></v-icon>
        <span style="font-weight: 500; letter-spacing: 0.5px;">添加分组</span>
      </v-btn>
    </div>

    <!-- 分组选择区域 -->
    <div class="pa-2" style="background-color: #FAFBFD; border-top: 1px solid rgba(25, 118, 210, 0.05);">
      <div class="d-flex flex-wrap gap-2 px-2 py-1">
        <v-btn
          v-for="group in groups"
          :key="group.tag"
          :value="group.tag"
          :color="modelValue === group.tag ? '#1976D2' : '#E8F1FF'"
          :variant="modelValue === group.tag ? 'flat' : 'outlined'"
          class="group-btn"
          size="small"
          rounded="pill"
          :class="{'v-btn--active elevation-2': modelValue === group.tag}"
          @click="$emit('update:modelValue', group.tag)"
          :style="{
            minWidth: 'auto',
            borderColor: '#1976D2',
            fontWeight: 500,
            color: modelValue === group.tag ? 'white !important' : '#0D47A1 !important'
          }"
        >
          <v-icon :icon="mdiDomain" size="x-small" class="mr-1" :style="{color: modelValue === group.tag ? 'white !important' : '#0D47A1 !important'}"></v-icon>
          <span :style="{color: modelValue === group.tag ? 'white !important' : '#0D47A1 !important', fontWeight: 500}">{{ group.tag }}</span>
        </v-btn>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { mdiDomain, mdiPlus } from '@mdi/js'

// 定义组件属性
defineProps<{
  modelValue: string;
  groups: Array<{ tag: string; hosts: Array<Record<string, string>> }>;
}>()

// 定义组件事件
defineEmits<{
  (e: 'update:modelValue', value: string): void;
  (e: 'add-group'): void;
}>()
</script>
