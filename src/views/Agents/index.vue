<template>
  <div ref="workspaceElement" class="agents-workspace" data-testid="agents-workspace">
    <header class="workspace-header">
      <div class="workspace-brand">
        <span class="workspace-icon" aria-hidden="true"><v-icon :icon="mdiAccountGroupOutline" size="22" /></span>
        <div>
          <h1>{{ t('agents.title') }}</h1>
          <p>{{ t('agents.subtitle') }}</p>
        </div>
      </div>
      <div class="workspace-actions">
        <RouterLink to="/agents/chat" class="agents-button quiet" data-testid="agents-legacy-chat">
          <v-icon :icon="mdiChatOutline" size="16" aria-hidden="true" />
          {{ t('agents.legacyChat') }}
        </RouterLink>
        <button
          type="button"
          class="agents-button quiet scan-button"
          :disabled="store.scanning || store.loading"
          :aria-label="t(store.scanning ? 'agents.scanning' : 'agents.scan')"
          :title="t(store.scanning ? 'agents.scanning' : 'agents.scan')"
          data-testid="agents-scan"
          @click="scan"
        >
          <v-progress-circular v-if="store.scanning" indeterminate size="16" width="2" aria-hidden="true" />
          <v-icon v-else :icon="mdiRefresh" size="17" aria-hidden="true" />
          <span>{{ t('agents.scan') }}</span>
        </button>
        <button
          type="button"
          class="agents-button primary"
          :disabled="store.loading || !store.initialized"
          data-testid="agents-create-group"
          @click="openCreate"
        >
          <v-icon :icon="mdiPlus" size="17" aria-hidden="true" />
          {{ t('agents.createGroup') }}
        </button>
      </div>
    </header>

    <div v-if="displayError" class="workspace-error-wrap">
      <div class="agents-error" role="alert" data-testid="agents-error">
        <v-icon :icon="mdiAlertCircleOutline" size="18" aria-hidden="true" />
        <div class="agents-error-content">
          <strong>{{ t('agents.errorTitle') }}</strong>
          <p>{{ displayError }}</p>
        </div>
        <button
          v-if="store.currentSaveError"
          type="button"
          class="agents-button"
          :disabled="store.saving"
          data-testid="agents-retry-save"
          @click="retrySave"
        >{{ t('agents.retrySave') }}</button>
      </div>
    </div>

    <div v-if="store.loading" class="workspace-state" role="status" data-testid="agents-loading">
      <v-progress-circular indeterminate size="28" width="2" color="primary" aria-hidden="true" />
      <p>{{ t('agents.loading') }}</p>
      <p v-if="store.scanning" class="state-hint">{{ t('agents.scanning') }}</p>
    </div>
    <div v-else-if="!store.initialized" class="workspace-state" data-testid="agents-load-failed">
      <v-icon :icon="mdiAlertCircleOutline" size="32" aria-hidden="true" />
      <h2>{{ t('agents.loadFailed') }}</h2>
      <button type="button" class="agents-button" data-testid="agents-retry-load" @click="initialize">
        <v-icon :icon="mdiRefresh" size="16" aria-hidden="true" />
        {{ t('agents.retryLoad') }}
      </button>
    </div>
    <div v-else class="workspace-grid" :class="{ 'compact-members': compactMembers || !store.currentGroup }">
      <aside class="groups-panel" :aria-label="t('agents.groups')">
        <div class="groups-heading">
          <h2>{{ t('agents.groups') }}</h2>
          <span :aria-label="t('agents.groupCount', { count: store.groups.length })">{{ store.groups.length }}</span>
        </div>
        <div class="group-list" data-testid="agents-group-list">
          <button
            v-for="group in store.groups"
            :key="group.id"
            type="button"
            class="group-item"
            :class="{ selected: group.id === store.currentGroupId }"
            :aria-pressed="group.id === store.currentGroupId"
            :data-group-id="group.id"
            data-testid="agents-group-item"
            @click="selectGroup(group.id)"
          >
            <span class="group-name"><v-icon :icon="mdiPound" size="17" aria-hidden="true" />{{ group.name }}</span>
            <span class="group-project" :title="group.projectPath">{{ group.projectPath }}</span>
            <span class="group-summary">
              <span>{{ t('agents.memberCount', { count: group.members.length }) }}</span>
              <span>{{ t(`agents.mode.${group.executionMode}`) }}</span>
            </span>
          </button>
        </div>
        <button type="button" class="agents-button quiet sidebar-create" data-testid="agents-list-create" @click="openCreate">
          <v-icon :icon="mdiPlus" size="16" aria-hidden="true" />
          {{ t('agents.createGroup') }}
        </button>
        <p class="groups-hint">{{ t('agents.groupHint') }}</p>
      </aside>

      <section v-if="store.currentGroup" class="conversation-panel" :aria-label="t('agents.thread.label')">
        <header class="conversation-header">
          <div class="conversation-identity">
            <h2><v-icon :icon="mdiPound" size="20" aria-hidden="true" />{{ store.currentGroup.name }}</h2>
            <div class="project-row">
              <v-icon :icon="mdiFolderOutline" size="14" aria-hidden="true" />
              <span class="project-path" :title="store.currentGroup.projectPath">{{ store.currentGroup.projectPath }}</span>
              <span class="agents-badge">{{ t(`agents.mode.${store.currentGroup.executionMode}`) }}</span>
            </div>
          </div>
          <div class="conversation-actions">
            <button
              v-if="compactMembers"
              type="button"
              class="agents-button members-toggle"
              :aria-expanded="showMembers"
              aria-haspopup="dialog"
              data-testid="agents-open-members"
              @click="showMembers = true"
            >
              <v-icon :icon="mdiAccountGroupOutline" size="16" aria-hidden="true" />
              {{ t('agents.openMembers') }}
              <span v-if="currentApprovals.length" class="approval-dot">{{ currentApprovals.length }}</span>
            </button>
            <button
              type="button"
              class="agents-icon-button quiet"
              :disabled="!store.activeRuns.some(isActiveRun)"
              :aria-label="t('agents.stopGroup')"
              :title="t('agents.stopGroup')"
              data-testid="agents-stop-group"
              @click="stopGroup"
            ><v-icon :icon="mdiStopCircleOutline" size="19" aria-hidden="true" /></button>
            <button
              type="button"
              class="agents-icon-button quiet"
              :disabled="store.saving"
              :aria-label="t('agents.deleteGroup')"
              :title="t('agents.deleteGroup')"
              data-testid="agents-delete-group"
              @click="openDelete"
            ><v-icon :icon="mdiDeleteOutline" size="19" aria-hidden="true" /></button>
          </div>
        </header>
        <div v-if="store.scanning || store.saving" class="conversation-notice" role="status">
          <v-progress-circular indeterminate size="11" width="1.5" aria-hidden="true" />
          {{ t(store.scanning ? 'agents.scanning' : 'agents.saving') }}
        </div>
        <MessageThread
          :key="store.currentGroup.id"
          :group="store.currentGroup"
          :engines="store.engineOptions"
          :saving="store.saving"
          :scanning="store.scanning"
          :retrying-run-ids="retryingRunIds"
          @retry-run="retryRun"
        />
        <AgentComposer
          :key="`composer-${store.currentGroup.id}`"
          :group="store.currentGroup"
          :engines="store.engineOptions"
          :active-runs="store.activeRuns"
          :scanning="store.scanning"
          :saving="store.saving"
          :send-message="sendMessage"
        />
      </section>

      <section v-else class="empty-workspace" data-testid="agents-empty">
        <div class="empty-content">
          <div class="empty-workspace-icon" aria-hidden="true"><v-icon :icon="mdiAccountGroupOutline" size="36" /></div>
          <h2>{{ t(store.groups.length ? 'agents.noSelection' : 'agents.noGroups') }}</h2>
          <p>{{ t(store.groups.length ? 'agents.noSelectionHint' : 'agents.noGroupsHint') }}</p>
          <button type="button" class="agents-button primary" data-testid="agents-empty-create" @click="openCreate">
            <v-icon :icon="mdiPlus" size="17" aria-hidden="true" />
            {{ t('agents.createGroup') }}
          </button>
          <div v-if="store.scanning" class="engine-empty-notice" role="status" data-testid="agents-empty-scanning">
            <v-progress-circular indeterminate size="16" width="2" aria-hidden="true" />
            <span>{{ t('agents.scanning') }}</span>
          </div>
          <div v-else-if="!hasSupportedEngine" class="engine-empty-notice" data-testid="agents-unavailable">
            <v-icon :icon="mdiInformationOutline" size="20" aria-hidden="true" />
            <div><strong>{{ t('agents.noEngines') }}</strong><p>{{ t('agents.noEnginesHint') }}</p></div>
          </div>
          <p class="empty-policy">{{ t('agents.policy.noLoop') }}</p>
        </div>
      </section>

      <aside v-if="store.currentGroup && !compactMembers" class="members-panel" :aria-label="t('agents.openMembers')">
        <MembersPanel
          :group="store.currentGroup"
          :engines="store.engineOptions"
          :approvals="currentApprovals"
          :active-runs="store.activeRuns"
          :busy-member-ids="busyMemberIds"
          :saving="store.saving"
          :scanning="store.scanning"
          @toggle-member="toggleMember"
          @stop-member="stopMember"
          @approval="decideApproval"
        />
      </aside>
    </div>

    <CreateGroupDialog
      v-model="showCreate"
      :engines="store.engineOptions"
      :scanning="store.scanning"
      :submitting="creating"
      :error="createError || (showCreate ? displayError : null)"
      @scan="scan"
      @create="createGroup"
    />

    <v-dialog
      v-if="store.currentGroup && compactMembers"
      v-model="showMembers"
      max-width="440"
      aria-labelledby="agents-members-dialog-title"
    >
      <v-card class="agents-members-dialog" data-testid="agents-members-dialog">
        <header class="members-dialog-header">
          <h2 id="agents-members-dialog-title">{{ t('agents.openMembers') }}</h2>
          <button
            type="button"
            class="agents-icon-button quiet"
            :aria-label="t('agents.closeMembers')"
            data-testid="agents-close-members"
            @click="showMembers = false"
          ><v-icon :icon="mdiClose" size="20" aria-hidden="true" /></button>
        </header>
        <div class="members-dialog-body">
          <div v-if="displayError" class="members-dialog-error">
            <div class="agents-error" role="alert" data-testid="agents-members-error">
              <v-icon :icon="mdiAlertCircleOutline" size="18" aria-hidden="true" />
              <div class="agents-error-content"><p>{{ displayError }}</p></div>
            </div>
          </div>
          <MembersPanel
            :group="store.currentGroup"
            :engines="store.engineOptions"
            :approvals="currentApprovals"
            :active-runs="store.activeRuns"
            :busy-member-ids="busyMemberIds"
            :saving="store.saving"
            :scanning="store.scanning"
            @toggle-member="toggleMember"
            @stop-member="stopMember"
            @approval="decideApproval"
          />
        </div>
      </v-card>
    </v-dialog>

    <v-dialog
      :model-value="!!deleteTarget"
      :persistent="deleting"
      max-width="440"
      aria-labelledby="agents-delete-title"
      aria-describedby="agents-delete-description"
      @update:model-value="!$event && (deleteTarget = null)"
      @after-enter="deleteCancel?.focus()"
    >
      <v-card class="agents-dialog delete-dialog" data-testid="agents-delete-dialog">
        <div class="delete-dialog-body">
          <div class="delete-icon" aria-hidden="true"><v-icon :icon="mdiDeleteOutline" size="24" /></div>
          <h2 id="agents-delete-title">{{ t('agents.delete.title') }}</h2>
          <p id="agents-delete-description">{{ t('agents.delete.description', { name: deleteTarget?.name || '' }) }}</p>
          <div v-if="deleteError" class="agents-error" role="alert" data-testid="agents-delete-error">
            <div class="agents-error-content"><p>{{ deleteError }}</p></div>
          </div>
        </div>
        <footer class="delete-dialog-footer">
          <button ref="deleteCancel" type="button" class="agents-button" :disabled="deleting" @click="deleteTarget = null">{{ t('common.cancel') }}</button>
          <button
            type="button"
            class="agents-button danger"
            :disabled="deleting"
            data-testid="agents-delete-confirm"
            @click="deleteGroup"
          >
            <v-progress-circular v-if="deleting" indeterminate size="14" width="2" aria-hidden="true" />
            {{ t(deleting ? 'agents.delete.deleting' : 'agents.delete.confirm') }}
          </button>
        </footer>
      </v-card>
    </v-dialog>
  </div>
</template>

<script setup lang="ts">
import {
  mdiAccountGroupOutline,
  mdiAlertCircleOutline,
  mdiChatOutline,
  mdiClose,
  mdiDeleteOutline,
  mdiFolderOutline,
  mdiInformationOutline,
  mdiPlus,
  mdiPound,
  mdiRefresh,
  mdiStopCircleOutline,
} from '@mdi/js'
import { computed, onMounted, onUnmounted, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { RouterLink } from 'vue-router'
import { useAgentsStore } from '@/stores/agents'
import type { AgentGroup, CreateAgentGroupInput } from '@/types/agents'
import AgentComposer from './agent-composer.vue'
import { isActiveRun } from './agent-ui'
import CreateGroupDialog from './create-group-dialog.vue'
import MembersPanel from './members-panel.vue'
import MessageThread from './message-thread.vue'
import { forgetAgentDraft } from './use-agent-drafts'
import './agents.css'

const { t } = useI18n()
const store = useAgentsStore()
const showCreate = ref(false)
const showMembers = ref(false)
const creating = ref(false)
const createError = ref<string | null>(null)
const localError = ref<string | null>(null)
const deleting = ref(false)
const deleteError = ref<string | null>(null)
const deleteTarget = ref<AgentGroup | null>(null)
const deleteCancel = ref<HTMLButtonElement | null>(null)
const busyMemberIds = ref<string[]>([])
const retryingRunIds = ref<string[]>([])
const workspaceElement = ref<HTMLElement | null>(null)
const compactMembers = ref(true)
let resizeObserver: ResizeObserver | undefined

const displayError = computed(() => localError.value || store.error)
const currentApprovals = computed(() =>
  store.approvals.filter(approval => approval.groupId === store.currentGroupId)
)
const hasSupportedEngine = computed(() =>
  store.engineOptions.some(engine => engine.available && engine.modes.length > 0)
)

function errorText(error: unknown): string {
  return error instanceof Error ? error.message : String(error)
}

async function initialize(): Promise<void> {
  localError.value = null
  try {
    await store.init()
  } catch (error) {
    localError.value = errorText(error)
  }
}

async function scan(): Promise<void> {
  localError.value = null
  try {
    await store.scan()
  } catch (error) {
    localError.value = errorText(error)
  }
}

function openCreate(): void {
  createError.value = null
  showCreate.value = true
}

async function createGroup(input: CreateAgentGroupInput): Promise<void> {
  if (creating.value) return
  creating.value = true
  createError.value = null
  localError.value = null
  try {
    const group = await store.createGroup(input)
    store.selectGroup(group.id)
    showCreate.value = false
  } catch (error) {
    createError.value = errorText(error)
  } finally {
    creating.value = false
  }
}

function selectGroup(groupId: string): void {
  localError.value = null
  store.selectGroup(groupId)
}

async function sendMessage(content: string, recipients: string[]): Promise<void> {
  localError.value = null
  try {
    await store.sendMessage(content, recipients)
  } catch (error) {
    localError.value = errorText(error)
    throw error
  }
}

async function retryRun(groupId: string, runId: string): Promise<void> {
  if (retryingRunIds.value.includes(runId)) return
  retryingRunIds.value.push(runId)
  localError.value = null
  try {
    await store.retryRun(groupId, runId)
  } catch (error) {
    localError.value = errorText(error)
  } finally {
    retryingRunIds.value = retryingRunIds.value.filter(id => id !== runId)
  }
}

async function toggleMember(memberId: string): Promise<void> {
  if (busyMemberIds.value.includes(memberId)) return
  busyMemberIds.value.push(memberId)
  localError.value = null
  try {
    await store.toggleMember(memberId)
  } catch (error) {
    localError.value = errorText(error)
  } finally {
    busyMemberIds.value = busyMemberIds.value.filter(id => id !== memberId)
  }
}

function stopMember(memberId: string): void {
  try {
    store.stopMember(memberId)
  } catch (error) {
    localError.value = errorText(error)
  }
}

function stopGroup(): void {
  try {
    store.stopGroup()
  } catch (error) {
    localError.value = errorText(error)
  }
}

async function decideApproval(approvalId: string, allow: boolean): Promise<void> {
  localError.value = null
  try {
    await store.decideApproval(approvalId, allow)
  } catch (error) {
    localError.value = errorText(error)
  }
}

async function retrySave(): Promise<void> {
  localError.value = null
  try {
    await store.retrySave()
  } catch (error) {
    localError.value = errorText(error)
  }
}

function openDelete(): void {
  deleteError.value = null
  deleteTarget.value = store.currentGroup
}

async function deleteGroup(): Promise<void> {
  const group = deleteTarget.value
  if (!group || deleting.value) return
  deleting.value = true
  deleteError.value = null
  try {
    await store.deleteGroup(group.id)
    forgetAgentDraft(group.id)
    deleteTarget.value = null
    localError.value = null
  } catch (error) {
    deleteError.value = errorText(error)
  } finally {
    deleting.value = false
  }
}

function updateLayout(): void {
  compactMembers.value = (workspaceElement.value?.clientWidth ?? 0) < 940
}

watch(compactMembers, compact => {
  if (!compact) showMembers.value = false
})
watch(
  () => store.currentGroupId,
  () => {
    showMembers.value = false
  }
)

onMounted(() => {
  updateLayout()
  if (typeof ResizeObserver !== 'undefined' && workspaceElement.value) {
    resizeObserver = new ResizeObserver(updateLayout)
    resizeObserver.observe(workspaceElement.value)
  } else {
    window.addEventListener('resize', updateLayout)
  }
  void initialize()
})
onUnmounted(() => {
  resizeObserver?.disconnect()
  window.removeEventListener('resize', updateLayout)
})
</script>

<style scoped>
.agents-workspace { display: flex; flex-direction: column; height: 100%; min-height: 0; min-width: 0; background: var(--bg); container-type: inline-size; }
.workspace-header { display: flex; flex-wrap: wrap; align-items: center; justify-content: space-between; gap: 12px 20px; padding: 16px 20px; border-bottom: 1px solid var(--border); flex-shrink: 0; }
.workspace-brand { display: flex; align-items: center; gap: 10px; min-width: 0; }
.workspace-brand > div { min-width: 0; }
.workspace-icon { display: flex; align-items: center; justify-content: center; width: 38px; height: 38px; border: 1px solid var(--border); border-radius: 10px; color: var(--accent); background: var(--bg-elevated); flex-shrink: 0; }
.workspace-brand h1 { margin: 0; font-size: 17px; font-weight: 650; line-height: 1.4; letter-spacing: -.2px; }
.workspace-brand p { margin: 2px 0 0; font-size: 11px; color: var(--text-muted); }
.workspace-actions { display: flex; flex-wrap: wrap; align-items: center; gap: 7px; }
.workspace-error-wrap { padding: 10px 14px; border-bottom: 1px solid var(--border); flex-shrink: 0; max-height: 190px; overflow-y: auto; }
.workspace-error-wrap .agents-error { flex-wrap: wrap; }
.workspace-error-wrap .agents-error-content { flex-basis: 220px; }
.workspace-error-wrap .agents-button { min-height: 30px; }
.workspace-state { display: flex; flex: 1; min-height: 0; flex-direction: column; align-items: center; justify-content: center; gap: 13px; padding: 24px; overflow-y: auto; text-align: center; }
.workspace-state p { margin: 0; color: var(--text-muted); }
.workspace-state h2 { margin: 0; font-weight: 600; font-size: 17px; }
.workspace-state .state-hint { font-size: 12px; }
.workspace-state > .v-icon { color: var(--text-muted); }
.workspace-grid { display: grid; grid-template-columns: 194px minmax(0, 1fr) 250px; flex: 1; min-height: 0; min-width: 0; }
.workspace-grid.compact-members { grid-template-columns: 194px minmax(0, 1fr); }
.groups-panel { display: flex; flex-direction: column; min-height: 0; min-width: 0; padding: 18px 10px 14px; background: var(--bg-elevated); border-right: 1px solid var(--border); overflow-y: auto; scroll-padding: 12px; }
.groups-heading { display: flex; align-items: center; justify-content: space-between; gap: 8px; margin: 0 8px 13px; }
.groups-heading h2 { margin: 0; font-size: 11px; font-weight: 600; color: var(--text-muted); }
.groups-heading > span { color: var(--text-muted); font-size: 11px; }
.group-list { display: flex; flex-direction: column; gap: 8px; }
.group-item { display: flex; flex-direction: column; gap: 8px; padding: 12px 10px; border: 1px solid var(--border); border-radius: 9px; background: var(--bg); text-align: left; color: var(--text); font: inherit; cursor: pointer; min-width: 0; width: 100%; transition: border-color var(--motion-fast), background var(--motion-fast); }
.group-item:hover { border-color: var(--border-strong); background: var(--bg-elevated-2); }
.group-item.selected { border-color: rgb(var(--accent-rgb) / 0.4); background: rgb(var(--accent-rgb) / 0.07); }
.group-name { display: flex; align-items: flex-start; gap: 5px; font-size: 12px; line-height: 1.5; font-weight: 600; overflow-wrap: anywhere; }
.group-name .v-icon { flex-shrink: 0; color: var(--text-muted); margin-top: 1px; }
.group-item.selected .group-name { color: var(--accent); }
.group-project { display: block; color: var(--text-muted); font-family: var(--jedi-font-mono); font-size: 10px; line-height: 1.7; overflow: hidden; white-space: nowrap; text-overflow: ellipsis; width: 100%; }
.group-summary { display: flex; flex-wrap: wrap; align-items: baseline; justify-content: space-between; gap: 3px 6px; font-size: 10px; color: var(--text-muted); }
.sidebar-create { justify-content: flex-start; margin-top: 9px; width: 100%; }
.groups-hint { margin: auto 7px 0; padding-top: 22px; color: var(--text-muted); font-size: 11px; line-height: 1.8; }
.conversation-panel { display: flex; flex-direction: column; min-width: 0; min-height: 0; background: var(--bg); }
.conversation-header { display: flex; flex-wrap: wrap; align-items: flex-start; gap: 10px 12px; padding: 17px 20px 15px; border-bottom: 1px solid var(--border); flex-shrink: 0; }
.conversation-identity { flex: 1 1 170px; min-width: 0; }
.conversation-identity h2 { display: flex; align-items: flex-start; gap: 5px; margin: 0; color: var(--text); font-size: 16px; font-weight: 650; line-height: 1.5; overflow-wrap: anywhere; }
.conversation-identity h2 .v-icon { margin-top: 2px; color: var(--text-muted); flex-shrink: 0; }
.project-row { display: flex; flex-wrap: wrap; align-items: center; gap: 5px 7px; margin-top: 5px; color: var(--text-muted); }
.project-row > .v-icon { flex-shrink: 0; }
.project-path { max-width: 100%; min-width: 0; color: var(--text-muted); font-family: var(--jedi-font-mono); font-size: 10px; overflow-wrap: anywhere; }
.project-row .agents-badge { font-size: 10px; padding: 0 5px; }
.conversation-actions { display: flex; align-items: center; flex-wrap: wrap; gap: 3px; }
.members-toggle { font-size: 11px; }
.approval-dot { display: inline-flex; align-items: center; justify-content: center; border-radius: 4px; min-width: 18px; padding: 0 4px; color: var(--warning); background: rgb(var(--warning-rgb) / 0.1); }
.conversation-notice { display: flex; align-items: center; gap: 6px; min-height: 24px; padding: 3px 20px; border-bottom: 1px solid var(--border); background: var(--bg-elevated); color: var(--text-muted); font-size: 10px; flex-shrink: 0; }
.members-panel { min-width: 0; min-height: 0; overflow-y: auto; background: var(--bg-elevated); border-left: 1px solid var(--border); scroll-padding: 12px; }
.empty-workspace { display: flex; align-items: center; justify-content: center; min-width: 0; min-height: 0; padding: 28px; overflow-y: auto; }
.empty-content { width: 100%; max-width: 480px; padding: 18px 0; }
.empty-workspace-icon { display: flex; align-items: center; justify-content: center; width: 64px; height: 64px; margin-bottom: 20px; border: 1px solid var(--border); border-radius: 15px; background: var(--bg-elevated); color: var(--accent); }
.empty-content h2 { margin: 0; font-size: 23px; font-weight: 600; line-height: 1.45; letter-spacing: -.3px; text-wrap: balance; }
.empty-content > p { margin: 12px 0 20px; color: var(--text-muted); font-size: 13px; line-height: 1.8; }
.engine-empty-notice { display: flex; align-items: flex-start; gap: 9px; padding: 14px; margin-top: 24px; background: var(--bg-elevated); border: 1px solid var(--border); border-radius: 9px; font-size: 12px; color: var(--text-muted); }
.engine-empty-notice > .v-icon, .engine-empty-notice > .v-progress-circular { flex-shrink: 0; margin-top: 2px; }
.engine-empty-notice strong { font-size: 12px; color: var(--text); font-weight: 600; }
.engine-empty-notice p { margin: 6px 0 0; font-size: 12px; line-height: 1.75; }
.empty-content .empty-policy { font-size: 11px; margin: 20px 0 0; }
.members-dialog-header { display: flex; align-items: center; justify-content: space-between; gap: 16px; padding: 15px 16px; border-bottom: 1px solid var(--border); flex-shrink: 0; }
.members-dialog-header h2 { font-size: 15px; font-weight: 600; margin: 0; }
.agents-members-dialog { display: flex; flex-direction: column; max-height: calc(100dvh - 48px); }
.members-dialog-body { overflow-y: auto; min-height: 0; scroll-padding: 16px; }
.members-dialog-error { padding: 14px 16px 0; }
.delete-dialog-body { padding: 24px; }
.delete-icon { display: flex; align-items: center; justify-content: center; width: 46px; height: 46px; border: 1px solid rgb(var(--danger-rgb) / 0.2); background: rgb(var(--danger-rgb) / 0.06); border-radius: 12px; color: var(--danger); margin-bottom: 14px; }
.delete-dialog-body h2 { font-size: 18px; font-weight: 600; margin: 0; }
.delete-dialog-body > p { color: var(--text-muted); font-size: 13px; line-height: 1.8; margin: 8px 0 0; overflow-wrap: anywhere; }
.delete-dialog-body .agents-error { margin-top: 14px; }
.delete-dialog-footer { padding: 16px 24px; display: flex; flex-wrap: wrap; justify-content: flex-end; gap: 8px; border-top: 1px solid var(--border); background: var(--bg-elevated); }
@container (max-width: 790px) {
  .scan-button { width: 36px; padding: 7px; }
  .scan-button > span { display: none; }
  .workspace-header { gap: 12px; }
  .workspace-actions { margin-left: auto; }
}
@container (max-width: 650px) {
  .workspace-header { padding: 14px; }
  .workspace-brand { align-items: flex-start; }
  .workspace-actions { width: 100%; justify-content: flex-end; gap: 5px; }
  .workspace-actions .agents-button { padding-inline: 9px; font-size: 11px; }
  .workspace-grid, .workspace-grid.compact-members { grid-template-columns: minmax(0, 1fr); grid-template-rows: auto minmax(0, 1fr); }
  .groups-panel { flex-direction: row; flex-wrap: wrap; align-items: center; gap: 8px; padding: 10px 12px; max-height: 154px; border-right: 0; border-bottom: 1px solid var(--border); overflow-y: hidden; }
  .groups-heading { margin: 0; flex: 1; }
  .group-list { flex-direction: row; gap: 7px; overflow-x: auto; padding: 4px 3px 6px; margin: -3px; order: 1; flex-basis: calc(100% + 6px); min-width: 0; }
  .group-item { flex: 0 0 178px; padding: 8px 9px; gap: 4px; }
  .group-list:empty { display: none; }
  .sidebar-create { width: auto; min-height: 26px; padding: 3px 5px; font-size: 11px; margin: 0; }
  .groups-hint { display: none; }
  .conversation-header { padding: 13px 14px 12px; gap: 8px; }
  .conversation-actions { margin-left: auto; }
  .conversation-identity { flex-basis: 100%; }
  .empty-workspace { padding: 20px; }
  .empty-content h2 { font-size: 20px; }
  .empty-workspace-icon { width: 52px; height: 52px; }
  .workspace-error-wrap { padding: 8px; max-height: 140px; }
}
@media (prefers-reduced-motion: reduce) { .group-item { transition: none; } }
</style>
