<template>
  <div class="command-popup" v-if="visible">
    <div class="popup-list">
      <div
        v-for="cmd in commands"
        :key="cmd.name"
        class="popup-item"
        @click="selectCommand(cmd)"
      >
        <span class="item-icon">{{ cmd.icon }}</span>
        <span class="item-name">{{ cmd.name }}</span>
        <span class="item-desc">{{ cmd.description }}</span>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { SLASH_COMMANDS, type SlashCommand } from '@/agent/slashCommands'

defineProps<{ visible: boolean }>()
const emit = defineEmits<{
  (e: 'select', command: SlashCommand): void
  (e: 'close'): void
}>()

const commands = SLASH_COMMANDS

function selectCommand(cmd: SlashCommand) {
  emit('select', cmd)
}
</script>

<style scoped>
.command-popup {
  position: absolute;
  bottom: 100%;
  left: 0;
  margin-bottom: 8px;
  width: 280px;
  background: #0a0e14;
  border: 1px solid rgba(0, 255, 255, 0.2);
  border-radius: 10px;
  overflow: hidden;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.4);
  z-index: 100;
}

.popup-list {
  padding: 6px;
  max-height: 280px;
  overflow-y: auto;
}

.popup-item {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 8px 10px;
  border-radius: 6px;
  cursor: pointer;
  transition: background 0.1s;
}

.popup-item:hover {
  background: rgba(0, 255, 255, 0.08);
}

.item-icon {
  font-size: 14px;
}

.item-name {
  font-size: 12px;
  font-weight: 600;
  color: #00ffff;
  min-width: 70px;
}

.item-desc {
  font-size: 11px;
  color: rgba(255, 255, 255, 0.5);
}
</style>
