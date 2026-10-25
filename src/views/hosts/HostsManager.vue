<template>
  <div class="hosts-container scifi-page">
    <!-- 页头：与聊天页 chat-header 统一 -->
    <div class="hosts-header">
      <div class="header-logo">
        <v-icon :icon="mdiDomain" size="18" aria-hidden="true" />
        <span class="header-title">{{ t('sidebar.hostsManager') }}</span>
      </div>
      <div class="header-actions">
        <button
          type="button"
          class="global-switch-btn"
          :class="{ on: hostsResolveSwitch }"
          :aria-pressed="hostsResolveSwitch"
          :title="t('hosts.globalSwitch.hint')"
          @click="toggleGlobalSwitch"
        >
          <v-icon
            :icon="hostsResolveSwitch ? mdiPowerPlugOutline : mdiPowerPlugOffOutline"
            size="16"
            aria-hidden="true"
          />
          <span>{{ t('hosts.globalSwitch.label') }}: {{ hostsResolveSwitch ? t('hosts.globalSwitch.enabled') : t('hosts.globalSwitch.disabled') }}</span>
        </button>
        <button
          type="button"
          class="header-primary-btn"
          @click="dialogs.addGroup = true"
        >
          <v-icon :icon="mdiPlus" size="16" aria-hidden="true" />
          <span>{{ t('hosts.groups.add') }}</span>
        </button>
      </div>
    </div>

    <div class="content-wrapper">
      <!-- 分组管理区域 -->
      <group-manager
        v-if="groups.length"
        v-model="selectedGroup"
        :groups="groups"
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

  <AppNotification />
</template>

<script setup lang="ts">
import { mdiDomain, mdiPlus, mdiPowerPlugOffOutline, mdiPowerPlugOutline } from '@mdi/js'
import { onMounted, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { getOsInfo } from '@/api/hosts'
import AppNotification from '@/components/common/AppNotification.vue'
import EmptyState from '@/components/hosts/common/EmptyState.vue'
import GroupManager from '@/components/hosts/common/GroupManager.vue'
import AddGroupDialog from '@/components/hosts/dialogs/AddGroupDialog.vue'
import AddHostDialog from '@/components/hosts/dialogs/AddHostDialog.vue'
import DeleteConfirmDialog from '@/components/hosts/dialogs/DeleteConfirmDialog.vue'
import EditHostDialog from '@/components/hosts/dialogs/EditHostDialog.vue'
import HostsTable from '@/components/hosts/tables/HostsTable.vue'
import { useAppNotification } from '@/composables/useAppNotification'
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

const search = ref('')

const { notify } = useAppNotification()

function showNotification(text: string, color: 'success' | 'error' | 'info' | 'warning') {
  notify(text, color)
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

// 与原悬浮开关等价：先乐观翻转状态，再调用统一切换逻辑（失败时由 useHostsData 回滚）
function toggleGlobalSwitch() {
  const next = !hostsResolveSwitch.value
  hostsResolveSwitch.value = next
  handleHostsSwitch(next)
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

/* 页头：对齐聊天页 .chat-header 令牌 */
.hosts-header {
  position: relative;
  z-index: 2;
  display: flex;
  align-items: center;
  justify-content: space-between;
  flex-shrink: 0;
  min-height: 60px;
  padding: 12px 24px;
  background: var(--bg);
  border-bottom: 1px solid var(--border);
}

.hosts-header .header-logo {
  display: flex;
  align-items: center;
  gap: 10px;
  color: var(--text);
}

.hosts-header .header-title {
  font-size: 15px;
  font-weight: 600;
  color: var(--text);
  font-family: var(--jedi-font-ui);
}

.header-primary-btn {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  min-height: 32px;
  padding: 0 14px;
  font-size: 13px;
  font-weight: 500;
  font-family: var(--jedi-font-ui);
  color: var(--on-accent);
  background: var(--accent);
  border: 1px solid var(--accent);
  border-radius: 8px;
  cursor: pointer;
  transition: background var(--motion-fast), border-color var(--motion-fast);
}

.header-primary-btn:hover {
  background: var(--accent-hover);
  border-color: var(--accent-hover);
}

.header-actions {
  display: flex;
  align-items: center;
  gap: 10px;
}

.global-switch-btn {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  min-height: 32px;
  padding: 0 12px;
  font-size: 13px;
  font-family: var(--jedi-font-ui);
  color: var(--text);
  background: transparent;
  border: 1px solid var(--border);
  border-radius: 8px;
  cursor: pointer;
  transition: border-color var(--motion-fast), color var(--motion-fast);
}

.global-switch-btn:hover {
  border-color: rgb(var(--accent-rgb) / 0.5);
}

.global-switch-btn.on {
  color: var(--success);
  border-color: rgb(var(--success-rgb) / 0.45);
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
