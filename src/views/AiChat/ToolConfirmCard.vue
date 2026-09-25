<template>
  <Transition name="confirm-fade">
    <div v-if="store.pendingConfirmation" class="confirm-overlay">
      <div class="confirm-card">
        <div class="confirm-head">
          <span class="confirm-risk" :class="`risk-${store.pendingConfirmation?.risk}`">
            {{ riskLabel }}
          </span>
          <span class="confirm-title">{{ store.pendingConfirmation?.skillName }}</span>
        </div>
        <p class="confirm-desc">{{ $t('skills.confirm.desc') }}</p>
        <pre class="confirm-args">{{ prettyArgs }}</pre>
        <p v-if="isSystemRisk" class="confirm-warning">{{ $t('skills.confirm.systemWarning') }}</p>
        <div class="confirm-actions">
          <button class="btn btn-deny" @click="store.resolveConfirmation(false)">
            {{ $t('skills.confirm.deny') }}
          </button>
          <button class="btn btn-approve" @click="store.resolveConfirmation(true)">
            {{ $t('skills.confirm.approve') }}
          </button>
          <button class="btn btn-always" @click="store.resolveConfirmation(true, true)">
            {{ $t('skills.confirm.alwaysApprove') }}
          </button>
        </div>
      </div>
    </div>
  </Transition>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { useAiChatStore } from '@/stores/aiChat'

const store = useAiChatStore()
const { t } = useI18n()

const isSystemRisk = computed(() => store.pendingConfirmation?.risk === 'system')

const riskLabel = computed(() => {
  switch (store.pendingConfirmation?.risk) {
    case 'system':
      return t('skills.confirm.riskSystem')
    case 'write':
      return t('skills.confirm.riskWrite')
    default:
      return t('skills.confirm.riskDefault')
  }
})

const prettyArgs = computed(() => {
  const args = store.pendingConfirmation?.args
  if (args == null) return ''
  try {
    return JSON.stringify(args, null, 2)
  } catch {
    return String(args)
  }
})
</script>

<style scoped>
.confirm-overlay {
  position: fixed;
  inset: 0;
  z-index: 3000;
  display: flex;
  align-items: flex-end;
  justify-content: center;
  padding: 0 0 32px;
  background: rgba(0, 0, 0, 0.32);
  backdrop-filter: blur(2px);
}

.confirm-card {
  width: min(560px, calc(100vw - 32px));
  border-radius: 14px;
  padding: 18px 20px 16px;
  background: rgba(24, 26, 32, 0.98);
  border: 1px solid rgba(255, 255, 255, 0.1);
  box-shadow: 0 16px 48px rgba(0, 0, 0, 0.45);
  color: #e8e8ec;
}

.confirm-head {
  display: flex;
  align-items: center;
  gap: 10px;
  margin-bottom: 8px;
}

.confirm-risk {
  font-size: 11px;
  font-weight: 600;
  letter-spacing: 0.3px;
  padding: 2px 8px;
  border-radius: 6px;
  text-transform: uppercase;
}

.risk-system {
  background: rgba(255, 86, 86, 0.16);
  color: #ff7a7a;
  border: 1px solid rgba(255, 86, 86, 0.35);
}

.risk-write {
  background: rgba(255, 176, 32, 0.14);
  color: #ffc451;
  border: 1px solid rgba(255, 176, 32, 0.32);
}

.confirm-title {
  font-size: 15px;
  font-weight: 600;
  font-family: var(--mono-font, monospace);
}

.confirm-desc {
  margin: 0 0 10px;
  font-size: 13px;
  opacity: 0.72;
}

.confirm-args {
  margin: 0 0 14px;
  max-height: 220px;
  overflow: auto;
  padding: 10px 12px;
  border-radius: 8px;
  background: rgba(0, 0, 0, 0.35);
  border: 1px solid rgba(255, 255, 255, 0.06);
  font-family: var(--mono-font, monospace);
  font-size: 12px;
  line-height: 1.5;
  white-space: pre-wrap;
  word-break: break-word;
}

.confirm-warning {
  margin: 0 0 12px;
  padding: 8px 10px;
  border-radius: 8px;
  font-size: 12px;
  background: rgba(255, 86, 86, 0.1);
  border: 1px solid rgba(255, 86, 86, 0.25);
  color: #ff9a9a;
}

.confirm-actions {
  display: flex;
  justify-content: flex-end;
  gap: 10px;
}

.btn {
  cursor: pointer;
  border: none;
  border-radius: 8px;
  padding: 8px 16px;
  font-size: 13px;
  font-weight: 600;
  transition: filter 0.15s ease;
}

.btn:hover {
  filter: brightness(1.12);
}

.btn-deny {
  background: rgba(255, 255, 255, 0.08);
  color: #d6d6da;
}

.btn-approve {
  background: linear-gradient(135deg, #5b8cff, #6a5bff);
  color: #fff;
}

.btn-always {
  background: rgba(251, 191, 36, 0.16);
  color: #fbbf24;
  border: 1px solid rgba(251, 191, 36, 0.35);
}

.confirm-fade-enter-active,
.confirm-fade-leave-active {
  transition: opacity 0.18s ease;
}

.confirm-fade-enter-from,
.confirm-fade-leave-to {
  opacity: 0;
}
</style>
