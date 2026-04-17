<template>
  <v-dialog
    :model-value="modelValue"
    @update:model-value="$emit('update:modelValue', $event)"
    max-width="520"
    persistent
  >
    <v-card class="model-settings-card">
      <!-- Header -->
      <div class="card-header">
        <div class="header-brand">
          <div class="brand-icon">
            <svg width="20" height="20" viewBox="0 0 24 24" fill="none">
              <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z" stroke="currentColor" stroke-width="1.5"/>
              <line x1="12" y1="11" x2="12" y2="17" stroke="currentColor" stroke-width="2"/>
              <line x1="9" y1="14" x2="15" y2="14" stroke="currentColor" stroke-width="2"/>
            </svg>
          </div>
          <div class="brand-text">
            <h2>Add Group</h2>
            <p>Create a new host group</p>
          </div>
        </div>
        <button class="close-btn" @click="closeDialog">
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none">
            <line x1="18" y1="6" x2="6" y2="18" stroke="currentColor" stroke-width="2"/>
            <line x1="6" y1="6" x2="18" y2="18" stroke="currentColor" stroke-width="2"/>
          </svg>
        </button>
      </div>

      <!-- Content -->
      <div class="card-body">
        <div class="form-group">
          <label class="form-label">
            <svg width="12" height="12" viewBox="0 0 24 24" fill="none">
              <path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2" stroke="currentColor" stroke-width="2"/>
              <circle cx="12" cy="7" r="4" stroke="currentColor" stroke-width="2"/>
            </svg>
            Group Name
          </label>
          <div class="input-wrapper">
            <span class="input-prefix">NAME</span>
            <input
              v-model="groupName"
              type="text"
              class="form-input with-prefix"
              placeholder="My Server Group"
              @keyup.enter="confirmAdd"
            />
          </div>
        </div>

        <div class="toggle-row">
          <div class="toggle-info">
            <span class="toggle-label">Use Remote Config</span>
            <span class="toggle-hint">Load hosts from a remote URL</span>
          </div>
          <button
            class="toggle-switch"
            :class="{ active: isRemote }"
            @click="isRemote = !isRemote"
          >
            <span class="toggle-handle"></span>
          </button>
        </div>

        <v-expand-transition>
          <div v-if="isRemote" class="form-group">
            <label class="form-label">
              <svg width="12" height="12" viewBox="0 0 24 24" fill="none">
                <path d="M10 13a5 5 0 0 0 7.54.54l3-3a5 5 0 0 0-7.07-7.07l-1.72 1.71" stroke="currentColor" stroke-width="2"/>
                <path d="M14 11a5 5 0 0 0-7.54-.54l-3 3a5 5 0 0 0 7.07 7.07l1.71-1.71" stroke="currentColor" stroke-width="2"/>
              </svg>
              Remote URL
            </label>
            <div class="input-wrapper">
              <span class="input-prefix">URL</span>
              <input
                v-model="remoteUrl"
                type="text"
                class="form-input with-prefix"
                placeholder="https://example.com/hosts.json"
                @keyup.enter="confirmAdd"
              />
            </div>
          </div>
        </v-expand-transition>

        <v-expand-transition>
          <div v-if="!isRemote" class="form-group">
            <label class="form-label">
              <svg width="12" height="12" viewBox="0 0 24 24" fill="none">
                <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z" stroke="currentColor" stroke-width="1.5"/>
                <polyline points="14 2 14 8 20 8" stroke="currentColor" stroke-width="1.5"/>
              </svg>
              Hosts Content
            </label>
            <textarea
              v-model="hostsContent"
              class="form-textarea"
              rows="4"
              placeholder="192.168.1.1 example.com&#10;10.0.0.1 api.example.com"
            ></textarea>
            <div class="input-hint">One entry per line: IP followed by domain, separated by space</div>
          </div>
        </v-expand-transition>
      </div>

      <!-- Footer -->
      <div class="card-footer">
        <v-btn variant="text" @click="closeDialog">Cancel</v-btn>
        <v-spacer />
        <v-btn variant="tonal" color="primary" @click="confirmAdd">Add Group</v-btn>
      </div>
    </v-card>
  </v-dialog>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { fetchRemoteConfig } from '@/api/hosts'
import type { HostEntry } from '@/types/hosts'

defineProps<{ modelValue: boolean }>()
const emit = defineEmits<{
  (e: 'update:modelValue', value: boolean): void
  (e: 'add', data: { name: string; isRemote: boolean; url?: string; hosts?: HostEntry[] }): void
  (e: 'error', message: string): void
}>()

const groupName = ref('')
const isRemote = ref(false)
const remoteUrl = ref('')
const hostsContent = ref('')

function closeDialog() {
  emit('update:modelValue', false)
  resetForm()
}

function resetForm() {
  groupName.value = ''
  isRemote.value = false
  remoteUrl.value = ''
  hostsContent.value = ''
}

async function _confirmAdd() {
  if (!groupName.value) {
    emit('error', 'Group name cannot be empty')
    return
  }

  if (isRemote.value) {
    if (!remoteUrl.value) {
      emit('error', 'Remote URL cannot be empty')
      return
    }
    try {
      const result = await fetchRemoteConfig(remoteUrl.value)
      if (!Array.isArray(result)) {
        emit('error', 'Invalid remote config format')
        return
      }
      const matchedGroup = result.find((g: any) => g.name === groupName.value)
      if (matchedGroup) {
        emit('add', {
          name: groupName.value,
          isRemote: true,
          url: remoteUrl.value,
          hosts: matchedGroup.hosts,
        })
        closeDialog()
      } else {
        emit('error', 'Group not found in remote config')
      }
    } catch (error) {
      emit('error', `Fetch failed: ${(error as Error).message}`)
    }
  } else {
    const parsedLines = hostsContent.value
      .split('\n')
      .map(line => line.trim())
      .filter(line => line.length > 0)
      .map(line => {
        const parts = line.split(/\s+/)
        if (parts.length >= 2) {
          return { domain: parts[1], ip: parts[0] }
        }
        return null
      })
      .filter(item => item !== null)

    const uniqueDomains = new Set()
    const hostsArray: HostEntry[] = parsedLines
      .filter(item => {
        if (item && !uniqueDomains.has(item.domain)) {
          uniqueDomains.add(item.domain)
          return true
        }
        return false
      })
      .map(item => ({
        domain: item?.domain,
        ip: item?.ip,
        disabled: false,
      }))

    emit('add', {
      name: groupName.value,
      isRemote: false,
      hosts: hostsArray,
    })
    closeDialog()
  }
}
</script>

<style scoped>
.model-settings-card {
  background: #0a0e14 !important;
  border-radius: 16px !important;
  overflow: hidden;
  max-height: 85vh;
  display: flex;
  flex-direction: column;
}

.card-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px 20px;
  background: rgba(20, 30, 40, 0.6);
  border-bottom: 1px solid rgba(255, 255, 255, 0.06);
  flex-shrink: 0;
}

.header-brand {
  display: flex;
  align-items: center;
  gap: 12px;
}

.brand-icon {
  width: 40px;
  height: 40px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: linear-gradient(135deg, rgba(0, 255, 255, 0.15) 0%, rgba(0, 255, 136, 0.05) 100%);
  border: 1px solid rgba(0, 255, 255, 0.25);
  border-radius: 10px;
  color: #00ffff;
}

.brand-text h2 {
  margin: 0;
  font-size: 16px;
  font-weight: 600;
  color: #ffffff;
}

.brand-text p {
  margin: 2px 0 0;
  font-size: 12px;
  color: rgba(255, 255, 255, 0.4);
}

.close-btn {
  width: 36px;
  height: 36px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: transparent;
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 8px;
  color: rgba(255, 255, 255, 0.5);
  cursor: pointer;
  transition: all 0.2s ease;
}

.close-btn:hover {
  background: rgba(255, 107, 107, 0.1);
  border-color: rgba(255, 107, 107, 0.3);
  color: #ff6b6b;
}

.card-body {
  flex: 1;
  overflow-y: auto;
  padding: 20px;
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.form-group {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.form-label {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 11px;
  font-weight: 700;
  letter-spacing: 1px;
  color: rgba(255, 255, 255, 0.5);
  text-transform: uppercase;
}

.form-label svg {
  color: #00ff88;
}

.input-wrapper {
  display: flex;
  align-items: center;
  background: rgba(0, 255, 255, 0.03);
  border: 1px solid rgba(0, 255, 255, 0.12);
  border-radius: 10px;
  overflow: hidden;
  transition: border-color 0.15s;
}

.input-wrapper:focus-within {
  border-color: rgba(0, 255, 255, 0.4);
  box-shadow: 0 0 0 2px rgba(0, 255, 255, 0.1);
}

.input-prefix {
  padding: 12px 14px;
  font-size: 10px;
  font-weight: 700;
  letter-spacing: 1px;
  color: rgba(0, 255, 255, 0.6);
  background: rgba(0, 255, 255, 0.05);
  border-right: 1px solid rgba(0, 255, 255, 0.12);
  min-width: 64px;
  text-align: center;
}

.form-input {
  flex: 1;
  padding: 12px 14px;
  background: transparent;
  border: none;
  outline: none;
  color: #ffffff;
  font-family: 'JetBrains Mono', monospace;
  font-size: 13px;
}

.form-input::placeholder {
  color: rgba(255, 255, 255, 0.2);
}

.form-input.with-prefix {
  padding-left: 12px;
}

.form-textarea {
  width: 100%;
  padding: 12px 14px;
  background: rgba(0, 255, 255, 0.03);
  border: 1px solid rgba(0, 255, 255, 0.12);
  border-radius: 10px;
  color: #ffffff;
  font-family: 'JetBrains Mono', monospace;
  font-size: 13px;
  resize: vertical;
  outline: none;
  transition: border-color 0.15s;
  box-sizing: border-box;
}

.form-textarea:focus {
  border-color: rgba(0, 255, 255, 0.4);
  box-shadow: 0 0 0 2px rgba(0, 255, 255, 0.1);
}

.form-textarea::placeholder {
  color: rgba(255, 255, 255, 0.2);
}

.input-hint {
  font-size: 11px;
  color: rgba(255, 255, 255, 0.3);
  margin-top: 4px;
}

.toggle-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 14px;
  background: rgba(255, 255, 255, 0.02);
  border: 1px solid rgba(255, 255, 255, 0.06);
  border-radius: 10px;
}

.toggle-info {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.toggle-label {
  font-size: 13px;
  font-weight: 500;
  color: rgba(255, 255, 255, 0.8);
}

.toggle-hint {
  font-size: 11px;
  color: rgba(255, 255, 255, 0.35);
}

.toggle-switch {
  width: 44px;
  height: 24px;
  background: rgba(255, 255, 255, 0.08);
  border: 1px solid rgba(255, 255, 255, 0.12);
  border-radius: 12px;
  cursor: pointer;
  position: relative;
  transition: all 0.2s ease;
}

.toggle-switch.active {
  background: rgba(0, 255, 136, 0.15);
  border-color: rgba(0, 255, 136, 0.4);
}

.toggle-handle {
  position: absolute;
  top: 2px;
  left: 2px;
  width: 18px;
  height: 18px;
  background: rgba(255, 255, 255, 0.5);
  border-radius: 50%;
  transition: all 0.2s ease;
}

.toggle-switch.active .toggle-handle {
  left: 22px;
  background: #00ff88;
}

.card-footer {
  display: flex;
  align-items: center;
  padding: 16px 20px;
  border-top: 1px solid rgba(255, 255, 255, 0.06);
  flex-shrink: 0;
}

::-webkit-scrollbar {
  width: 4px;
}

::-webkit-scrollbar-track {
  background: transparent;
}

::-webkit-scrollbar-thumb {
  background: rgba(255, 255, 255, 0.1);
  border-radius: 2px;
}

::-webkit-scrollbar-thumb:hover {
  background: rgba(255, 255, 255, 0.15);
}
</style>
