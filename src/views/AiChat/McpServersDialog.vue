<template>
  <Transition name="mcp-fade">
    <div v-if="modelValue" class="mcp-overlay" @click.self="close">
      <div class="mcp-dialog">
        <div class="mcp-head">
          <h3>第三方 MCP 服务器</h3>
          <button class="mcp-close" @click="close">✕</button>
        </div>

        <p class="mcp-hint">
          连接外部 MCP 服务器（stdio），其工具会自动加入 AI 的可用工具集。第三方工具默认需要确认后执行。
        </p>

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
                {{ store.isConnecting(s.id) ? '连接中…' : '连接' }}
              </button>
              <button v-else class="mcp-btn ghost" @click="store.disconnect(s.id)">断开</button>
              <button class="mcp-btn danger" @click="store.removeServer(s.id)">删除</button>
            </div>
          </div>
          <div v-if="store.servers.length === 0" class="mcp-empty">还没有配置 MCP 服务器</div>
        </div>

        <div class="mcp-add">
          <div class="mcp-add-title">添加服务器</div>
          <div class="mcp-mode">
            <button class="mcp-tab" :class="{ on: form.mode === 'stdio' }" @click="form.mode = 'stdio'">
              本地 (stdio)
            </button>
            <button class="mcp-tab" :class="{ on: form.mode === 'sse' }" @click="form.mode = 'sse'">
              远程 (SSE)
            </button>
          </div>
          <input v-model="form.name" class="mcp-input" placeholder="名称，如 Filesystem" />
          <template v-if="form.mode === 'stdio'">
            <input v-model="form.command" class="mcp-input" placeholder="命令，如 npx" />
            <input v-model="form.argsText" class="mcp-input" placeholder="参数（空格分隔），如 -y @modelcontextprotocol/server-filesystem /tmp" />
          </template>
          <template v-else>
            <input v-model="form.url" class="mcp-input" placeholder="URL，如 http://localhost:3000/sse" />
          </template>
          <button class="mcp-btn primary" :disabled="!canAdd" @click="add">添加</button>
        </div>

        <div class="mcp-export">
          <div class="mcp-add-title">把 Jedi 作为 MCP server</div>
          <p class="mcp-hint">
            在其他 MCP 客户端（如 Claude Desktop）中，将 Jedi 配置为以下命令，即可使用 Jedi 的只读工具（记忆 / 网页）：
          </p>
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

const form = reactive({ mode: 'stdio' as 'stdio' | 'sse', name: '', command: '', argsText: '', url: '' })

const canAdd = computed(() => {
  if (form.name.trim() === '') return false
  return form.mode === 'stdio' ? form.command.trim() !== '' : form.url.trim() !== ''
})

function close(): void {
  emit('update:modelValue', false)
}

function add(): void {
  if (!canAdd.value) return
  const id = `${form.name.trim().toLowerCase().replace(/[^a-z0-9]+/g, '-')}-${Date.now()}`
  if (form.mode === 'sse') {
    store.addServer({ id, name: form.name.trim(), command: '', args: [], env: [], url: form.url.trim() })
  } else {
    const args = form.argsText.trim() ? form.argsText.trim().split(/\s+/) : []
    store.addServer({ id, name: form.name.trim(), command: form.command.trim(), args, env: [] })
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
  z-index: 3000;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(0, 0, 0, 0.4);
  backdrop-filter: blur(2px);
}

.mcp-dialog {
  width: min(600px, calc(100vw - 32px));
  max-height: 80vh;
  overflow: auto;
  border-radius: 14px;
  padding: 20px 22px;
  background: rgba(24, 26, 32, 0.98);
  border: 1px solid rgba(255, 255, 255, 0.1);
  box-shadow: 0 16px 48px rgba(0, 0, 0, 0.45);
  color: #e8e8ec;
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
  color: #aaa;
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
  background: rgba(255, 86, 86, 0.14);
  color: #ff7a7a;
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
  background: rgba(255, 255, 255, 0.04);
  border: 1px solid rgba(255, 255, 255, 0.06);
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
  background: #666;
}

.mcp-dot.on {
  background: #4ade80;
  box-shadow: 0 0 6px #4ade80;
}

.mcp-item-cmd {
  margin-top: 2px;
  font-size: 11px;
  opacity: 0.55;
  font-family: var(--mono-font, monospace);
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
  border-top: 1px solid rgba(255, 255, 255, 0.08);
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
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 7px;
  padding: 6px 10px;
  font-size: 12px;
  background: rgba(255, 255, 255, 0.04);
  color: #bdbdc2;
}

.mcp-tab.on {
  background: rgba(91, 140, 255, 0.18);
  border-color: rgba(91, 140, 255, 0.5);
  color: #fff;
}

.mcp-export {
  margin-top: 16px;
  padding-top: 14px;
  border-top: 1px solid rgba(255, 255, 255, 0.08);
}

.mcp-code {
  display: inline-block;
  margin-top: 4px;
  padding: 6px 10px;
  border-radius: 7px;
  background: rgba(0, 0, 0, 0.4);
  border: 1px solid rgba(255, 255, 255, 0.08);
  font-family: var(--mono-font, monospace);
  font-size: 12px;
  color: #9ecbff;
}

.mcp-input {
  padding: 8px 10px;
  border-radius: 8px;
  background: rgba(0, 0, 0, 0.3);
  border: 1px solid rgba(255, 255, 255, 0.1);
  color: #e8e8ec;
  font-size: 13px;
  outline: none;
}

.mcp-input:focus {
  border-color: rgba(91, 140, 255, 0.6);
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
  background: rgba(255, 255, 255, 0.08);
  color: #d6d6da;
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
  background: linear-gradient(135deg, #5b8cff, #6a5bff);
  color: #fff;
}

.mcp-btn.ghost {
  background: rgba(255, 255, 255, 0.06);
}

.mcp-btn.danger {
  background: rgba(255, 86, 86, 0.16);
  color: #ff7a7a;
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
