<template>
  <!-- 空状态显示 -->
  <div class="jedi-card-content text-center pa-12" style="background-color: #f5f7fa;">
    <div class="d-flex flex-column align-center">
      <v-icon :icon="mdiDomain" size="80" color="#1976D2" class="death-star jedi-hover-scale" style="filter: drop-shadow(0 4px 8px rgba(25, 118, 210, 0.3));"></v-icon>
    </div>
    <h2 class="text-h5 font-weight-bold mb-2" style="color: #1976D2; letter-spacing: 0.5px;">暂无解析配置</h2>
    <p class="text-body-1 text-grey-darken-1 mb-8 max-width-500 mx-auto" style="letter-spacing: 0.3px;">您可以手动添加分组或使用默认配置来开始管理您的 hosts 文件</p>
    <div class="d-flex justify-center">
      <v-btn
        color="#1976D2"
        variant="flat"
        class="mr-4 px-6 jedi-hover-lift"
        rounded="pill"
        size="large"
        :prepend-icon="mdiPlus"
        @click="$emit('add-group')"
        style="box-shadow: 0 2px 4px rgba(25, 118, 210, 0.2);"
      >
        <span style="font-weight: 500; letter-spacing: 0.5px;">添加分组</span>
      </v-btn>
      <v-btn
        color="#4CAF50"
        variant="flat"
        class="px-6 jedi-hover-lift"
        rounded="pill"
        size="large"
        :prepend-icon="mdiDomain"
        @click="$emit('use-default')"
        style="box-shadow: 0 2px 4px rgba(76, 175, 80, 0.2);"
      >
        <span style="font-weight: 500; letter-spacing: 0.5px;">使用默认配置</span>
      </v-btn>
    </div>
  </div>
</template>

<script setup lang="ts">
import { mdiDomain, mdiPlus } from '@mdi/js'

// 定义组件事件
defineEmits<{
  (e: 'add-group'): void;
  (e: 'use-default'): void;
}>()
</script>
