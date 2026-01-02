<template>
  <v-dialog v-model="dialogModel" max-width="700" class="jedi-dialog-card">
    <v-card class="jedi-dialog-card">
      <v-toolbar color="surface" class="px-4 jedi-dialog-header border-b">
        <v-icon :icon="mdiCog" color="primary" class="mr-2"></v-icon>
        <v-toolbar-title class="font-weight-medium">应用设置</v-toolbar-title>
        <v-spacer></v-spacer>
        <v-btn :icon="mdiClose" variant="text" color="medium-emphasis" @click="dialogModel = false"></v-btn>
      </v-toolbar>
      <v-card-text class="pa-6">
        <v-tabs v-model="settingsTab" color="var(--jedi-accent)">
          <v-tab value="general">常规设置</v-tab>
          <v-tab value="advanced">高级设置</v-tab>
        </v-tabs>

        <v-window v-model="settingsTab" class="mt-4">
          <!-- 常规设置 -->
          <v-window-item value="general">
            <v-list>
              <v-list-item>
                <template v-slot:prepend>
                  <v-icon :icon="mdiLaunch" color="var(--jedi-primary)" class="mr-3"></v-icon>
                </template>
                <v-list-item-title>开机自启动</v-list-item-title>
                <template v-slot:append>
                  <v-switch
                    v-model="autostartEnabled"
                    color="var(--jedi-accent)"
                    hide-details
                    :loading="autostartLoading"
                    @update:model-value="toggleAutostart"
                  ></v-switch>
                </template>
              </v-list-item>

              <v-list-item>
                <template v-slot:prepend>
                  <v-icon :icon="mdiTrayArrowDown" color="var(--jedi-primary)" class="mr-3"></v-icon>
                </template>
                <v-list-item-title>最小化到托盘</v-list-item-title>
                <template v-slot:append>
                  <v-switch color="var(--jedi-accent)" hide-details></v-switch>
                </template>
              </v-list-item>

              <v-list-item>
                <template v-slot:prepend>
                  <v-icon :icon="mdiUpdate" color="var(--jedi-primary)" class="mr-3"></v-icon>
                </template>
                <v-list-item-title>自动检查更新</v-list-item-title>
                <template v-slot:append>
                  <v-switch color="var(--jedi-accent)" hide-details></v-switch>
                </template>
              </v-list-item>
            </v-list>
          </v-window-item>

          <!-- 高级设置 -->
          <v-window-item value="advanced">
            <v-list>
              <v-list-item>
                <template v-slot:prepend>
                  <v-icon :icon="mdiFileDocument" color="var(--jedi-primary)" class="mr-3"></v-icon>
                </template>
                <v-list-item-title>Hosts 文件路径</v-list-item-title>
                <template v-slot:append>
                  <v-text-field
                    variant="outlined"
                    density="compact"
                    hide-details
                    readonly
                    value="/etc/hosts"
                    style="width: 250px"
                  ></v-text-field>
                </template>
              </v-list-item>

              <v-list-item>
                <template v-slot:prepend>
                  <v-icon :icon="mdiBackupRestore" color="var(--jedi-primary)" class="mr-3"></v-icon>
                </template>
                <v-list-item-title>备份设置</v-list-item-title>
                <template v-slot:append>
                  <v-btn color="var(--jedi-accent)" variant="tonal" size="small" rounded="sm">备份</v-btn>
                </template>
              </v-list-item>

              <v-list-item>
                <template v-slot:prepend>
                  <v-icon :icon="mdiRefresh" color="var(--jedi-primary)" class="mr-3"></v-icon>
                </template>
                <v-list-item-title>重置应用</v-list-item-title>
                <template v-slot:append>
                  <v-btn color="var(--jedi-danger)" variant="tonal" size="small" rounded="sm">重置</v-btn>
                </template>
              </v-list-item>
            </v-list>
          </v-window-item>
        </v-window>
      </v-card-text>
      <v-card-actions class="pa-4 pt-0">
        <v-spacer></v-spacer>
        <v-btn variant="text" @click="dialogModel = false" rounded="sm" class="mr-2">
          取消
        </v-btn>
        <v-btn color="var(--jedi-accent)" variant="elevated" @click="dialogModel = false" rounded="sm">
          保存
        </v-btn>
      </v-card-actions>
    </v-card>
  </v-dialog>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import {
  mdiCog,
  mdiClose,
  mdiLaunch,
  mdiTrayArrowDown,
  mdiUpdate,
  mdiFileDocument,
  mdiBackupRestore,
  mdiRefresh
} from '@mdi/js'
import { enableAutostart, disableAutostart, isAutostartEnabled } from '@/api/app'

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

// 设置选项卡
const settingsTab = ref('general')

// 自启动相关状态
const autostartEnabled = ref(false)
const autostartLoading = ref(false)

// 切换自启动状态
async function toggleAutostart(value: boolean | null) {
  if (value === null) return
  try {
    autostartLoading.value = true
    if (value) {
      await enableAutostart()
    } else {
      await disableAutostart()
    }
  } catch (error) {
    console.error('切换自启动状态失败:', error)
    // 恢复原来的状态
    autostartEnabled.value = !value
  } finally {
    autostartLoading.value = false
  }
}

// 检查自启动状态
async function checkAutostartStatus() {
  try {
    autostartLoading.value = true
    const enabled = await isAutostartEnabled()
    autostartEnabled.value = enabled
  } catch (error) {
    console.error('检查自启动状态失败:', error)
  } finally {
    autostartLoading.value = false
  }
}

// 组件挂载时检查自启动状态
onMounted(() => {
  checkAutostartStatus()
})
</script>
