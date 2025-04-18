<template>
  <v-dialog v-model="dialogModel" max-width="700" class="jedi-dialog-card">
    <v-card class="jedi-dialog-card">
      <v-toolbar style="background: linear-gradient(135deg, #1A2530 0%, #2C3E50 100%); border-bottom: 1px solid rgba(52, 152, 219, 0.3);" class="px-4 jedi-dialog-header">
        <v-icon :icon="mdiCog" color="white" class="mr-2"></v-icon>
        <v-toolbar-title class="font-weight-medium">应用设置</v-toolbar-title>
        <v-spacer></v-spacer>
        <v-btn :icon="mdiClose" variant="text" color="white" @click="dialogModel = false"></v-btn>
      </v-toolbar>
      <v-card-text class="pa-6">
        <v-tabs v-model="settingsTab" color="var(--jedi-accent)">
          <v-tab value="general">常规设置</v-tab>
          <v-tab value="appearance">外观设置</v-tab>
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
                  <v-switch color="var(--jedi-accent)" hide-details></v-switch>
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

          <!-- 外观设置 -->
          <v-window-item value="appearance">
            <v-list>
              <v-list-item>
                <template v-slot:prepend>
                  <v-icon :icon="mdiThemeLightDark" color="var(--jedi-primary)" class="mr-3"></v-icon>
                </template>
                <v-list-item-title>主题模式</v-list-item-title>
                <template v-slot:append>
                  <v-select
                    :items="['浅色', '深色', '跟随系统']"
                    variant="outlined"
                    density="compact"
                    hide-details
                    style="width: 150px"
                  ></v-select>
                </template>
              </v-list-item>

              <v-list-item>
                <template v-slot:prepend>
                  <v-icon :icon="mdiPalette" color="var(--jedi-primary)" class="mr-3"></v-icon>
                </template>
                <v-list-item-title>主题颜色</v-list-item-title>
                <template v-slot:append>
                  <div class="d-flex">
                    <v-btn :icon="mdiCheckboxBlankCircle" size="small" color="#2C3E50" class="mr-1"></v-btn>
                    <v-btn :icon="mdiCheckboxBlankCircle" size="small" color="#3498DB" class="mr-1"></v-btn>
                    <v-btn :icon="mdiCheckboxBlankCircle" size="small" color="#D4AF37" class="mr-1"></v-btn>
                    <v-btn :icon="mdiCheckboxBlankCircle" size="small" color="#4CAF50" class="mr-1"></v-btn>
                  </div>
                </template>
              </v-list-item>

              <v-list-item>
                <template v-slot:prepend>
                  <v-icon :icon="mdiFormatFont" color="var(--jedi-primary)" class="mr-3"></v-icon>
                </template>
                <v-list-item-title>字体大小</v-list-item-title>
                <template v-slot:append>
                  <v-slider
                    color="var(--jedi-accent)"
                    min="12"
                    max="18"
                    step="1"
                    hide-details
                    style="width: 150px"
                  ></v-slider>
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
        <v-btn color="var(--jedi-accent)" variant="flat" @click="dialogModel = false" rounded="sm">
          保存
        </v-btn>
      </v-card-actions>
    </v-card>
  </v-dialog>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import {
  mdiCog,
  mdiClose,
  mdiLaunch,
  mdiTrayArrowDown,
  mdiUpdate,
  mdiThemeLightDark,
  mdiPalette,
  mdiFormatFont,
  mdiFileDocument,
  mdiBackupRestore,
  mdiRefresh,
  mdiCheckboxBlankCircle
} from '@mdi/js'

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
</script>
