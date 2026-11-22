<template>
  <v-dialog
    :model-value="modelValue"
    :persistent="submitting"
    max-width="720"
    aria-labelledby="agents-create-title"
    aria-describedby="agents-create-subtitle"
    @update:model-value="emit('update:modelValue', $event)"
    @after-enter="nameInput?.focus()"
  >
    <v-card class="agents-dialog" data-testid="agents-create-dialog">
      <form ref="formElement" class="create-form" novalidate @submit.prevent="submit">
        <header class="dialog-header">
          <div>
            <h2 id="agents-create-title">{{ t('agents.create.title') }}</h2>
            <p id="agents-create-subtitle">{{ t('agents.create.subtitle') }}</p>
          </div>
          <button
            type="button"
            class="agents-icon-button quiet"
            :aria-label="t('common.close')"
            :disabled="submitting"
            data-testid="agents-create-close"
            @click="emit('update:modelValue', false)"
          >
            <v-icon :icon="mdiClose" size="20" aria-hidden="true" />
          </button>
        </header>

        <div class="dialog-body">
          <div v-if="error" class="agents-error" role="alert" data-testid="agents-create-error">
            <v-icon :icon="mdiAlertCircleOutline" size="18" aria-hidden="true" />
            <div class="agents-error-content">
              <strong>{{ t('agents.errorTitle') }}</strong>
              <p>{{ error }}</p>
            </div>
          </div>
          <div v-if="Object.keys(errors).length" class="validation-summary" role="alert">
            {{ t('agents.create.validationTitle') }}
          </div>

          <div class="form-field">
            <label class="agents-field-label" for="agents-group-name">{{ t('agents.create.name') }}</label>
            <input
              id="agents-group-name"
              ref="nameInput"
              v-model="name"
              class="agents-input"
              type="text"
              autocomplete="off"
              maxlength="80"
              required
              :placeholder="t('agents.create.namePlaceholder')"
              :disabled="submitting"
              :aria-invalid="!!errors.name"
              :aria-describedby="errors.name ? 'agents-name-error' : undefined"
              data-testid="agents-create-name"
              @input="delete errors.name"
            />
            <p v-if="errors.name" id="agents-name-error" class="agents-field-error">{{ errors.name }}</p>
          </div>

          <div class="form-field">
            <label class="agents-field-label" for="agents-project-path">{{ t('agents.create.projectPath') }}</label>
            <input
              id="agents-project-path"
              v-model="projectPath"
              class="agents-input project-input"
              type="text"
              autocomplete="off"
              spellcheck="false"
              maxlength="2048"
              required
              :placeholder="t('agents.create.projectPlaceholder')"
              :disabled="submitting"
              :aria-invalid="!!errors.path"
              :aria-describedby="errors.path ? 'agents-path-error agents-path-hint' : 'agents-path-hint'"
              data-testid="agents-create-path"
              @input="delete errors.path"
            />
            <p id="agents-path-hint" class="agents-field-hint">{{ t('agents.create.projectHint') }}</p>
            <p v-if="errors.path" id="agents-path-error" class="agents-field-error">{{ errors.path }}</p>
          </div>

          <fieldset class="mode-fieldset" :disabled="submitting">
            <legend class="agents-field-label">{{ t('agents.create.mode') }}</legend>
            <div class="mode-options">
              <label
                v-for="mode in modes"
                :key="mode"
                class="mode-choice"
                :class="{ selected: executionMode === mode, unavailable: !modeAvailable(mode) }"
              >
                <input
                  v-model="executionMode"
                  type="radio"
                  name="agents-execution-mode"
                  :value="mode"
                  :disabled="!modeAvailable(mode)"
                  :aria-describedby="mode === 'approval' && !modeAvailable(mode) ? 'agents-approval-unavailable' : undefined"
                  :data-testid="`agents-mode-${mode}`"
                />
                <span>
                  <strong>{{ t(`agents.mode.${mode}`) }}</strong>
                  <span>{{ t(`agents.create.${mode}Hint`) }}</span>
                </span>
              </label>
            </div>
            <p v-if="!modeAvailable('approval')" id="agents-approval-unavailable" class="agents-field-hint" data-testid="agents-approval-unavailable">{{ t('agents.create.approvalUnavailable') }}</p>
            <p class="agents-field-hint">{{ t(`agents.policy.${executionMode}`) }}</p>
          </fieldset>

          <fieldset class="members-fieldset" :disabled="submitting || scanning">
            <legend class="agents-field-label">{{ t('agents.create.members') }}</legend>
            <div class="members-heading">
              <p class="agents-field-hint">{{ t('agents.create.membersHint') }}</p>
              <button
                type="button"
                class="agents-button quiet"
                :disabled="scanning || submitting"
                data-testid="agents-create-scan"
                @click="emit('scan')"
              >
                <v-progress-circular v-if="scanning" indeterminate size="15" width="2" aria-hidden="true" />
                <v-icon v-else :icon="mdiRefresh" size="16" aria-hidden="true" />
                {{ t(scanning ? 'agents.scanning' : 'agents.scan') }}
              </button>
            </div>
            <p v-if="selectionAdjusted" class="agents-field-hint" role="status">{{ t('agents.create.selectionAdjusted') }}</p>
            <div v-if="scanning" class="scan-notice" role="status">{{ t('agents.scanning') }}</div>
            <p v-else-if="!engines.length" class="empty-engines">{{ t('agents.noEnginesDetected') }}</p>
            <div class="engine-list">
              <div
                v-for="engine in engines"
                :key="engine.id"
                class="engine-choice"
                :class="{
                  selected: selectedIds.includes(engine.id),
                  unavailable: !engineSupportsMode(engine, executionMode),
                }"
                :data-engine-id="engine.id"
                data-testid="agents-engine-option"
              >
                <label class="engine-label">
                  <input
                    type="checkbox"
                    :checked="selectedIds.includes(engine.id)"
                    :disabled="!engineSupportsMode(engine, executionMode)"
                    :value="engine.id"
                    data-testid="agents-engine-checkbox"
                    @change="toggleEngine(engine.id, ($event.target as HTMLInputElement).checked)"
                  />
                  <v-icon :icon="mdiRobotOutline" size="20" aria-hidden="true" />
                  <span class="engine-identity">
                    <strong>{{ engine.name }}</strong>
                    <span>{{ engine.location || engine.id }}</span>
                  </span>
                  <span
                    class="agents-badge engine-status"
                    :class="{ 'is-active': engineSupportsMode(engine, executionMode) }"
                    :title="t('agents.engine.statusDetail', { status: engine.status })"
                  >{{ t(engineStatusKey(engine, executionMode)) }}</span>
                </label>
                <div v-if="selectedIds.includes(engine.id)" class="engine-role">
                  <label class="agents-field-label" :for="roleInputId(engine.id)">
                    {{ t('agents.create.role', { name: engine.name }) }}
                  </label>
                  <input
                    :id="roleInputId(engine.id)"
                    v-model="roles[engine.id]"
                    class="agents-input"
                    type="text"
                    maxlength="160"
                    required
                    :placeholder="t('agents.create.rolePlaceholder')"
                    :aria-invalid="!!errors.roles"
                    :aria-describedby="errors.roles ? 'agents-roles-error' : undefined"
                    data-testid="agents-engine-role"
                    @input="delete errors.roles"
                  />
                </div>
              </div>
            </div>
            <p v-if="errors.members" class="agents-field-error">{{ errors.members }}</p>
            <p v-if="errors.roles" id="agents-roles-error" class="agents-field-error">{{ errors.roles }}</p>
            <p v-if="!scanning && !availableEngines.length" class="agents-field-hint no-supported-engines">
              {{ t('agents.create.noMembersForMode') }}
            </p>
            <p class="agents-field-hint">{{ t('agents.create.cliHint') }}</p>
          </fieldset>
          <p class="agents-field-hint credentials-hint">{{ t('agents.policy.credentials') }}</p>
        </div>

        <footer class="dialog-footer">
          <button
            type="button"
            class="agents-button"
            :disabled="submitting"
            @click="emit('update:modelValue', false)"
          >{{ t('common.cancel') }}</button>
          <button
            type="submit"
            class="agents-button primary"
            :disabled="submitting || scanning"
            data-testid="agents-create-submit"
          >
            <v-progress-circular v-if="submitting" indeterminate size="15" width="2" aria-hidden="true" />
            <v-icon v-else :icon="mdiPlus" size="17" aria-hidden="true" />
            {{ t(submitting ? 'agents.create.creating' : 'agents.create.submit') }}
          </button>
        </footer>
      </form>
    </v-card>
  </v-dialog>
</template>

<script setup lang="ts">
import { mdiAlertCircleOutline, mdiClose, mdiPlus, mdiRefresh, mdiRobotOutline } from '@mdi/js'
import { computed, nextTick, reactive, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import type { AgentEngineOption, AgentExecutionMode, CreateAgentGroupInput } from '@/types/agents'
import { isAbsoluteProjectPath } from '@/utils/path'
import { engineStatusKey, engineSupportsMode } from './agent-ui'

const props = defineProps<{
  modelValue: boolean
  engines: AgentEngineOption[]
  scanning: boolean
  submitting: boolean
  error: string | null
}>()

const emit = defineEmits<{
  'update:modelValue': [value: boolean]
  scan: []
  create: [input: CreateAgentGroupInput]
}>()

const { t } = useI18n()
const modes: AgentExecutionMode[] = ['discussion', 'approval']
const name = ref('')
const projectPath = ref('')
const executionMode = ref<AgentExecutionMode>('discussion')
const selectedIds = ref<string[]>([])
const roles = reactive<Record<string, string>>({})
const errors = reactive<Record<string, string>>({})
const selectionAdjusted = ref(false)
const nameInput = ref<HTMLInputElement | null>(null)
const formElement = ref<HTMLFormElement | null>(null)
const availableEngines = computed(() =>
  props.engines.filter(engine => engineSupportsMode(engine, executionMode.value))
)

watch(
  () => props.modelValue,
  open => {
    if (!open) return
    name.value = ''
    projectPath.value = ''
    executionMode.value = 'discussion'
    selectedIds.value = []
    selectionAdjusted.value = false
    for (const key of Object.keys(errors)) delete errors[key]
    for (const key of Object.keys(roles)) delete roles[key]
  }
)

watch(
  () => [executionMode.value, props.engines, props.scanning],
  () => {
    if (props.scanning) return
    if (!modeAvailable(executionMode.value)) executionMode.value = 'discussion'
    const allowed = new Set(availableEngines.value.map(engine => engine.id))
    const remaining = selectedIds.value.filter(id => allowed.has(id))
    if (remaining.length !== selectedIds.value.length) {
      selectedIds.value = remaining
      selectionAdjusted.value = true
    }
    delete errors.members
  },
  { deep: true }
)

function roleInputId(engineId: string): string {
  return `agents-role-${encodeURIComponent(engineId)}`
}

function toggleEngine(engineId: string, checked: boolean): void {
  selectionAdjusted.value = false
  delete errors.members
  if (checked) {
    if (!selectedIds.value.includes(engineId)) selectedIds.value.push(engineId)
    if (!roles[engineId]) roles[engineId] = t('agents.create.defaultRole')
  } else {
    selectedIds.value = selectedIds.value.filter(id => id !== engineId)
  }
}

function modeAvailable(mode: AgentExecutionMode): boolean {
  return mode === 'discussion' || props.engines.some(engine => engineSupportsMode(engine, mode))
}

async function submit(): Promise<void> {
  if (props.submitting || props.scanning) return
  for (const key of Object.keys(errors)) delete errors[key]
  const groupName = name.value.trim()
  const path = projectPath.value.trim()
  if (!groupName) errors.name = t('agents.create.nameRequired')
  else if (groupName.length > 80) errors.name = t('agents.create.nameTooLong')
  if (!path) errors.path = t('agents.create.pathRequired')
  else if (!isAbsoluteProjectPath(path)) errors.path = t('agents.create.pathAbsolute')
  const members = selectedIds.value
    .filter(id => availableEngines.value.some(engine => engine.id === id))
    .map(engineId => ({ engineId, role: (roles[engineId] || '').trim() }))
  if (!members.length) errors.members = t('agents.create.membersRequired')
  if (members.some(member => !member.role)) errors.roles = t('agents.create.roleRequired')
  else if (members.some(member => member.role.length > 160))
    errors.roles = t('agents.create.roleTooLong')
  if (Object.keys(errors).length) {
    await nextTick()
    const invalid = formElement.value?.querySelector<HTMLInputElement>('[aria-invalid="true"]')
    if (invalid) invalid.focus()
    else
      formElement.value
        ?.querySelector<HTMLInputElement>('input[type="checkbox"]:not(:disabled)')
        ?.focus()
    return
  }
  emit('create', {
    name: groupName,
    projectPath: path,
    executionMode: executionMode.value,
    members,
  })
}
</script>

<style scoped>
.create-form { display: flex; flex-direction: column; min-height: 0; max-height: calc(100dvh - 64px); }
.dialog-header { display: flex; align-items: flex-start; gap: 20px; justify-content: space-between; padding: 22px 24px 18px; border-bottom: 1px solid var(--border); flex-shrink: 0; }
.dialog-header h2 { margin: 0; font-size: 18px; font-weight: 650; line-height: 1.45; }
.dialog-header p { margin: 4px 0 0; color: var(--text-muted); font-size: 12px; }
.dialog-body { padding: 22px 24px; overflow-y: auto; min-height: 0; display: flex; flex-direction: column; gap: 20px; scroll-padding: 12px; }
.form-field { min-width: 0; }
.project-input { font-family: var(--jedi-font-mono); font-size: 12px; }
.mode-fieldset, .members-fieldset { border: 0; min-width: 0; padding: 0; margin: 0; }
.mode-options { display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 10px; }
.mode-choice { display: flex; align-items: flex-start; gap: 9px; border: 1px solid var(--border-strong); border-radius: var(--radius-control); padding: 12px; cursor: pointer; background: var(--bg); }
.mode-choice.selected { border-color: var(--accent); background: rgb(var(--accent-rgb) / 0.05); }
.mode-choice.unavailable { cursor: default; background: var(--bg-elevated); color: var(--text-muted); }
.mode-choice.unavailable strong { color: var(--text-muted); }
.mode-choice input { accent-color: var(--accent); margin-top: 4px; flex-shrink: 0; }
.mode-choice strong { display: block; font-size: 12px; font-weight: 600; color: var(--text); }
.mode-choice span span { display: block; color: var(--text-muted); font-size: 11px; margin-top: 4px; }
.members-heading { display: flex; align-items: flex-start; justify-content: space-between; gap: 10px; margin-bottom: 10px; }
.members-heading p { margin-top: 0; }
.members-heading .agents-button { white-space: normal; max-width: 50%; min-height: 30px; padding-top: 3px; padding-bottom: 3px; }
.engine-list { display: flex; flex-direction: column; gap: 8px; }
.engine-choice { border: 1px solid var(--border); border-radius: var(--radius-control); background: var(--bg); min-width: 0; }
.engine-choice.selected { border-color: rgb(var(--accent-rgb) / 0.55); }
.engine-label { display: flex; flex-wrap: wrap; align-items: center; gap: 10px; min-height: 64px; padding: 11px 12px; cursor: pointer; }
.engine-label input { accent-color: var(--accent); width: 16px; height: 16px; flex-shrink: 0; }
.engine-label > .v-icon { color: var(--text-muted); flex-shrink: 0; }
.engine-choice.unavailable .engine-label { cursor: default; }
.engine-identity { flex: 1; min-width: 90px; }
.engine-identity strong { display: block; font-size: 12px; font-weight: 600; }
.engine-identity > span { display: block; font-size: 11px; color: var(--text-muted); overflow-wrap: anywhere; }
.engine-status { max-width: 180px; }
.engine-role { padding: 0 12px 13px 48px; }
.engine-role .agents-field-label { font-size: 11px; color: var(--text-muted); }
.engine-role input { font-size: 12px; }
.validation-summary { color: var(--danger); font-size: 12px; font-weight: 600; }
.scan-notice, .empty-engines { padding: 12px; color: var(--text-muted); background: var(--bg-elevated); border-radius: var(--radius-control); font-size: 12px; }
.no-supported-engines { color: var(--warning); }
.credentials-hint { padding-top: 14px; margin: 0; border-top: 1px solid var(--border); }
.dialog-footer { display: flex; justify-content: flex-end; gap: 9px; padding: 16px 24px; background: var(--bg-elevated); border-top: 1px solid var(--border); flex-shrink: 0; }
@media (max-width: 540px) {
  .dialog-header { padding: 18px 16px; }
  .dialog-body { padding: 18px 16px; }
  .dialog-footer { padding: 14px 16px; }
  .mode-options { grid-template-columns: minmax(0, 1fr); }
  .engine-label { gap: 8px; }
  .engine-status { margin-left: 26px; max-width: calc(100% - 26px); }
}
</style>
