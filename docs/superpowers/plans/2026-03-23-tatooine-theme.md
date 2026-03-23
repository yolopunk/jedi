# Tatooine Theme Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps uses checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement the Tatooine warm sand-yellow metal theme as the new light theme, replacing the existing neon daylight theme.

**Architecture:** Replace CSS variables in `.light-theme` class in `theme.css` and update Vuetify theme configuration in `vuetify.ts`. No new files created, just modifying existing ones.

**Tech Stack:** CSS, Vue 3, Vuetify 3

---

## File Structure

| File | Operation | Purpose |
|------|-----------|---------|
| `src/assets/theme.css` | Modify | Replace `.light-theme` CSS variables with Tatooine colors |
| `src/plugins/vuetify.ts` | Modify | Update light theme color definitions |

---

### Task 1: Update theme.css - Light Theme CSS Variables

**Files:**
- Modify: `src/assets/theme.css:95-178`

- [ ] **Step 1: Read current theme.css to confirm line numbers**

Read: `src/assets/theme.css` to verify the `.light-theme` block is at lines 95-178

- [ ] **Step 2: Replace the .light-theme block**

Replace lines 95-178 with:

```css
/* =========================================
   Light Theme (TATOOINE OUTPOST - Sand & Metal)
   ========================================= */
.light-theme {
  /* Backgrounds - Warm Sand Gradient */
  --jedi-bg-app: #f5e6d3;
  --jedi-bg-surface: #efe0cc;
  --jedi-bg-surface-hover: #e8d4bc;
  --jedi-bg-sidebar: #e8d4bc;
  --jedi-bg-input: #faf3e8;

  /* Borders - Brass */
  --jedi-border: #b8860b;
  --jedi-border-focus: #cd7f32;

  /* Text - Sandstone Browns */
  --jedi-text-primary: #3d2914;
  --jedi-text-secondary: #6b4423;
  --jedi-text-tertiary: #9a7b5a;
  --jedi-text-inverse: #ffffff;

  /* Functional - Metal Tones */
  --jedi-primary: #cd7f32;
  --jedi-primary-hover: #e6be8a;
  --jedi-accent: #daa520;
  --jedi-success: #daa520;
  --jedi-warning: #cd853f;
  --jedi-danger: #b22222;

  /* Syntax */
  --jedi-syntax-ip: #cd7f32;
  --jedi-syntax-domain: #b8860b;

  /* Sci-Fi Console (Tatooine Version) */
  --scifi-bg: #f5e6d3;
  --scifi-bg-terminal: #faf3e8;
  --scifi-cyan: #b8860b;
  --scifi-green: #daa520;
  --scifi-magenta: #cd7f32;
  --scifi-amber: #cd853f;
  --scifi-red: #b22222;
  --scifi-border: #b8860b;

  /* Shadows - Metal Glow (No Neon) */
  --jedi-shadow-sm: 0 2px 4px rgba(184, 134, 11, 0.1);
  --jedi-shadow-md: 0 2px 8px rgba(184, 134, 11, 0.2);

  /* Glow effects - SUBTLE METAL ONLY */
  --glow-border: rgba(205, 127, 50, 0.15);
  --glow-text: rgba(184, 134, 11, 0.3);
  --glow-hover: rgba(205, 127, 50, 0.08);

  /* Vuetify Theme Variables (RGB channels) */
  --v-theme-background: 245, 230, 211;
  --v-theme-surface: 239, 224, 204;
  --v-theme-surface-light: 232, 212, 188;
  --v-theme-surface-dark: 250, 243, 232;
  --v-theme-primary: 205, 127, 50;
  --v-theme-primary-light: 230, 190, 138;
  --v-theme-primary-dark: 184, 134, 11;
  --v-theme-secondary: 107, 68, 35;
  --v-theme-secondary-light: 154, 123, 90;
  --v-theme-secondary-dark: 61, 41, 20;
  --v-theme-error: 178, 34, 34;
  --v-theme-error-light: 205, 68, 68;
  --v-theme-error-dark: 139, 0, 0;
  --v-theme-info: 184, 134, 11;
  --v-theme-info-light: 218, 165, 32;
  --v-theme-info-dark: 139, 105, 20;
  --v-theme-success: 218, 165, 32;
  --v-theme-success-light: 230, 190, 138;
  --v-theme-success-dark: 139, 105, 20;
  --v-theme-warning: 205, 133, 63;
  --v-theme-warning-light: 222, 170, 120;
  --v-theme-warning-dark: 139, 90, 43;
  --v-theme-on-background: 61, 41, 20;
  --v-theme-on-surface: 61, 41, 20;
  --v-theme-on-primary: 255, 255, 255;
  --v-theme-on-secondary: 255, 255, 255;
  --v-theme-on-error: 255, 255, 255;
  --v-theme-on-info: 61, 41, 20;
  --v-theme-on-success: 61, 41, 20;
  --v-theme-on-warning: 61, 41, 20;
}
```

- [ ] **Step 3: Verify the change**

Read the modified file to confirm the `.light-theme` block was replaced correctly.

- [ ] **Step 4: Commit**

```bash
cd /Users/cynosure/workspace/github/jedi
git add src/assets/theme.css
git commit -m "feat: update light theme to Tatooine sand & metal colors

- Replace neon daylight theme with warm sand-yellow palette
- Use brass/copper/metal tones instead of neon
- Remove neon glow effects, use subtle metal shadows

Co-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>"
```

---

### Task 2: Update vuetify.ts - Light Theme Colors

**Files:**
- Modify: `src/plugins/vuetify.ts:24-36`

- [ ] **Step 1: Read current vuetify.ts to confirm**

Read: `src/plugins/vuetify.ts`

- [ ] **Step 2: Update the light theme colors**

Replace the `light` theme colors block (lines 24-36) with:

```typescript
      light: {
        dark: false,
        colors: {
          background: '#f5e6d3',      // Warm sand
          surface: '#efe0cc',         // Light sand
          primary: '#cd7f32',         // Copper
          secondary: '#6b4423',       // Sandstone brown
          accent: '#daa520',          // Dark gold
          error: '#b22222',           // Firebrick red
          info: '#b8860b',            // Dark goldenrod
          success: '#daa520',         // Dark gold
          warning: '#cd853f',         // Peru
        }
      },
```

- [ ] **Step 3: Verify the change**

Read the modified file to confirm the light theme colors were updated correctly.

- [ ] **Step 4: Commit**

```bash
cd /Users/cynosure/workspace/github/jedi
git add src/plugins/vuetify.ts
git commit -m "feat: update Vuetify light theme to Tatooine colors

- Match Vuetify theme colors to CSS variables
- Use warm sand, brass, and copper tones

Co-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>"
```

---

### Task 3: Test the Theme

**Files:** None - manual verification

- [ ] **Step 1: Build the frontend**

Run:
```bash
cd /Users/cynosure/workspace/github/jedi
pnpm build
```

Expected: Build completes successfully with no errors.

- [ ] **Step 2: Manual verification checklist**

If you can run the app, verify:
- [ ] Theme toggle switches between dark and light themes
- [ ] Light theme uses sand-yellow background
- [ ] No neon glow effects visible in light theme
- [ ] Text is readable (good contrast)
- [ ] All components render correctly in both themes
- [ ] Dark theme remains unchanged

- [ ] **Step 3: Verify git status**

Run:
```bash
cd /Users/cynosure/workspace/github/jedi
git status
```

Expected: Clean working tree, all changes committed.

---

## Acceptance Criteria

- [x] CSS variables in `.light-theme` replaced with Tatooine colors
- [x] Vuetify light theme colors updated
- [x] No neon effects in light theme
- [x] Dark theme unchanged
- [x] All changes committed
