<template>
  <div class="command-palette-overlay" v-if="visible" @click.self="$emit('close')">
    <div class="command-palette">
      <div class="palette-header">
        <span class="palette-title">Commands</span>
        <button class="close-btn" @click="$emit('close')">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none">
            <line x1="18" y1="6" x2="6" y2="18" stroke="currentColor" stroke-width="2"/>
            <line x1="6" y1="6" x2="18" y2="18" stroke="currentColor" stroke-width="2"/>
          </svg>
        </button>
      </div>
      <div class="palette-list">
        <div
          v-for="cmd in commands"
          :key="cmd.name"
          class="palette-item"
          @click="selectCommand(cmd)"
        >
          <span class="item-icon">{{ cmd.icon }}</span>
          <span class="item-name">{{ cmd.name }}</span>
          <span class="item-desc">{{ cmd.description }}</span>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { SLASH_COMMANDS, type SlashCommand } from '@/agent/slashCommands'

defineProps<{ visible: boolean }>()
const emit = defineEmits<{
  (e: 'close'): void
  (e: 'select', command: SlashCommand): void
}>()

const commands = SLASH_COMMANDS

function selectCommand(cmd: SlashCommand) {
  emit('select', cmd)
  emit('close')
}
</script>

<style scoped>
.command-palette-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
}

.command-palette {
  width: 320px;
  max-height: 400px;
  background: #0a0e14;
  border: 1px solid rgba(0, 255, 255, 0.2);
  border-radius: 12px;
  overflow: hidden;
  box-shadow: 0 16px 48px rgba(0, 0, 0, 0.5);
}

.palette-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 16px;
  background: rgba(20, 30, 40, 0.6);
  border-bottom: 1px solid rgba(255, 255, 255, 0.06);
}

.palette-title {
  font-size: 13px;
  font-weight: 600;
  color: #ffffff;
}

.close-btn {
  width: 24px;
  height: 24px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: transparent;
  border: none;
  color: rgba(255, 255, 255, 0.4);
  cursor: pointer;
}

.close-btn:hover {
  color: #ff6b6b;
}

.palette-list {
  padding: 8px;
  max-height: 320px;
  overflow-y: auto;
}

.palette-item {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 10px 12px;
  border-radius: 8px;
  cursor: pointer;
  transition: background 0.15s;
}

.palette-item:hover {
  background: rgba(0, 255, 255, 0.08);
}

.item-icon {
  font-size: 16px;
}

.item-name {
  font-size: 12px;
  font-weight: 600;
  color: #00ffff;
  min-width: 80px;
}

.item-desc {
  font-size: 11px;
  color: rgba(255, 255, 255, 0.5);
}
</style>
