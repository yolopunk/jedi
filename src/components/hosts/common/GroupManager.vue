<template>
  <!-- 分组管理区域 -->
  <div class="group-tabs-container sticky-header">

    <!-- 分组标签页 -->
    <v-tabs
      v-model="selectedTab"
      bg-color="transparent"
      color="var(--jedi-accent)"
      align-tabs="start"
      show-arrows
      slider-color="var(--jedi-accent)"
      class="group-tabs"
    >
      <v-tab
        v-for="group in groups"
        :key="group.name"
        :value="group.name"
        class="group-tab"
        @click="$emit('update:modelValue', group.name)"
      >
        <v-icon :icon="mdiDomain" size="small" class="mr-1"></v-icon>
        <span class="group-name">{{ group.name }}</span>
        <v-btn
          v-if="remoteGroupNames.includes(group.name)"
          icon
          size="x-small"
          variant="text"
          class="ml-1 group-rename-btn"
          :aria-label="$t('hosts.groups.refresh')"
          :title="$t('hosts.groups.refresh')"
          :disabled="syncing || refreshingName === group.name"
          @click.stop="$emit('refresh-group', group.name)"
        >
          <v-icon
            :icon="mdiRefresh"
            size="x-small"
            :class="{ spinning: syncing || refreshingName === group.name }"
          ></v-icon>
        </v-btn>
        <v-btn
          v-if="remoteGroupNames.includes(group.name)"
          icon
          size="x-small"
          variant="text"
          class="ml-1 group-rename-btn"
          :aria-label="$t('hosts.groups.editSource')"
          :title="$t('hosts.groups.editSource')"
          @click.stop="$emit('edit-source', group.name)"
        >
          <v-icon :icon="mdiLinkVariant" size="x-small"></v-icon>
        </v-btn>
        <v-btn
          icon
          size="x-small"
          variant="text"
          class="ml-1 group-rename-btn"
          :aria-label="$t('hosts.dialog.renameGroupTitle')"
          :title="$t('hosts.dialog.renameGroupTitle')"
          @click.stop="$emit('rename-group', group.name)"
        >
          <v-icon :icon="mdiPencil" size="x-small"></v-icon>
        </v-btn>
        <v-btn
          icon
          size="x-small"
          variant="text"
          class="ml-1 group-rename-btn"
          :aria-label="$t('hosts.groups.delete')"
          :title="$t('hosts.groups.delete')"
          @click.stop="$emit('delete-group', group.name)"
        >
          <v-icon :icon="mdiDeleteOutline" size="x-small"></v-icon>
        </v-btn>
      </v-tab>
    </v-tabs>
  </div>
</template>

<script setup lang="ts">
import { mdiDeleteOutline, mdiDomain, mdiLinkVariant, mdiPencil, mdiRefresh } from '@mdi/js'
import { computed } from 'vue'
import type { Group } from '@/types/hosts'

// 定义组件属性
const props = withDefaults(
  defineProps<{
    modelValue: string
    groups: Group[]
    /** 远程数据源对应的分组名（显示刷新按钮） */
    remoteGroupNames?: string[]
    /** 后台同步中（禁用刷新按钮并旋转图标） */
    syncing?: boolean
    /** 正在手动刷新的分组名（只让该分组的按钮转圈） */
    refreshingName?: string | null
  }>(),
  {
    remoteGroupNames: () => [],
    syncing: false,
    refreshingName: null,
  }
)

// 定义组件事件
const emit = defineEmits<{
  (e: 'update:modelValue', value: string): void
  (e: 'rename-group', value: string): void
  (e: 'refresh-group', value: string): void
  (e: 'edit-source', value: string): void
  (e: 'delete-group', value: string): void
}>()

// 计算属性：当前选中的标签
const selectedTab = computed({
  get: () => props.modelValue,
  set: value => {
    emit('update:modelValue', value)
  },
})
</script>

<style scoped>
.sticky-header {
  position: sticky;
  top: 0;
  z-index: 10;
  background-color: var(--jedi-bg-surface);
  border-bottom: 1px solid var(--jedi-border);
  transition: background-color 0.3s ease;
}

.group-tabs-container {
  /* Border handled by sticky-header */
}

.group-tabs {
  border-radius: 0;
}

.group-tab {
  min-width: 100px;
  font-weight: 500;
  text-transform: none;
  letter-spacing: 0.3px;
  transition: all 0.3s ease;
  color: var(--jedi-text-secondary) !important;
}

.group-name {
  max-width: 160px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.group-rename-btn {
  opacity: 0;
}

.group-tab:hover .group-rename-btn,
.group-tab:focus-within .group-rename-btn {
  opacity: 1;
}

.group-tab.v-tab--selected {
  color: var(--jedi-text-primary) !important;
  font-weight: 600;
}

.spinning {
  animation: group-refresh-spin 1s linear infinite;
}

@keyframes group-refresh-spin {
  from {
    transform: rotate(0deg);
  }
  to {
    transform: rotate(360deg);
  }
}
</style>
