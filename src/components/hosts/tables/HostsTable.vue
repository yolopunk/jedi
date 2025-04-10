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
          style="min-width: 200px;"
        ></v-text-field>
      </div>
      <v-btn
        color="#4CAF50"
        variant="flat"
        :prepend-icon="mdiPlus"
        @click="$emit('add-host', currentGroup.tag)"
        size="small"
        rounded="pill"
        class="jedi-btn jedi-hover-lift"
        style="box-shadow: 0 2px 4px rgba(76, 175, 80, 0.2);"
      >
        <span style="font-weight: 500; letter-spacing: 0.3px;">新增条目</span>
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
      style="border-radius: 8px; overflow: hidden; border: 1px solid rgba(0,0,0,0.08);"
      :footer-props="{
        'items-per-page-options': [5, 10, 15, 20, -1],
        'items-per-page-text': '每页显示',
        'page-text': '{0}-{1} 共 {2}'
      }"
    >
      <!-- IP地址列 -->
      <template v-slot:item.ip="{ item }">
        <div class="d-flex align-center">
          <v-icon :icon="mdiIpNetwork" size="small" color="var(--jedi-blue)" class="mr-2"></v-icon>
          <span>{{ item.ip }}</span>
        </div>
      </template>

      <!-- 域名列 -->
      <template v-slot:item.domain="{ item }">
        <div class="d-flex align-center">
          <v-icon :icon="mdiDomain" size="small" color="var(--jedi-blue)" class="mr-2"></v-icon>
          <span>{{ item.domain }}</span>
          <v-btn
            icon
            variant="text"
            size="x-small"
            class="ml-2 jedi-hover-scale"
            @click="handleOpenDomain(item.domain)"
            style="background-color: #E3F2FD; border-radius: 50%; box-shadow: 0 1px 2px rgba(0,0,0,0.05);"
          >
            <v-icon :icon="mdiWeb" size="x-small" color="#1976D2"></v-icon>
          </v-btn>
        </div>
      </template>

      <!-- 状态列 -->
      <template v-slot:item.enabled="{ item }">
        <v-switch
          v-model="item.enabled"
          hide-details
          color="success"
          density="compact"
          @update:model-value="$emit('update-status', item)"
          class="ma-0 pa-0 lightsaber-switch green-switch"
          :ripple="false"
          style="transform: scale(0.9); background-color: transparent;"
        ></v-switch>
      </template>

      <!-- 操作列 -->
      <template v-slot:item.actions="{ item }">
        <div class="d-flex">
          <v-tooltip location="top" text="编辑">
            <template v-slot:activator="{ props }">
              <v-btn
                icon
                variant="flat"
                color="#1976D2"
                class="mr-1 jedi-hover-lift"
                @click="$emit('edit-host', item)"
                v-bind="props"
                size="x-small"
                style="background-color: #E3F2FD; border: 1px solid rgba(25, 118, 210, 0.2); box-shadow: 0 1px 2px rgba(0,0,0,0.05);"
              >
                <v-icon :icon="mdiPencil" size="small" color="#1976D2"></v-icon>
              </v-btn>
            </template>
          </v-tooltip>

          <v-tooltip location="top" text="删除">
            <template v-slot:activator="{ props }">
              <v-btn
                icon
                variant="flat"
                color="#F44336"
                @click="$emit('delete-host', item)"
                v-bind="props"
                size="x-small"
                class="jedi-hover-lift"
                style="background-color: #FFEBEE; border: 1px solid rgba(244, 67, 54, 0.2); box-shadow: 0 1px 2px rgba(0,0,0,0.05);"
              >
                <v-icon :icon="mdiDelete" size="small" color="#F44336"></v-icon>
              </v-btn>
            </template>
          </v-tooltip>
        </div>
      </template>

      <!-- 空状态 -->
      <template v-slot:no-data>
        <div class="text-center pa-4">
          <v-icon :icon="mdiDomain" size="large" color="var(--jedi-blue-light)" class="mb-2"></v-icon>
          <div class="text-subtitle-1 font-weight-medium">暂无数据</div>
          <div class="text-body-2 text-grey">请添加新的hosts条目</div>
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

// 定义组件属性
const props = defineProps<{
  currentGroup: { tag: string; hosts: Array<Record<string, string>> };
  search?: string;
}>()

// 定义组件事件
const emit = defineEmits<{
  (e: 'update:search', value: string): void;
  (e: 'update-status', host: any): void;
  (e: 'edit-host', host: any): void;
  (e: 'delete-host', host: any): void;
  (e: 'add-host', tag: string): void;
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
