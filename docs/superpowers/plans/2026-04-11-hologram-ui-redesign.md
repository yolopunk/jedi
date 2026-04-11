# Hologram UI Redesign Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Redesign AI Chat interface with holographic terminal aesthetic - blue glow, frosted glass, unified Star Wars inspired style.

**Architecture:** CSS cleanup first (remove duplicates), then progressively update Header → Input Area → Message Bubble → Model Dropdown. Vue template updates accompany CSS changes.

**Tech Stack:** Vue 3, CSS (no preprocessor), TypeScript

---

## File Map

| File | Responsibility |
|------|----------------|
| `src/views/AiChat/chat.css` | All hologram styles, remove duplicates |
| `src/views/AiChat/index.vue` | Header template, input area HTML/JS |
| `src/views/AiChat/ModelSettings.vue` | Provider display styling (if needed) |

---

## Task 1: CSS Cleanup — Remove Duplicate Styles

**Files:**
- Modify: `src/views/AiChat/chat.css`

- [ ] **Step 1: Identify duplicate `.input-console` definitions**

Read lines 730-740 and lines 1430-1445. Keep the version at ~1430 (newer AI chat style). Delete the version at ~730.

- [ ] **Step 2: Identify duplicate `.input-row` definitions**

Read lines 738-745 and lines 1444-1450. Keep the version at ~1444. Delete the version at ~738.

- [ ] **Step 3: Identify duplicate `.console-input` definitions**

Read lines 761-783 and lines 1485-1507. Keep the version at ~1485. Delete the version at ~761.

- [ ] **Step 4: Identify duplicate `.send-btn` definitions**

Read lines 809-818 and lines 1581-1605. Keep the version at ~1581. Delete the version at ~809.

- [ ] **Step 5: Check for other duplicate toolbar/slash-btn styles**

Grep for `.slash-btn` — this old button style may conflict with new `.toolbar-btn`.

- [ ] **Step 6: Commit**

```bash
git add src/views/AiChat/chat.css
git commit -m "refactor: remove duplicate CSS definitions in chat.css"
```

---

## Task 2: Add Hologram Foundation Variables

**Files:**
- Modify: `src/views/AiChat/chat.css`

- [ ] **Step 1: Read current CSS file start to find where to add variables**

Read `src/views/AiChat/chat.css` lines 1-30.

- [ ] **Step 2: Add CSS custom properties at file start**

After line 1 comment block, add:

```css
:root {
  /* Hologram Color Palette */
  --hologram-bg: #0a0a0f;
  --hologram-primary: #00d4ff;
  --hologram-dark: #0891b2;
  --hologram-glow: rgba(0, 212, 255, 0.3);
  --hologram-glow-strong: rgba(0, 212, 255, 0.5);
  --hologram-surface: rgba(0, 20, 40, 0.6);
  --hologram-surface-light: rgba(0, 20, 40, 0.8);
  --hologram-text: #e0f7ff;
  --hologram-text-muted: #64748b;

  /* Effects */
  --hologram-blur: blur(12px);
  --hologram-shadow: 0 0 20px rgba(0, 212, 255, 0.3), inset 0 0 20px rgba(0, 212, 255, 0.05);

  /* Border Radius */
  --radius-card: 12px;
  --radius-button: 10px;
  --radius-input: 10px;
}
```

- [ ] **Step 3: Commit**

```bash
git add src/views/AiChat/chat.css
git commit -m "feat: add hologram CSS custom properties"
```

---

## Task 3: Update Header with Hologram Styles

**Files:**
- Modify: `src/views/AiChat/chat.css`
- Modify: `src/views/AiChat/index.vue`

- [ ] **Step 1: Read current header template**

Read `src/views/AiChat/index.vue` lines 15-35 to see current header structure.

- [ ] **Step 2: Add `.chat-header` hologram styles to chat.css**

Find where `.chat-header` is defined. Replace with:

```css
.chat-header {
  height: 48px;
  padding: 0 16px;
  background: var(--hologram-surface-light);
  border-bottom: 1px solid rgba(0, 212, 255, 0.2);
  box-shadow: 0 2px 20px rgba(0, 212, 255, 0.1);
  backdrop-filter: var(--hologram-blur);
  display: flex;
  align-items: center;
  justify-content: space-between;
}
```

- [ ] **Step 3: Add `.header-logo` styles**

Add after `.chat-header`:

```css
.header-logo {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 11px;
  color: var(--hologram-text-muted);
}

.header-logo .menu-icon {
  color: var(--hologram-primary);
}

.header-logo .holocron {
  color: var(--hologram-primary);
  text-shadow: 0 0 10px var(--hologram-glow);
}

.header-logo .path {
  color: var(--hologram-text-muted);
}
```

- [ ] **Step 4: Add `.provider-display` styles (currently missing)**

Add after `.header-logo`:

```css
.provider-display {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 6px 12px;
  background: rgba(0, 212, 255, 0.08);
  border: 1px solid rgba(0, 212, 255, 0.25);
  border-radius: 8px;
  cursor: pointer;
  transition: all 0.2s;
}

.provider-display:hover {
  border-color: var(--hologram-glow-strong);
  box-shadow: 0 0 15px rgba(0, 212, 255, 0.2);
}

.provider-label {
  font-size: 10px;
  color: var(--hologram-text-muted);
  letter-spacing: 1px;
}

.provider-name {
  font-size: 12px;
  color: var(--hologram-primary);
  font-weight: 600;
}
```

- [ ] **Step 5: Verify header template has correct class bindings**

Read index.vue lines 25-34. Ensure classes match:
- `.chat-header` on header element
- `.header-logo` on logo div
- `.provider-display` on provider div

- [ ] **Step 6: Commit**

```bash
git add src/views/AiChat/chat.css src/views/AiChat/index.vue
git commit -m "feat: apply hologram styling to header"
```

---

## Task 4: Restructure Input Area Layout

**Files:**
- Modify: `src/views/AiChat/index.vue`
- Modify: `src/views/AiChat/chat.css`

- [ ] **Step 1: Read current input area template**

Read `src/views/AiChat/index.vue` lines 159-228 to see current structure.

- [ ] **Step 2: Replace input area template with new layout**

Replace lines 159-228 with:

```vue
<!-- 输入区域 - 全息终端风格 -->
<div class="input-console">
  <!-- 输入框 -->
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
    <AttachmentMenu
      v-if="showAttachmentMenu"
      @close="showAttachmentMenu = false"
      @select="handleAttachmentSelect"
    />
  </div>

  <!-- 底部操作栏 -->
  <div class="input-actions">
    <!-- 左下角: 工具栏 -->
    <div class="input-toolbar">
      <button class="toolbar-btn" @click="showCommands = !showCommands" title="Commands (/)">
        <span>/</span>
      </button>
      <button class="toolbar-btn" @click="showAttachmentMenu = !showAttachmentMenu" title="Add (attachment, skills, web search)">
        <span>+</span>
      </button>
    </div>

    <!-- 右下角: Model选择 + 发送 -->
    <div class="input-right">
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
</div>
```

- [ ] **Step 3: Add new script variables**

Read `src/views/AiChat/index.vue` around lines 270-300. Add to existing reactive variables:

```typescript
const showAttachmentMenu = ref(false)
const showModelDropdown = ref(false)
const showCommands = ref(false)

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

- [ ] **Step 4: Replace input area CSS with hologram styles**

Find `.input-console` in chat.css and replace with:

```css
.input-console {
  padding: 16px 20px;
  background: var(--hologram-surface);
  border-top: 1px solid rgba(0, 212, 255, 0.15);
  backdrop-filter: var(--hologram-blur);
  position: relative;
  z-index: 2;
}
```

- [ ] **Step 5: Add/replace `.input-wrapper` styles**

Find or add `.input-wrapper`:

```css
.input-wrapper {
  position: relative;
  width: 100%;
}
```

- [ ] **Step 6: Add `.input-actions` and `.input-toolbar` styles**

Add after `.input-wrapper`:

```css
.input-actions {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-top: 8px;
}

.input-toolbar {
  display: flex;
  gap: 8px;
}

.input-right {
  display: flex;
  align-items: center;
  gap: 8px;
}
```

- [ ] **Step 7: Replace `.toolbar-btn` styles**

Find `.toolbar-btn` in chat.css and replace with:

```css
.toolbar-btn {
  width: 36px;
  height: 36px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(0, 212, 255, 0.05);
  border: 1px solid rgba(0, 212, 255, 0.2);
  border-radius: 8px;
  color: rgba(0, 212, 255, 0.8);
  font-size: 16px;
  cursor: pointer;
  transition: all 0.15s;
}

.toolbar-btn:hover {
  background: rgba(0, 212, 255, 0.12);
  border-color: rgba(0, 212, 255, 0.4);
  box-shadow: 0 0 15px rgba(0, 212, 255, 0.2);
  color: var(--hologram-primary);
}
```

- [ ] **Step 8: Replace `.console-input` with hologram styles**

Find `.console-input` and replace with:

```css
.console-input {
  width: 100%;
  min-height: 44px;
  max-height: 160px;
  padding: 10px 16px;
  background: rgba(0, 10, 20, 0.6);
  border: 1px solid rgba(0, 212, 255, 0.15);
  border-radius: 10px;
  color: var(--hologram-text);
  font-size: 14px;
  font-family: inherit;
  resize: none;
  outline: none;
  transition: border-color 0.15s, box-shadow 0.15s;
}

.console-input:focus {
  border-color: rgba(0, 212, 255, 0.4);
  box-shadow: 0 0 20px rgba(0, 212, 255, 0.15),
              inset 0 0 10px rgba(0, 212, 255, 0.05);
}

.console-input::placeholder {
  color: rgba(255, 255, 255, 0.3);
}
```

- [ ] **Step 9: Replace `.model-selector` and `.model-dropdown-btn` styles**

Find and replace:

```css
.model-selector {
  position: relative;
}

.model-dropdown-btn {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 8px 14px;
  background: rgba(0, 212, 255, 0.1);
  border: 1px solid rgba(0, 212, 255, 0.3);
  border-radius: 8px;
  color: var(--hologram-primary);
  font-size: 12px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.15s;
}

.model-dropdown-btn:hover {
  background: rgba(0, 212, 255, 0.15);
  border-color: rgba(0, 212, 255, 0.5);
  box-shadow: 0 0 15px rgba(0, 212, 255, 0.25);
}
```

- [ ] **Step 10: Replace `.send-btn` styles**

Find `.send-btn` and replace with:

```css
.send-btn {
  width: 44px;
  height: 44px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: linear-gradient(135deg, rgba(0, 212, 255, 0.2), rgba(0, 212, 255, 0.1));
  border: 1px solid rgba(0, 212, 255, 0.4);
  border-radius: 10px;
  color: var(--hologram-primary);
  font-size: 18px;
  cursor: pointer;
  transition: all 0.15s;
  box-shadow: 0 0 15px rgba(0, 212, 255, 0.2);
}

.send-btn:hover:not(.disabled) {
  background: linear-gradient(135deg, rgba(0, 212, 255, 0.3), rgba(0, 212, 255, 0.15));
  box-shadow: 0 0 25px rgba(0, 212, 255, 0.4);
}

.send-btn.disabled {
  opacity: 0.4;
  cursor: not-allowed;
}
```

- [ ] **Step 11: Commit**

```bash
git add src/views/AiChat/index.vue src/views/AiChat/chat.css
git commit -m "feat: restructure input area with hologram UI"
```

---

## Task 5: Add Message Bubble Hover Metadata

**Files:**
- Modify: `src/views/AiChat/index.vue`
- Modify: `src/views/AiChat/chat.css`

- [ ] **Step 1: Read current message template**

Read `src/views/AiChat/index.vue` lines 66-130 to see message structure.

- [ ] **Step 2: Add metadata row to message template**

Find the message content area and add a metadata row that shows on hover. The structure should be inside `.console-message`:

```vue
<div class="console-message" :class="message.role">
  <!-- 时间戳 -->
  <div class="message-timestamp">
    <span class="timestamp">[{{ formatTimestamp(message.timestamp) }}]</span>
  </div>

  <div class="message-row">
    <!-- Avatar -->
    ...
    <!-- 消息内容 -->
    <div class="message-content">
      <div class="message-text">{{ message.content }}</div>
      <!-- Hover 元数据 -->
      <div class="message-meta">
        <span class="meta-item model-tag">
          <span class="meta-label">MODEL:</span>
          <span class="meta-value">{{ message.model || 'N/A' }}</span>
        </span>
        <span class="meta-item copy-btn" @click="copyMessage(message.content)">
          <svg width="12" height="12" viewBox="0 0 24 24" fill="none">
            <rect x="9" y="9" width="13" height="13" rx="2" stroke="currentColor" stroke-width="2"/>
            <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1" stroke="currentColor" stroke-width="2"/>
          </svg>
          复制
        </span>
        <span class="meta-item retry-btn" @click="retryMessage(message)">
          <svg width="12" height="12" viewBox="0 0 24 24" fill="none">
            <path d="M1 4v6h6M23 20v-6h-6" stroke="currentColor" stroke-width="2"/>
            <path d="M20.49 9A9 9 0 0 0 5.64 5.64L1 10m22 4l-4.64 4.36A9 9 0 0 1 3.51 15" stroke="currentColor" stroke-width="2"/>
          </svg>
          重试
        </span>
        <span class="meta-item time">{{ formatTimestamp(message.timestamp) }}</span>
      </div>
    </div>
  </div>
</div>
```

- [ ] **Step 3: Add message metadata CSS**

Add to chat.css:

```css
.message-meta {
  display: none;
  flex-wrap: wrap;
  gap: 8px;
  padding: 8px 0;
  opacity: 0.6;
  font-size: 11px;
  color: var(--hologram-text-muted);
  transition: opacity 0.15s;
}

.console-message:hover .message-meta {
  display: flex;
  opacity: 1;
}

.meta-item {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  padding: 4px 8px;
  background: rgba(0, 212, 255, 0.08);
  border: 1px solid rgba(0, 212, 255, 0.15);
  border-radius: 6px;
  cursor: pointer;
  transition: all 0.15s;
}

.meta-item:hover {
  background: rgba(0, 212, 255, 0.15);
  border-color: rgba(0, 212, 255, 0.3);
  color: var(--hologram-primary);
}

.model-tag .meta-label {
  color: var(--hologram-text-muted);
}

.model-tag .meta-value {
  color: var(--hologram-primary);
  font-weight: 600;
}

.meta-item.time {
  cursor: default;
}

.meta-item.time:hover {
  background: rgba(0, 212, 255, 0.08);
  border-color: rgba(0, 212, 255, 0.15);
  color: var(--hologram-text-muted);
}
```

- [ ] **Step 4: Add copy and retry handler functions to script**

Read the script section and add:

```typescript
function copyMessage(content: string) {
  navigator.clipboard.writeText(content)
}

function retryMessage(message: any) {
  // Re-send the message - implement based on store structure
  store.sendMessage(message.content)
}
```

- [ ] **Step 5: Commit**

```bash
git add src/views/AiChat/index.vue src/views/AiChat/chat.css
git commit -m "feat: add message bubble hover metadata"
```

---

## Task 6: Polish Model Dropdown Menu

**Files:**
- Modify: `src/views/AiChat/chat.css`

- [ ] **Step 1: Find and replace `.model-dropdown-menu` styles**

Find `.model-dropdown-menu` and replace with:

```css
.model-dropdown-menu {
  position: absolute;
  bottom: 100%;
  right: 0;
  margin-bottom: 8px;
  min-width: 240px;
  max-height: 300px;
  overflow-y: auto;
  background: rgba(0, 20, 40, 0.95);
  border: 1px solid rgba(0, 212, 255, 0.25);
  border-radius: 10px;
  padding: 6px;
  backdrop-filter: blur(12px);
  box-shadow: 0 0 30px rgba(0, 212, 255, 0.2),
              0 8px 32px rgba(0, 0, 0, 0.4);
  z-index: 100;
}

.model-dropdown-menu::-webkit-scrollbar {
  width: 6px;
}

.model-dropdown-menu::-webkit-scrollbar-track {
  background: transparent;
}

.model-dropdown-menu::-webkit-scrollbar-thumb {
  background: rgba(0, 212, 255, 0.2);
  border-radius: 3px;
}
```

- [ ] **Step 2: Find and replace `.model-dropdown-item` styles**

Replace with:

```css
.model-dropdown-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 10px 12px;
  border-radius: 8px;
  cursor: pointer;
  transition: background 0.15s;
  margin-bottom: 2px;
}

.model-dropdown-item:hover {
  background: rgba(0, 212, 255, 0.1);
}

.model-dropdown-item.selected {
  background: rgba(0, 212, 255, 0.15);
  border: 1px solid rgba(0, 212, 255, 0.3);
}

.model-item-name {
  font-size: 13px;
  color: var(--hologram-text);
}

.model-item-context {
  font-size: 10px;
  color: var(--hologram-text-muted);
  padding: 2px 6px;
  background: rgba(255, 255, 255, 0.08);
  border-radius: 4px;
}
```

- [ ] **Step 3: Add selected checkmark indicator**

Add to `.model-dropdown-item.selected::after` or as a span:

```css
.model-dropdown-item.selected .model-item-name::after {
  content: ' ✓';
  color: var(--hologram-primary);
}
```

- [ ] **Step 4: Commit**

```bash
git add src/views/AiChat/chat.css
git commit -m "feat: polish model dropdown with hologram styling"
```

---

## Task 7: Final Cleanup and Light Theme Update

**Files:**
- Modify: `src/views/AiChat/chat.css`

- [ ] **Step 1: Search for any remaining old terminal styles that conflict**

Grep for `60a5fa` (old blue) and `#3b82f6` (old gradient) in chat.css to find remaining old styles.

- [ ] **Step 2: Update light theme section**

Find `.light-theme` section (~line 1329) and update to match hologram palette.

- [ ] **Step 3: Commit**

```bash
git add src/views/AiChat/chat.css
git commit -m "refactor: update light theme to match hologram palette"
```

---

## Task 8: Verification

**Files:**
- Verify: `src/views/AiChat/index.vue`
- Verify: `src/views/AiChat/chat.css`

- [ ] **Step 1: Run build**

```bash
pnpm build
```

Expected: Build succeeds

- [ ] **Step 2: Run cargo check**

```bash
cd src-tauri && cargo check
```

Expected: No errors

- [ ] **Step 3: Commit all remaining changes**

```bash
git add -A
git commit -m "feat: complete hologram UI redesign"
```

---

## Self-Review Checklist

- [ ] All hologram colors use CSS custom properties
- [ ] No duplicate CSS class definitions remain
- [ ] Input area has correct layout (toolbar left, model+send right)
- [ ] Message hover shows metadata row
- [ ] Model dropdown has frosted glass effect
- [ ] Header has `.provider-display` with hologram styling
- [ ] Light theme updated to match
