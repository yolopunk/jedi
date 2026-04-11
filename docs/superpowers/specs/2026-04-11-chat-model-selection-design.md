# Chat Model Selection - Implementation Design

## Goal

Enable users to select a model and chat after configuring a provider's API key. The chat input area follows an AI chat style with adaptive height and quick-access tool buttons.

---

## Problem Analysis

| Issue | Root Cause |
|-------|------------|
| Configured provider still shows "API key needed" | `loadConfiguredProviders()` only called when ModelSettings dialog opens, not at startup |
| Select model has no interaction | `createSession()` uses hardcoded `openai/gpt-4o-mini`, ignores `modelsDevStore` |
| Can't resume last-used model | No persistence of last selected provider/model |

---

## Design

### 1. Chat Page Header - Provider Display

**Current:** Shows "MODEL: {modelName}" and opens ModelSettings on click

**Changed:**
- Header shows **PROVIDER** name (e.g., "DEEPSEEK")
- Click opens ModelSettings dialog for configuration
- Model name shown in input area (not header)

### 2. Chat Page Input Area

**Layout (left to right):**
```
[+ ] [  /  ] | [ Adaptive height textarea                        ] | [ GPT-4o ▼ ]
```

**Elements:**
- `+` button → popup for attachment upload, skills, web search
- `/` button → slash commands popup (existing CommandPalette)
- Textarea → auto-resize height, max 200px
- Model dropdown → shows current model, opens model selector (provider's models)

**Height behavior:**
- Default: single line (40px)
- Auto-expand with content up to 200px
- Shift+Enter for new line

### 3. Model Dropdown

Click opens a dropdown showing all models for the current provider:
- Model name
- Context window badge (e.g., "128K")
- Reasoning / Tool badges if applicable

Selecting a model updates `modelsDevStore.selectedModelId` and persists to storage.

### 4. Provider Configuration Dialog (ModelSettings)

**No changes to dialog structure.** Add the following enhancements:

**API Key viewing:**
- When a provider has API key configured, show masked key (e.g., `sk-****abcd`)
- Eye icon button to toggle reveal/hide
- "Remove Key" button styled as red text button (not broken outlined button)

### 5. Startup Behavior

1. `onMounted` in index.vue loads both:
   - `modelsDevStore.fetchProviders()`
   - `providerConfigStore.loadConfiguredProviders()`
2. Load last-used `providerId` + `modelId` from localStorage
3. Auto-select if both exist and provider is configured
4. If no saved selection or provider not configured → show welcome screen

### 6. Persistence

Store in localStorage:
```json
{
  "lastProviderId": "deepseek",
  "lastModelId": "deepseek-chat"
}
```

---

## Files to Modify

| File | Change |
|------|--------|
| `src/views/AiChat/index.vue` | Header shows provider; input area rebuild with model dropdown |
| `src/stores/aiChat.ts` | `createSession()` accepts provider/model args instead of hardcode |
| `src/stores/modelsDev.ts` | Persist/load last selected provider+model; add `lastSelectedProviderId`, `lastSelectedModelId` |
| `src/stores/providerConfig.ts` | Called at startup in index.vue |
| `src/views/AiChat/ModelSettings.vue` | Fix "Remove Key" button styling; add API key reveal toggle |
| `src/components/CommandPalette.vue` | Already exists for `/` slash commands |
| `src/views/AiChat/chat.css` | Input area styles |

---

## Implementation Order

1. Fix `loadConfiguredProviders()` called at startup
2. Add persistence for last selected provider/model
3. Update header to show provider instead of model
4. Rebuild input area with model dropdown
5. Fix ModelSettings API key button styling
6. Wire up model selection to update session

---

## Success Criteria

- [ ] Provider shows in header, ModelSettings opens on click
- [ ] Input area has `+`, `/`, and model dropdown
- [ ] Model dropdown shows current provider's models
- [ ] Selecting model persists across app restarts
- [ ] `+` opens attachment/skills/web search popup
- [ ] `/` opens slash commands (existing behavior)
- [ ] Already-configured provider shows API key with reveal toggle
- [ ] Chat sends message using selected provider/model
