<template>
  <div class="hosts-table-container">
    <!-- 表格工具栏 -->
    <div class="d-flex justify-space-between align-center pa-4">
      <!-- 搜索框 -->
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
        class="search-field"
        style="max-width: 300px;"
      ></v-text-field>

      <v-btn
        color="var(--jedi-success)"
        variant="flat"
        rounded="lg"
        size="small"
        elevation="2"
        class="add-host-btn"
        @click="emit('add-host', currentGroup.name)"
        style="color: white; letter-spacing: 0.5px;"
      >
        <v-icon :icon="mdiPlus" size="small" class="mr-1"></v-icon>
        <span style="font-weight: 500;">添加条目</span>
      </v-btn>
    </div>

    <!-- 数据表格区域 -->
    <v-data-table
      :headers="headers"
      :items="tableItems"
      :search="searchModel"
      :loading="loading"
      density="comfortable"
      hover
      fixed-header
      class="hosts-table jedi-table"
      :items-per-page="10"
      bg-color="white"
      style="border-radius: 16px; overflow: hidden; border: 1px solid rgba(0,0,0,0.08); box-shadow: 0 3px 10px rgba(0,0,0,0.05);"
      :footer-props="{
        'items-per-page-options': [5, 10, 15, 20, -1],
        'items-per-page-text': '每页显示',
        'page-text': '{0}-{1} 共 {2}',
        'show-first-last-page': true,
        'class': 'justify-start'
      }"
    >
      <template v-slot:loading>
        <div class="pa-4">
          <v-skeleton-loader
            type="table-heading, table-row@10"
            class="mb-2"
          ></v-skeleton-loader>
        </div>
      </template>

      <template v-slot:no-data>
        <div class="d-flex flex-column align-center justify-center pa-6">
          <v-icon
            :icon="mdiDatabaseOff"
            size="large"
            color="grey-lighten-1"
            class="mb-4"
          ></v-icon>
          <span class="text-grey-darken-1">暂无数据</span>
        </div>
      </template>

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
            color="var(--jedi-primary)"
            class="mr-2 action-btn edit-btn"
            @click="emit('edit-host', item)"
          >
            <v-icon :icon="mdiPencil" size="small"></v-icon>
          </v-btn>
          <v-btn
            icon
            size="small"
            variant="text"
            color="var(--jedi-danger)"
            class="action-btn delete-btn"
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
  mdiPlus,
  mdiIpNetwork,
  mdiDomain,
  mdiWeb,
  mdiPencil,
  mdiDelete,
  mdiMagnify,
  mdiDatabaseOff
} from '@mdi/js'
import { getHostsAsItems, openDomainLink } from '@/utils/hostsUtils'
import { Group } from '@/types/hosts'

// 定义组件属性
const props = defineProps<{
  currentGroup: Group;
  search?: string;
  loading?: boolean;
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
  // 如果在加载状态且没有currentGroup，返回空数组
  if (props.loading && (!props.currentGroup || !props.currentGroup.hosts)) {
    return []
  }
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

.search-field {
  transition: all 0.3s ease;
  box-shadow: 0 2px 5px rgba(0, 0, 0, 0.05);
}

.search-field:focus-within,
.search-field:hover {
  box-shadow: 0 3px 8px rgba(0, 0, 0, 0.1);
  transform: translateY(-1px);
}

:deep(.v-data-table__tr:hover) {
  background-color: rgba(52, 152, 219, 0.05) !important;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.05);
}

:deep(.v-data-table__thead) {
  position: sticky;
  top: 0;
  z-index: 2;
}

:deep(.v-data-table__thead th) {
  background-color: #f5f7fa !important;
  border-bottom: 2px solid rgba(0, 0, 0, 0.1) !important;
  font-weight: 600 !important;
  color: var(--jedi-primary) !important;
}

:deep(.v-data-table__wrapper) {
  height: calc(100vh - 300px);
  overflow-y: auto;
}

:deep(.v-data-table__tr) {
  transition: all 0.2s ease;
  border-bottom: 1px solid rgba(0, 0, 0, 0.03) !important;
}

.add-host-btn {
  transition: all 0.3s ease;
  box-shadow: 0 2px 8px rgba(76, 175, 80, 0.2) !important;
}

.add-host-btn:hover {
  transform: translateY(-2px);
  box-shadow: 0 4px 12px rgba(76, 175, 80, 0.3) !important;
}

.action-btn {
  opacity: 0.75;
  transition: all 0.2s ease;
  border-radius: 8px;
}

.action-btn:hover {
  opacity: 1;
  transform: translateY(-1px);
}

.edit-btn {
  color: var(--jedi-primary) !important;
}

.edit-btn:hover {
  background-color: rgba(44, 62, 80, 0.05) !important;
}

.delete-btn {
  color: var(--jedi-danger) !important;
}

.delete-btn:hover {
  background-color: rgba(231, 76, 60, 0.05) !important;
}

.hosts-table-container {
  display: flex;
  flex-direction: column;
  height: 100%;
  overflow: hidden;
}

:deep(.v-data-table) {
  flex: 1;
  display: flex;
  flex-direction: column;
}

:deep(.v-data-table__wrapper) {
  flex: 1;
}

:deep(.v-data-table-footer) {
  justify-content: flex-start !important;
}

:deep(.v-data-table-footer__items-per-page) {
  margin-right: 16px;
}

:deep(.v-data-table-footer__pagination) {
  margin-left: 0 !important;
}
</style>
