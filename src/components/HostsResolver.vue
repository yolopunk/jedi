<template>
  <!--
  ====================================
  JEDIHOSTS管理器 - 主界面
  功能：管理hosts文件配置，支持分组、启用/禁用、添加/编辑/删除条目
  ====================================
  -->

  <!-- 主内容区域 -->
  <div class="hosts-container fade-in-up">
    <v-card class="jedi-card fade-in" style="border-radius: 16px; overflow: hidden; box-shadow: 0 3px 10px rgba(0,0,0,0.08); transition: all 0.3s ease;">
    <!-- 分组管理区域 -->
    <group-manager
      v-if="groups.length"
      v-model="selectedGroup"
      :groups="groups"
      @add-group="showAddGroupDialog = true"
      class="fade-in"
    />

    <!-- 数据展示区域（包括加载状态） -->
    <template v-if="loading || currentGroup">
      <hosts-table
        :current-group="currentGroup || { name: '', hosts: [] }"
        v-model:search="search"
        :loading="loading"
        @update-status="updateHostStatus"
        @edit-host="openEditHostDialog"
        @delete-host="removeHost"
        @add-host="openAddHostDialog"
        @open-domain="handleOpenDomain"
        class="fade-in-scale"
      />
    </template>

    <!-- 空状态显示 -->
    <template v-else>
      <empty-state
        @add-group="showAddGroupDialog = true"
        @use-default="initializeDefaultConfig"
        class="fade-in-scale"
      />
    </template>
  </v-card>
  </div>

  <!-- 3. 对话框区域 -->
  <!-- 添加分组对话框 -->
  <add-group-dialog
    v-model="showAddGroupDialog"
    @add="addGroup"
    @error="showNotification($event, 'error')"
  />

  <!-- 添加条目对话框 -->
  <add-host-dialog
    v-model="showAddHostDialog"
    :group-name="currentAddGroupName"
    @add="addHost"
    @error="showNotification($event, 'error')"
  />

  <!-- 编辑条目对话框 -->
  <edit-host-dialog
    v-model="showEditHostDialog"
    :host="currentEditHost"
    @edit="editHost"
    @error="showNotification($event, 'error')"
  />

  <!-- 删除确认对话框 -->
  <delete-confirm-dialog
    v-model="showDeleteConfirmDialog"
    :host="hostToDelete"
    @delete="confirmDeleteHost"
  />

  <!-- 4. 通知消息组件 -->
  <notification-snackbar
    v-model="showSnackbar"
    :text="snackbarText"
    :color="snackbarColor"
    :timeout="3000"
  />

  <!-- 5. 全局开关悬浮按钮 -->
  <global-switch-fab
    v-model="hostsResolveSwitch"
    @update:model-value="handleHostsSwitch"
    class="fade-in"
  />
</template>

<script setup lang="ts">
/**
 * JEDIHOSTS管理器 - 主组件
 * 功能：管理hosts文件配置，支持分组、启用/禁用、添加/编辑/删除条目
 */

// ===== 导入依赖 =====
import { ref, onMounted } from 'vue'

// 导入子组件
import GroupManager from '@/components/hosts/common/GroupManager.vue'
import HostsTable from '@/components/hosts/tables/HostsTable.vue'
import EmptyState from '@/components/hosts/common/EmptyState.vue'
import AddGroupDialog from '@/components/hosts/dialogs/AddGroupDialog.vue'
import AddHostDialog from '@/components/hosts/dialogs/AddHostDialog.vue'
import EditHostDialog from '@/components/hosts/dialogs/EditHostDialog.vue'
import DeleteConfirmDialog from '@/components/hosts/dialogs/DeleteConfirmDialog.vue'
import NotificationSnackbar from '@/components/hosts/common/NotificationSnackbar.vue'
import GlobalSwitchFab from '@/components/hosts/common/GlobalSwitchFab.vue'

// 导入工具和服务
import { getOsInfo } from '@/api/hosts'
import { useHostsData } from '@/composables/useHostsData'

// ===== 状态变量 - UI相关 =====

// 对话框状态
const showAddGroupDialog = ref(false)
const showAddHostDialog = ref(false)
const currentAddGroupName = ref('')
const showEditHostDialog = ref(false)
const currentEditHost = ref<any>(null)
const showDeleteConfirmDialog = ref(false)
const hostToDelete = ref<any>(null)

// 提示消息状态
const showSnackbar = ref(false)
const snackbarText = ref('')
const snackbarColor = ref<'success' | 'error' | 'info' | 'warning'>('success')

// 搜索关键词 (纯UI状态)
const search = ref('')

// ===== 核心逻辑 (Composable) =====

/**
 * 显示通知
 */
function showNotification(text: string, color: 'success' | 'error' | 'info' | 'warning') {
  snackbarText.value = text
  snackbarColor.value = color
  showSnackbar.value = true
}

const {
  groups,
  selectedGroup,
  hostsResolveSwitch,
  loading,
  currentGroup,
  loadSystemHosts,
  handleHostsSwitch,
  initializeDefaultConfig,
  addGroup,
  addHost,
  editHost,
  updateHostStatus,
  confirmDeleteHost
} = useHostsData(showNotification)

// ===== UI 交互方法 =====

/**
 * 打开添加主机对话框
 */
function openAddHostDialog(groupName: string) {
  currentAddGroupName.value = groupName
  showAddHostDialog.value = true
}

/**
 * 打开编辑主机对话框
 */
function openEditHostDialog(host: any) {
  currentEditHost.value = host
  showEditHostDialog.value = true
}

/**
 * 打开删除确认对话框
 */
function removeHost(host: any) {
  hostToDelete.value = host
  showDeleteConfirmDialog.value = true
}

/**
 * 处理打开域名链接
 */
function handleOpenDomain(_domain: string, message: string) {
  showNotification(message, 'info')
}

// ===== 生命周期钩子 =====

onMounted(async () => {
  await getOsInfo()
  await loadSystemHosts()
})

</script>

<style scoped>
.hosts-container {
  display: flex;
  flex-direction: column;
  height: calc(100vh - 80px);
  overflow: hidden;
}

.jedi-card {
  display: flex;
  flex-direction: column;
  flex: 1;
  overflow: hidden;
}
</style>

<style scoped>
/* 引入全局样式 */
@import '@/assets/style.css';
</style>
