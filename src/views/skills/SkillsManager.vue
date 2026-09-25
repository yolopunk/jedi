<template>
  <div class="skills-page">
    <!-- Console 工具栏 -->
    <div class="console-header-bar">
      <span class="bar-title">[SKILLS]</span>
      <span class="bar-count">
        {{ filteredSkills.length }}/{{ skillsStore.allSkills.length }} ·
        {{ $t('skills.enabledCount', { count: skillsStore.enabledCount }) }}
      </span>

      <input
        v-model="search"
        class="bar-search"
        type="search"
        :placeholder="$t('skills.page.searchPlaceholder')"
        :aria-label="$t('skills.page.searchPlaceholder')"
      />

      <div class="bar-filters" role="group" :aria-label="$t('skills.page.filterLabel')">
        <button
          v-for="f in FILTERS"
          :key="f"
          class="bar-chip"
          :class="{ on: sourceFilter === f }"
          @click="sourceFilter = f"
        >
          {{ filterLabel(f) }}
        </button>
      </div>

      <button class="bar-btn" :class="{ on: sortByUsage }" @click="sortByUsage = !sortByUsage">
        ⇅ {{ $t('skills.stats.sortByUsage') }}
      </button>
      <button class="bar-btn" @click="startCreate">{{ $t('skills.newSkill') }}</button>
      <button class="bar-btn danger" @click="handleClearStats">
        {{ armedId === CLEAR_ID ? $t('skills.stats.confirmClear') : $t('skills.stats.clear') }}
      </button>
    </div>

    <div v-if="skillsStore.error" class="page-error">{{ skillsStore.error }}</div>

    <!-- 新建/编辑面板 -->
    <SkillEditor
      v-if="editorOpen"
      :editing="editingDef"
      class="page-editor"
      @saved="closeEditor"
      @cancel="closeEditor"
    />

    <!-- 平铺技能列表（筛选 + 搜索，密度优先） -->
    <div class="skills-list">
      <SkillRow
        v-for="skill in filteredSkills"
        :key="skill.id"
        :skill="skill"
        :expanded="expandedId === skill.id"
        @expand="toggleExpand(skill.id)"
        @edit="startEdit"
        @remove="handleRemove"
      />
    </div>

    <div v-if="filteredSkills.length === 0" class="page-empty">{{ $t('skills.page.empty') }}</div>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import SkillEditor from '@/components/skills/SkillEditor.vue'
import SkillRow from '@/components/skills/SkillRow.vue'
import { useTwoStepConfirm } from '@/composables/useTwoStepConfirm'
import type { CustomSkillDef, Skill, SkillSource } from '@/skills/types'
import { useSkillStatsStore } from '@/stores/skillStats'
import { useSkillsStore } from '@/stores/skills'

const { t } = useI18n()
const skillsStore = useSkillsStore()
const statsStore = useSkillStatsStore()
const { armedId, arm } = useTwoStepConfirm()

type SourceFilter = 'all' | SkillSource
const FILTERS: SourceFilter[] = ['all', 'builtin', 'custom', 'mcp']

const CLEAR_ID = 'clear-stats'
const search = ref('')
const sourceFilter = ref<SourceFilter>('all')
const sortByUsage = ref(false)
const expandedId = ref<string | null>(null)
const editorOpen = ref(false)
const editingDef = ref<CustomSkillDef | null>(null)

const filteredSkills = computed<Skill[]>(() => {
  let list = [...skillsStore.allSkills]
  if (sourceFilter.value !== 'all') {
    list = list.filter(s => s.source === sourceFilter.value)
  }
  const q = search.value.trim().toLowerCase()
  if (q) {
    list = list.filter(
      s =>
        s.name.toLowerCase().includes(q) ||
        s.description.toLowerCase().includes(q) ||
        s.id.toLowerCase().includes(q)
    )
  }
  if (sortByUsage.value) {
    list.sort(
      (a, b) => (statsStore.statsFor(b.id)?.calls ?? 0) - (statsStore.statsFor(a.id)?.calls ?? 0)
    )
  }
  return list
})

function filterLabel(f: SourceFilter): string {
  switch (f) {
    case 'builtin':
      return t('skills.groupBuiltin')
    case 'custom':
      return t('skills.groupCustom')
    case 'mcp':
      return t('skills.groupMcp')
    default:
      return t('skills.page.filterAll')
  }
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

onMounted(() => {
  // 页面可能先于聊天页访问：loadConfig/loadStats 均幂等
  skillsStore.loadConfig()
  statsStore.loadStats()
})
</script>

<style scoped>
.skills-page {
  height: 100%;
  display: flex;
  flex-direction: column;
  gap: 14px;
  padding: 16px 18px;
  overflow: auto;
  color: var(--text);
}

.console-header-bar {
  display: flex;
  align-items: center;
  gap: 10px;
  flex-wrap: wrap;
  padding-bottom: 12px;
  border-bottom: 1px solid var(--border);
}

.bar-title {
  font-family: var(--jedi-font-mono);
  font-size: 15px;
  font-weight: 700;
  letter-spacing: 0.5px;
  color: var(--accent);
}

.bar-count {
  font-family: var(--jedi-font-mono);
  font-size: 11px;
  opacity: 0.6;
}

.bar-search {
  flex: 1;
  min-width: 140px;
  max-width: 260px;
  padding: 6px 10px;
  border-radius: 7px;
  background: var(--bg-terminal);
  border: 1px solid var(--border);
  color: var(--text);
  font-size: 12px;
  outline: none;
}

.bar-search:focus {
  border-color: rgb(var(--accent-rgb) / 0.6);
}

.bar-filters {
  display: flex;
  gap: 6px;
}

.bar-chip {
  cursor: pointer;
  border: 1px solid var(--border);
  border-radius: 7px;
  padding: 5px 10px;
  font-size: 11px;
  background: rgb(var(--text-rgb) / 0.04);
  color: var(--text-muted);
}

.bar-chip.on {
  background: rgb(var(--accent-rgb) / 0.18);
  border-color: rgb(var(--accent-rgb) / 0.5);
  color: var(--accent);
}

.bar-btn {
  cursor: pointer;
  border: 1px solid var(--border);
  border-radius: 7px;
  padding: 5px 10px;
  font-size: 11px;
  background: rgb(var(--text-rgb) / 0.04);
  color: var(--text);
}

.bar-btn.on {
  border-color: rgb(var(--accent-rgb) / 0.5);
  color: var(--accent);
}

.bar-btn.danger {
  background: rgb(var(--danger-rgb) / 0.16);
  color: var(--danger);
}

.page-error {
  padding: 8px 10px;
  border-radius: 8px;
  font-size: 12px;
  background: rgb(var(--danger-rgb) / 0.14);
  color: var(--danger);
}

.page-editor {
  max-width: 720px;
}

.skills-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
  max-width: 900px;
}

.page-empty {
  padding: 40px;
  text-align: center;
  font-size: 13px;
  opacity: 0.5;
}
</style>
