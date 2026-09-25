<template>
  <Transition name="skl-fade">
    <div v-if="modelValue" class="skl-overlay" @click.self="close">
      <div class="skl-dialog">
        <div class="skl-head">
          <h3>{{ $t('skills.title') }}</h3>
          <div class="skl-head-right">
            <span class="skl-count">{{ $t('skills.enabledCount', { count: skillsStore.enabledCount }) }}</span>
            <button class="skl-close" :aria-label="$t('common.close')" @click="close">✕</button>
          </div>
        </div>

        <p class="skl-hint">{{ $t('skills.hint') }}</p>

        <div class="skl-toolbar">
          <button class="skl-mini" :class="{ on: sortByUsage }" @click="sortByUsage = !sortByUsage">
            ⇅ {{ $t('skills.stats.sortByUsage') }}
          </button>
          <button class="skl-mini danger" @click="handleClearStats">
            {{ armedId === CLEAR_ID ? $t('skills.stats.confirmClear') : $t('skills.stats.clear') }}
          </button>
        </div>

        <div v-if="skillsStore.error" class="skl-error">{{ skillsStore.error }}</div>

        <SkillEditor
          v-if="editorOpen"
          :editing="editingDef"
          @saved="closeEditor"
          @cancel="closeEditor"
        />

        <div class="skl-list">
          <div v-for="group in groups" :key="group.key" class="skl-group">
            <div class="skl-group-head">
              <span class="skl-group-label">{{ group.label }}</span>
              <span class="skl-group-count">{{ group.skills.length }}</span>
              <button
                v-if="group.key === 'custom' && !editorOpen"
                class="skl-btn slim"
                @click="startCreate"
              >
                {{ $t('skills.newSkill') }}
              </button>
            </div>
            <div v-if="group.skills.length === 0" class="skl-empty">
              {{ group.key === 'custom' ? $t('skills.emptyCustom') : $t('skills.emptyMcp') }}
            </div>
            <SkillRow
              v-for="skill in group.skills"
              :key="skill.id"
              :skill="skill"
              :expanded="expandedId === skill.id"
              @expand="toggleExpand(skill.id)"
              @edit="startEdit"
              @remove="handleRemove"
            />
          </div>
        </div>
      </div>
    </div>
  </Transition>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import SkillEditor from '@/components/skills/SkillEditor.vue'
import SkillRow from '@/components/skills/SkillRow.vue'
import { useTwoStepConfirm } from '@/composables/useTwoStepConfirm'
import type { CustomSkillDef, Skill, SkillSource } from '@/skills/types'
import { useSkillStatsStore } from '@/stores/skillStats'
import { useSkillsStore } from '@/stores/skills'

defineProps<{ modelValue: boolean }>()
const emit = defineEmits<{ 'update:modelValue': [boolean] }>()

const { t } = useI18n()
const skillsStore = useSkillsStore()
const statsStore = useSkillStatsStore()
const { armedId, arm } = useTwoStepConfirm()

const CLEAR_ID = 'clear-stats'
const expandedId = ref<string | null>(null)
const editorOpen = ref(false)
const editingDef = ref<CustomSkillDef | null>(null)
const sortByUsage = ref(false)

function groupSkills(source: SkillSource) {
  const list = [...skillsStore.skillsBySource(source)]
  if (sortByUsage.value) {
    list.sort(
      (a, b) => (statsStore.statsFor(b.id)?.calls ?? 0) - (statsStore.statsFor(a.id)?.calls ?? 0)
    )
  }
  return list
}

const groups = computed(() => [
  {
    key: 'builtin',
    label: t('skills.groupBuiltin'),
    skills: groupSkills('builtin'),
  },
  { key: 'custom', label: t('skills.groupCustom'), skills: groupSkills('custom') },
  { key: 'mcp', label: t('skills.groupMcp'), skills: groupSkills('mcp') },
])

function close(): void {
  emit('update:modelValue', false)
}

function toggleExpand(id: string): void {
  expandedId.value = expandedId.value === id ? null : id
}

function startCreate(): void {
  editingDef.value = null
  editorOpen.value = true
}

function startEdit(skill: Skill): void {
  const def = skillsStore.customDefs.find(d => d.id === skill.id)
  if (!def) return
  editingDef.value = def
  editorOpen.value = true
}

function closeEditor(): void {
  editorOpen.value = false
  editingDef.value = null
}

async function handleRemove(id: string): Promise<void> {
  await skillsStore.removeCustom(id)
  if (editingDef.value?.id === id) closeEditor()
}

async function handleClearStats(): Promise<void> {
  // 两段式确认：第一次点击进入确认态（3 秒自动复位），再点一次才真正清零
  if (!arm(CLEAR_ID)) return
  await statsStore.clearAll()
}
</script>

<style scoped>
.skl-overlay {
  position: fixed;
  inset: 0;
  z-index: var(--z-panel);
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgb(var(--ink-rgb) / 0.4);
  backdrop-filter: blur(2px);
}

.skl-dialog {
  width: min(640px, calc(100vw - 32px));
  max-height: 82vh;
  overflow: auto;
  border-radius: 14px;
  padding: 20px 22px;
  background: var(--bg-elevated);
  border: 1px solid var(--border);
  box-shadow: var(--shadow-lg);
  color: var(--text);
}

.skl-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 6px;
}

.skl-head h3 {
  margin: 0;
  font-size: 16px;
}

.skl-head-right {
  display: flex;
  align-items: center;
  gap: 10px;
}

.skl-count {
  font-size: 11px;
  opacity: 0.66;
}

.skl-close {
  background: none;
  border: none;
  color: var(--text-muted);
  cursor: pointer;
  font-size: 15px;
}

.skl-hint {
  margin: 0 0 12px;
  font-size: 12px;
  opacity: 0.66;
  line-height: 1.5;
}

.skl-error {
  margin-bottom: 10px;
  padding: 8px 10px;
  border-radius: 8px;
  font-size: 12px;
  background: rgb(var(--danger-rgb) / 0.14);
  color: var(--danger);
}

.skl-list {
  display: flex;
  flex-direction: column;
  gap: 14px;
}

.skl-group-head {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 8px;
}

.skl-group-label {
  font-size: 12px;
  font-weight: 600;
  opacity: 0.72;
}

.skl-group-count {
  font-size: 10px;
  padding: 1px 6px;
  border-radius: 8px;
  background: rgb(var(--text-rgb) / 0.08);
  opacity: 0.7;
}

.skl-group-head .skl-btn {
  margin-left: auto;
}

.skl-empty {
  padding: 12px;
  text-align: center;
  font-size: 12px;
  opacity: 0.5;
}

.skl-toolbar {
  display: flex;
  gap: 8px;
  margin-bottom: 12px;
}

.skl-mini {
  cursor: pointer;
  border: 1px solid var(--border);
  border-radius: 6px;
  padding: 3px 8px;
  font-size: 11px;
  background: rgb(var(--text-rgb) / 0.05);
  color: var(--text);
}

.skl-mini.on {
  border-color: rgb(var(--accent-rgb) / 0.5);
  color: var(--accent);
}

.skl-mini.danger {
  background: rgb(var(--danger-rgb) / 0.16);
  color: var(--danger);
}

.skl-btn {
  cursor: pointer;
  border: none;
  border-radius: 7px;
  padding: 6px 12px;
  font-size: 12px;
  font-weight: 600;
  background: rgb(var(--text-rgb) / 0.08);
  color: var(--text);
  transition: filter 0.15s ease;
}

.skl-btn:hover:not(:disabled) {
  filter: brightness(1.15);
}

.skl-btn.slim {
  padding: 3px 10px;
}

.skl-fade-enter-active,
.skl-fade-leave-active {
  transition: opacity 0.18s ease;
}

.skl-fade-enter-from,
.skl-fade-leave-to {
  opacity: 0;
}
</style>
