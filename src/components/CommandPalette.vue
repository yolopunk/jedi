<template>
  <div class="command-popup" v-if="visible && commands.length > 0">
    <div class="popup-list">
      <div
        v-for="(cmd, i) in commands"
        :key="cmd.name"
        class="popup-item"
        :class="{ active: i === activeIndex }"
        @click="selectCommand(cmd)"
        @mouseenter="$emit('hover', i)"
      >
        <span class="item-icon">{{ cmd.icon }}</span>
        <span class="item-name">{{ cmd.name }}</span>
        <span class="item-desc">{{ cmd.description }}</span>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import type { SlashCommand } from '@/agent/slashCommands'

withDefaults(
  defineProps<{
    visible: boolean
    commands: SlashCommand[]
    activeIndex?: number
  }>(),
  { activeIndex: 0 }
)

const emit = defineEmits<{
  (e: 'select', command: SlashCommand): void
  (e: 'hover', index: number): void
  (e: 'close'): void
}>()

function selectCommand(cmd: SlashCommand) {
  emit('select', cmd)
  emit('close')
}
</script>

<style scoped>
.command-popup {
  position: absolute;
  bottom: 100%;
  left: 16px;
  margin-bottom: 8px;
  width: 320px;
  max-width: calc(100% - 32px);
  background: var(--surface-raised);
  border: 1px solid var(--border-strong);
  border-radius: 10px;
  overflow: hidden;
  box-shadow: var(--shadow-lg);
  z-index: 100;
}

.popup-list {
  padding: 6px;
  max-height: min(280px, 40vh);
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

.popup-item:hover,
.popup-item.active {
  background: rgb(var(--accent-rgb) / 0.08);
}

.item-icon {
  font-size: 14px;
}

.item-name {
  font-size: 12px;
  font-weight: 600;
  color: var(--text);
  min-width: 70px;
}

.item-desc {
  font-size: 11px;
  color: var(--text-muted);
  overflow-wrap: anywhere;
}
</style>
