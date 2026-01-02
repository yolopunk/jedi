<template>
  <!-- 编辑条目对话框 -->
  <v-dialog v-model="dialogModel" max-width="500" persistent>
    <v-card class="rounded-lg overflow-hidden jedi-dialog-card">
      <v-toolbar class="px-4 jedi-dialog-header">
        <v-icon :icon="mdiPencil" class="mr-2"></v-icon>
        <v-toolbar-title class="font-weight-medium">{{ $t('hosts.dialog.editTitle') }}</v-toolbar-title>
        <v-spacer></v-spacer>
        <v-btn icon @click="closeDialog">
          <v-icon :icon="mdiClose"></v-icon>
        </v-btn>
      </v-toolbar>
      <v-card-text class="pa-6">
        <v-text-field
          v-model="hostIp"
          :label="$t('hosts.dialog.ipLabel')"
          variant="outlined"
          :placeholder="$t('hosts.dialog.ipPlaceholder')"
          required
          :prepend-inner-icon="mdiIpNetwork"
          @update:model-value="debouncedUpdateIp"
          validate-on="blur"
          :rules="[rules.required]"
        ></v-text-field>
        <v-text-field
          v-model="hostDomain"
          :label="$t('hosts.dialog.domainLabel')"
          variant="outlined"
          :placeholder="$t('hosts.dialog.domainPlaceholder')"
          required
          :prepend-inner-icon="mdiDomain"
          @update:model-value="debouncedUpdateDomain"
          validate-on="blur"
          :rules="[rules.required]"
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
          color="success"
          variant="elevated"
          @click="confirmEdit"
          rounded="sm"
        >
          {{ $t('common.confirm') }}
        </v-btn>
      </v-card-actions>
    </v-card>
  </v-dialog>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { mdiPencil, mdiClose, mdiIpNetwork, mdiDomain } from '@mdi/js'
import { validateHostInput } from '@/utils/hostsUtils'

// 定义组件属性
const props = defineProps<{
  modelValue: boolean;
  host: any | null;
}>()

const { t } = useI18n()

// 定义组件事件
const emit = defineEmits<{
  (e: 'update:modelValue', value: boolean): void;
  (e: 'edit', data: {
    originalHost: any;
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

// 验证规则
const rules = {
  required: (value: string) => !!value?.trim() || t('hosts.dialog.required')
}

// 表单数据
const hostIp = ref('')
const hostDomain = ref('')

// 防抖函数
const debounce = (fn: Function, delay: number) => {
  let timeoutId: NodeJS.Timeout
  return (...args: any[]) => {
    clearTimeout(timeoutId)
    timeoutId = setTimeout(() => fn(...args), delay)
  }
}

// 防抖更新
const debouncedUpdateIp = debounce((value: string) => {
  hostIp.value = value
}, 300)

const debouncedUpdateDomain = debounce((value: string) => {
  hostDomain.value = value
}, 300)

// 监听主机数据变化
watch(() => props.host, (newHost) => {
  if (newHost) {
    hostIp.value = newHost.ip
    hostDomain.value = newHost.domain
  }
}, { immediate: true })

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

// 确认编辑
function confirmEdit() {
  // 验证输入
  if (!validateHostInput(hostIp.value, hostDomain.value)) {
    emit('error', 'IP和域名不能为空')
    return
  }

  // 验证编辑数据
  if (!props.host) {
    emit('error', '编辑数据丢失')
    return
  }

  // 提交编辑事件
  emit('edit', {
    originalHost: props.host,
    ip: hostIp.value.trim(),
    domain: hostDomain.value.trim()
  })

  // 关闭对话框
  closeDialog()
}
</script>
