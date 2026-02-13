<template>
  <!-- 重命名分组对话框 -->
  <v-dialog v-model="dialogModel" max-width="400">
    <v-card class="rounded-lg overflow-hidden jedi-dialog-card">
      <v-toolbar class="px-4 jedi-dialog-header">
        <v-icon :icon="mdiPencil" class="mr-2"></v-icon>
        <v-toolbar-title class="font-weight-medium">{{ $t('hosts.dialog.renameGroupTitle') }}</v-toolbar-title>
        <v-spacer></v-spacer>
        <v-btn icon @click="closeDialog">
          <v-icon :icon="mdiClose"></v-icon>
        </v-btn>
      </v-toolbar>
      <v-card-text class="pa-6">
        <v-text-field
          v-model="newGroupName"
          :label="$t('hosts.dialog.groupNameLabel')"
          variant="outlined"
          :placeholder="$t('hosts.dialog.groupNamePlaceholder')"
          required
          autofocus
          @keyup.enter="confirmRename"
        ></v-text-field>
      </v-card-text>
      <v-card-actions class="pa-6 pt-0">
        <v-spacer></v-spacer>
        <v-btn
          variant="text"
          @click="closeDialog"
          class="mr-2"
          color="grey-darken-1"
          rounded="sm"
        >
          {{ $t('common.cancel') }}
        </v-btn>
        <v-btn
          color="primary"
          variant="elevated"
          @click="confirmRename"
          rounded="sm"
          :disabled="!isValid"
        >
          {{ $t('common.confirm') }}
        </v-btn>
      </v-card-actions>
    </v-card>
  </v-dialog>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { mdiPencil, mdiClose } from '@mdi/js'

// 定义组件属性
const props = defineProps<{
  modelValue: boolean;
  groupName: string;
}>()

// 定义组件事件
const emit = defineEmits<{
  (e: 'update:modelValue', value: boolean): void;
  (e: 'rename', oldName: string, newName: string): void;
}>()

const newGroupName = ref('')

// 监听 props 变化
watch(() => props.modelValue, (val) => {
  if (val) {
    newGroupName.value = props.groupName
  }
})

// 计算属性：控制对话框显示
const dialogModel = computed({
  get: () => props.modelValue,
  set: (value) => emit('update:modelValue', value)
})

const isValid = computed(() => {
  return newGroupName.value.trim().length > 0 && newGroupName.value.trim() !== props.groupName
})

// 关闭对话框
function closeDialog() {
  dialogModel.value = false
}

// 确认重命名
function confirmRename() {
  if (!isValid.value) return
  emit('rename', props.groupName, newGroupName.value.trim())
  closeDialog()
}
</script>

<style scoped>
.jedi-dialog-card {
  border: 1px solid rgba(var(--v-border-color), var(--v-border-opacity));
}

.jedi-dialog-header {
  background-color: rgb(var(--v-theme-surface-variant));
  color: rgb(var(--v-theme-on-surface-variant));
}
</style>
