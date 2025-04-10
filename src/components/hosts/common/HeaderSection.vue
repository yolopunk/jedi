<template>
  <!-- 应用标题区域 -->
  <div class="app-header mb-4 pa-4 jedi-hover-bright lightsaber-glow" 
       style="background: linear-gradient(135deg, #1565C0 0%, #1976D2 100%); 
              border-radius: 12px; 
              box-shadow: 0 4px 12px rgba(25, 118, 210, 0.2);">
    <div class="d-flex justify-space-between align-center">
      <div class="d-flex align-center">
        <div class="jedi-logo mr-4">
          <v-icon :icon="mdiSwordCross" size="x-large" color="white" 
                 class="lightsaber-icon jedi-hover-scale" 
                 style="filter: drop-shadow(0 0 8px rgba(255,255,255,0.5));"></v-icon>
        </div>
        <div>
          <h1 class="text-h4 font-weight-bold mb-1 jedi-title">
            <span class="jedi-text" style="letter-spacing: 1px; text-shadow: 0 2px 4px rgba(0,0,0,0.2);">JEDI</span>
            <span class="hosts-text" style="letter-spacing: 1px; text-shadow: 0 2px 4px rgba(0,0,0,0.2);">HOSTS</span>
            <span class="text-h5 font-weight-medium" style="color: white; text-shadow: 0 0 10px rgba(255,255,255,0.8);">管理器</span>
          </h1>
          <p class="text-subtitle-1 mb-0" style="color: rgba(255,255,255,0.9); font-weight: 500; letter-spacing: 0.3px;">
            管理您的 hosts 文件配置，提高开发效率
          </p>
        </div>
      </div>
      <div class="d-flex align-center">
        <v-chip
          :color="modelValue ? '#4CAF50' : '#757575'"
          :text-color="'white'"
          size="small"
          class="mr-3 px-4 force-chip jedi-hover-lift"
          variant="elevated"
          elevation="2"
          style="border: 1px solid rgba(255,255,255,0.2); letter-spacing: 0.5px;"
        >
          <v-icon start :icon="modelValue ? mdiCheckCircle : mdiPauseCircle" size="small"></v-icon>
          <span style="font-weight: 500;">{{ modelValue ? '已启用' : '已禁用' }}</span>
        </v-chip>
        <div class="d-flex align-center global-switch-container pa-1 px-3 rounded-pill" 
             style="background: rgba(255,255,255,0.15); backdrop-filter: blur(5px); 
                    border: 1px solid rgba(255,255,255,0.2); min-width: 140px; height: 36px;">
          <span class="mr-2" style="color: white; font-weight: 500; font-size: 0.85rem; 
                                    text-shadow: 0 0 5px rgba(0,0,0,0.3);">全局开关</span>
          <v-switch
            :model-value="modelValue"
            hide-details
            color="success"
            @update:model-value="(val) => val !== null && $emit('update:modelValue', val)"
            density="compact"
            class="global-switch lightsaber-switch green-switch"
            style="transform: scale(0.75); margin-right: 8px; width: 40px;"
          ></v-switch>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { mdiSwordCross, mdiCheckCircle, mdiPauseCircle } from '@mdi/js'

// 定义组件属性
defineProps<{
  modelValue: boolean;
}>()

// 定义组件事件
defineEmits<{
  (e: 'update:modelValue', value: boolean): void;
}>()
</script>
