<template>
  <div class="hosts-container scifi-page">
    <!-- CRT Effects -->
    <div class="scanlines"></div>
    <div class="crt-vignette"></div>

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
        <span class="dialog-title">[ RENAME_GROUP ]</span>
      </v-card-title>
      <v-card-text class="console-card-text">
        <div class="input-wrapper">
          <span class="input-prompt">>></span>
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
          <span class="btn-text">{{ t('common.cancel').toUpperCase() }}</span>
        </button>
        <button
          class="console-btn primary"
          @click="handleConfirmRenameGroup"
        >
          <span class="btn-text">{{ t('common.save').toUpperCase() }}</span>
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

/* Sci-Fi Card Overrides for Dialog */
.console-title-bar {
  background: linear-gradient(180deg, #0f0f1a 0%, #0a0a12 100%);
  border-bottom: 1px solid rgba(0, 255, 255, 0.2);
  padding: 12px 16px;
}

.dialog-title {
  font-size: 11px;
  font-weight: 700;
  letter-spacing: 2px;
  color: #00ff88;
  text-shadow: 0 0 10px rgba(0, 255, 136, 0.5);
  font-family: 'JetBrains Mono', monospace;
}

.console-card-text {
  background: #0a0a0f;
  padding: 20px 16px;
}

.console-card-actions {
  background: linear-gradient(0deg, #0f0f1a 0%, #0a0a12 100%);
  border-top: 1px solid rgba(0, 255, 255, 0.15);
  padding: 12px 16px;
}

.console-btn {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  background: transparent;
  border: 1px solid #00ff88;
  color: #00ff88;
  padding: 8px 16px;
  border-radius: 4px;
  font-family: 'JetBrains Mono', monospace;
  font-size: 11px;
  font-weight: 700;
  letter-spacing: 1px;
  cursor: pointer;
  transition: all 0.2s ease;
}

.console-btn:hover {
  background: rgba(0, 255, 136, 0.07);
  box-shadow: 0 0 15px rgba(0, 255, 136, 0.27), inset 0 0 15px rgba(0, 255, 136, 0.07);
}

.console-btn.primary {
  border-color: #00ffff;
  color: #00ffff;
}

.console-btn.primary:hover {
  background: rgba(0, 255, 255, 0.07);
  box-shadow: 0 0 15px rgba(0, 255, 255, 0.27), inset 0 0 15px rgba(0, 255, 255, 0.07);
}

.btn-text {
  letter-spacing: 1px;
}

/* =========================================
   Light Theme Styles (Tatooine Outpost)
   ========================================= */
.light-theme .console-title-bar {
  background: linear-gradient(180deg, #efe0cc 0%, #e8d4bc 100%);
  border-bottom: 1px solid #b8860b;
}

.light-theme .dialog-title {
  color: #cd7f32;
  text-shadow: 0 0 8px rgba(205, 127, 50, 0.3);
}

.light-theme .console-card-text {
  background: #faf3e8;
}

.light-theme .console-card-actions {
  background: linear-gradient(0deg, #e8d4bc 0%, #efe0cc 100%);
  border-top: 1px solid #b8860b;
}

.light-theme .console-btn {
  border-color: #cd7f32;
  color: #cd7f32;
}

.light-theme .console-btn:hover {
  background: rgba(205, 127, 50, 0.1);
  box-shadow: 0 2px 8px rgba(205, 127, 50, 0.25);
}

.light-theme .console-btn.primary {
  border-color: #cd7f32;
  color: #cd7f32;
}

.light-theme .console-btn.primary:hover {
  background: rgba(205, 127, 50, 0.1);
  box-shadow: 0 2px 8px rgba(205, 127, 50, 0.3);
}

/* Tatooine Texture Enhancements */
.light-theme .hosts-container {
  background: 
    linear-gradient(180deg, rgba(245, 230, 211, 0.95) 0%, rgba(239, 224, 204, 0.98) 100%),
    repeating-linear-gradient(
      -45deg,
      transparent,
      transparent 20px,
      rgba(184, 134, 11, 0.015) 20px,
      rgba(184, 134, 11, 0.015) 40px
    );
}

.light-theme .content-wrapper {
  background: transparent;
}

.light-theme .scifi-card {
  background: linear-gradient(135deg, #efe0cc 0%, #e8d4bc 100%);
  border-color: #b8860b;
  box-shadow: 
    0 4px 20px rgba(107, 68, 35, 0.2),
    inset 0 1px 0 rgba(255, 255, 255, 0.3);
}

.light-theme .console-title-bar {
  background: linear-gradient(180deg, #e8d4bc 0%, #dcc8a8 100%);
  border-bottom-color: #b8860b;
}

.light-theme .console-card-text {
  background: #faf3e8;
  border-top: none;
}

.light-theme .console-card-actions {
  background: linear-gradient(0deg, #dcc8a8 0%, #e8d4bc 100%);
  border-top-color: #b8860b;
}
</style>
