# Frontend Refactoring Plan

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- []`) syntax for tracking.

**Goal:** Refactor the Vue 3 frontend to eliminate code smells — massive CSS duplication, god components, TypeScript gaps, dead code, and inconsistent patterns — without changing any functionality or design.

**Architecture:** Six incremental chunks, each independently shippable. Chunks are ordered by risk (lowest first) and impact (highest first). No chunk changes runtime behavior.

**Tech Stack:** Vue 3 + TypeScript + Vuetify 3 + Vite

---

## Chunk 1: Dead Code Removal & Quick Wins

*Risk: Zero. These are unused files, refs, and CSS.*

### Task 1.1: Delete dead files

- [ ] Delete `src/composables/useAiChat.ts` (204 lines, never imported, all TODO stubs)
- [ ] Delete `src/components/hosts/dialogs/RenameGroupDialog.vue` (empty placeholder `<div></div>`)

### Task 1.2: Remove unused refs and timers

**File:** `src/components/common/SystemInfoBar.vue`
- [ ] Remove `now` ref (line 288) and `timeTimer` (lines 285, 340-342) — `now` is never used in template

**File:** `src/components/hosts/tables/HostsTable.vue`
- [ ] Remove `currentTimestamp` ref (line 252) and `updateTimestamp` function (lines 283-289) — never displayed

### Task 1.3: Remove dead CSS classes

**File:** `src/components/hosts/tables/HostsTable.vue`
- [ ] Remove `.console-header`, `.console-title-bar`, `.console-status-indicators`, `.status-light` CSS (lines 419-503) — never referenced in template

### Task 1.4: Fix `@ts-ignore` on Tauri imports

- [ ] In `src/api/app.ts:1`, `src/api/hosts.ts:5`, `src/api/ai-chat.ts:5`, `src/composables/useStorage.ts:1` — remove `// @ts-ignore` lines (check if current Tauri v2 types resolve correctly; if not, add a declaration in `src/types/tauri.d.ts`)

### Task 1.5: Commit

```bash
git add -A
git commit -m "refactor: remove dead code, unused refs, and @ts-ignore suppressions"
```

---

## Chunk 2: Extract Shared CSS into Utility Stylesheets

*Risk: Low. Moving CSS from components to shared files; selectors and values stay identical. Visual regression test by comparing screenshots.*

This chunk eliminates ~5000 lines of duplicated CSS across 15+ files.

### Task 2.1: Create `src/assets/styles/console-ui.css`

Extract these duplicated patterns into one file:

**CRT Effects** (duplicated in 8 files: HostsManager, PodcastManager, WallpaperManager, AiChat/index, HostsTable):
```css
/* .scanlines */
.scanlines { ... }
/* .crt-vignette */
.crt-vignette { ... }
/* Light theme variants */
.light-theme .scanlines { ... }
.light-theme .crt-vignette { ... }
```

**Console Buttons** (duplicated in 5+ files: HostsManager, PodcastManager, WallpaperManager, HostsTable, SettingsDialog):
```css
.console-btn { ... }
.console-btn:hover { ... }
.console-btn.primary { ... }
.light-theme .console-btn { ... }
```

**Console Input** (duplicated in 4+ files: HostsManager, PodcastManager, WallpaperManager, HostsTable):
```css
.input-wrapper { ... }
.input-prompt { ... }
.console-input { ... }
.light-theme .input-wrapper { ... }
.light-theme .console-input { ... }
```

**Status Indicators** (duplicated in 4+ files):
```css
.status-light { ... }
.status-light.online { ... }
.status-light.standby { ... }
.status-light.scanning { ... }
```

**Grid Background** (duplicated in 5+ files):
```css
.grid-background { ... }
.light-theme .grid-background { ... }
```

**Empty State** (duplicated in 3+ files):
```css
.empty-state { ... }
.empty-state-icon { ... }
.light-theme .empty-state { ... }
```

**Loading Skeleton** (duplicated in 3+ files):
```css
.loading-grid { ... }
.skeleton-item { ... }
```

- [ ] Create `src/assets/styles/console-ui.css` with all extracted styles
- [ ] Import in `src/main.ts`: `import '@/assets/styles/console-ui.css'`

### Task 2.2: Create `src/assets/styles/console-dialog.css`

Extract duplicated dialog CSS from: EditHostDialog, AddGroupDialog, AddHostDialog, DeleteConfirmDialog:
```css
.dialog-with-glow { ... }
.dialog-decorator { ... }
.dialog-icon { ... }
.dialog-title { ... }
.close-btn { ... }
.form-section { ... }
.input-label { ... }
.label-icon { ... }
.console-title-bar { ... }
.light-theme .dialog-with-glow { ... }
/* etc. */
```

- [ ] Create `src/assets/styles/console-dialog.css`
- [ ] Import in `src/main.ts`

### Task 2.3: Remove duplicated CSS from source components

For each file that had styles extracted:
- [ ] `src/views/hosts/HostsManager.vue` — remove scanlines, CRT, console-btn, input-wrapper, grid-background CSS
- [ ] `src/views/podcast/PodcastManager.vue` — remove same patterns (~300 lines)
- [ ] `src/views/wallpapers/WallpaperManager.vue` — remove same patterns (~300 lines)
- [ ] `src/views/AiChat/index.vue` — remove same patterns (~300 lines)
- [ ] `src/components/hosts/tables/HostsTable.vue` — remove same patterns (~400 lines)
- [ ] `src/components/dialogs/SettingsDialog.vue` — remove console-btn CSS
- [ ] `src/components/hosts/dialogs/EditHostDialog.vue` — remove dialog CSS
- [ ] `src/components/hosts/dialogs/AddGroupDialog.vue` — remove dialog CSS
- [ ] `src/components/hosts/dialogs/AddHostDialog.vue` — remove dialog CSS
- [ ] `src/components/hosts/dialogs/DeleteConfirmDialog.vue` — remove dialog CSS

### Task 2.4: Extract duplicated scrollbar CSS

The same custom scrollbar CSS is in 10+ files. Create a shared rule:
```css
/* src/assets/styles/scrollbar.css */
.custom-scrollbar::-webkit-scrollbar { width: 4px; }
.custom-scrollbar::-webkit-scrollbar-track { background: transparent; }
.custom-scrollbar::-webkit-scrollbar-thumb { background: rgba(0, 255, 255, 0.2); border-radius: 2px; }
/* light theme variant */
```

- [ ] Create `src/assets/styles/scrollbar.css`
- [ ] Replace duplicated scrollbar CSS in all files with `class="custom-scrollbar"`

### Task 2.5: Commit

```bash
git add -A
git commit -m "refactor: extract shared CSS into console-ui.css and console-dialog.css"
```

---

## Chunk 3: Extract Shared Components & Composables

*Risk: Low-Medium. Creating new components/composables, then wiring existing views to use them. Behavior stays identical.*

### Task 3.1: Create `CrtOverlay.vue` component

Replace duplicated CRT template markup in 5+ views:
```vue
<!-- src/components/common/CrtOverlay.vue -->
<template>
  <div class="scanlines"></div>
  <div class="crt-vignette"></div>
</template>
```

- [ ] Create `src/components/common/CrtOverlay.vue`
- [ ] Replace CRT markup in: HostsManager, PodcastManager, WallpaperManager, AiChat/index, HostsTable

### Task 3.2: Create `useSnackbar()` composable

Unify the 3 different snackbar patterns:
```typescript
// src/composables/useSnackbar.ts
export function useSnackbar() {
  const show = ref(false)
  const text = ref('')
  const color = ref<'success' | 'error' | 'info' | 'warning'>('success')

  function notify(message: string, type: 'success' | 'error' | 'info' | 'warning' = 'success') {
    text.value = message
    color.value = type
    show.value = true
  }

  return { show, text, color, notify }
}
```

- [ ] Create `src/composables/useSnackbar.ts`
- [ ] Refactor `src/views/hosts/HostsManager.vue` to use `useSnackbar()`
- [ ] Refactor `src/views/podcast/PodcastManager.vue` to use `useSnackbar()`
- [ ] Refactor `src/views/wallpapers/WallpaperManager.vue` to use `useSnackbar()`
- [ ] Evaluate if `NotificationSnackbar.vue` component can be simplified or removed

### Task 3.3: Extract theme toggle logic into composable

`toggleTheme()`, `themeIcon`, `themeTooltip` are duplicated identically in AppSidebar and AppFooter:
```typescript
// Add to src/composables/useTheme.ts
export function useThemeToggle() {
  const { themeMode, setTheme } = useTheme()

  const themeIcon = computed(() => { ... })
  const themeTooltip = computed(() => { ... })

  function toggleTheme() {
    if (themeMode.value === 'light') setTheme('dark')
    else if (themeMode.value === 'dark') setTheme('system')
    else setTheme('light')
  }

  return { themeIcon, themeTooltip, toggleTheme }
}
```

- [ ] Add `useThemeToggle()` to `src/composables/useTheme.ts`
- [ ] Refactor `AppSidebar.vue` to use `useThemeToggle()` (remove lines 317-337)
- [ ] Refactor `AppFooter.vue` to use `useThemeToggle()` (remove lines 143-171)

### Task 3.4: Extract markdown rendering utility

MarkdownIt + highlight.js + DOMPurify setup is duplicated in 3 files:
```typescript
// src/utils/markdown.ts
import MarkdownIt from 'markdown-it'
import hljs from 'highlight.js'
import DOMPurify from 'dompurify'

export function createMarkdownRenderer(): MarkdownIt { ... }
export function renderSafe(html: string): string { ... }
```

- [ ] Create `src/utils/markdown.ts`
- [ ] Refactor `src/views/AiChat/ChatMessage.vue` to use it
- [ ] Refactor `src/views/AiChat/index.vue` to use it
- [ ] Refactor `src/components/podcast/EpisodeShowNotes.vue` to use it

### Task 3.5: Fix `AppSidebar.vue` localStorage usage

`AppSidebar.vue` uses raw `localStorage` (lines 407, 422). Refactor to use `useStorage()`:
- [ ] Refactor `AppSidebar.vue` `loadWidth`/`saveWidth` to use `useStorage()` composable

### Task 3.6: Commit

```bash
git add -A
git commit -m "refactor: extract CrtOverlay, useSnackbar, useThemeToggle, markdown utils"
```

---

## Chunk 4: Fix TypeScript Issues

*Risk: Medium. Changing type annotations; behavior unchanged but compiler may surface new errors.*

### Task 4.1: Replace `any` in hosts module

- [ ] `src/composables/useHostsData.ts:215` — change `originalHost: any` to `originalHost: HostEntry`
- [ ] `src/composables/useHostsData.ts:254` — change `host: any` to `host: HostEntry`
- [ ] `src/composables/useHostsData.ts:276` — change `host: any` to `host: HostEntry`
- [ ] `src/utils/hostsUtils.ts:42` — change `host: any` to `host: HostEntry`
- [ ] `src/utils/hostsUtils.ts:52` — change `host: any` to `host: HostEntry`
- [ ] `src/components/hosts/tables/HostsTable.vue:242-246` — change emit types from `any` to `HostEntry`
- [ ] `src/components/hosts/tables/HostsTable.vue:305` — change `item: any` to `item: HostEntry`
- [ ] `src/components/hosts/dialogs/EditHostDialog.vue:67` — change `host: any | null` to `host: HostEntry | null`
- [ ] `src/components/hosts/dialogs/DeleteConfirmDialog.vue:48` — change `host: any | null` to `host: HostEntry | null`

### Task 4.2: Replace `any` in API layer

- [ ] `src/api/ai-chat.ts:103` — change `metadata?: any` to `metadata?: Record<string, unknown>`
- [ ] `src/api/ai-chat.ts:125` — same
- [ ] Add return type annotations to all functions in `src/api/app.ts`
- [ ] Add return type annotations to all functions in `src/api/hosts.ts`
- [ ] Add return type annotations to all functions in `src/api/podcast.ts`

### Task 4.3: Replace `any` in other components

- [ ] `src/views/AiChat/ProviderForm.vue:272` — replace `type Model = any` with proper interface
- [ ] `src/views/AiChat/ChatInput.vue:154` — change `ref<any>(null)` to proper ref type
- [ ] `src/views/AiChat/ModelSettings.vue:308` — change emit type from `any` to proper interface
- [ ] `src/components/common/SystemInfoBar.vue:329` — remove `as any` cast, use proper type

### Task 4.4: Fix duplicate type definitions

- [ ] `src/utils/security.ts:291-301` — import `ChatMessageValidation` from `src/api/ai-chat.ts` instead of redefining
- [ ] `src/utils/security.ts:11-15` — import `ValidationResult` from `src/api/ai-chat.ts` instead of redefining
- [ ] Standardize field naming (camelCase in TypeScript)

### Task 4.5: Commit

```bash
git add -A
git commit -m "refactor: replace any types with proper interfaces, add API return types"
```

---

## Chunk 5: Composable Cleanup & Memory Leak Fixes

*Risk: Medium. Fixing lifecycle issues that could cause subtle bugs.*

### Task 5.1: Fix missing event listener cleanup

**File:** `src/composables/useTheme.ts`
- [ ] Add `onUnmounted` to remove `prefersDark` change listener (line 87)

**File:** `src/composables/useAudioPlayer.ts`
- [ ] Store event handler references in variables
- [ ] Add `cleanupAudioRef()` function that removes all 5 listeners
- [ ] Call cleanup in `onUnmounted`

**File:** `src/components/common/LogoShaderBg.vue`
- [ ] Fix `window.addEventListener('resize', ...)` — store reference and remove in `onUnmounted`

### Task 5.2: Fix missing interval cleanup

**File:** `src/composables/useWallpaper.ts`
- [ ] Add `onUnmounted` that calls `stopAutoUpdateCheck()`

### Task 5.3: Move module-level composable calls

**File:** `src/composables/useWallpaper.ts:5`
- [ ] Move `const { getItem, setItem } = useStorage()` inside `useWallpaper()` function body

**File:** `src/composables/useTheme.ts:9`
- [ ] Move `const storage = useStorage()` inside `useTheme()` function body (requires restructuring the module-level refs)

### Task 5.4: Fix `useHostsData.ts` missing `finally` block

- [ ] `src/composables/useHostsData.ts:153-164` — add `finally { loading.value = false }` to `initializeDefaultConfig()`

### Task 5.5: Refactor store to use API layer

**File:** `src/stores/aiChat.ts`
- [ ] Replace all direct `invoke()` calls (lines 168, 179, 197, 209, 222, 240, 296, 316, 333) with imports from `src/api/ai-chat.ts`
- [ ] Remove duplicate `localStorage` access (lines 375-412), use `useStorage()` composable instead

### Task 5.6: Commit

```bash
git add -A
git commit -m "refactor: fix memory leaks, move module-level side effects, unify API usage in store"
```

---

## Chunk 6: Component Decomposition (God Components)

*Risk: Higher. Splitting large files into sub-components. Test each split carefully.*

### Task 6.1: Split `AppSidebar.vue` (1030 → ~300 lines)

Extract nav items as data-driven rendering:
- [ ] Create `src/components/layout/SidebarNavItem.vue` — single nav item with icon, label, tooltip, active state
- [ ] Define `navItems` array and use `v-for` instead of 4 copy-pasted blocks
- [ ] Extract collapsed/expanded action buttons into `SidebarActions.vue`

### Task 6.2: Split `SettingsDialog.vue` (1183 → ~300 lines)

- [ ] Create `src/components/dialogs/settings/GeneralSettingsTab.vue`
- [ ] Create `src/components/dialogs/settings/WallpaperSettingsTab.vue`
- [ ] Create `src/components/dialogs/settings/ChatSettingsTab.vue`
- [ ] Create `src/components/dialogs/settings/AdvancedSettingsTab.vue`
- [ ] `SettingsDialog.vue` becomes a shell with `v-tabs` and lazy-loaded tab components

### Task 6.3: Split `PodcastManager.vue` (1955 → ~500 lines)

- [ ] Extract `src/views/podcast/PodcastLibrary.vue` — grid view with search/filter
- [ ] Extract `src/views/podcast/PodcastDetail.vue` — episode list with pagination
- [ ] Extract `src/views/podcast/podcast.css` — podcast-specific styles (not shared)
- [ ] `PodcastManager.vue` becomes a router shell

### Task 6.4: Split `AiChat/index.vue` (1930 → ~500 lines)

- [ ] Extract `src/views/AiChat/BootScreen.vue` — boot sequence animation
- [ ] Extract `src/views/AiChat/R2D2Avatar.vue` — R2-D2 CSS avatar component
- [ ] Extract `src/views/AiChat/BB8Animation.vue` — BB-8 CSS animation component
- [ ] Extract `src/views/AiChat/ChatPanel.vue` — messages area + input

### Task 6.5: Commit each split separately

```bash
git commit -m "refactor: split AppSidebar into SidebarNavItem + SidebarActions"
git commit -m "refactor: split SettingsDialog into 4 tab components"
git commit -m "refactor: split PodcastManager into Library + Detail components"
git commit -m "refactor: split AiChat into BootScreen, Avatar, and ChatPanel"
```

---

## Verification

After each chunk:
- [ ] Run `npx vue-tsc --noEmit` — no new type errors
- [ ] Run `pnpm build` — builds successfully
- [ ] Visual check: launch `pnpm tauri dev` and verify each affected page looks identical

## Estimated Impact

| Metric | Before | After |
|--------|--------|-------|
| Total frontend lines | ~18,000 | ~12,000 |
| Files over 1000 lines | 6 | 0 |
| Files over 500 lines | 10 | 3-4 |
| CSS duplication | ~5000 lines | ~200 lines |
| `any` type count | 36+ | 0 |
| Dead code files | 2 | 0 |
