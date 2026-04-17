<template>
  <div class="session-list-wrapper">
    <!-- Header -->
    <div class="list-header">
      <v-btn
        block
        color="primary"
        variant="flat"
        prepend-icon="mdi-plus"
        class="new-chat-btn"
        @click="$emit('new-session')"
      >
        New Chat
      </v-btn>
    </div>

    <!-- Search -->
    <div class="list-search">
      <v-text-field
        v-model="searchQuery"
        :placeholder="$t('chat.searchPlaceholder')"
        prepend-inner-icon="mdi-magnify"
        variant="solo-filled"
        density="compact"
        hide-details
        single-line
        clearable
        class="search-input"
      />
    </div>

    <!-- Sessions List -->
    <div class="sessions-container">
      <div v-if="filteredSessions.length === 0" class="empty-state">
        <v-icon icon="mdi-chat-outline" size="48" color="surface-variant" />
        <p v-if="searchQuery">No conversations found</p>
        <p v-else>No conversations yet</p>
        <v-btn
          v-if="!searchQuery"
          color="primary"
          variant="text"
          @click="$emit('new-session')"
        >
          Start a new chat
        </v-btn>
      </div>

      <TransitionGroup v-else name="list" tag="div" class="sessions-list">
        <div
          v-for="session in filteredSessions"
          :key="session.id"
          :class="['session-item', { active: session.id === currentSessionId }]"
          @click="$emit('select-session', session.id)"
        >
          <div class="session-icon">
            <v-icon 
              :icon="getSessionIcon(session)" 
              size="20"
              :color="session.id === currentSessionId ? 'primary' : 'default'"
            />
          </div>
          
          <div class="session-content">
            <div class="session-title">
              {{ session.title || 'New Chat' }}
            </div>
            <div class="session-meta">
              <span class="session-date">{{ formatDate(session.updated_at) }}</span>
              <span v-if="session.messages.length" class="session-count">
                {{ session.messages.length }} messages
              </span>
            </div>
          </div>

          <v-menu location="bottom end">
            <template v-slot:activator="{ props: menuProps }">
              <v-btn
                icon
                variant="text"
                size="small"
                class="session-menu-btn"
                v-bind="menuProps"
                @click.stop
              >
                <v-icon icon="mdi-dots-vertical" size="18" />
              </v-btn>
            </template>
            
            <v-list density="compact">
              <v-list-item @click.stop="$emit('rename-session', session.id)">
                <template v-slot:prepend>
                  <v-icon icon="mdi-pencil" />
                </template>
                <v-list-item-title>Rename</v-list-item-title>
              </v-list-item>
              
              <v-list-item @click.stop="$emit('export-session', session.id)">
                <template v-slot:prepend>
                  <v-icon icon="mdi-export" />
                </template>
                <v-list-item-title>Export</v-list-item-title>
              </v-list-item>
              
              <v-divider />
              
              <v-list-item 
                @click.stop="$emit('delete-session', session.id)"
                class="delete-item"
              >
                <template v-slot:prepend>
                  <v-icon icon="mdi-delete" color="error" />
                </template>
                <v-list-item-title class="text-error">Delete</v-list-item-title>
              </v-list-item>
            </v-list>
          </v-menu>
        </div>
      </TransitionGroup>
    </div>

    <!-- Footer -->
    <div class="list-footer">
      <v-btn
        variant="text"
        size="small"
        prepend-icon="mdi-cog"
        @click="$emit('open-settings')"
      >
        Settings
      </v-btn>
    </div>
  </div>
</template>

<script setup lang="ts">
import dayjs from 'dayjs'
import relativeTime from 'dayjs/plugin/relativeTime'
import { computed, ref } from 'vue'
import type { Session } from '@/stores/aiChat'

dayjs.extend(relativeTime)

const props = defineProps<{
  sessions: Session[]
  currentSessionId: string | null
}>()

defineEmits<{
  (e: 'new-session'): void
  (e: 'select-session', sessionId: string): void
  (e: 'rename-session', sessionId: string): void
  (e: 'export-session', sessionId: string): void
  (e: 'delete-session', sessionId: string): void
  (e: 'open-settings'): void
}>()

const searchQuery = ref('')

const _filteredSessions = computed(() => {
  if (!searchQuery.value) {
    return props.sessions
  }

  const query = searchQuery.value.toLowerCase()
  return props.sessions.filter(
    session =>
      session.title.toLowerCase().includes(query) ||
      session.messages.some(m => m.content.toLowerCase().includes(query))
  )
})

function _getSessionIcon(session: Session): string {
  if (session.messages.length === 0) return 'mdi-chat-plus'
  if (session.messages.length < 5) return 'mdi-chat'
  if (session.messages.length < 20) return 'mdi-chat-processing'
  return 'mdi-forum'
}

function _formatDate(dateStr: string): string {
  const date = dayjs(dateStr)
  const now = dayjs()

  if (now.diff(date, 'day') < 1) {
    return date.fromNow()
  } else if (now.diff(date, 'day') < 7) {
    return date.format('ddd')
  } else {
    return date.format('MMM D')
  }
}
</script>

<style scoped>
.session-list-wrapper {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: rgba(var(--v-theme-surface-variant), 0.3);
}

.list-header {
  padding: 16px;
}

.new-chat-btn {
  border-radius: 12px;
  font-weight: 600;
  text-transform: none;
  letter-spacing: 0;
}

.list-search {
  padding: 0 16px 12px;
}

.search-input :deep(.v-field) {
  border-radius: 12px;
  background: rgba(var(--v-theme-background), 0.6);
}

.sessions-container {
  flex: 1;
  overflow-y: auto;
  padding: 0 8px;
}

.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 48px 24px;
  text-align: center;
  color: rgba(var(--v-theme-on-surface), 0.5);
}

.empty-state p {
  margin: 16px 0 8px;
  font-size: 0.875rem;
}

.sessions-list {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.session-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px;
  border-radius: 12px;
  cursor: pointer;
  transition: all 0.2s ease;
  position: relative;
}

.session-item:hover {
  background: rgba(var(--v-theme-on-surface), 0.05);
}

.session-item.active {
  background: rgba(var(--v-theme-primary), 0.1);
}

.session-item.active::before {
  content: '';
  position: absolute;
  left: 0;
  top: 50%;
  transform: translateY(-50%);
  width: 3px;
  height: 24px;
  background: rgb(var(--v-theme-primary));
  border-radius: 0 3px 3px 0;
}

.session-icon {
  flex-shrink: 0;
  width: 36px;
  height: 36px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 10px;
  background: rgba(var(--v-theme-surface-variant), 0.5);
}

.session-item.active .session-icon {
  background: rgba(var(--v-theme-primary), 0.15);
}

.session-content {
  flex: 1;
  min-width: 0;
}

.session-title {
  font-size: 0.9375rem;
  font-weight: 500;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.session-meta {
  display: flex;
  gap: 8px;
  margin-top: 2px;
  font-size: 0.75rem;
  color: rgba(var(--v-theme-on-surface), 0.5);
}

.session-count::before {
  content: '·';
  margin-right: 8px;
}

.session-menu-btn {
  opacity: 0;
  transition: opacity 0.2s ease;
}

.session-item:hover .session-menu-btn {
  opacity: 1;
}

.delete-item {
  color: rgb(var(--v-theme-error));
}

.list-footer {
  padding: 16px;
  border-top: 1px solid rgba(var(--v-theme-on-surface), 0.1);
}

/* List animations */
.list-enter-active,
.list-leave-active {
  transition: all 0.3s ease;
}

.list-enter-from {
  opacity: 0;
  transform: translateX(-20px);
}

.list-leave-to {
  opacity: 0;
  transform: translateX(20px);
}

/* Scrollbar styling */
.sessions-container::-webkit-scrollbar {
  width: 6px;
}

.sessions-container::-webkit-scrollbar-track {
  background: transparent;
}

.sessions-container::-webkit-scrollbar-thumb {
  background: rgba(var(--v-theme-on-surface), 0.2);
  border-radius: 3px;
}

.sessions-container::-webkit-scrollbar-thumb:hover {
  background: rgba(var(--v-theme-on-surface), 0.3);
}

/* Dark mode */
:global(.v-theme--dark) .session-list-wrapper {
  background: rgba(0, 0, 0, 0.2);
}

:global(.v-theme--dark) .session-icon {
  background: rgba(255, 255, 255, 0.05);
}

:global(.v-theme--dark) .session-item.active .session-icon {
  background: rgba(var(--v-theme-primary), 0.2);
}
</style>
