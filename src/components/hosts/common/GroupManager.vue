<template>
  <!-- 分组管理区域 -->
  <div class="group-tabs-container sticky-header">
    <!-- 分组标签页：只承载选择语义，不放任何可交互元素 -->
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
        <v-icon :icon="mdiDomain" size="small" class="mr-1" aria-hidden="true"></v-icon>
        <span class="group-name">{{ group.name }}</span>
        <!-- 远程数据源标记：纯展示，不参与交互 -->
        <v-icon
          v-if="isRemote(group.name)"
          :icon="mdiLinkVariant"
          size="x-small"
          class="remote-badge"
          aria-hidden="true"
        ></v-icon>
      </v-tab>
    </v-tabs>

    <!-- 当前分组的操作区：常驻显示，作用于选中分组 -->
    <div v-if="currentGroup" class="group-actions">
      <button
        v-if="isRemote(currentGroup.name)"
        type="button"
        class="group-action-btn"
        :disabled="syncing || refreshingName === currentGroup.name"
        :aria-label="`${$t('hosts.groups.refresh')} · ${currentGroup.name}`"
        :title="$t('hosts.groups.refresh')"
        @click="$emit('refresh-group', currentGroup.name)"
      >
        <v-icon
          :icon="mdiRefresh"
          size="18"
          aria-hidden="true"
          :class="{ spinning: syncing || refreshingName === currentGroup.name }"
        ></v-icon>
      </button>

      <v-menu location="bottom end" offset="6">
        <template v-slot:activator="{ props }">
          <button
            v-bind="props"
            type="button"
            class="group-action-btn"
            :aria-label="`${$t('hosts.groups.more')} · ${currentGroup.name}`"
            :title="`${$t('hosts.groups.more')} · ${currentGroup.name}`"
          >
            <v-icon :icon="mdiDotsVertical" size="18" aria-hidden="true"></v-icon>
          </button>
        </template>

        <div class="console-menu group-actions-menu">
          <button
            v-if="isRemote(currentGroup.name)"
            type="button"
            class="menu-item"
            @click="$emit('edit-source', currentGroup.name)"
          >
            <v-icon :icon="mdiLinkVariant" size="x-small" aria-hidden="true"></v-icon>
            <span class="menu-text">{{ $t('hosts.groups.editSource') }}</span>
          </button>
          <button type="button" class="menu-item" @click="$emit('rename-group', currentGroup.name)">
            <v-icon :icon="mdiPencil" size="x-small" aria-hidden="true"></v-icon>
            <span class="menu-text">{{ $t('hosts.dialog.renameGroupTitle') }}</span>
          </button>
          <div class="menu-divider"></div>
          <button type="button" class="menu-item danger" @click="$emit('delete-group', currentGroup.name)">
            <v-icon :icon="mdiDeleteOutline" size="x-small" aria-hidden="true"></v-icon>
            <span class="menu-text">{{ $t('hosts.groups.delete') }}</span>
          </button>
        </div>
      </v-menu>
    </div>
  </div>
</template>

<script setup lang="ts">
import {
  mdiDeleteOutline,
  mdiDomain,
  mdiDotsVertical,
  mdiLinkVariant,
  mdiPencil,
  mdiRefresh,
} from '@mdi/js'
import { computed } from 'vue'
import type { Group } from '@/types/hosts'

// 定义组件属性
const props = withDefaults(
  defineProps<{
    modelValue: string
    groups: Group[]
    /** 远程数据源对应的分组名（显示同步按钮与来源标记） */
    remoteGroupNames?: string[]
    /** 后台同步中（禁用同步按钮并旋转图标） */
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

// 操作区作用于当前选中的分组
const currentGroup = computed(() => props.groups.find(g => g.name === props.modelValue) ?? null)

function isRemote(name: string): boolean {
  return props.remoteGroupNames.includes(name)
}
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

/* 标签栏与右侧操作区同一行：操作区常驻，宽度不随悬停变化 */
.group-tabs-container {
  display: flex;
  align-items: center;
}

.group-tabs {
  flex: 1;
  min-width: 0;
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

.remote-badge {
  margin-left: 6px;
  opacity: 0.45;
}

.group-tab.v-tab--selected {
  color: var(--jedi-text-primary) !important;
  font-weight: 600;
}

.group-actions {
  display: flex;
  align-items: center;
  gap: 2px;
  padding: 0 8px;
  flex-shrink: 0;
}

.group-action-btn {
  width: 28px;
  height: 28px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  background: transparent;
  border: none;
  border-radius: 6px;
  color: rgb(var(--text-rgb) / 0.45);
  cursor: pointer;
  transition:
    background-color 0.15s ease,
    color 0.15s ease;
}

.group-action-btn:hover:not(:disabled) {
  background: rgb(var(--text-rgb) / 0.06);
  color: var(--jedi-text-primary);
}

.group-action-btn:focus-visible {
  outline: 2px solid rgb(var(--accent-rgb) / 0.5);
  outline-offset: 1px;
}

.group-action-btn:disabled {
  opacity: 0.45;
  cursor: default;
}

.group-actions-menu {
  min-width: 176px;
}

/* 复用全局 .menu-item 外观，但要清掉 button 的默认样式 */
.group-actions-menu .menu-item {
  width: 100%;
  background: transparent;
  border: 0;
  text-align: left;
}

.group-actions-menu .menu-item.danger {
  color: rgb(var(--danger-rgb) / 0.85);
}

.group-actions-menu .menu-item.danger:hover {
  background: rgb(var(--danger-rgb) / 0.1);
  color: var(--danger);
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
