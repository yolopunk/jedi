<template>
  <Transition name="skl-fade" @after-leave="restoreFocus">
    <div
      v-if="modelValue"
      class="skl-overlay"
      @click.self="onOverlayClick"
    >
      <div
        ref="dialogRef"
        class="skl-dialog"
        role="dialog"
        aria-modal="true"
        :aria-label="$t('skills.title')"
        @keydown="onDialogKeydown"
      >
        <div class="skl-head">
          <h3>{{ $t('skills.title') }}</h3>
          <div class="skl-head-right">
            <span class="skl-count">
              {{
                $t('skills.enabledCount', {
                  count: toolsStore.enabledCount + skillsStore.enabledCount,
                })
              }}
            </span>
            <button class="skl-close" :aria-label="$t('common.close')" @click="close">✕</button>
          </div>
        </div>

        <p class="skl-hint">{{ $t('skills.hint') }}</p>

        <SkillsManagerContent @editor-change="editorOpen = $event" />
      </div>
    </div>
  </Transition>
</template>

<script setup lang="ts">
import { nextTick, ref, watch } from 'vue'
import SkillsManagerContent from '@/components/skills/SkillsManagerContent.vue'
import { useSkillsStore } from '@/stores/skills'
import { useToolsStore } from '@/stores/tools'

const props = defineProps<{ modelValue: boolean }>()
const emit = defineEmits<{ 'update:modelValue': [boolean] }>()

const skillsStore = useSkillsStore()
const toolsStore = useToolsStore()

const dialogRef = ref<HTMLElement | null>(null)
const editorOpen = ref(false)
let previouslyFocused: HTMLElement | null = null

function close(): void {
  emit('update:modelValue', false)
}

// 编辑器打开时点击遮罩不关闭，避免丢失未保存的表单内容（仍可用 ✕ / Esc 关闭）
function onOverlayClick(): void {
  if (editorOpen.value) return
  close()
}

// 对话框内 Tab 循环（焦点陷阱）：到达首/尾可聚焦元素时回绕
function onDialogKeydown(e: KeyboardEvent): void {
  if (e.key === 'Escape') {
    e.stopPropagation()
    close()
    return
  }
  if (e.key !== 'Tab' || !dialogRef.value) return
  const focusables = dialogRef.value.querySelectorAll<HTMLElement>(
    'a[href], button:not([disabled]), input:not([disabled]), select, textarea, [tabindex]:not([tabindex="-1"])'
  )
  if (focusables.length === 0) return
  const first = focusables[0]
  const last = focusables[focusables.length - 1]
  const active = document.activeElement as HTMLElement | null
  if (e.shiftKey && (active === first || !dialogRef.value.contains(active))) {
    e.preventDefault()
    last.focus()
  } else if (!e.shiftKey && (active === last || !dialogRef.value.contains(active))) {
    e.preventDefault()
    first.focus()
  }
}

function restoreFocus(): void {
  previouslyFocused?.focus()
  previouslyFocused = null
}

watch(
  () => props.modelValue,
  async open => {
    if (open) {
      previouslyFocused = document.activeElement as HTMLElement | null
      // 等过渡把节点挂载后再聚焦第一个控件
      await nextTick()
      dialogRef.value?.querySelector<HTMLElement>('input, button')?.focus()
    }
  }
)
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
  width: min(680px, calc(100vw - 32px));
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
  color: var(--text-muted);
}

.skl-close {
  background: none;
  border: none;
  color: var(--text-muted);
  cursor: pointer;
  font-size: 15px;
}

.skl-close:hover {
  color: var(--text);
}

.skl-hint {
  margin: 0 0 14px;
  font-size: 12px;
  color: var(--text-muted);
  line-height: 1.5;
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
