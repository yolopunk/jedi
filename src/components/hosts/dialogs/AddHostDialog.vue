<template>
  <!-- 添加条目对话框 -->
  <v-dialog v-model="dialogModel" max-width="500" persistent>
    <v-card class="rounded-lg overflow-hidden">
      <v-toolbar color="#42b983" class="px-4">
        <v-icon :icon="mdiDns" class="mr-2" color="white"></v-icon>
        <v-toolbar-title class="font-weight-medium">新增条目</v-toolbar-title>
        <v-spacer></v-spacer>
        <v-btn icon @click="closeDialog">
          <v-icon :icon="mdiClose" color="white"></v-icon>
        </v-btn>
      </v-toolbar>
      <v-card-text class="pa-6">
        <v-text-field
          v-model="hostIp"
          label="IP地址"
          variant="outlined"
          placeholder="例如: 127.0.0.1"
          required
          bg-color="white"
          :prepend-inner-icon="mdiIpNetwork"
        ></v-text-field>
        <v-text-field
          v-model="hostDomain"
          label="域名"
          variant="outlined"
          placeholder="例如: example.com"
          required
          bg-color="white"
          :prepend-inner-icon="mdiDomain"
        ></v-text-field>
      </v-card-text>
      <v-card-actions class="pa-6 pt-0">
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
          color="#4CAF50"
          variant="elevated"
          @click="confirmAdd"
          class="lightsaber-btn green"
        >
          确认
        </v-btn>
      </v-card-actions>
    </v-card>
  </v-dialog>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { mdiDns, mdiClose, mdiIpNetwork, mdiDomain } from '@mdi/js'
import { validateHostInput } from '@/utils/hostsUtils'

// 定义组件属性
const props = defineProps<{
  modelValue: boolean;
  groupTag: string;
}>()

// 定义组件事件
const emit = defineEmits<{
  (e: 'update:modelValue', value: boolean): void;
  (e: 'add', data: {
    groupTag: string;
    ip: string;
    domain: string;
  }): void;
  (e: 'error', message: string): void;
}>()

// 对话框状态
const dialogModel = computed({
  get: () => props.modelValue,
  set: (value) => emit('update:modelValue', value)
})

// 表单数据
const hostIp = ref('')
const hostDomain = ref('')

// 关闭对话框
function closeDialog() {
  dialogModel.value = false
  resetForm()
}

// 重置表单
function resetForm() {
  hostIp.value = ''
  hostDomain.value = ''
}

// 确认添加
function confirmAdd() {
  // 验证输入
  if (!validateHostInput(hostIp.value, hostDomain.value)) {
    emit('error', 'IP和域名不能为空')
    return
  }

  // 提交添加事件
  emit('add', {
    groupTag: props.groupTag,
    ip: hostIp.value.trim(),
    domain: hostDomain.value.trim()
  })

  // 关闭对话框
  closeDialog()
}
</script>
