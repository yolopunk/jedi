<template>
  <div class="hosts-container fade-in-up">
    <v-card class="main-card">
    <!-- 分组管理区域 -->
    <group-manager
      v-if="groups.length"
      v-model="selectedGroup"
      :groups="groups"
      @add-group="dialogs.addGroup = true"
      @rename-group="openRenameGroupDialog"
      class="fade-in"
    />

    <!-- 数据展示区域 -->
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
        @add-group="dialogs.addGroup = true"
        @use-default="initializeDefaultConfig"
        class="fade-in-scale"
      />
    </template>
  </v-card>
  </div>

  <!-- 对话框区域 -->
  <add-group-dialog
    v-model="dialogs.addGroup"
    @add="addGroup"
    @error="showNotification($event, 'error')"
  />

  <v-dialog v-model="dialogs.renameGroup" max-width="400">
    <v-card>
      <v-card-title>{{ $t('hosts.dialog.renameGroupTitle') }}</v-card-title>
      <v-card-text>
        <v-text-field
          v-model="renameGroupName"
          :label="$t('hosts.dialog.groupNameLabel')"
          variant="outlined"
        />
      </v-card-text>
      <v-card-actions>
        <v-spacer />
        <v-btn
          variant="text"
          color="grey-darken-1"
          @click="dialogs.renameGroup = false"
        >
          {{ $t('common.cancel') }}
        </v-btn>
        <v-btn
          color="primary"
          variant="elevated"
          @click="handleConfirmRenameGroup"
        >
          {{ $t('common.save') }}
        </v-btn>
      </v-card-actions>
    </v-card>
  </v-dialog>

  <add-host-dialog
    v-model="dialogs.addHost"
    :group-name="currentAddGroupName"
    @add="addHost"
    @error="showNotification($event, 'error')"
  />

  <edit-host-dialog
    v-model="dialogs.editHost"
    :host="currentEditHost"
    @edit="editHost"
    @error="showNotification($event, 'error')"
  />

  <delete-confirm-dialog
    v-model="dialogs.deleteConfirm"
    :host="hostToDelete"
    @delete="confirmDeleteHost"
  />

  <notification-snackbar
    v-model="showSnackbar"
    :text="snackbarText"
    :color="snackbarColor"
    :timeout="3000"
  />

  <!-- 全局开关悬浮按钮 -->
  <global-switch-fab
    v-model="hostsResolveSwitch"
    @update:model-value="handleHostsSwitch"
    class="fade-in"
  />
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import type { HostEntry } from '@/types/hosts'

import GroupManager from '@/components/hosts/common/GroupManager.vue'
import HostsTable from '@/components/hosts/tables/HostsTable.vue'
import EmptyState from '@/components/hosts/common/EmptyState.vue'
import AddGroupDialog from '@/components/hosts/dialogs/AddGroupDialog.vue'
import AddHostDialog from '@/components/hosts/dialogs/AddHostDialog.vue'
import EditHostDialog from '@/components/hosts/dialogs/EditHostDialog.vue'
import DeleteConfirmDialog from '@/components/hosts/dialogs/DeleteConfirmDialog.vue'
import NotificationSnackbar from '@/components/hosts/common/NotificationSnackbar.vue'
import GlobalSwitchFab from '@/components/hosts/common/GlobalSwitchFab.vue'

import { getOsInfo } from '@/api/hosts'
import { useHostsData } from '@/composables/useHostsData'

const dialogs = ref({
  addGroup: false,
  addHost: false,
  editHost: false,
  deleteConfirm: false,
  renameGroup: false
})

const currentAddGroupName = ref('')
const currentEditHost = ref<HostEntry | null>(null)
const hostToDelete = ref<HostEntry | null>(null)
const renameGroupOriginalName = ref('')
const renameGroupName = ref('')

const showSnackbar = ref(false)
const snackbarText = ref('')
const snackbarColor = ref<'success' | 'error' | 'info' | 'warning'>('success')

const search = ref('')

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
  confirmDeleteHost,
  renameGroup
} = useHostsData(showNotification)

function openAddHostDialog(groupName: string) {
  currentAddGroupName.value = groupName
  dialogs.value.addHost = true
}

function openEditHostDialog(host: HostEntry) {
  currentEditHost.value = host
  dialogs.value.editHost = true
}

function removeHost(host: HostEntry) {
  hostToDelete.value = host
  dialogs.value.deleteConfirm = true
}

function handleOpenDomain(_domain: string, message: string) {
  showNotification(message, 'info')
}

function openRenameGroupDialog(name: string) {
  renameGroupOriginalName.value = name
  renameGroupName.value = name
  dialogs.value.renameGroup = true
}

async function handleConfirmRenameGroup() {
  const trimmed = renameGroupName.value.trim()
  if (!trimmed) {
    showNotification('分组名称不能为空', 'error')
    return
  }
  await renameGroup(renameGroupOriginalName.value, trimmed)
  dialogs.value.renameGroup = false
}

onMounted(async () => {
  await getOsInfo()
  await loadSystemHosts()
})

</script>

<style scoped>
.hosts-container {
  display: flex;
  flex-direction: column;
  height: 100%;
  overflow: hidden;
}

.main-card {
  display: flex;
  flex-direction: column;
  flex: 1;
  border-radius: 16px;
  overflow: hidden;
  box-shadow: 0 3px 10px rgba(0, 0, 0, 0.08);
  transition: all 0.3s ease;
  min-height: 0;
}
</style>
