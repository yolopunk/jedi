<template>
  <div class="hosts-table-container pa-4 d-flex flex-column flex-grow-1">
    <!-- Toolbar -->
    <div class="d-flex justify-space-between align-center mb-4 flex-shrink-0">
      <v-text-field
        v-model="searchModel"
        :label="$t('common.search')"
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
    <div class="table-wrapper flex-grow-1" ref="tableWrapperRef">
      <v-data-table
        :headers="headers"
        :items="displayItems"
        :search="searchModel"
        :loading="loading"
        density="compact"
        hover
        fixed-header
        class="jedi-data-table"
        :items-per-page="-1"
        hide-default-footer
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

      <!-- Progress indicator at bottom -->
      <template v-slot:body.append>
        <tr v-if="hasMore && !loading" class="load-more-row">
          <td :colspan="headers.length" class="text-center py-4">
            <v-progress-circular indeterminate size="24" color="primary"></v-progress-circular>
          </td>
        </tr>
        <tr v-else-if="!hasMore && filteredItems.length > 0" class="load-more-row">
          <td :colspan="headers.length" class="text-center py-4 text-medium-emphasis">
            {{ $t('hosts.table.noMore') }}
          </td>
        </tr>
      </template>
      </v-data-table>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted } from 'vue'
import { useI18n } from 'vue-i18n'
import {
  mdiMagnify,
  mdiPlus,
  mdiWeb,
  mdiPencil,
  mdiDelete
} from '@mdi/js'
import { getHostsAsItems, openDomainLink } from '@/utils/hostsUtils'
import { Group } from '@/types/hosts'

const { t } = useI18n()

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

const tableWrapperRef = ref<HTMLElement | null>(null)
const itemsPerPage = 20
const currentDisplayCount = ref(itemsPerPage)

// 表格列配置
const headers = computed(() => [
  { title: t('hosts.table.ip'), key: 'ip', sortable: true },
  { title: t('hosts.table.domain'), key: 'domain', sortable: true },
  { title: t('hosts.table.status'), key: 'enabled', sortable: false },
  { title: t('hosts.table.actions'), key: 'actions', sortable: false }
])

// 搜索模型
const searchModel = computed({
  get: () => props.search || '',
  set: (value) => emit('update:search', value)
})

// 过滤后的所有数据
const filteredItems = computed(() => {
  // 如果在加载状态且没有currentGroup，返回空数组
  if (props.loading && (!props.currentGroup || !props.currentGroup.hosts)) {
    return []
  }
  return getHostsAsItems(props.currentGroup.hosts)
})

// 当前显示的数据
const displayItems = computed(() => {
  return filteredItems.value.slice(0, currentDisplayCount.value)
})

// 是否还有更多数据
const hasMore = computed(() => {
  return currentDisplayCount.value < filteredItems.value.length
})

// 重置显示数量当数据变化时
watch(filteredItems, () => {
  currentDisplayCount.value = itemsPerPage
})

// 加载更多
function loadMore() {
  if (hasMore.value) {
    currentDisplayCount.value += itemsPerPage
  }
}

// 滚动监听
let scrollHandler: (() => void) | null = null

function setupScrollListener() {
  if (!tableWrapperRef.value) return

  const wrapper = tableWrapperRef.value
  const tableWrapper = wrapper.querySelector('.v-data-table__wrapper')

  if (tableWrapper) {
    scrollHandler = () => {
      const { scrollTop, scrollHeight, clientHeight } = tableWrapper as HTMLElement
      // 当滚动到底部附近时加载更多
      if (scrollTop + clientHeight >= scrollHeight - 100) {
        loadMore()
      }
    }
    tableWrapper.addEventListener('scroll', scrollHandler)
  }
}

function removeScrollListener() {
  if (!tableWrapperRef.value || !scrollHandler) return

  const wrapper = tableWrapperRef.value
  const tableWrapper = wrapper.querySelector('.v-data-table__wrapper')

  if (tableWrapper) {
    tableWrapper.removeEventListener('scroll', scrollHandler)
  }
  scrollHandler = null
}

onMounted(() => {
  // 延迟设置滚动监听器，确保表格已渲染
  setTimeout(setupScrollListener, 100)
})

onUnmounted(() => {
  removeScrollListener()
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
.hosts-table-container {
  display: flex;
  flex-direction: column;
  min-height: 0;
  height: 100%;
}

.table-wrapper {
  flex-grow: 1;
  min-height: 0;
  overflow: hidden;
}

/* Ensure data table handles scrolling properly */
.table-wrapper :deep(.v-data-table) {
  height: 100%;
}

.table-wrapper :deep(.v-data-table__wrapper) {
  max-height: calc(100vh - 300px);
  overflow-y: auto;
}

.load-more-row {
  background-color: transparent;
}

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
