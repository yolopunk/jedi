<template>
  <!-- 添加分组对话框 -->
  <v-dialog v-model="dialogModel" max-width="550" persistent>
    <v-card class="rounded-lg overflow-hidden">
      <v-toolbar color="#4a90e2" class="px-4">
        <v-icon :icon="mdiDomainPlus" class="mr-2" color="white"></v-icon>
        <v-toolbar-title class="font-weight-medium">添加分组</v-toolbar-title>
        <v-spacer></v-spacer>
        <v-btn icon @click="closeDialog">
          <v-icon :icon="mdiClose" color="white"></v-icon>
        </v-btn>
      </v-toolbar>
      <v-card-text class="pa-6">
        <v-text-field
          v-model="groupTag"
          label="分组名称"
          variant="outlined"
          placeholder="例如: 开发环境"
          required
          bg-color="white"
        ></v-text-field>

        <v-switch
          v-model="isRemote"
          label="使用远程配置"
          color="#2196F3"
          hide-details
          class="mb-4"
        ></v-switch>

        <v-expand-transition>
          <v-text-field
            v-if="isRemote"
            v-model="remoteUrl"
            label="远程配置URL"
            variant="outlined"
            placeholder="例如: https://example.com/hosts.json"
            required
            bg-color="white"
            :prepend-inner-icon="mdiLinkVariant"
          ></v-text-field>
        </v-expand-transition>

        <v-expand-transition>
          <div v-if="!isRemote">
            <v-textarea
              v-model="hostsContent"
              label="Hosts 列表"
              variant="outlined"
              placeholder="格式: IP 域名，每行一条
例如:
127.0.0.1 localhost
192.168.1.1 router.local"
              rows="6"
              auto-grow
              required
              bg-color="white"
              class="font-monospace"
            ></v-textarea>
            <v-alert
              type="info"
              variant="tonal"
              density="compact"
              class="mt-2"
            >
              <div class="text-body-2">
                每行一条记录，格式为: <code>IP地址 域名</code>
              </div>
            </v-alert>
          </div>
        </v-expand-transition>
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
          color="#2196F3"
          variant="elevated"
          @click="confirmAdd"
          class="lightsaber-btn blue"
        >
          确认
        </v-btn>
      </v-card-actions>
    </v-card>
  </v-dialog>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { mdiDomainPlus, mdiClose, mdiLinkVariant } from '@mdi/js'
import { fetchRemoteConfig } from '@/services/hostsService'

// 定义组件属性
const props = defineProps<{
  modelValue: boolean;
}>()

// 定义组件事件
const emit = defineEmits<{
  (e: 'update:modelValue', value: boolean): void;
  (e: 'add', data: {
    tag: string;
    isRemote: boolean;
    url?: string;
    hosts?: Array<Record<string, string>>
  }): void;
  (e: 'error', message: string): void;
}>()

// 对话框状态
const dialogModel = computed({
  get: () => props.modelValue,
  set: (value) => emit('update:modelValue', value)
})

// 表单数据
const groupTag = ref('')
const isRemote = ref(false)
const remoteUrl = ref('')
const hostsContent = ref('')

// 关闭对话框
function closeDialog() {
  dialogModel.value = false
  resetForm()
}

// 重置表单
function resetForm() {
  groupTag.value = ''
  isRemote.value = false
  remoteUrl.value = ''
  hostsContent.value = ''
}

// 确认添加
async function confirmAdd() {
  // 验证分组名称
  if (!groupTag.value) {
    emit('error', '分组名称不能为空')
    return
  }

  // 处理远程配置
  if (isRemote.value) {
    if (!remoteUrl.value) {
      emit('error', '远程配置URL不能为空')
      return
    }

    try {
      // 获取远程配置
      const result = await fetchRemoteConfig(remoteUrl.value)

      // 验证远程配置
      if (!Array.isArray(result)) {
        emit('error', '远程配置格式错误')
        return
      }

      // 查找匹配的分组
      const matchedGroup = result.find(g => g.tag === groupTag.value)

      if (matchedGroup) {
        // 提交添加事件
        emit('add', {
          tag: groupTag.value,
          isRemote: true,
          url: remoteUrl.value,
          hosts: matchedGroup.hosts
        })

        // 关闭对话框
        closeDialog()
      } else {
        emit('error', '远程配置中未找到该分组')
      }
    } catch (error) {
      emit('error', `远程加载失败: ${(error as Error).message}`)
    }
  } else {
    // 处理本地配置
    const hostsArray = hostsContent.value
      .split('\n')
      .map(line => line.trim())
      .filter(line => line.length > 0)
      .map(line => {
        const [ip, domain] = line.split(/\s+/)
        return { [domain]: ip }
      })

    // 提交添加事件
    emit('add', {
      tag: groupTag.value,
      isRemote: false,
      hosts: hostsArray
    })

    // 关闭对话框
    closeDialog()
  }
}
</script>
