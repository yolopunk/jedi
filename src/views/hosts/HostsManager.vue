<template>
  <div class="hosts-container scifi-page">
    <div class="content-wrapper">
      <!-- 分组管理区域 -->
      <group-manager
        v-if="groups.length"
        v-model="selectedGroup"
        :groups="groups"
        @add-group="dialogs.addGroup = true"
        @rename-group="openRenameGroupDialog"
        class="fade-in flex-shrink-0"
      />

      <!-- 数据展示区域 -->
      <template v-if="loading || currentGroup">
        <div class="flex-grow-1 overflow-hidden d-flex flex-column">
          <hosts-table
            :current-group="currentGroup || { name: '', hosts: [] }"
            v-model:search="search"
            :loading="loading"
            @update-status="updateHostStatus"
            @edit-host="openEditHostDialog"
            @delete-host="removeHost"
            @add-host="openAddHostDialog"
            @open-domain="handleOpenDomain"
            class="fade-in-scale h-100"
          />
        </div>
      </template>

      <!-- 空状态显示 -->
      <template v-else>
        <empty-state
          @add-group="dialogs.addGroup = true"
          @use-default="initializeDefaultConfig"
          class="fade-in-scale h-100"
        />
      </template>
    </div>
  </div>

  <!-- 对话框区域 -->
  <add-group-dialog
    v-model="dialogs.addGroup"
    @add="addGroup"
    @error="showNotification($event, 'error')"
  />

  <v-dialog v-model="dialogs.renameGroup" max-width="400">
    <v-card class="scifi-card">
      <v-card-title class="console-title-bar">
        <span class="dialog-title">{{ $t('hosts.dialog.renameGroupTitle') }}</span>
      </v-card-title>
      <v-card-text class="console-card-text">
        <div class="input-wrapper">
          <input
            v-model="renameGroupName"
            type="text"
            class="console-input"
            :placeholder="$t('hosts.dialog.groupNameLabel')"
          />
        </div>
      </v-card-text>
      <v-card-actions class="console-card-actions">
        <v-spacer />
        <button
          class="console-btn"
          @click="dialogs.renameGroup = false"
        >
          <span class="btn-text">{{ t('common.cancel') }}</span>
        </button>
        <button
          class="console-btn primary"
          @click="handleConfirmRenameGroup"
        >
          <span class="btn-text">{{ t('common.save') }}</span>
        </button>
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
import { onMounted, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import EmptyState from '@/components/hosts/common/EmptyState.vue'
import GlobalSwitchFab from '@/components/hosts/common/GlobalSwitchFab.vue'
import GroupManager from '@/components/hosts/common/GroupManager.vue'
import NotificationSnackbar from '@/components/hosts/common/NotificationSnackbar.vue'
import AddGroupDialog from '@/components/hosts/dialogs/AddGroupDialog.vue'
import AddHostDialog from '@/components/hosts/dialogs/AddHostDialog.vue'
import DeleteConfirmDialog from '@/components/hosts/dialogs/DeleteConfirmDialog.vue'
import EditHostDialog from '@/components/hosts/dialogs/EditHostDialog.vue'
import HostsTable from '@/components/hosts/tables/HostsTable.vue'
import { getOsInfo } from '@/api/hosts'
import { useHostsData } from '@/composables/useHostsData'
import type { HostEntry } from '@/types/hosts'

const { t } = useI18n()

const dialogs = ref({
  addGroup: false,
  addHost: false,
  editHost: false,
  deleteConfirm: false,
  renameGroup: false,
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
  renameGroup,
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
    showNotification(t('hosts.validation.groupNameRequired'), 'error')
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
  flex: 1;
  min-height: 0;
  overflow: hidden;
  position: relative;
  font-family: 'JetBrains Mono', 'Fira Code', 'SF Mono', monospace;
}

.content-wrapper {
  position: relative;
  z-index: 1;
  display: flex;
  flex-direction: column;
  flex: 1;
  min-height: 0;
  overflow: hidden;
  padding: 8px;
}

/* Clean dialog chrome */
.console-title-bar {
  background: var(--bg);
  border-bottom: 1px solid var(--border);
  padding: 14px 16px;
}

.dialog-title {
  font-size: 15px;
  font-weight: 600;
  color: var(--text);
  font-family: var(--jedi-font-ui);
}

.dialog-input {
  width: 100%;
}

.console-card-text {
  background: var(--bg-terminal);
  padding: 20px 16px;
}

.console-card-actions {
  background: linear-gradient(0deg, var(--bg-terminal) 0%, var(--bg-terminal) 100%);
  border-top: 1px solid rgb(var(--accent-rgb) / 0.15);
  padding: 12px 16px;
}

</style>
