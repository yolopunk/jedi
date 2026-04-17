<template>
  <Teleport to="body">
    <Transition name="panel">
      <div v-if="showPanel" class="agent-pool-panel">
        <!-- Header -->
        <div class="panel-header">
          <div class="header-left">
            <svg width="18" height="18" viewBox="0 0 24 24" fill="none">
              <rect x="2" y="7" width="20" height="10" rx="2" stroke="currentColor" stroke-width="1.5"/>
              <path d="M6 7V5a2 2 0 0 1 2-2h8a2 2 0 0 1 2 2v2" stroke="currentColor" stroke-width="1.5"/>
              <line x1="12" y1="12" x2="12" y2="12.01" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>
            </svg>
            <span class="header-title">Workers</span>
            <span v-if="workers.length > 0" class="worker-count">{{ workers.length }}</span>
          </div>
          <button class="close-btn" @click="$emit('update:showPanel', false)">
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none">
              <line x1="18" y1="6" x2="6" y2="18" stroke="currentColor" stroke-width="2"/>
              <line x1="6" y1="6" x2="18" y2="18" stroke="currentColor" stroke-width="2"/>
            </svg>
          </button>
        </div>

        <!-- Worker List -->
        <div class="panel-body">
          <div v-if="workers.length === 0" class="empty-state">
            <svg width="32" height="32" viewBox="0 0 24 24" fill="none">
              <circle cx="12" cy="12" r="9" stroke="currentColor" stroke-width="1.5"/>
              <path d="M9 10h.01M15 10h.01M9 14s1 1 3 1 3-1 3-1" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
            </svg>
            <span>No active workers</span>
          </div>
          <div v-else class="worker-list">
            <WorkerCard
              v-for="worker in workers"
              :key="worker.id"
              :worker="worker"
              @stop="handleStop"
            />
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<script setup lang="ts">
import { stopWorker } from '@/agent/pool'
import { useAgentPool } from '@/agent/useAgentPool'

const _props = defineProps<{
  showPanel: boolean
}>()

const _emit = defineEmits<{
  'update:showPanel': [value: boolean]
}>()

const { workers, refresh } = useAgentPool()

async function _handleStop(workerId: string) {
  try {
    await stopWorker(workerId)
    await refresh()
  } catch (e) {
    console.error('Failed to stop worker:', e)
  }
}
</script>

<style scoped>
.agent-pool-panel {
  position: fixed;
  bottom: 80px;
  right: 20px;
  width: 340px;
  max-height: 400px;
  background: #0a0e14;
  border: 1px solid rgba(0, 255, 255, 0.15);
  border-radius: 14px;
  box-shadow:
    0 0 0 1px rgba(0, 0, 0, 0.3),
    0 8px 32px rgba(0, 0, 0, 0.5),
    0 0 60px rgba(0, 255, 255, 0.05);
  display: flex;
  flex-direction: column;
  overflow: hidden;
  z-index: 9999;
}

.panel-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 14px 16px;
  background: rgba(20, 30, 40, 0.6);
  border-bottom: 1px solid rgba(255, 255, 255, 0.06);
  flex-shrink: 0;
}

.header-left {
  display: flex;
  align-items: center;
  gap: 10px;
  color: #00ffff;
}

.header-title {
  font-size: 14px;
  font-weight: 600;
  color: #ffffff;
}

.worker-count {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-width: 20px;
  height: 20px;
  padding: 0 6px;
  background: rgba(0, 255, 255, 0.15);
  border: 1px solid rgba(0, 255, 255, 0.25);
  border-radius: 10px;
  font-size: 11px;
  font-weight: 600;
  color: #00ffff;
}

.close-btn {
  width: 30px;
  height: 30px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: transparent;
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 7px;
  color: rgba(255, 255, 255, 0.4);
  cursor: pointer;
  transition: all 0.15s ease;
}

.close-btn:hover {
  background: rgba(255, 107, 107, 0.1);
  border-color: rgba(255, 107, 107, 0.3);
  color: #ff6b6b;
}

.panel-body {
  flex: 1;
  overflow-y: auto;
  padding: 12px;
}

.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 10px;
  padding: 32px 16px;
  color: rgba(255, 255, 255, 0.25);
}

.empty-state span {
  font-size: 13px;
}

.worker-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

/* Transitions */
.panel-enter-active,
.panel-leave-active {
  transition: all 0.2s ease;
}

.panel-enter-from,
.panel-leave-to {
  opacity: 0;
  transform: translateY(10px) scale(0.98);
}

/* Scrollbar */
::-webkit-scrollbar {
  width: 4px;
}

::-webkit-scrollbar-track {
  background: transparent;
}

::-webkit-scrollbar-thumb {
  background: rgba(255, 255, 255, 0.08);
  border-radius: 2px;
}

::-webkit-scrollbar-thumb:hover {
  background: rgba(255, 255, 255, 0.12);
}
</style>
