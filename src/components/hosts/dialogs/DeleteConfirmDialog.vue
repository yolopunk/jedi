<template>
  <!-- 删除确认对话框 -->
  <v-dialog v-model="dialogModel" max-width="400" persistent>
    <v-card class="rounded-lg">
      <v-card-title class="text-h5 pa-4 d-flex align-center" style="background-color: #FFEBEE; color: #D32F2F;">
        <v-icon :icon="mdiAlertCircle" color="#D32F2F" class="mr-2"></v-icon>
        <span>确认删除</span>
      </v-card-title>
      <v-card-text class="pa-4 pt-5">
        <p class="text-body-1">您确定要删除以下条目吗？</p>
        <div v-if="host" class="mt-3 pa-3" style="background-color: #F5F5F5; border-radius: 8px;">
          <div class="d-flex align-center mb-1">
            <v-icon :icon="mdiIpNetwork" size="small" color="#1976D2" class="mr-2"></v-icon>
            <span class="font-weight-medium">IP地址：</span>
            <span class="ml-2">{{ host.ip }}</span>
          </div>
          <div class="d-flex align-center">
            <v-icon :icon="mdiDomain" size="small" color="#1976D2" class="mr-2"></v-icon>
            <span class="font-weight-medium">域名：</span>
            <span class="ml-2">{{ host.domain }}</span>
          </div>
        </div>
        <p class="text-body-2 mt-4" style="color: #F44336;">此操作不可撤销，删除后将立即生效。</p>
      </v-card-text>
      <v-card-actions class="pa-4 pt-0">
        <v-spacer></v-spacer>
        <v-btn
          variant="text"
          @click="closeDialog"
          class="mr-2"
          color="grey-darken-1"
        >
          取消
        </v-btn>
        <v-btn
          color="#F44336"
          variant="flat"
          @click="confirmDelete"
          class="px-4"
        >
          确认删除
        </v-btn>
      </v-card-actions>
    </v-card>
  </v-dialog>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { mdiAlertCircle, mdiIpNetwork, mdiDomain } from '@mdi/js'

// 定义组件属性
const props = defineProps<{
  modelValue: boolean;
  host: any | null;
}>()

// 定义组件事件
const emit = defineEmits<{
  (e: 'update:modelValue', value: boolean): void;
  (e: 'delete', host: any): void;
}>()

// 对话框状态
const dialogModel = computed({
  get: () => props.modelValue,
  set: (value) => emit('update:modelValue', value)
})

// 关闭对话框
function closeDialog() {
  dialogModel.value = false
}

// 确认删除
function confirmDelete() {
  if (props.host) {
    emit('delete', props.host)
  }
  closeDialog()
}
</script>
