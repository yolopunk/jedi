# Chat Model Selection Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Enable chat with selected provider/model after configuring API key. Add model dropdown in input area, fix startup loading, persist last selection.

**Architecture:** Provider/model selection via modelsDev store, session uses provider/model from modelsDev store instead of hardcoded values. Persistence via localStorage.

**Tech Stack:** Vue 3, Pinia, Tauri v2, TypeScript

---

## Task 1: Fix Startup Loading of Configured Providers

**Files:**
- Modify: `src/views/AiChat/index.vue:445-449`

**Changes:**

In `onMounted`, add `providerConfigStore.loadConfiguredProviders()` alongside `modelsDevStore.fetchProviders()`.

```typescript
// src/views/AiChat/index.vue:445-449
onMounted(async () => {
  skillsStore.loadFromStorage()
  await Promise.all([
    modelsDevStore.fetchProviders(),
    providerConfigStore.loadConfiguredProviders()  // ADD THIS
  ])
  scrollToBottom()
  // ... rest
})
```

- [ ] **Step 1: Add loadConfiguredProviders call**

Read `src/views/AiChat/index.vue` lines 445-459, find `onMounted`, add `providerConfigStore.loadConfiguredProviders()` to the Promise.all.

- [ ] **Step 2: Commit**

```bash
git add src/views/AiChat/index.vue
git commit -m "fix: load configured providers at startup"
```

---

## Task 2: Add Persistence for Last Selected Provider/Model

**Files:**
- Modify: `src/stores/modelsDev.ts:21-30` (state)
- Modify: `src/stores/modelsDev.ts:84-100` (fetchProviders action)
- Modify: `src/stores/modelsDev.ts:106-114` (selectProvider action)
- Modify: `src/stores/modelsDev.ts:116-120` (selectModel action)

**Changes:**

Add `lastSelectedProviderId` and `lastSelectedModelId` refs, persist to localStorage on change, restore on fetchProviders.

```typescript
// In modelsDev store state section (after line 29):
const lastSelectedProviderId = ref<string | null>(null)
const lastSelectedModelId = ref<string | null>(null)

// Add helper functions for localStorage:
function persistSelection() {
  localStorage.setItem('jedi-last-model', JSON.stringify({
    providerId: lastSelectedProviderId.value,
    modelId: lastSelectedModelId.value
  }))
}

function loadPersistedSelection() {
  try {
    const saved = localStorage.getItem('jedi-last-model')
    if (saved) {
      const { providerId, modelId } = JSON.parse(saved)
      lastSelectedProviderId.value = providerId
      lastSelectedModelId.value = modelId
    }
  } catch (e) {
    console.error('Failed to load last model selection:', e)
  }
}

// In fetchProviders (line 84-100):
// ADD at end of try block before catch:
loadPersistedSelection()
if (lastSelectedProviderId.value && !selectedProviderId.value) {
  const provider = providersData.value[lastSelectedProviderId.value]
  if (provider && providerConfigStore.isProviderConfigured(lastSelectedProviderId.value)) {
    selectedProviderId.value = lastSelectedProviderId.value
    if (lastSelectedModelId.value && provider.models[lastSelectedModelId.value]) {
      selectedModelId.value = lastSelectedModelId.value
    }
  }
}

// In selectProvider (line 106-110):
function selectProvider(providerId: string): void {
  selectedProviderId.value = providerId
  lastSelectedProviderId.value = providerId  // ADD
  selectedModelId.value = null  // Clear model when provider changes
  persistSelection()  // ADD
}

// In selectModel (line 112-114):
function selectModel(modelId: string): void {
  selectedModelId.value = modelId
  lastSelectedModelId.value = modelId  // ADD
  persistSelection()  // ADD
}
```

Also add `lastSelectedProviderId` and `lastSelectedModelId` to the return statement.

- [ ] **Step 1: Add state and helper functions**

Read modelsDev.ts lines 21-30, add state refs and persistence helpers.

- [ ] **Step 2: Modify fetchProviders**

Read lines 84-100, add `loadPersistedSelection()` call and auto-select logic.

- [ ] **Step 3: Modify selectProvider and selectModel**

Read lines 106-120, add persistence calls and lastSelected updates.

- [ ] **Step 4: Update return statement**

Add new state vars to return object.

- [ ] **Step 5: Commit**

```bash
git add src/stores/modelsDev.ts
git commit -m "feat: persist last selected provider/model"
```

---

## Task 3: Update Header to Show Provider

**Files:**
- Modify: `src/views/AiChat/index.vue:25-34`
- Modify: `src/views/AiChat/index.vue:296-299`

**Changes:**

Replace model display with provider display in header.

```vue
<!-- Line 25-34: Change header-right content -->
<div class="header-right">
  <div class="status-badge">
    <span class="status-dot"></span>
    <span class="status-text">{{ connectionStatus }}</span>
  </div>
  <div class="provider-display" @click="showModelSettings = true">
    <span class="provider-label">PROVIDER:</span>
    <span class="provider-name">{{ currentProviderName }}</span>
  </div>
</div>

<!-- Line 296-299: Replace currentModelName computed with provider name -->
const currentProviderName = computed(() => {
  return modelsDevStore.selectedProvider?.name?.toUpperCase() || 'SELECT PROVIDER'
})
```

- [ ] **Step 1: Update header template**

Read index.vue lines 25-34, replace `.model-display` with `.provider-display` and update content.

- [ ] **Step 2: Add computed property**

Read lines 296-299, replace/add `currentProviderName` computed.

- [ ] **Step 3: Commit**

```bash
git add src/views/AiChat/index.vue
git commit -m "feat: show provider in header instead of model"
```

---

## Task 4: Rebuild Input Area with Model Dropdown and + Button

**Files:**
- Modify: `src/views/AiChat/index.vue:159-214` (template)
- Modify: `src/views/AiChat/chat.css` (styles)

**Changes:**

Replace the terminal-style input with AI chat style input:

```vue
<!-- Line 159-214: Replace input-console with new layout -->
<!-- 输入区域 - AI Chat 风格 -->
<div class="input-console">
  <div class="input-row">
    <!-- Left toolbar: / and + buttons -->
    <div class="input-toolbar">
      <button class="toolbar-btn" @click="showCommands = !showCommands" title="Commands (/)">
        <span>/</span>
      </button>
      <button class="toolbar-btn" @click="showAttachmentMenu = !showAttachmentMenu" title="Add (attachment, skills, web search)">
        <span>+</span>
      </button>
    </div>

    <!-- Textarea wrapper -->
    <div class="input-wrapper">
      <textarea
        ref="inputRef"
        v-model="inputText"
        class="console-input"
        :placeholder="$t('chat.commandPlaceholder')"
        rows="1"
        @keydown="handleKeydown"
        @input="autoResize"
      ></textarea>
      <CommandPalette
        :visible="showCommands"
        @select="handleCommandSelect"
        @close="showCommands = false"
      />
      <!-- Attachment popup -->
      <AttachmentMenu
        v-if="showAttachmentMenu"
        @close="showAttachmentMenu = false"
        @select="handleAttachmentSelect"
      />
    </div>

    <!-- Right: Model dropdown -->
    <div class="model-selector">
      <button class="model-dropdown-btn" @click="showModelDropdown = !showModelDropdown">
        <span class="model-dropdown-name">{{ currentModelName }}</span>
        <svg width="12" height="12" viewBox="0 0 24 24" fill="none">
          <polyline points="6 9 12 15 18 9" stroke="currentColor" stroke-width="2"/>
        </svg>
      </button>
      <div v-if="showModelDropdown" class="model-dropdown-menu">
        <div
          v-for="model in selectedProviderModels"
          :key="model.id"
          class="model-dropdown-item"
          :class="{ selected: model.id === modelsDevStore.selectedModelId }"
          @click="selectModelFromDropdown(model)"
        >
          <span class="model-item-name">{{ model.name }}</span>
          <span class="model-item-context">{{ formatContextShort(model.limit?.context) }}</span>
        </div>
      </div>
    </div>

    <!-- Send button -->
    <button
      class="send-btn"
      :class="{ disabled: !inputText.trim() || store.isLoading }"
      @click="handleSend"
      :disabled="!inputText.trim() || store.isLoading"
    >
      <span class="send-icon">↑</span>
    </button>
  </div>
</div>
```

Add to script:
```typescript
const showAttachmentMenu = ref(false)
const showModelDropdown = ref(false)

const selectedProviderModels = computed(() => {
  return modelsDevStore.selectedProviderModels
})

function selectModelFromDropdown(model: any) {
  modelsDevStore.selectModel(model.id)
  showModelDropdown.value = false
}

function handleAttachmentSelect(action: string) {
  showAttachmentMenu.value = false
  // Handle: attachment, skills, web-search
}

function formatContextShort(len?: number): string {
  if (!len) return 'N/A'
  if (len >= 1000000) return `${(len / 1000000).toFixed(0)}M`
  if (len >= 1000) return `${(len / 1000).toFixed(0)}K`
  return len.toString()
}
```

Add new reactive vars to `showModelSettings`:
```typescript
const showModelSettings = ref(false)
```

**CSS additions to chat.css:**
```css
/* Input area AI chat style */
.input-console {
  padding: 12px 16px;
  background: rgba(10, 14, 20, 0.95);
  border-top: 1px solid rgba(255, 255, 255, 0.06);
}

.input-row {
  display: flex;
  align-items: flex-end;
  gap: 8px;
}

.input-toolbar {
  display: flex;
  flex-direction: column;
  gap: 4px;
  flex-shrink: 0;
}

.toolbar-btn {
  width: 36px;
  height: 36px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(255, 255, 255, 0.03);
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 8px;
  color: rgba(255, 255, 255, 0.6);
  font-size: 18px;
  cursor: pointer;
  transition: all 0.15s;
}

.toolbar-btn:hover {
  background: rgba(255, 255, 255, 0.06);
  border-color: rgba(255, 255, 255, 0.2);
  color: #ffffff;
}

.input-wrapper {
  flex: 1;
  position: relative;
  display: flex;
  flex-direction: column;
}

.console-input {
  width: 100%;
  min-height: 40px;
  max-height: 200px;
  padding: 10px 14px;
  background: rgba(255, 255, 255, 0.03);
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 12px;
  color: #ffffff;
  font-size: 14px;
  font-family: inherit;
  resize: none;
  outline: none;
  transition: border-color 0.15s;
}

.console-input:focus {
  border-color: rgba(0, 255, 255, 0.4);
}

.console-input::placeholder {
  color: rgba(255, 255, 255, 0.3);
}

.model-selector {
  position: relative;
  flex-shrink: 0;
}

.model-dropdown-btn {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 8px 12px;
  background: rgba(0, 255, 255, 0.08);
  border: 1px solid rgba(0, 255, 255, 0.25);
  border-radius: 8px;
  color: #00ffff;
  font-size: 12px;
  cursor: pointer;
  transition: all 0.15s;
  white-space: nowrap;
}

.model-dropdown-btn:hover {
  background: rgba(0, 255, 255, 0.12);
  border-color: rgba(0, 255, 255, 0.4);
}

.model-dropdown-menu {
  position: absolute;
  bottom: 100%;
  right: 0;
  margin-bottom: 8px;
  min-width: 220px;
  max-height: 280px;
  overflow-y: auto;
  background: #0d1117;
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 10px;
  padding: 6px;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.4);
  z-index: 100;
}

.model-dropdown-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 10px;
  border-radius: 6px;
  cursor: pointer;
  transition: background 0.1s;
}

.model-dropdown-item:hover {
  background: rgba(255, 255, 255, 0.05);
}

.model-dropdown-item.selected {
  background: rgba(0, 255, 255, 0.1);
}

.model-item-name {
  font-size: 13px;
  color: #ffffff;
}

.model-item-context {
  font-size: 10px;
  color: rgba(255, 255, 255, 0.4);
  padding: 2px 6px;
  background: rgba(255, 255, 255, 0.08);
  border-radius: 4px;
}

.send-btn {
  width: 40px;
  height: 40px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(0, 255, 255, 0.15);
  border: 1px solid rgba(0, 255, 255, 0.3);
  border-radius: 10px;
  color: #00ffff;
  font-size: 18px;
  cursor: pointer;
  transition: all 0.15s;
  flex-shrink: 0;
}

.send-btn:hover:not(.disabled) {
  background: rgba(0, 255, 255, 0.25);
  border-color: rgba(0, 255, 255, 0.5);
}

.send-btn.disabled {
  opacity: 0.4;
  cursor: not-allowed;
}
```

- [ ] **Step 1: Replace input-console template**

Read index.vue lines 159-214, replace with new layout.

- [ ] **Step 2: Add new script variables and functions**

Add `showAttachmentMenu`, `showModelDropdown` refs, `selectedProviderModels` computed, and handler functions.

- [ ] **Step 3: Add CSS to chat.css**

Append new CSS for AI chat style input area.

- [ ] **Step 4: Commit**

```bash
git add src/views/AiChat/index.vue src/views/AiChat/chat.css
git commit -m "feat: rebuild input area with model dropdown and toolbar"
```

---

## Task 5: Create Attachment Menu Component

**Files:**
- Create: `src/components/AttachmentMenu.vue`

**Component:**
```vue
<template>
  <div class="attachment-menu">
    <div class="menu-item" @click="$emit('select', 'attachment')">
      <svg width="16" height="16" viewBox="0 0 24 24" fill="none">
        <path d="M21.44 11.05l-9.19 9.19a6 6 0 0 1-8.49-8.49l9.19-9.19a4 4 0 0 1 5.66 5.66l-9.2 9.19a2 2 0 0 1-2.83-2.83l8.49-8.48" stroke="currentColor" stroke-width="2"/>
      </svg>
      <span>Attachment</span>
    </div>
    <div class="menu-item" @click="$emit('select', 'skills')">
      <svg width="16" height="16" viewBox="0 0 24 24" fill="none">
        <polygon points="12 2 15.09 8.26 22 9.27 17 14.14 18.18 21.02 12 17.77 5.82 21.02 7 14.14 2 9.27 8.91 8.26 12 2" stroke="currentColor" stroke-width="2"/>
      </svg>
      <span>Skills</span>
    </div>
    <div class="menu-item" @click="$emit('select', 'web-search')">
      <svg width="16" height="16" viewBox="0 0 24 24" fill="none">
        <circle cx="11" cy="11" r="8" stroke="currentColor" stroke-width="2"/>
        <line x1="21" y1="21" x2="16.65" y2="16.65" stroke="currentColor" stroke-width="2"/>
      </svg>
      <span>Web Search</span>
    </div>
  </div>
</template>

<script setup lang="ts">
defineEmits<{
  (e: 'close'): void
  (e: 'select', action: string): void
}>()
</script>

<style scoped>
.attachment-menu {
  position: absolute;
  bottom: 100%;
  left: 0;
  margin-bottom: 8px;
  min-width: 160px;
  background: #0d1117;
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 10px;
  padding: 6px;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.4);
  z-index: 100;
}

.menu-item {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 10px 12px;
  border-radius: 6px;
  cursor: pointer;
  color: rgba(255, 255, 255, 0.7);
  font-size: 13px;
  transition: all 0.1s;
}

.menu-item:hover {
  background: rgba(255, 255, 255, 0.05);
  color: #ffffff;
}
</style>
```

- [ ] **Step 1: Create AttachmentMenu.vue**

Create the component at `src/components/AttachmentMenu.vue`.

- [ ] **Step 2: Import in index.vue**

Add import and register component.

- [ ] **Step 3: Commit**

```bash
git add src/components/AttachmentMenu.vue
git add src/views/AiChat/index.vue  # for import
git commit -m "feat: add attachment menu component"
```

---

## Task 6: Wire Model Selection to Session

**Files:**
- Modify: `src/views/AiChat/index.vue:369-389` (handleSend)
- Modify: `src/stores/aiChat.ts:106-113` (sendMessage)

**Changes:**

When sending a message, use provider/model from `modelsDevStore` instead of session defaults.

```typescript
// In handleSend (index.vue), before sendMessage:
if (!store.currentSession) {
  await store.createSession(
    '新对话',
    modelsDevStore.selectedProviderId,   // ADD
    modelsDevStore.selectedModelId        // ADD
  )
}
```

- [ ] **Step 1: Update handleSend**

Read lines 369-389, update createSession call to pass selected provider/model.

- [ ] **Step 2: Commit**

```bash
git add src/views/AiChat/index.vue
git commit -m "fix: use selected provider/model when creating session"
```

---

## Task 7: Fix ModelSettings API Key Button Styling

**Files:**
- Modify: `src/views/AiChat/ModelSettings.vue:214-256` (already done in previous session)
- Modify: `src/views/AiChat/ModelSettings.vue:1058-1080` (add CSS)

**Changes:**

Already partially done. Ensure CSS for `.delete-btn` is properly added. Verify the button is styled as a proper red text button with hover state.

- [ ] **Step 1: Verify CSS exists**

Read ModelSettings.vue lines 1058-1080, check `.delete-btn` CSS is present.

- [ ] **Step 2: Commit if changes needed**

```bash
git add src/views/AiChat/ModelSettings.vue
git commit -m "fix: delete key button styling in ModelSettings"
```

---

## Verification

After all tasks:

1. Run `pnpm build` - should succeed
2. Run `cargo check` in src-tauri - should succeed
3. App startup - configured providers should load without opening dialog
4. Model dropdown - should show current provider's models
5. Send message - should use selected provider/model

---

## Dependencies

Tasks must be completed in order. Tasks 1-3 are prerequisites for Tasks 4-6 to work correctly.
