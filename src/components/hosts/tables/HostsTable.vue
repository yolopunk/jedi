<template>
  <div class="hosts-table-container pa-4 d-flex flex-column flex-grow-1" style="min-height: 0;">
    <!-- Toolbar -->
    <div class="d-flex justify-space-between align-center mb-4">
      <v-text-field
        v-model="searchModel"
        :placeholder="$t('hosts.table.searchPlaceholder')"
        :prepend-inner-icon="mdiMagnify"
        variant="outlined"
        density="compact"
        hide-details
        class="search-field"
        style="max-width: 300px;"
        rounded="lg"
      ></v-text-field>

      <v-btn
        color="success"
        variant="flat"
        class="text-none px-4"
        @click="emit('add-host', currentGroup.name)"
      >
        <v-icon :icon="mdiPlus" class="mr-1"></v-icon>
        {{ $t('hosts.table.addHost') }}
      </v-btn>
    </div>

    <!-- Table -->
    <v-data-table-virtual
      :headers="headers"
      :items="tableItems"
      :search="searchModel"
      :loading="loading"
      density="compact"
      hover
      fixed-header
      height="100%"
      class="flex-grow-1 jedi-data-table"
    >
      <!-- IP Column -->
      <template v-slot:item.ip="{ item }">
        <span class="jedi-table-ip">{{ item.ip }}</span>
      </template>

      <!-- Domain Column -->
      <template v-slot:item.domain="{ item }">
        <div class="d-flex align-center">
          <span class="jedi-table-domain mr-2">{{ item.domain }}</span>
          <v-btn
            icon
            size="x-small"
            variant="text"
            color="primary"
            class="opacity-50 hover-opacity-100"
            @click="handleOpenDomain(item.domain)"
          >
            <v-icon :icon="mdiWeb" size="small"></v-icon>
          </v-btn>
        </div>
      </template>

      <!-- Status Column -->
      <template v-slot:item.enabled="{ item }">
        <v-switch
          v-model="item.enabled"
          color="success"
          hide-details
          density="compact"
          inset
          @update:model-value="emit('update-status', item)"
        ></v-switch>
      </template>

      <!-- Actions Column -->
      <template v-slot:item.actions="{ item }">
        <div class="d-flex">
          <v-btn
            icon
            size="small"
            variant="text"
            color="primary"
            class="mr-1"
            @click="emit('edit-host', item)"
          >
            <v-icon :icon="mdiPencil" size="small"></v-icon>
          </v-btn>
          <v-btn
            icon
            size="small"
            variant="text"
            color="error"
            @click="emit('delete-host', item.id)"
          >
            <v-icon :icon="mdiDelete" size="small"></v-icon>
          </v-btn>
        </div>
      </template>
    </v-data-table-virtual>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import {
  mdiMagnify,
  mdiPlus,
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
  openDomainLink(domain)
    .then((message) => {
      emit('open-domain', domain, message)
    })
    .catch((error) => {
      console.error('打开域名失败:', error)
      emit('open-domain', domain, `打开域名失败: ${domain}`)
    })
}
</script>

<style scoped>
/* Jedi Switch Styling */
:deep(.jedi-switch) {
  display: flex;
  justify-content: center;
}

:deep(.jedi-switch .v-selection-control) {
  min-height: auto;
}

/* Track Styling */
:deep(.jedi-switch .v-switch__track) {
  background-color: rgba(255, 255, 255, 0.1) !important;
  border: 1px solid rgba(255, 255, 255, 0.15);
  opacity: 1 !important;
  height: 12px; /* Reduced to micro size */
  width: 24px;  /* Reduced to micro size */
  border-radius: 12px;
  transition: all 0.3s ease;
}

:deep(.jedi-switch.v-input--is-dirty .v-switch__track) {
  background-color: rgba(0, 255, 128, 0.15) !important;
  border-color: rgba(0, 255, 128, 0.3);
  box-shadow: 0 0 8px rgba(0, 255, 128, 0.2) inset;
}

/* Thumb Styling */
:deep(.jedi-switch .v-selection-control__input input) {
  opacity: 0;
}

:deep(.jedi-switch .v-switch__thumb) {
  height: 8px; /* Micro size */
  width: 8px;  /* Micro size */
  background-color: #95a5a6;
  box-shadow: 0 1px 2px rgba(0,0,0,0.3);
  transition: all 0.3s cubic-bezier(0.25, 0.8, 0.25, 1);
  top: calc(50% - 4px);
}

:deep(.jedi-switch.v-input--is-dirty .v-switch__thumb) {
  background-color: #00ff80;
  box-shadow: 0 0 8px rgba(0, 255, 128, 0.6), 0 0 10px rgba(0, 255, 128, 0.4);
  transform: translateX(12px);
}

/* Hover effects */
:deep(.jedi-switch:hover .v-switch__track) {
  border-color: rgba(255, 255, 255, 0.3);
}

:deep(.jedi-switch.v-input--is-dirty:hover .v-switch__track) {
  border-color: rgba(0, 255, 128, 0.5);
  box-shadow: 0 0 10px rgba(0, 255, 128, 0.3) inset;
}
</style>
