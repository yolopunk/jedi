<template>
  <div class="worker-card" :class="`status-${worker.status}`">
    <!-- Status Icon -->
    <div class="status-icon">
      <!-- idle -->
      <svg v-if="worker.status === 'idle'" width="18" height="18" viewBox="0 0 24 24" fill="none">
        <circle cx="12" cy="12" r="9" stroke="currentColor" stroke-width="1.5"/>
        <path d="M12 7v5l3 3" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
      </svg>
      <!-- running -->
      <svg v-else-if="worker.status === 'running'" width="18" height="18" viewBox="0 0 24 24" fill="none">
        <circle cx="12" cy="12" r="9" stroke="currentColor" stroke-width="1.5"/>
        <path d="M12 6v6l4 2" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
      </svg>
      <!-- completed -->
      <svg v-else-if="worker.status === 'completed'" width="18" height="18" viewBox="0 0 24 24" fill="none">
        <circle cx="12" cy="12" r="9" stroke="currentColor" stroke-width="1.5"/>
        <path d="M8 12l3 3 5-5" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/>
      </svg>
      <!-- failed -->
      <svg v-else-if="worker.status === 'failed'" width="18" height="18" viewBox="0 0 24 24" fill="none">
        <circle cx="12" cy="12" r="9" stroke="currentColor" stroke-width="1.5"/>
        <path d="M15 9l-6 6M9 9l6 6" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
      </svg>
      <!-- stopped -->
      <svg v-else-if="worker.status === 'stopped'" width="18" height="18" viewBox="0 0 24 24" fill="none">
        <circle cx="12" cy="12" r="9" stroke="currentColor" stroke-width="1.5"/>
        <rect x="9" y="9" width="6" height="6" rx="1" fill="currentColor"/>
      </svg>
    </div>

    <!-- Info -->
    <div class="worker-info">
      <div class="worker-description">{{ worker.description }}</div>
      <div class="worker-meta">
        <span v-if="worker.progress" class="worker-progress">{{ worker.progress }}</span>
        <span v-else class="worker-status-text">{{ statusText }}</span>
        <span v-if="elapsedTime" class="worker-elapsed">{{ elapsedTime }}</span>
      </div>
    </div>

    <!-- Stop Button -->
    <button
      v-if="worker.status === 'running'"
      class="stop-btn"
      title="Stop worker"
      @click.stop="$emit('stop', worker.id)"
    >
      <svg width="14" height="14" viewBox="0 0 24 24" fill="none">
        <rect x="6" y="6" width="12" height="12" rx="2" fill="currentColor"/>
      </svg>
    </button>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import type { WorkerStatus, WorkerStatusKind } from '@/agent/poolTypes'

const props = defineProps<{
  worker: WorkerStatus
}>()

defineEmits<(e: 'stop', workerId: string) => void>()

const statusLabels: Record<WorkerStatusKind, string> = {
  idle: 'Idle',
  running: 'Running',
  completed: 'Completed',
  failed: 'Failed',
  stopped: 'Stopped',
}

const statusText = computed(() => statusLabels[props.worker.status])

const elapsedTime = computed(() => {
  if (!props.worker.started_at || props.worker.status === 'idle') return null
  const elapsed = Date.now() - props.worker.started_at
  const seconds = Math.floor(elapsed / 1000)
  if (seconds < 60) return `${seconds}s`
  const minutes = Math.floor(seconds / 60)
  const remainingSeconds = seconds % 60
  if (minutes < 60) return `${minutes}m ${remainingSeconds}s`
  const hours = Math.floor(minutes / 60)
  return `${hours}h ${minutes % 60}m`
})
</script>

<style scoped>
.worker-card {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px 14px;
  background: rgba(255, 255, 255, 0.03);
  border: 1px solid rgba(255, 255, 255, 0.06);
  border-radius: 10px;
  transition: all 0.2s ease;
}

.worker-card:hover {
  background: rgba(255, 255, 255, 0.05);
}

.worker-card.status-idle {
  border-color: rgba(255, 255, 255, 0.08);
}

.worker-card.status-running {
  border-color: rgba(0, 255, 255, 0.25);
  background: rgba(0, 255, 255, 0.04);
}

.worker-card.status-completed {
  border-color: rgba(0, 255, 136, 0.25);
  background: rgba(0, 255, 136, 0.03);
}

.worker-card.status-failed {
  border-color: rgba(255, 107, 107, 0.25);
  background: rgba(255, 107, 107, 0.03);
}

.worker-card.status-stopped {
  border-color: rgba(255, 255, 255, 0.08);
  opacity: 0.7;
}

.status-icon {
  width: 32px;
  height: 32px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 8px;
  flex-shrink: 0;
}

.status-idle .status-icon {
  background: rgba(255, 255, 255, 0.06);
  color: rgba(255, 255, 255, 0.3);
}

.status-running .status-icon {
  background: rgba(0, 255, 255, 0.1);
  color: #00ffff;
}

.status-completed .status-icon {
  background: rgba(0, 255, 136, 0.1);
  color: #00ff88;
}

.status-failed .status-icon {
  background: rgba(255, 107, 107, 0.1);
  color: #ff6b6b;
}

.status-stopped .status-icon {
  background: rgba(255, 255, 255, 0.04);
  color: rgba(255, 255, 255, 0.3);
}

.worker-info {
  flex: 1;
  min-width: 0;
}

.worker-description {
  font-size: 13px;
  font-weight: 500;
  color: rgba(255, 255, 255, 0.85);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.worker-meta {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-top: 3px;
}

.worker-status-text {
  font-size: 11px;
  color: rgba(255, 255, 255, 0.35);
}

.status-running .worker-status-text {
  color: #00ffff;
}

.status-completed .worker-status-text {
  color: #00ff88;
}

.status-failed .worker-status-text {
  color: #ff6b6b;
}

.worker-progress {
  font-size: 11px;
  color: #00ffff;
  font-family: 'JetBrains Mono', monospace;
}

.worker-elapsed {
  font-size: 11px;
  color: rgba(255, 255, 255, 0.25);
  font-family: 'JetBrains Mono', monospace;
}

.worker-elapsed::before {
  content: '·';
  margin-right: 8px;
}

.stop-btn {
  width: 28px;
  height: 28px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(255, 107, 107, 0.08);
  border: 1px solid rgba(255, 107, 107, 0.2);
  border-radius: 6px;
  color: #ff6b6b;
  cursor: pointer;
  transition: all 0.15s ease;
  flex-shrink: 0;
}

.stop-btn:hover {
  background: rgba(255, 107, 107, 0.15);
  border-color: rgba(255, 107, 107, 0.4);
}
</style>
