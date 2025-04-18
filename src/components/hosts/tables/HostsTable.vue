<template>
  <div>
    <!-- 表格工具栏 -->
    <div class="d-flex justify-space-between align-center pa-4">
      <div class="d-flex align-center">
        <v-text-field
          v-model="searchModel"
          label="搜索域名或IP"
          :prepend-inner-icon="mdiMagnify"
          variant="outlined"
          density="compact"
          hide-details
          bg-color="white"
          color="var(--jedi-text-medium)"
          rounded="pill"
          style="width: 400px; min-width: 400px;"
        ></v-text-field>
      </div>
      <v-btn
        color="var(--jedi-accent)"
        variant="flat"
        rounded="sm"
        size="small"
        class="jedi-btn jedi-hover-lift"
        @click="emit('add-host', currentGroup.name)"
        style="color: white; text-transform: uppercase; letter-spacing: 0.5px;"
      >
        <v-icon :icon="mdiPlus" size="small" class="mr-1"></v-icon>
        <span>添加条目</span>
      </v-btn>
    </div>

    <!-- 数据表格区域 -->
    <v-data-table
      :headers="headers"
      :items="tableItems"
      :search="searchModel"
      density="comfortable"
      hover
      class="hosts-table jedi-table"
      :items-per-page="10"
      bg-color="white"
      style="border-radius: 12px; overflow: hidden; border: 1px solid rgba(0,0,0,0.08); box-shadow: 0 2px 8px rgba(0,0,0,0.05);"
      :footer-props="{
        'items-per-page-options': [5, 10, 15, 20, -1],
        'items-per-page-text': '每页显示',
        'page-text': '{0}-{1} 共 {2}'
      }"
    >
      <!-- IP地址列 -->
      <template v-slot:item.ip="{ item }">
        <div class="d-flex align-center">
          <v-icon :icon="mdiIpNetwork" size="small" color="primary" class="mr-2"></v-icon>
          <span>{{ item.ip }}</span>
        </div>
      </template>

      <!-- 域名列 -->
      <template v-slot:item.domain="{ item }">
        <div class="d-flex align-center">
          <v-icon :icon="mdiDomain" size="small" color="primary" class="mr-2"></v-icon>
          <span class="domain-text">{{ item.domain }}</span>
          <v-btn
            icon
            size="x-small"
            variant="text"
            color="primary"
            class="ml-2 jedi-hover-scale"
            @click="handleOpenDomain(item.domain)"
          >
            <v-icon :icon="mdiWeb" size="small"></v-icon>
          </v-btn>
        </div>
      </template>

      <!-- 状态列 -->
      <template v-slot:item.enabled="{ item }">
        <v-switch
          v-model="item.enabled"
          color="success"
          hide-details
          density="compact"
          @update:model-value="emit('update-status', item)"
          class="status-switch"
        ></v-switch>
      </template>

      <!-- 操作列 -->
      <template v-slot:item.actions="{ item }">
        <div class="d-flex">
          <v-btn
            icon
            size="small"
            variant="text"
            color="primary"
            class="mr-2"
            @click="emit('edit-host', item)"
          >
            <v-icon :icon="mdiPencil" size="small"></v-icon>
          </v-btn>
          <v-btn
            icon
            size="small"
            variant="text"
            color="var(--jedi-danger)"
            @click="emit('delete-host', item)"
          >
            <v-icon :icon="mdiDelete" size="small"></v-icon>
          </v-btn>
        </div>
      </template>
    </v-data-table>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import {
  mdiMagnify,
  mdiPlus,
  mdiIpNetwork,
  mdiDomain,
  mdiWeb,
  mdiPencil,
  mdiDelete
} from '@mdi/js'
import { getHostsAsItems, openDomainLink } from '@/utils/hostsUtils'
import { Group } from '@/types/hosts'

// 定义组件属性
const props = defineProps<{
  currentGroup: Group;
  search?: string;
}>()

// 定义组件事件
const emit = defineEmits<{
  (e: 'update:search', value: string): void;
  (e: 'update-status', host: any): void;
  (e: 'edit-host', host: any): void;
  (e: 'delete-host', host: any): void;
  (e: 'add-host', name: string): void;
  (e: 'open-domain', domain: string, message: string): void;
}>()

// 表格列配置
const headers = [
  { title: 'IP地址', key: 'ip', sortable: true },
  { title: '域名', key: 'domain', sortable: true },
  { title: '状态', key: 'enabled', sortable: false },
  { title: '操作', key: 'actions', sortable: false }
]

// 搜索模型
const searchModel = computed({
  get: () => props.search || '',
  set: (value) => emit('update:search', value)
})

// 表格数据
const tableItems = computed(() => {
  return getHostsAsItems(props.currentGroup.hosts)
})

// 处理打开域名
function handleOpenDomain(domain: string) {
  const message = openDomainLink(domain)
  emit('open-domain', domain, message)
}
</script>

<style scoped>
.domain-text {
  max-width: 300px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.status-switch {
  margin-top: 0 !important;
  margin-bottom: 0 !important;
}
</style>
