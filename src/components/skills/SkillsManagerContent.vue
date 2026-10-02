<template>
  <div class="mgr-body">
    <div v-if="showSearch" class="mgr-toolbar">
      <input
        v-model="search"
        class="mgr-search"
        type="search"
        :placeholder="$t('skills.page.searchPlaceholder')"
        :aria-label="$t('skills.page.searchPlaceholder')"
      />
    </div>
    <div class="mgr-toolbar">
      <button
        class="mgr-mini"
        :class="{ on: sortByUsage }"
        :aria-pressed="sortByUsage"
        @click="sortByUsage = !sortByUsage"
      >
        ⇅ {{ $t('skills.stats.sortByUsage') }}
      </button>
      <button class="mgr-mini danger" @click="handleClearStats">
        {{ armedId === CLEAR_ID ? $t('skills.stats.confirmClear') : $t('skills.stats.clear') }}
      </button>
    </div>

    <div v-if="skillsStore.error || editError" class="mgr-error" role="alert">
      {{ skillsStore.error || editError }}
    </div>

    <SkillEditor v-if="editorOpen" :editing="editingDef" @saved="closeEditor" @cancel="closeEditor" />

    <!-- 工具区：内置 + MCP 桥接，可执行能力 -->
    <div class="mgr-section">
      <div class="mgr-group-head">
        <span class="mgr-group-label">{{ $t('skills.toolsSection') }}</span>
        <span class="mgr-group-hint">{{ $t('skills.toolsSectionHint') }}</span>
      </div>
      <div v-for="group in toolGroups" :key="group.key" class="mgr-group">
        <div class="mgr-subgroup-head">
          <span class="mgr-subgroup-label">{{ group.label }}</span>
          <span class="mgr-group-count">{{ group.items.length }}</span>
        </div>
        <div v-if="group.items.length === 0" class="mgr-empty">
          {{ group.key === 'mcp' ? $t('skills.emptyMcp') : $t('skills.emptyTools') }}
        </div>
        <ToolRow
          v-for="tool in group.items"
          :key="tool.id"
          :tool="tool"
          :expanded="expandedId === tool.id"
          @expand="toggleExpand(tool.id)"
        />
      </div>
    </div>

    <!-- 技能区：SKILL.md 指令包 -->
    <div class="mgr-section">
      <div class="mgr-group-head">
        <span class="mgr-group-label">{{ $t('skills.skillsSection') }}</span>
        <span class="mgr-group-hint">{{ $t('skills.skillsSectionHint') }}</span>
        <button v-if="!editorOpen" class="mgr-btn slim" @click="startCreate">
          {{ $t('skills.newSkill') }}
        </button>
      </div>
      <div v-for="group in skillGroups" :key="group.key" class="mgr-group">
        <div class="mgr-subgroup-head">
          <span class="mgr-subgroup-label">{{ group.label }}</span>
          <span class="mgr-group-count">{{ group.items.length }}</span>
        </div>
        <div v-if="group.items.length === 0" class="mgr-empty">{{ $t('skills.emptyUser') }}</div>
        <SkillRow
          v-for="skill in group.items"
          :key="skill.name"
          :skill="skill"
          :expanded="expandedId === skill.name"
          @expand="toggleExpand(skill.name)"
          @edit="startEdit"
          @remove="handleRemove"
        />
      </div>
    </div>

    <div v-if="nothingMatches" class="mgr-empty">{{ $t('skills.page.empty') }}</div>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import type { Tool } from '@/agent/tools/types'
import { readSkill } from '@/api/skills'
import { useTwoStepConfirm } from '@/composables/useTwoStepConfirm'
import type { SkillDetail, SkillManifest } from '@/skills/types'
import { useSkillsStore } from '@/stores/skills'
import { useToolStatsStore } from '@/stores/toolStats'
import { useToolsStore } from '@/stores/tools'
import SkillEditor from './SkillEditor.vue'
import SkillRow from './SkillRow.vue'
import ToolRow from './ToolRow.vue'

defineProps<{ showSearch?: boolean }>()
const emit = defineEmits<{ 'editor-change': [open: boolean] }>()

const { t } = useI18n()
const skillsStore = useSkillsStore()
const toolsStore = useToolsStore()
const statsStore = useToolStatsStore()
const { armedId, arm } = useTwoStepConfirm()

const CLEAR_ID = 'clear-stats'
const search = ref('')
const sortByUsage = ref(false)
const expandedId = ref<string | null>(null)
const editorOpen = ref(false)
const editingDef = ref<SkillDetail | null>(null)
const editError = ref<string | null>(null)

// 宿主弹窗据此在编辑器打开时禁用「点击遮罩即关闭」，避免丢失未保存内容
watch(editorOpen, open => emit('editor-change', open), { immediate: true })

function matchSearch(text: string): boolean {
  const q = search.value.trim().toLowerCase()
  if (!q) return true
  return text.toLowerCase().includes(q)
}

const toolGroups = computed(() => {
  // 依赖 flagMap 使开关变化驱动重渲染（registry 本身非响应式）
  void toolsStore.allTools
  const bySource = (source: Tool['source']) =>
    toolsStore.allTools
      .filter(t => t.source === source)
      .filter(t => matchSearch(`${t.name} ${t.description} ${t.id}`))
      .sort((a, b) => {
        if (sortByUsage.value) {
          return (statsStore.statsFor(b.id)?.calls ?? 0) - (statsStore.statsFor(a.id)?.calls ?? 0)
        }
        return a.name.localeCompare(b.name)
      })
  return [
    { key: 'builtin', label: t('skills.groupBuiltin'), items: bySource('builtin') },
    { key: 'mcp', label: t('skills.groupMcp'), items: bySource('mcp') },
  ]
})

const skillGroups = computed(() => {
  void skillsStore.allSkills
  const bySource = (source: SkillManifest['source']) =>
    skillsStore
      .skillsBySource(source)
      .filter(s => matchSearch(`${s.name} ${s.description}`))
      .sort((a, b) => a.name.localeCompare(b.name))
  return [
    { key: 'skills-builtin', label: t('skills.groupSkillsBuiltin'), items: bySource('builtin') },
    { key: 'user', label: t('skills.groupUser'), items: bySource('user') },
  ]
})

const nothingMatches = computed(() => {
  const q = search.value.trim()
  if (!q) return false
  return (
    toolGroups.value.every(g => g.items.length === 0) &&
    skillGroups.value.every(g => g.items.length === 0)
  )
})

function toggleExpand(id: string): void {
  expandedId.value = expandedId.value === id ? null : id
}

function startCreate(): void {
  editingDef.value = null
  editorOpen.value = true
}

async function startEdit(skill: SkillManifest): Promise<void> {
  editError.value = null
  try {
    editingDef.value = await readSkill(skill.name)
    editorOpen.value = true
  } catch (e) {
    console.error('Failed to read skill for edit:', e)
    editError.value = t('skills.editReadFailed')
  }
}

function closeEditor(): void {
  editorOpen.value = false
  editingDef.value = null
}

async function handleRemove(name: string): Promise<void> {
  await skillsStore.removeUserSkill(name)
  if (editingDef.value?.name === name) closeEditor()
}

async function handleClearStats(): Promise<void> {
  // 两段式确认：第一次点击进入确认态（3 秒自动复位），再点一次才真正清零
  if (!arm(CLEAR_ID)) return
  await statsStore.clearAll()
}

onMounted(() => {
  // 页面可能先于聊天页访问：loadConfig/loadStats 均幂等
  skillsStore.loadConfig()
  statsStore.loadStats()
})
</script>

<style scoped>
.mgr-body {
  display: flex;
  flex-direction: column;
  gap: 14px;
}

.mgr-toolbar {
  display: flex;
  gap: 8px;
  align-items: center;
}

.mgr-search {
  flex: 1;
  min-width: 140px;
  padding: 6px 10px;
  border-radius: 7px;
  background: var(--bg-terminal);
  border: 1px solid var(--border);
  color: var(--text);
  font-size: 12px;
  outline: none;
}

.mgr-search:focus {
  border-color: var(--accent);
  box-shadow: 0 0 0 1px rgb(var(--accent-rgb) / 0.3);
}

.mgr-mini {
  cursor: pointer;
  border: 1px solid var(--border);
  border-radius: 6px;
  padding: 3px 8px;
  font-size: 11px;
  background: rgb(var(--text-rgb) / 0.05);
  color: var(--text);
  transition: background 0.15s ease, border-color 0.15s ease;
}

.mgr-mini:hover {
  border-color: var(--border-strong);
  background: rgb(var(--text-rgb) / 0.09);
}

.mgr-mini:active {
  background: rgb(var(--text-rgb) / 0.13);
}

.mgr-mini.on {
  border-color: rgb(var(--accent-rgb) / 0.5);
  color: var(--accent);
}

.mgr-mini.danger {
  background: rgb(var(--danger-rgb) / 0.16);
  color: var(--danger);
}

.mgr-error {
  padding: 8px 10px;
  border-radius: 8px;
  font-size: 12px;
  background: rgb(var(--danger-rgb) / 0.14);
  color: var(--danger);
}

.mgr-section {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.mgr-group {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.mgr-group-head {
  display: flex;
  align-items: baseline;
  gap: 8px;
}

.mgr-group-label {
  font-size: 13px;
  font-weight: 700;
}

.mgr-group-hint {
  font-size: 11px;
  color: var(--text-muted);
}

.mgr-group-head .mgr-btn {
  margin-left: auto;
}

.mgr-subgroup-head {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-top: 2px;
}

.mgr-subgroup-label {
  font-size: 11px;
  font-weight: 600;
  color: var(--text-muted);
}

.mgr-group-count {
  font-size: 10px;
  padding: 1px 6px;
  border-radius: 8px;
  background: rgb(var(--text-rgb) / 0.08);
  color: var(--text-muted);
}

.mgr-empty {
  padding: 12px;
  text-align: center;
  font-size: 12px;
  color: var(--text-muted);
}

.mgr-btn {
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

.mgr-btn:hover:not(:disabled) {
  filter: brightness(1.15);
}

.mgr-btn.slim {
  padding: 3px 10px;
}
</style>
