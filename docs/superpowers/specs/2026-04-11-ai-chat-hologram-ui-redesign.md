# AI Chat Hologram UI Redesign

> **Date:** 2026-04-11
> **Status:** Draft

## Overview

Redesign the AI Chat interface to unify terminal and AI chat styles into a cohesive **Holographic Terminal** aesthetic, inspired by Star Wars holographic projections and Jedi theme.

---

## Visual Foundation

### Color Palette

| Token | Hex | Usage |
|-------|-----|-------|
| Background | `#0a0a0f` | Page background |
| Hologram Primary | `#00d4ff` | Main glow, highlights |
| Hologram Dark | `#0891b2` | Secondary accents |
| Glow Border | `rgba(0, 212, 255, 0.3)` | Box shadows, borders |
| Text Primary | `#e0f7ff` | Main text |
| Text Secondary | `#64748b` | Muted text |
| Surface | `rgba(0, 20, 40, 0.6)` | Cards, panels |

### Effects

- **Frosted Glass:** `backdrop-filter: blur(12px)`
- **Glow Border:** `box-shadow: 0 0 20px rgba(0, 212, 255, 0.3), inset 0 0 20px rgba(0, 212, 255, 0.05)`
- **Border Glow:** `border: 1px solid rgba(0, 212, 255, 0.25)`

### Border Radius

- Cards: `12px`
- Buttons: `10px`
- Inputs: `10px`

---

## Header Bar

### Layout

```
┌─────────────────────────────────────────────────────────────┐
│ [≡] holocron:~/chat                        [⚡] [PROVIDER: CLAUDE] │
└─────────────────────────────────────────────────────────────┘
```

### Styling

**`.chat-header`**
```css
height: 48px;
padding: 0 16px;
background: rgba(0, 20, 40, 0.8);
border-bottom: 1px solid rgba(0, 212, 255, 0.2);
box-shadow: 0 2px 20px rgba(0, 212, 255, 0.1);
backdrop-filter: blur(12px);
```

**`.header-logo`**
```css
font-size: 11px;
color: #64748b;
gap: 8px;
```
- Menu icon: `#00d4ff`
- `holocron`: `#00d4ff` (glowing terminal style)
- `~/chat`: muted color

**`.provider-display`**
```css
padding: 6px 12px;
background: rgba(0, 212, 255, 0.08);
border: 1px solid rgba(0, 212, 255, 0.25);
border-radius: 8px;
cursor: pointer;
transition: all 0.2s;
```
- Hover: border glows stronger
- Click: opens ModelSettings

---

## Input Area

### Layout

```
┌─────────────────────────────────────────────────────────────┐
│  输入框...                                                  │
│  输入框...                                                  │
│  输入框...                                                  │
│  [/] [+]                           [MODEL ▼] [↑]          │
│  ← 左下角                          右下角 →                │
└─────────────────────────────────────────────────────────────┘
```

### Structure

- Input textarea: 100% width, auto height (min 44px, max 160px)
- Left bottom: toolbar buttons `/`, `+`
- Right bottom: model selector + send button

### Styling

**`.input-console`**
```css
padding: 16px 20px;
background: rgba(0, 20, 40, 0.6);
border-top: 1px solid rgba(0, 212, 255, 0.15);
backdrop-filter: blur(12px);
```

**`.input-row`** (new layout)
```css
display: flex;
flex-direction: column;
gap: 8px;
```

**`.console-input`**
```css
width: 100%;
min-height: 44px;
max-height: 160px;
padding: 10px 16px;
background: rgba(0, 10, 20, 0.6);
border: 1px solid rgba(0, 212, 255, 0.15);
border-radius: 10px;
color: #e0f7ff;
font-size: 14px;
backdrop-filter: blur(8px);
resize: none;
outline: none;
transition: border-color 0.15s, box-shadow 0.15s;
```

**`.console-input:focus`**
```css
border-color: rgba(0, 212, 255, 0.4);
box-shadow: 0 0 20px rgba(0, 212, 255, 0.15),
            inset 0 0 10px rgba(0, 212, 255, 0.05);
```

**`.input-actions`** (bottom row)
```css
display: flex;
justify-content: space-between;
align-items: center;
```

**`.toolbar-btn`**
```css
width: 36px;
height: 36px;
background: rgba(0, 212, 255, 0.05);
border: 1px solid rgba(0, 212, 255, 0.2);
border-radius: 8px;
color: rgba(0, 212, 255, 0.8);
font-size: 16px;
transition: all 0.15s;
```

**`.toolbar-btn:hover`**
```css
background: rgba(0, 212, 255, 0.12);
border-color: rgba(0, 212, 255, 0.4);
box-shadow: 0 0 15px rgba(0, 212, 255, 0.2);
color: #00d4ff;
```

**`.model-selector`**
```css
display: flex;
align-items: center;
gap: 8px;
```

**`.model-dropdown-btn`**
```css
padding: 8px 14px;
background: rgba(0, 212, 255, 0.1);
border: 1px solid rgba(0, 212, 255, 0.3);
border-radius: 8px;
color: #00d4ff;
font-size: 12px;
font-weight: 600;
transition: all 0.15s;
```

**`.model-dropdown-btn:hover`**
```css
background: rgba(0, 212, 255, 0.15);
border-color: rgba(0, 212, 255, 0.5);
box-shadow: 0 0 15px rgba(0, 212, 255, 0.25);
```

**`.send-btn`**
```css
width: 44px;
height: 44px;
background: linear-gradient(135deg, rgba(0, 212, 255, 0.2), rgba(0, 212, 255, 0.1));
border: 1px solid rgba(0, 212, 255, 0.4);
border-radius: 10px;
color: #00d4ff;
font-size: 18px;
box-shadow: 0 0 15px rgba(0, 212, 255, 0.2);
transition: all 0.15s;
```

**`.send-btn:hover:not(.disabled)`**
```css
background: linear-gradient(135deg, rgba(0, 212, 255, 0.3), rgba(0, 212, 255, 0.15));
box-shadow: 0 0 25px rgba(0, 212, 255, 0.4);
```

---

## Message Bubble

### Hover State

On hover, each message displays metadata row:

```
┌─────────────────────────────────────┐
│  消息内容...                         │
│  [MODEL: CLAUDE-3-5] [📋 复制] [🔄 重试] [10:30:15] │
└─────────────────────────────────────┘
```

**Styling**

**`.message-meta`** (hidden by default)
```css
display: none;
gap: 8px;
padding: 8px 0;
opacity: 0.6;
font-size: 11px;
color: #64748b;
transition: opacity 0.15s;
```

**`.console-message:hover .message-meta`**
```css
display: flex;
opacity: 1;
```

**`.meta-item`**
```css
padding: 4px 8px;
background: rgba(0, 212, 255, 0.08);
border: 1px solid rgba(0, 212, 255, 0.15);
border-radius: 6px;
cursor: pointer;
transition: all 0.15s;
```

**`.meta-item:hover`**
```css
background: rgba(0, 212, 255, 0.15);
border-color: rgba(0, 212, 255, 0.3);
color: #00d4ff;
```

---

## Model Dropdown Menu

### Styling

**`.model-dropdown-menu`**
```css
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
```

**`.model-dropdown-item`**
```css
display: flex;
align-items: center;
justify-content: space-between;
padding: 10px 12px;
border-radius: 8px;
cursor: pointer;
transition: background 0.15s;
```

**`.model-dropdown-item:hover`**
```css
background: rgba(0, 212, 255, 0.1);
```

**`.model-dropdown-item.selected`**
```css
background: rgba(0, 212, 255, 0.15);
border: 1px solid rgba(0, 212, 255, 0.3);
```

---

## CSS Cleanup

Remove duplicate CSS definitions:
- `.input-console` (lines ~731, ~1438) → keep one
- `.input-row` (lines ~739, ~1444) → keep one
- `.console-input` (lines ~761, ~1485) → keep one
- `.send-btn` (lines ~809, ~1581) → keep one

---

## Components to Update

| File | Changes |
|------|---------|
| `src/views/AiChat/chat.css` | Replace all duplicate styles, add new hologram styles |
| `src/views/AiChat/index.vue` | Update header template, update input area layout |
| `src/views/AiChat/ModelSettings.vue` | Add `.provider-display` styles if needed |

---

## Implementation Order

1. **CSS Cleanup** — Remove duplicate styles first
2. **Header** — Update `.chat-header` and `.provider-display`
3. **Input Area** — Restructure layout, add hologram styles
4. **Message Bubble** — Add hover metadata row
5. **Model Dropdown** — Polish hologram styling
