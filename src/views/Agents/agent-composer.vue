<template>
  <form class="composer" @submit.prevent="send">
    <div class="recipient-row">
      <label for="agents-recipient">{{ t('agents.composer.recipientLabel') }}</label>
      <div class="recipient-select-wrap">
        <select
          id="agents-recipient"
          v-model="draft.recipientId"
          class="recipient-select"
          :disabled="draft.sending"
          aria-describedby="agents-recipient-hint"
          data-testid="agents-recipient"
        >
          <option value="all">{{ t('agents.composer.all', { count: enabledMembers.length }) }}</option>
          <option
            v-for="member in group.members"
            :key="member.id"
            :value="member.id"
            :disabled="!member.enabled || !memberAvailable(member.engineId)"
          >{{ t(!member.enabled ? 'agents.composer.pausedRecipient' : !memberAvailable(member.engineId) ? 'agents.composer.unavailableRecipient' : 'agents.composer.recipient', { name: member.name }) }}</option>
        </select>
        <v-icon :icon="mdiChevronDown" size="16" aria-hidden="true" />
      </div>
      <p id="agents-recipient-hint">{{ recipientHint }}</p>
    </div>
    <div class="composer-box" :class="{ sending: draft.sending }">
      <label for="agents-message" class="agents-sr-only">{{ t('agents.composer.label') }}</label>
      <textarea
        id="agents-message"
        v-model="draft.content"
        :placeholder="t('agents.composer.placeholder')"
        rows="3"
        maxlength="20000"
        :aria-describedby="blockedReason ? 'agents-composer-shortcut agents-composer-state' : 'agents-composer-shortcut'"
        data-testid="agents-composer-input"
        @keydown="handleKeydown"
        @compositionstart="composing = true"
        @compositionend="composing = false"
      ></textarea>
      <div class="composer-footer">
        <span id="agents-composer-shortcut">{{ t('agents.composer.shortcut') }}</span>
        <button
          class="agents-button primary send-button"
          type="submit"
          :disabled="!canSend"
          data-testid="agents-send"
        >
          <v-progress-circular v-if="draft.sending" indeterminate size="15" width="2" aria-hidden="true" />
          <v-icon v-else :icon="mdiSendOutline" size="17" aria-hidden="true" />
          {{ t(draft.sending ? 'agents.composer.sending' : 'agents.composer.send') }}
        </button>
      </div>
    </div>
    <p v-if="blockedReason" id="agents-composer-state" class="composer-state" role="status" data-testid="agents-composer-state">
      <v-icon :icon="mdiInformationOutline" size="14" aria-hidden="true" />
      {{ blockedReason }}
    </p>
  </form>
</template>

<script setup lang="ts">
import { mdiChevronDown, mdiInformationOutline, mdiSendOutline } from '@mdi/js'
import { computed, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import type { AgentEngineOption, AgentGroup, AgentGroupRun } from '@/types/agents'
import { engineSupportsMode, isActiveRun } from './agent-ui'
import { agentDraftFor } from './use-agent-drafts'

const props = defineProps<{
  group: AgentGroup
  engines: AgentEngineOption[]
  activeRuns: AgentGroupRun[]
  scanning: boolean
  saving: boolean
  sendMessage: (content: string, recipients: string[]) => Promise<void>
}>()

const { t } = useI18n()
const draft = computed(() => agentDraftFor(props.group.id))
const composing = ref(false)
const enabledMembers = computed(() => props.group.members.filter(member => member.enabled))
const selectedMember = computed(() =>
  props.group.members.find(member => member.id === draft.value.recipientId)
)
const recipients = computed(() =>
  draft.value.recipientId === 'all'
    ? enabledMembers.value
    : selectedMember.value
      ? [selectedMember.value]
      : []
)
const recipientHint = computed(() =>
  draft.value.recipientId === 'all'
    ? t('agents.composer.groupHint')
    : t('agents.composer.directHint', { name: selectedMember.value?.name ?? '' })
)

function memberAvailable(engineId: string): boolean {
  return engineSupportsMode(
    props.engines.find(engine => engine.id === engineId),
    props.group.executionMode
  )
}

const blockedReason = computed(() => {
  if (props.scanning) return t('agents.composer.scanning')
  if (!recipients.value.length) return t('agents.composer.allPaused')
  if (recipients.value.some(member => !member.enabled)) return t('agents.composer.recipientPaused')
  if (recipients.value.some(member => !memberAvailable(member.engineId)))
    return t('agents.composer.recipientUnavailable')
  if (
    props.activeRuns.some(
      run => isActiveRun(run) && recipients.value.some(member => member.id === run.memberId)
    )
  ) {
    return t('agents.composer.recipientsBusy')
  }
  if (props.saving) return t('agents.composer.savePending')
  return ''
})
const canSend = computed(
  () => !!draft.value.content.trim() && !draft.value.sending && !blockedReason.value
)

function handleKeydown(event: KeyboardEvent): void {
  if (event.key !== 'Enter' || !(event.ctrlKey || event.metaKey)) return
  if (event.isComposing || composing.value || event.keyCode === 229) return
  event.preventDefault()
  void send()
}

async function send(): Promise<void> {
  if (!canSend.value || composing.value) return
  const sendingDraft = draft.value
  const original = sendingDraft.content
  const targetIds = sendingDraft.recipientId === 'all' ? [] : [sendingDraft.recipientId]
  sendingDraft.sending = true
  try {
    await props.sendMessage(original.trim(), targetIds)
    // A user can keep writing, or switch groups, while dispatch is persisted.
    if (sendingDraft.content === original) sendingDraft.content = ''
  } catch {
    // The page displays the store/dispatch error. Keep the draft for retry.
  } finally {
    sendingDraft.sending = false
  }
}
</script>

<style scoped>
.composer { flex-shrink: 0; padding: 15px 22px 18px; border-top: 1px solid var(--border); background: var(--bg); min-width: 0; }
.recipient-row { display: flex; align-items: center; flex-wrap: wrap; gap: 7px 10px; margin-bottom: 10px; }
.recipient-row label { font-size: 11px; color: var(--text-muted); }
.recipient-select-wrap { position: relative; display: inline-flex; align-items: center; max-width: 100%; min-width: 0; }
.recipient-select-wrap > .v-icon { position: absolute; right: 7px; pointer-events: none; color: var(--text-muted); }
.recipient-select { appearance: none; border: 1px solid var(--border); border-radius: 6px; padding: 5px 27px 5px 9px; background: var(--bg-elevated); color: var(--text); font: inherit; font-size: 11px; min-height: 30px; max-width: 210px; cursor: pointer; text-overflow: ellipsis; }
.recipient-select option { background: var(--bg); color: var(--text); }
.recipient-row p { flex: 1 1 220px; margin: 0; font-size: 10px; color: var(--text-muted); }
.composer-box { padding: 12px 12px 10px; border: 1px solid var(--border-strong); border-radius: 12px; background: var(--bg-input); transition: border-color var(--motion-fast); }
.composer-box:focus-within { border-color: var(--accent); }
.composer-box textarea { display: block; width: 100%; min-height: 65px; max-height: 24vh; resize: vertical; border: 0; background: transparent; color: var(--text); padding: 0 2px 8px; font: inherit; font-size: 13px; line-height: 1.7; }
.composer-box textarea::placeholder { color: var(--text-subtle); opacity: 1; }
.composer-box textarea:focus-visible { outline-offset: 3px; }
.composer-footer { display: flex; align-items: center; justify-content: space-between; flex-wrap: wrap; gap: 8px; }
.composer-footer > span { color: var(--text-muted); font-size: 10px; }
.send-button { min-height: 34px; margin-left: auto; }
.composer-state { display: flex; align-items: baseline; gap: 5px; margin: 8px 0 0; color: var(--text-muted); font-size: 11px; line-height: 1.6; overflow-wrap: anywhere; }
.composer-state .v-icon { flex-shrink: 0; }
@media (max-width: 700px) {
  .composer { padding: 12px 14px 14px; }
  .recipient-row p { flex-basis: 100%; }
  .composer-box { padding: 11px; }
  .composer-box textarea { min-height: 50px; }
  .composer-footer > span { font-size: 9px; }
}
@media (prefers-reduced-motion: reduce) { .composer-box { transition: none; } }
</style>
