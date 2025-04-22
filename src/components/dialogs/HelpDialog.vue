<template>
  <v-dialog v-model="dialogModel" max-width="600" class="jedi-dialog-card">
    <v-card class="jedi-dialog-card">
      <v-toolbar style="background: linear-gradient(135deg, #1A2530 0%, #2C3E50 100%); border-bottom: 1px solid rgba(52, 152, 219, 0.3);" class="px-4 jedi-dialog-header">
        <v-icon :icon="mdiHelpCircle" color="white" class="mr-2"></v-icon>
        <v-toolbar-title class="font-weight-medium">帮助中心</v-toolbar-title>
        <v-spacer></v-spacer>
        <v-btn :icon="mdiClose" variant="text" color="white" @click="dialogModel = false"></v-btn>
      </v-toolbar>
      <v-card-text class="pa-6">
        <h3 class="text-h6 mb-4">基本操作指南</h3>
        <v-list>
          <v-list-item>
            <template v-slot:prepend>
              <v-icon :icon="mdiPlusCircle" color="var(--jedi-accent)" class="mr-2"></v-icon>
            </template>
            <v-list-item-title>添加分组：点击 "+" 按钮创建新的 hosts 配置组</v-list-item-title>
          </v-list-item>
          <v-list-item>
            <template v-slot:prepend>
              <v-icon :icon="mdiPlus" color="var(--jedi-accent)" class="mr-2"></v-icon>
            </template>
            <v-list-item-title>添加条目：在分组中点击 "添加条目" 按钮添加新的 hosts 条目</v-list-item-title>
          </v-list-item>
          <v-list-item>
            <template v-slot:prepend>
              <v-icon :icon="mdiToggleSwitch" color="var(--jedi-accent)" class="mr-2"></v-icon>
            </template>
            <v-list-item-title>启用/禁用条目：使用每个条目旁边的开关控制其状态</v-list-item-title>
          </v-list-item>
          <v-list-item>
            <template v-slot:prepend>
              <v-icon :icon="mdiPower" color="var(--jedi-accent)" class="mr-2"></v-icon>
            </template>
            <v-list-item-title>全局开关：使用顶部的主开关控制所有 hosts 配置</v-list-item-title>
          </v-list-item>
        </v-list>
      </v-card-text>
      <v-card-actions class="pa-4 pt-0">
        <v-spacer></v-spacer>
        <v-btn color="var(--jedi-accent)" variant="elevated" @click="dialogModel = false" rounded="sm">
          关闭
        </v-btn>
      </v-card-actions>
    </v-card>
  </v-dialog>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { mdiHelpCircle, mdiClose, mdiPlusCircle, mdiPlus, mdiToggleSwitch, mdiPower } from '@mdi/js'

// 定义组件属性
const props = defineProps<{
  modelValue: boolean;
}>()

// 定义组件事件
const emit = defineEmits<{
  (e: 'update:modelValue', value: boolean): void;
}>()

// 对话框状态
const dialogModel = computed({
  get: () => props.modelValue,
  set: (value) => emit('update:modelValue', value)
})
</script>
