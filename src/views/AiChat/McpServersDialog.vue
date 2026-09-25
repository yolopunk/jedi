<template>
  <Transition name="mcp-fade">
    <div v-if="modelValue" class="mcp-overlay" @click.self="close">
      <div class="mcp-dialog">
        <div class="mcp-head">
          <h3>{{ $t('mcp.title') }}</h3>
          <button class="mcp-close" :aria-label="$t('common.close')" @click="close">✕</button>
        </div>

        <p class="mcp-hint">{{ $t('mcp.hint') }}</p>

        <div v-if="store.error" class="mcp-error">{{ store.error }}</div>

        <div class="mcp-list">
          <div v-for="s in store.servers" :key="s.id" class="mcp-item">
            <div class="mcp-item-main">
              <div class="mcp-item-name">
                <span class="mcp-dot" :class="{ on: store.isConnected(s.id) }"></span>
                {{ s.name }}
              </div>
              <div class="mcp-item-cmd">{{ s.url ? s.url : `${s.command} ${s.args.join(' ')}` }}</div>
            </div>
            <div class="mcp-item-actions">
              <button
                v-if="!store.isConnected(s.id)"
                class="mcp-btn"
                :disabled="store.isConnecting(s.id)"
                @click="store.connect(s.id)"
              >
                {{ store.isConnecting(s.id) ? $t('mcp.connecting') : $t('mcp.connect') }}
              </button>
              <button v-else class="mcp-btn ghost" @click="store.disconnect(s.id)">
                {{ $t('mcp.disconnect') }}
              </button>
              <button class="mcp-btn danger" @click="store.removeServer(s.id)">
                {{ $t('mcp.remove') }}
              </button>
            </div>
          </div>
          <div v-if="store.servers.length === 0" class="mcp-empty">{{ $t('mcp.empty') }}</div>
        </div>

        <div class="mcp-add">
          <div class="mcp-add-title">{{ $t('mcp.addTitle') }}</div>
          <div class="mcp-mode">
            <button class="mcp-tab" :class="{ on: form.mode === 'stdio' }" @click="form.mode = 'stdio'">
              {{ $t('mcp.modeLocal') }}
            </button>
            <button class="mcp-tab" :class="{ on: form.mode === 'sse' }" @click="form.mode = 'sse'">
              {{ $t('mcp.modeRemote') }}
            </button>
          </div>
          <input v-model="form.name" class="mcp-input" :placeholder="$t('mcp.namePlaceholder')" />
          <template v-if="form.mode === 'stdio'">
            <input v-model="form.command" class="mcp-input" :placeholder="$t('mcp.commandPlaceholder')" />
            <input v-model="form.argsText" class="mcp-input" :placeholder="$t('mcp.argsPlaceholder')" />
          </template>
          <template v-else>
            <input v-model="form.url" class="mcp-input" :placeholder="$t('mcp.urlPlaceholder')" />
          </template>
          <button class="mcp-btn primary" :disabled="!canAdd" @click="add">
            {{ $t('mcp.add') }}
          </button>
        </div>

        <div class="mcp-export">
          <div class="mcp-add-title">{{ $t('mcp.exportTitle') }}</div>
          <p class="mcp-hint">{{ $t('mcp.exportHint') }}</p>
          <code class="mcp-code">jedi --mcp-server</code>
        </div>
      </div>
    </div>
  </Transition>
</template>

<script setup lang="ts">
import { computed, reactive } from 'vue'
import { useMcpClientStore } from '@/stores/mcpClient'

defineProps<{ modelValue: boolean }>()
const emit = defineEmits<{ 'update:modelValue': [boolean] }>()

const store = useMcpClientStore()

const form = reactive({
  mode: 'stdio' as 'stdio' | 'sse',
  name: '',
  command: '',
  argsText: '',
  url: '',
})

const canAdd = computed(() => {
  if (form.name.trim() === '') return false
  return form.mode === 'stdio' ? form.command.trim() !== '' : form.url.trim() !== ''
})

function close(): void {
  emit('update:modelValue', false)
}

function add(): void {
  if (!canAdd.value) return
  const id = `${form.name
    .trim()
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, '-')}-${Date.now()}`
  if (form.mode === 'sse') {
    store.addServer({
      id,
      name: form.name.trim(),
      command: '',
      args: [],
      env: [],
      url: form.url.trim(),
    })
  } else {
    const args = form.argsText.trim() ? form.argsText.trim().split(/\s+/) : []
    store.addServer({
      id,
      name: form.name.trim(),
      command: form.command.trim(),
      args,
      env: [],
    })
  }
  form.name = ''
  form.command = ''
  form.argsText = ''
  form.url = ''
}
</script>

<style scoped>
.mcp-overlay {
  position: fixed;
  inset: 0;
  z-index: var(--z-panel);
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgb(var(--ink-rgb) / 0.4);
  backdrop-filter: blur(2px);
}

.mcp-dialog {
  width: min(600px, calc(100vw - 32px));
  max-height: 80vh;
  overflow: auto;
  border-radius: 14px;
  padding: 20px 22px;
  background: var(--bg-elevated);
  border: 1px solid var(--border);
  box-shadow: var(--shadow-lg);
  color: var(--text);
}

.mcp-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 6px;
}

.mcp-head h3 {
  margin: 0;
  font-size: 16px;
}

.mcp-close {
  background: none;
  border: none;
  color: var(--text-muted);
  cursor: pointer;
  font-size: 15px;
}

.mcp-hint {
  margin: 0 0 12px;
  font-size: 12px;
  opacity: 0.66;
  line-height: 1.5;
}

.mcp-error {
  margin-bottom: 10px;
  padding: 8px 10px;
  border-radius: 8px;
  font-size: 12px;
  background: rgb(var(--danger-rgb) / 0.14);
  color: var(--danger);
}

.mcp-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
  margin-bottom: 16px;
}

.mcp-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  padding: 10px 12px;
  border-radius: 10px;
  background: rgb(var(--text-rgb) / 0.04);
  border: 1px solid rgb(var(--text-rgb) / 0.06);
}

.mcp-item-main {
  min-width: 0;
  flex: 1;
}

.mcp-item-name {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 14px;
  font-weight: 600;
}

.mcp-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: var(--text-subtle);
}

.mcp-dot.on {
  background: var(--success);
  box-shadow: var(--glow-success);
}

.mcp-item-cmd {
  margin-top: 2px;
  font-size: 11px;
  opacity: 0.55;
  font-family: var(--jedi-font-mono);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.mcp-item-actions {
  display: flex;
  gap: 6px;
  flex-shrink: 0;
}

.mcp-add {
  display: flex;
  flex-direction: column;
  gap: 8px;
  padding-top: 14px;
  border-top: 1px solid var(--border);
}

.mcp-add-title {
  font-size: 12px;
  font-weight: 600;
  opacity: 0.72;
}

.mcp-mode {
  display: flex;
  gap: 6px;
}

.mcp-tab {
  flex: 1;
  cursor: pointer;
  border: 1px solid var(--border);
  border-radius: 7px;
  padding: 6px 10px;
  font-size: 12px;
  background: rgb(var(--text-rgb) / 0.04);
  color: var(--text-muted);
}

.mcp-tab.on {
  background: rgb(var(--accent-rgb) / 0.18);
  border-color: rgb(var(--accent-rgb) / 0.5);
  color: var(--accent);
}

.mcp-export {
  margin-top: 16px;
  padding-top: 14px;
  border-top: 1px solid var(--border);
}

.mcp-code {
  display: inline-block;
  margin-top: 4px;
  padding: 6px 10px;
  border-radius: 7px;
  background: rgb(var(--ink-rgb) / 0.15);
  border: 1px solid var(--border);
  font-family: var(--jedi-font-mono);
  font-size: 12px;
  color: var(--accent);
}

.mcp-input {
  padding: 8px 10px;
  border-radius: 8px;
  background: var(--bg-terminal);
  border: 1px solid var(--border);
  color: var(--text);
  font-size: 13px;
  outline: none;
}

.mcp-input:focus {
  border-color: rgb(var(--accent-rgb) / 0.6);
}

.mcp-empty {
  padding: 14px;
  text-align: center;
  font-size: 12px;
  opacity: 0.5;
}

.mcp-btn {
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

.mcp-btn:hover:not(:disabled) {
  filter: brightness(1.15);
}

.mcp-btn:disabled {
  opacity: 0.5;
  cursor: default;
}

.mcp-btn.primary {
  background: var(--accent);
  color: var(--on-accent);
}

.mcp-btn.ghost {
  background: rgb(var(--text-rgb) / 0.06);
}

.mcp-btn.danger {
  background: rgb(var(--danger-rgb) / 0.16);
  color: var(--danger);
}

.mcp-fade-enter-active,
.mcp-fade-leave-active {
  transition: opacity 0.18s ease;
}

.mcp-fade-enter-from,
.mcp-fade-leave-to {
  opacity: 0;
}
</style>
