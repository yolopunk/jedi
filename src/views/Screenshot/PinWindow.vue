<template>
  <div class="pin-root" @pointerdown="onDragStart" @wheel.prevent="onWheel" @pointerenter="hovering = true" @pointerleave="hovering = false">
    <!-- 顶部悬浮区：已贴住徽标 + 悬停工具条 -->
    <span class="pin-badge">
      <svg width="10" height="10" viewBox="0 0 12 12" fill="none" aria-hidden="true"><path d="M6 1.4 7.3 4l2.9.4-2.1 2 .5 2.9L6 8l-2.6 1.3L3.9 6.4l-2.1-2L4.7 4Z" stroke="#fff" stroke-width="1.1" stroke-linejoin="round"/></svg>
      {{ t('screenshot.pin.pinned') }}
    </span>

    <div class="pin-toolbar" :class="{ show: hovering }" @pointerdown.stop>
      <button class="pt-btn" type="button" :aria-label="t('screenshot.pin.zoomOut')" @click="stepZoom(-0.25)">
        <svg width="12" height="12" viewBox="0 0 16 16" fill="none" aria-hidden="true"><path d="M4 8h8" stroke="currentColor" stroke-width="1.6" stroke-linecap="round"/></svg>
      </button>
      <span class="pt-zoom">{{ zoomLabel }}</span>
      <button class="pt-btn" type="button" :aria-label="t('screenshot.pin.zoomIn')" @click="stepZoom(0.25)">
        <svg width="12" height="12" viewBox="0 0 16 16" fill="none" aria-hidden="true"><path d="M8 4v8M4 8h8" stroke="currentColor" stroke-width="1.6" stroke-linecap="round"/></svg>
      </button>
      <span class="pt-divider" aria-hidden="true"></span>
      <span class="pt-opacity" @pointerdown.stop>
        {{ t('screenshot.pin.opacity', { value: opacity }) }}
        <input v-model.number="opacity" class="pt-slider" type="range" min="20" max="100" step="5" :aria-label="t('screenshot.pin.opacityLabel')" />
      </span>
      <span class="pt-divider" aria-hidden="true"></span>
      <button class="pt-btn" type="button" :aria-label="t('screenshot.copy')" :title="t('screenshot.copy')" @click="copyImage">
        <svg width="12" height="12" viewBox="0 0 16 16" fill="none" aria-hidden="true"><rect x="5.5" y="5.5" width="8" height="8" rx="1.8" stroke="currentColor" stroke-width="1.3"/><path d="M10.5 5.5V3.8A1.3 1.3 0 0 0 9.2 2.5H3.8A1.3 1.3 0 0 0 2.5 3.8v5.4a1.3 1.3 0 0 0 1.3 1.3h1.7" stroke="currentColor" stroke-width="1.3" stroke-linecap="round"/></svg>
      </button>
      <button class="pt-btn pt-close" type="button" :aria-label="t('screenshot.cancel')" :title="t('screenshot.pin.unpin')" @click="closeSelf">
        <svg width="12" height="12" viewBox="0 0 16 16" fill="none" aria-hidden="true"><path d="M4 4l8 8M12 4l-8 8" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/></svg>
      </button>
    </div>

    <!-- 贴住的截图本体 -->
    <div class="pin-card" :style="{ opacity: opacity / 100 }">
      <div class="pin-stage">
        <div v-if="failed" class="pin-broken">
          <span>{{ t('screenshot.imageMissing') }}</span>
          <button class="console-btn" type="button" @click="closeSelf">{{ t('common.close') }}</button>
        </div>
        <img
          v-else-if="src"
          ref="imgEl"
          :src="src"
          :alt="t('screenshot.pin.pinned')"
          draggable="false"
          :style="imgStyle"
          @load="onLoaded"
          @error="failed = true"
        />
      </div>
      <span class="pin-corner" aria-hidden="true"></span>
    </div>

    <span class="pin-hint" :class="{ hide: hintHidden }">{{ t('screenshot.pin.hint') }}</span>
  </div>
</template>

<script setup lang="ts">
import { convertFileSrc } from '@tauri-apps/api/core'
import { getCurrentWindow, LogicalSize } from '@tauri-apps/api/window'
import { computed, onMounted, onUnmounted, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { copyScreenshotFile, takePinPath } from '@/api/screenshot'

const TOP_ZONE = 44
const CARD_PAD = 7

const { t } = useI18n()
const window_ = getCurrentWindow()
const src = ref('')
const rawPath = ref('')
const failed = ref(false)
const hovering = ref(false)
const opacity = ref(100)
const zoom = ref(1)
const imgEl = ref<HTMLImageElement | null>(null)
const natural = ref<{ w: number; h: number } | null>(null)
const hintHidden = ref(false)

const zoomLabel = computed(() => `${Math.round(zoom.value * 100)}%`)
const imgStyle = computed(() =>
  natural.value
    ? { width: `${Math.round(natural.value.w * zoom.value)}px`, height: 'auto' }
    : undefined
)

async function applySize() {
  if (!natural.value) return
  const w = Math.round(natural.value.w * zoom.value) + CARD_PAD * 2
  const h = Math.round(natural.value.h * zoom.value) + CARD_PAD * 2 + TOP_ZONE
  try {
    await window_.setSize(new LogicalSize(Math.max(w, 120), Math.max(h, 100)))
  } catch {
    /* 权限缺失时保持窗口原尺寸，仅缩放图片 */
  }
}

function onLoaded() {
  const img = imgEl.value
  if (!img) return
  natural.value = { w: img.naturalWidth, h: img.naturalHeight }
  // 初始缩放 = Rust 设定的窗口内容宽 / 图片原始宽
  const fitW = (window.innerWidth - CARD_PAD * 2) / img.naturalWidth
  const fitH = (window.innerHeight - CARD_PAD * 2 - TOP_ZONE) / img.naturalHeight
  zoom.value = Math.max(0.05, Math.min(fitW, fitH, 1))
}

function stepZoom(delta: number) {
  const next = Math.min(4, Math.max(0.2, zoom.value + delta))
  if (next === zoom.value) return
  zoom.value = next
  void applySize()
}

function onWheel(event: WheelEvent) {
  const step = event.deltaY < 0 ? 0.1 : -0.1
  const next = Math.min(4, Math.max(0.2, zoom.value + step))
  if (next === zoom.value) return
  zoom.value = next
  void applySize()
}

function onDragStart(event: PointerEvent) {
  if (event.button !== 0) return
  const target = event.target as HTMLElement
  if (target.closest('.pin-toolbar') || target.closest('input')) return
  void window_.startDragging()
}

async function copyImage() {
  if (!rawPath.value) return
  try {
    await copyScreenshotFile(rawPath.value)
  } catch {
    /* 静默：贴图窗口内不弹错误条 */
  }
}

async function closeSelf() {
  try {
    await window_.close()
  } catch {
    /* already gone */
  }
}

function onKeyDown(event: KeyboardEvent) {
  if (event.key === 'Escape') {
    event.preventDefault()
    void closeSelf()
  }
}

onMounted(async () => {
  window.addEventListener('keydown', onKeyDown)
  try {
    const path = await takePinPath()
    rawPath.value = path
    src.value = convertFileSrc(path)
  } catch {
    failed.value = true
  }
  window.setTimeout(() => {
    hintHidden.value = true
  }, 4000)
})

onUnmounted(() => {
  window.removeEventListener('keydown', onKeyDown)
})
</script>

<style scoped>
.pin-root {
  position: fixed;
  inset: 0;
  box-sizing: border-box;
  padding-top: 44px;
  background: transparent;
  user-select: none;
  overflow: visible;
  font-family: var(--jedi-font-ui);
}

.pin-card {
  position: relative;
  height: 100%;
  box-sizing: border-box;
  border: 1.5px solid var(--accent);
  border-radius: 9px;
  background: #fff;
  padding: 6px;
  box-shadow: 0 26px 64px rgba(39, 40, 46, 0.28), 0 2px 8px rgb(var(--accent-rgb) / 0.18);
}

.pin-stage {
  height: 100%;
  border: 1px solid var(--border);
  border-radius: 5px;
  background: #fff;
  overflow: hidden;
  display: grid;
  place-items: center;
}

.pin-stage img {
  display: block;
  max-width: none;
  user-select: none;
}

.pin-broken {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 10px;
  color: var(--text-muted);
  font-size: 12px;
}

.pin-badge {
  position: absolute;
  top: 2px;
  left: 10px;
  display: inline-flex;
  align-items: center;
  gap: 5px;
  height: 24px;
  padding: 0 9px;
  border-radius: 6px;
  background: var(--accent);
  color: #fff;
  font-size: 10.5px;
  font-weight: 600;
  box-shadow: 0 6px 16px rgb(var(--accent-rgb) / 0.35);
  z-index: 2;
}

.pin-corner {
  position: absolute;
  right: 8px;
  bottom: 8px;
  width: 12px;
  height: 12px;
  border-right: 2px solid var(--accent);
  border-bottom: 2px solid var(--accent);
  border-radius: 0 0 2px 0;
  opacity: 0.7;
  pointer-events: none;
}

.pin-toolbar {
  position: absolute;
  top: 5px;
  left: 50%;
  transform: translateX(-50%);
  display: flex;
  align-items: center;
  gap: 2px;
  height: 34px;
  padding: 0 6px;
  border-radius: 9px;
  border: 1px solid var(--border);
  background: var(--bg);
  box-shadow: 0 10px 26px rgba(39, 40, 46, 0.16);
  opacity: 0;
  pointer-events: none;
  transition: opacity 0.15s ease;
  z-index: 3;
}

.pin-toolbar.show {
  opacity: 1;
  pointer-events: auto;
}

.pt-btn {
  display: grid;
  place-items: center;
  width: 26px;
  height: 24px;
  border: 0;
  border-radius: 6px;
  background: transparent;
  color: var(--text-muted);
  cursor: pointer;
}

.pt-btn:hover {
  background: rgb(var(--ink-rgb) / 0.06);
  color: var(--text);
}

.pt-btn.pt-close {
  color: var(--danger);
}

.pt-btn.pt-close:hover {
  background: rgb(var(--danger-rgb) / 0.1);
}

.pt-zoom {
  padding: 0 2px;
  font: 500 10px var(--jedi-font-mono);
  color: var(--text);
}

.pt-divider {
  width: 1px;
  height: 16px;
  margin: 0 3px;
  background: var(--border);
}

.pt-opacity {
  display: inline-flex;
  align-items: center;
  gap: 7px;
  height: 24px;
  padding: 0 8px;
  border-radius: 6px;
  background: rgb(var(--accent-rgb) / 0.08);
  color: var(--accent);
  font-size: 10.5px;
  font-weight: 600;
}

.pt-slider {
  width: 72px;
  accent-color: var(--accent);
  cursor: pointer;
}

.pin-hint {
  position: absolute;
  left: 50%;
  bottom: 6px;
  transform: translateX(-50%);
  display: inline-flex;
  align-items: center;
  height: 26px;
  padding: 0 10px;
  border-radius: 7px;
  border: 1px solid var(--border);
  background: rgb(var(--bg-rgb) / 0.92);
  font: 500 10px var(--jedi-font-mono);
  color: var(--text-muted);
  white-space: nowrap;
  transition: opacity 0.4s ease;
  z-index: 2;
}

.pin-hint.hide {
  opacity: 0;
}

@media (prefers-reduced-motion: reduce) {
  .pin-toolbar,
  .pin-hint {
    transition: none;
  }
}
</style>
