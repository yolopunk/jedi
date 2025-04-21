<template>
  <!-- 通知消息组件 -->
  <v-snackbar
    v-model="show"
    :color="color"
    :timeout="timeout"
    location="top"
    rounded="pill"
    class="mt-6"
  >
    <div class="d-flex align-center">
      <v-icon
        :icon="getIcon"
        class="mr-2"
      ></v-icon>
      {{ text }}
    </div>
    <template v-slot:actions>
      <v-btn
        variant="text"
        @click="show = false"
        icon
        size="small"
      >
        <v-icon :icon="mdiClose"></v-icon>
      </v-btn>
    </template>
  </v-snackbar>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { mdiCheckCircle, mdiAlertCircle, mdiInformation, mdiClose } from '@mdi/js'

// 定义组件属性
const props = defineProps<{
  modelValue: boolean;
  text: string;
  color: 'success' | 'error' | 'info' | 'warning';
  timeout?: number;
}>()

// 定义组件事件
const emit = defineEmits<{
  (e: 'update:modelValue', value: boolean): void;
}>()

// 本地状态
const show = computed({
  get: () => props.modelValue,
  set: (value) => emit('update:modelValue', value)
})

// 计算图标
const getIcon = computed(() => {
  switch (props.color) {
    case 'success':
      return mdiCheckCircle
    case 'error':
      return mdiAlertCircle
    default:
      return mdiInformation
  }
})
</script>
