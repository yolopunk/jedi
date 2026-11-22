<template>
  <div class="members-content" data-testid="agents-members-panel">
    <section class="member-section" :aria-label="t('agents.members.title')">
      <div class="section-heading">
        <h3>{{ t('agents.members.title') }}</h3>
        <span class="section-count">{{ t('agents.memberCount', { count: group.members.length }) }}</span>
      </div>
      <div class="member-list">
        <article
          v-for="row in memberRows"
          :key="row.member.id"
          class="member"
          :data-member-id="row.member.id"
          data-testid="agents-member"
        >
          <div class="member-identity">
            <div class="member-avatar" aria-hidden="true"><v-icon :icon="mdiRobotOutline" size="19" /></div>
            <div class="member-name">
              <h4>{{ row.member.name }}</h4>
              <p>{{ row.engine?.location || row.member.engineId }}</p>
            </div>
          </div>
          <p class="member-role">{{ row.member.role }}</p>
          <div class="member-status" aria-live="polite">
            <span
              class="agents-badge"
              :class="row.active || row.member.enabled ? runStatusClass(row.run) : ''"
              data-testid="agents-member-status"
            >
              <v-progress-circular v-if="row.active && row.run?.status !== 'awaiting_approval'" indeterminate size="11" width="1.5" aria-hidden="true" />
              {{ t(row.active && row.run ? `agents.status.${row.run.status}` : !row.member.enabled ? 'agents.members.paused' : row.run ? `agents.status.${row.run.status}` : 'agents.members.idle') }}
            </span>
            <span v-if="row.active && !row.member.enabled" class="agents-badge">{{ t('agents.members.paused') }}</span>
          </div>
          <p v-if="!row.available && !scanning" class="member-unavailable">
            <v-icon :icon="mdiAlertCircleOutline" size="13" aria-hidden="true" />
            {{ t(engineStatusKey(row.engine, group.executionMode)) }}
          </p>
          <p v-if="row.run?.error" class="member-error">{{ row.run.error }}</p>
          <details v-if="row.run?.tools.length" class="member-tools">
            <summary>{{ t('agents.thread.toolActivity', { count: row.run.tools.length }) }}</summary>
            <ul>
              <li v-for="tool in row.run.tools" :key="tool.id">
                <div><strong>{{ tool.name }}</strong><span>{{ t(`agents.toolStatus.${tool.status}`) }}</span></div>
                <p v-if="tool.detail">{{ tool.detail }}</p>
              </li>
            </ul>
          </details>
          <div class="member-actions">
            <button
              type="button"
              class="agents-button quiet"
              :disabled="saving || busyMemberIds.includes(row.member.id)"
              :aria-label="t(row.member.enabled ? 'agents.members.pauseNamed' : 'agents.members.resumeNamed', { name: row.member.name })"
              :aria-pressed="!row.member.enabled"
              :data-member-id="row.member.id"
              data-testid="agents-member-toggle"
              @click="emit('toggleMember', row.member.id)"
            >
              <v-icon :icon="row.member.enabled ? mdiPause : mdiPlay" size="15" aria-hidden="true" />
              {{ t(row.member.enabled ? 'agents.members.pause' : 'agents.members.resume') }}
            </button>
            <button
              type="button"
              class="agents-button quiet"
              :disabled="!row.active"
              :aria-label="t('agents.members.stopNamed', { name: row.member.name })"
              :data-member-id="row.member.id"
              data-testid="agents-member-stop"
              @click="emit('stopMember', row.member.id)"
            >
              <v-icon :icon="mdiStop" size="15" aria-hidden="true" />
              {{ t('agents.members.stop') }}
            </button>
          </div>
        </article>
      </div>
      <p class="panel-hint">{{ t('agents.members.pausedHint') }}</p>
    </section>

    <section class="approval-section" :aria-label="t('agents.approvals.title')">
      <div class="section-heading">
        <h3>{{ t('agents.approvals.title') }}</h3>
        <span v-if="approvals.length" class="agents-badge is-warning">{{ t('agents.approvals.count', { count: approvals.length }) }}</span>
      </div>
      <p v-if="!approvals.length" class="panel-hint" data-testid="agents-no-approvals">{{ t('agents.approvals.none') }}</p>
      <div v-else class="approval-list">
        <article
          v-for="approval in approvals"
          :key="approval.id"
          class="approval-card"
          :data-approval-id="approval.id"
          data-testid="agents-approval"
        >
          <h4>
            <v-icon :icon="mdiShieldCheckOutline" size="16" aria-hidden="true" />
            {{ t('agents.approvals.request', { name: approval.memberName, tool: approval.toolName }) }}
          </h4>
          <details open class="approval-details">
            <summary>{{ t('agents.approvals.arguments') }}</summary>
            <pre>{{ formatArguments(approval.args) }}</pre>
          </details>
          <p class="panel-hint">{{ t('agents.approvals.hint') }}</p>
          <div class="approval-actions">
            <button
              type="button"
              class="agents-button approval-allow"
              :aria-label="t('agents.approvals.allowNamed', { name: approval.memberName })"
              :data-approval-id="approval.id"
              data-testid="agents-approval-allow"
              @click="emit('approval', approval.id, true)"
            >{{ t('agents.approvals.allowOnce') }}</button>
            <button
              type="button"
              class="agents-button"
              :aria-label="t('agents.approvals.denyNamed', { name: approval.memberName })"
              :data-approval-id="approval.id"
              data-testid="agents-approval-deny"
              @click="emit('approval', approval.id, false)"
            >{{ t('agents.approvals.deny') }}</button>
          </div>
        </article>
      </div>
    </section>

    <div class="panel-policy">
      <p><v-icon :icon="mdiShieldCheckOutline" size="15" aria-hidden="true" />{{ t(`agents.policy.${group.executionMode}`) }}</p>
      <p>{{ t('agents.policy.noLoop') }}</p>
      <p>{{ t('agents.policy.credentials') }}</p>
    </div>
  </div>
</template>

<script setup lang="ts">
import {
  mdiAlertCircleOutline,
  mdiPause,
  mdiPlay,
  mdiRobotOutline,
  mdiShieldCheckOutline,
  mdiStop,
} from '@mdi/js'
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import type { AgentApproval, AgentEngineOption, AgentGroup, AgentGroupRun } from '@/types/agents'
import { engineStatusKey, engineSupportsMode, isActiveRun, runStatusClass } from './agent-ui'

const props = defineProps<{
  group: AgentGroup
  engines: AgentEngineOption[]
  approvals: AgentApproval[]
  activeRuns: AgentGroupRun[]
  busyMemberIds: string[]
  saving: boolean
  scanning: boolean
}>()

const emit = defineEmits<{
  toggleMember: [memberId: string]
  stopMember: [memberId: string]
  approval: [approvalId: string, allow: boolean]
}>()

const { t } = useI18n()
const memberRows = computed(() => {
  const latestRuns = new Map<string, AgentGroupRun>()
  for (const run of props.group.runs) latestRuns.set(run.memberId, run)
  for (const run of props.activeRuns) {
    if (isActiveRun(run)) latestRuns.set(run.memberId, run)
  }
  const engines = new Map(props.engines.map(engine => [engine.id, engine]))
  return props.group.members.map(member => {
    const engine = engines.get(member.engineId)
    const run = latestRuns.get(member.id)
    return {
      member,
      engine,
      run,
      active: isActiveRun(run),
      available: engineSupportsMode(engine, props.group.executionMode),
    }
  })
})

function formatArguments(args: unknown): string {
  if (typeof args === 'string') return args
  try {
    return JSON.stringify(args, null, 2) ?? ''
  } catch {
    return String(args)
  }
}
</script>

<style scoped>
.members-content { display: flex; flex-direction: column; gap: 24px; padding: 20px 16px; min-width: 0; }
.section-heading { display: flex; flex-wrap: wrap; align-items: baseline; justify-content: space-between; gap: 5px 8px; margin-bottom: 14px; }
.section-heading h3 { margin: 0; font-size: 12px; font-weight: 650; }
.section-count { color: var(--text-muted); font-size: 11px; }
.member-list { display: flex; flex-direction: column; gap: 18px; }
.member { min-width: 0; }
.member + .member { padding-top: 16px; border-top: 1px solid var(--border); }
.member-identity { display: flex; align-items: center; gap: 9px; }
.member-avatar { display: flex; align-items: center; justify-content: center; flex-shrink: 0; width: 32px; height: 32px; border-radius: var(--radius-control); border: 1px solid var(--border); background: var(--bg); color: var(--text-muted); }
.member-name { min-width: 0; }
.member-name h4 { margin: 0; font-size: 12px; font-weight: 600; overflow-wrap: anywhere; }
.member-name p { margin: 1px 0 0; color: var(--text-muted); font-size: 10px; overflow-wrap: anywhere; }
.member-role { margin: 8px 0 7px; color: var(--text-muted); font-size: 12px; overflow-wrap: anywhere; }
.member-status { display: flex; flex-wrap: wrap; align-items: center; gap: 5px; }
.member-unavailable { display: flex; align-items: baseline; gap: 4px; margin: 7px 0 0; color: var(--warning); font-size: 11px; overflow-wrap: anywhere; }
.member-error { margin: 8px 0 0; color: var(--danger); white-space: pre-wrap; overflow-wrap: anywhere; font-size: 11px; max-height: 160px; overflow-y: auto; }
.member-actions { display: flex; flex-wrap: wrap; gap: 4px; margin-top: 7px; }
.member-actions .agents-button { min-height: 30px; padding: 4px 8px; font-size: 11px; }
.member-tools { margin-top: 9px; font-size: 11px; color: var(--text-muted); }
.member-tools summary, .approval-details summary { cursor: pointer; overflow-wrap: anywhere; }
.member-tools ul { padding: 0; margin: 8px 0 0; list-style: none; }
.member-tools li { padding: 6px 0; border-top: 1px solid var(--border); }
.member-tools li div { display: flex; flex-wrap: wrap; gap: 3px 8px; justify-content: space-between; }
.member-tools strong { font-weight: 500; color: var(--text); overflow-wrap: anywhere; }
.member-tools li p { margin: 3px 0 0; white-space: pre-wrap; overflow-wrap: anywhere; max-height: 160px; overflow-y: auto; }
.panel-hint { margin: 10px 0 0; font-size: 11px; line-height: 1.7; color: var(--text-muted); overflow-wrap: anywhere; }
.approval-section { border-top: 1px solid var(--border); padding-top: 20px; }
.approval-list { display: flex; flex-direction: column; gap: 12px; }
.approval-card { padding: 12px; border: 1px solid rgb(var(--warning-rgb) / 0.35); border-radius: var(--radius-control); background: rgb(var(--warning-rgb) / 0.04); min-width: 0; }
.approval-card h4 { margin: 0 0 10px; color: var(--text); font-weight: 600; font-size: 12px; line-height: 1.6; overflow-wrap: anywhere; }
.approval-card h4 .v-icon { color: var(--warning); margin-right: 3px; vertical-align: text-bottom; }
.approval-details { color: var(--text-muted); font-size: 11px; }
.approval-details pre { margin: 7px 0 0; padding: 9px; border-radius: 5px; background: var(--bg); border: 1px solid var(--border); color: var(--text); font-family: var(--jedi-font-mono); font-size: 11px; white-space: pre-wrap; overflow-wrap: anywhere; max-height: 220px; overflow-y: auto; }
.approval-actions { display: flex; flex-wrap: wrap; gap: 6px; margin-top: 12px; }
.approval-actions .agents-button { flex: 1 1 auto; padding: 6px 8px; min-height: 34px; font-size: 11px; }
.approval-actions .approval-allow { color: var(--warning); border-color: rgb(var(--warning-rgb) / 0.5); }
.panel-policy { margin-top: auto; padding-top: 18px; border-top: 1px solid var(--border); }
.panel-policy p { margin: 0 0 9px; color: var(--text-muted); font-size: 11px; line-height: 1.75; overflow-wrap: anywhere; }
.panel-policy p:first-child { color: var(--text); }
.panel-policy .v-icon { margin-right: 5px; vertical-align: text-bottom; color: var(--accent); }
.panel-policy p:last-child { margin-bottom: 0; }
</style>
