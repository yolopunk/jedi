<template>
  <v-dialog v-model="dialogModel" max-width="500" class="jedi-dialog-card">
    <v-card class="jedi-dialog-card">
      <v-toolbar style="background: linear-gradient(135deg, #1A2530 0%, #2C3E50 100%); border-bottom: 1px solid rgba(52, 152, 219, 0.3);" class="px-4 jedi-dialog-header">
        <v-icon :icon="mdiInformation" color="white" class="mr-2"></v-icon>
        <v-toolbar-title class="font-weight-medium">关于 Jedi 工具箱</v-toolbar-title>
        <v-spacer></v-spacer>
        <v-btn :icon="mdiClose" variant="text" color="white" @click="dialogModel = false"></v-btn>
      </v-toolbar>
      <v-card-text class="pa-6 text-center">
        <v-img src="/icon.png" width="100" class="mx-auto mb-4"></v-img>
        <h2 class="text-h5 font-weight-bold mb-2">Jedi 工具箱</h2>
        <p class="text-body-1 mb-4">版本 v{{ appInfo.version }}</p>
        <p class="text-body-2 mb-1">多功能开发辅助工具集</p>
        <p class="text-body-2 mb-4">基于 Tauri + Vue + Vuetify 构建</p>

        <v-divider class="mb-4"></v-divider>

        <div class="d-flex justify-center mb-2">
          <v-btn :icon="mdiGithub" variant="text" color="var(--jedi-primary)" @click="$emit('open-github')" size="small" class="mx-1"></v-btn>
          <v-btn :icon="mdiWeb" variant="text" color="var(--jedi-primary)" @click="$emit('open-website')" size="small" class="mx-1"></v-btn>
          <v-btn :icon="mdiEmail" variant="text" color="var(--jedi-primary)" @click="$emit('open-email')" size="small" class="mx-1"></v-btn>
        </div>

        <p class="text-caption text-grey-darken-1 mt-2">© 2025 Jedi. 保留所有权利。</p>
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
import { computed, ref, onMounted } from 'vue'
import { mdiInformation, mdiClose, mdiGithub, mdiWeb, mdiEmail } from '@mdi/js'
import { getAppInfo, type AppInfo } from '@/api/app'

// 定义组件属性
const props = defineProps<{
  modelValue: boolean;
}>()

// 定义组件事件
const emit = defineEmits<{
  (e: 'update:modelValue', value: boolean): void;
  (e: 'open-github'): void;
  (e: 'open-website'): void;
  (e: 'open-email'): void;
}>()

// 对话框状态
const dialogModel = computed({
  get: () => props.modelValue,
  set: (value) => emit('update:modelValue', value)
})

// 应用信息
const appInfo = ref<AppInfo>({
  version: '0.0.0',
  name: 'Jedi'
})

// 获取应用信息
onMounted(async () => {
  try {
    appInfo.value = await getAppInfo()
  } catch (error) {
    console.error('Failed to get app info:', error)
  }
})
</script>
