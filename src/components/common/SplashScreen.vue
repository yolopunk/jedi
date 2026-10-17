<template>
  <div
    class="app-splash"
    :class="{ 'splash-leaving': leaving }"
    role="status"
  >
    <!-- 黑洞主视觉（原侧边栏 logo 区着色器，作为开机画面的品牌瞬间） -->
    <div class="splash-visual" aria-hidden="true">
      <BlackHoleCanvas />
    </div>

    <!-- 氛围层：暗角聚焦 + 胶片颗粒 -->
    <div class="splash-vignette" aria-hidden="true"></div>
    <div class="splash-grain" aria-hidden="true"></div>

    <div class="splash-brand">
      <img
        class="splash-logo"
        src="/icon.png?v=jedi-koala-20261003"
        alt=""
      />
      <h1 class="splash-title">{{ t('sidebar.title') }}</h1>
      <p class="splash-subtitle">{{ t('sidebar.subtitle') }}</p>
    </div>

    <div class="splash-progress" aria-hidden="true">
      <div class="splash-progress-fill"></div>
    </div>

    <!-- 控制台元信息：左侧模块自检轮播，右侧版本 -->
    <div class="splash-meta" aria-hidden="true">
      <span class="splash-status">
        <span class="splash-prompt">&gt;</span>
        <span class="splash-status-text">{{ statusText }}</span>
        <span class="splash-caret"></span>
      </span>
      <span class="splash-version">v{{ appVersion }}</span>
    </div>

    <span class="sr-only">{{ t('splash.loading') }}</span>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import BlackHoleCanvas from '@/components/common/BlackHoleCanvas.vue'
import pkg from '../../../package.json'

const props = defineProps<{
  /** 由宿主在初始化完成后置 true，触发「坠入奇点」退场编排 */
  leaving?: boolean
}>()

const emit = defineEmits<{
  /** 启动页实际渲染完成，宿主以此为最短停留计时起点 */
  (e: 'ready'): void
}>()

const { t } = useI18n()

const appVersion = pkg.version

// 模块自检轮播：对应应用的真实功能模块
const statusKeys = [
  'splash.stepHosts',
  'splash.stepWallpaper',
  'splash.stepPodcast',
  'splash.stepChat',
] as const

const stepIndex = ref(0)
const statusText = computed(() => {
  const key = statusKeys[stepIndex.value % statusKeys.length]
  return key ? t(key) : ''
})

let statusTimer: number | null = null

onMounted(() => {
  emit('ready')
  statusTimer = window.setInterval(() => {
    stepIndex.value = (stepIndex.value + 1) % statusKeys.length
  }, 460)
})

// 退场时停止自检轮播
watch(
  () => props.leaving,
  leaving => {
    if (leaving && statusTimer !== null) {
      window.clearInterval(statusTimer)
      statusTimer = null
    }
  }
)

onUnmounted(() => {
  if (statusTimer !== null) {
    window.clearInterval(statusTimer)
    statusTimer = null
  }
})
</script>

<style scoped>
/* 启动页始终以暗色「开机画面」呈现，与应用主题无关 */
.app-splash {
  position: fixed;
  inset: 0;
  z-index: 5000;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  overflow: hidden;
  background: #0a0a0f;
  user-select: none;
  transition: opacity 0.6s ease;
}

.app-splash.splash-leaving {
  opacity: 0;
  pointer-events: none;
}

/* ── 第一幕：黑洞坍缩入场 ─────────────────────────────── */

.splash-visual {
  position: relative;
  /* 同时受 vmin/vh 双向约束，矮窗口下不溢出视口 */
  width: min(86vmin, 68vh, 720px);
  aspect-ratio: 1;
  margin-bottom: -6vh;
  /* 径向蒙版把方形画布边缘融进背景；收口在品牌区之前，
     避免外圈薄环横穿 logo/标题 */
  -webkit-mask-image: radial-gradient(circle at 50% 50%, #000 52%, transparent 76%);
  mask-image: radial-gradient(circle at 50% 50%, #000 52%, transparent 76%);
  /* 只动 opacity + transform：filter 叠加 mask + WebGL 层在部分
     WebView2/驱动组合下会整层不合成，导致启动页空白 */
  animation: bh-in 1.15s cubic-bezier(0.16, 1, 0.3, 1) 0.05s both;
  will-change: transform, opacity;
}

/* 启动阶段禁止画布拖拽交互 */
.splash-visual :deep(.black-hole-canvas) {
  pointer-events: none;
}

@keyframes bh-in {
  from {
    opacity: 0;
    transform: scale(0.78);
  }
  to {
    opacity: 1;
    transform: scale(1);
  }
}

/* ── 氛围层 ──────────────────────────────────────────── */

.splash-vignette {
  position: absolute;
  inset: 0;
  pointer-events: none;
  background: radial-gradient(
    ellipse 92% 78% at 50% 44%,
    transparent 52%,
    rgb(0 0 0 / 55%) 100%
  );
}

.splash-grain {
  position: absolute;
  inset: 0;
  pointer-events: none;
  opacity: 0.05;
  background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='180' height='180'%3E%3Cfilter id='n'%3E%3CfeTurbulence type='fractalNoise' baseFrequency='0.9' numOctaves='2' stitchTiles='stitch'/%3E%3C/filter%3E%3Crect width='100%25' height='100%25' filter='url(%23n)'/%3E%3C/svg%3E");
  background-size: 180px 180px;
}

/* ── 第二幕：品牌浮现 ────────────────────────────────── */

.splash-brand {
  position: relative;
  display: flex;
  flex-direction: column;
  align-items: center;
}

.splash-logo {
  width: 52px;
  height: 52px;
  border-radius: 12px;
  border: 1px solid rgb(255 255 255 / 0.14);
  animation: logo-in 0.7s cubic-bezier(0.34, 1.56, 0.64, 1) 0.5s both;
}

@keyframes logo-in {
  from {
    opacity: 0;
    transform: scale(0.6);
  }
  to {
    opacity: 1;
    transform: scale(1);
  }
}

.splash-title {
  margin: 14px 0 0;
  font-family: var(--jedi-font-display);
  font-size: 22px;
  font-weight: 600;
  line-height: 1;
  letter-spacing: 6px;
  text-indent: 6px;
  color: #f4f4f5;
  /* 动力学显影：字距从宽松收拢到最终位置 */
  animation: title-in 0.9s cubic-bezier(0.16, 1, 0.3, 1) 0.65s both;
}

@keyframes title-in {
  from {
    opacity: 0;
    letter-spacing: 16px;
    text-indent: 16px;
    filter: blur(8px);
  }
  to {
    opacity: 1;
    letter-spacing: 6px;
    text-indent: 6px;
    filter: blur(0);
  }
}

.splash-subtitle {
  margin: 8px 0 0;
  font-size: 11px;
  line-height: 1;
  letter-spacing: 4px;
  text-indent: 4px;
  color: rgb(244 244 245 / 0.42);
  animation: rise-in 0.7s cubic-bezier(0.16, 1, 0.3, 1) 0.85s both;
}

@keyframes rise-in {
  from {
    opacity: 0;
    transform: translateY(10px);
  }
  to {
    opacity: 1;
    transform: none;
  }
}

/* ── 第三幕：系统自检 ────────────────────────────────── */

.splash-progress {
  position: absolute;
  left: 50%;
  bottom: 13vh;
  width: 148px;
  height: 2px;
  border-radius: 1px;
  background: rgb(255 255 255 / 0.08);
  overflow: hidden;
  transform: translateX(-50%);
  animation: fade-in 0.5s ease 1.05s both;
}

@keyframes fade-in {
  from {
    opacity: 0;
  }
  to {
    opacity: 1;
  }
}

.splash-progress-fill {
  /* 基准 90%：分段充能动画覆盖显示；reduced-motion 下静止于此 */
  width: 90%;
  height: 100%;
  border-radius: 1px;
  background: linear-gradient(
    90deg,
    rgb(255 178 102 / 0.45),
    rgb(255 199 138 / 95%)
  );
  /* 分段充能：快进、停顿、再推进，模拟真实加载节奏 */
  animation: progress-fill 2.4s linear 0.35s both;
}

@keyframes progress-fill {
  0% {
    width: 0%;
  }
  14% {
    width: 24%;
  }
  30% {
    width: 28%;
  }
  58% {
    width: 58%;
  }
  82% {
    width: 74%;
  }
  100% {
    width: 90%;
  }
}

.splash-meta {
  position: absolute;
  left: 0;
  right: 0;
  bottom: 5vh;
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 32px;
  font-family: var(--jedi-font-mono);
  font-size: 10px;
  letter-spacing: 0.08em;
  color: rgb(244 244 245 / 50%);
  animation: fade-in 0.6s ease 1.3s both;
}

.splash-status {
  display: inline-flex;
  align-items: center;
}

.splash-prompt {
  margin-right: 8px;
  color: rgb(255 178 102 / 90%);
}

.splash-caret {
  display: inline-block;
  width: 5px;
  height: 11px;
  margin-left: 7px;
  background: rgb(255 178 102 / 85%);
  animation: caret-blink 1s steps(1) infinite;
}

@keyframes caret-blink {
  50% {
    opacity: 0;
  }
}

.splash-version {
  color: rgb(244 244 245 / 30%);
}

/* ── 退场：坠入奇点 ──────────────────────────────────── */
/* 退场时机由宿主保证（最短停留 1.8s），此时所有入场动画均已结束 */

.splash-visual,
.splash-brand,
.splash-progress,
.splash-meta {
  transition:
    opacity 0.5s cubic-bezier(0.5, 0, 0.75, 0.35),
    transform 0.5s cubic-bezier(0.5, 0, 0.75, 0.35);
}

.splash-leaving .splash-visual {
  opacity: 0;
  transform: scale(0.82);
}

.splash-leaving .splash-brand {
  opacity: 0;
  transform: translateY(-16px) scale(1.06);
}

.splash-leaving .splash-progress,
.splash-leaving .splash-meta {
  opacity: 0;
}

.splash-leaving .splash-progress-fill {
  animation: none;
  width: 100%;
  transition: width 0.25s ease-out;
}

/* ── Reduced motion：全部动画退化为静态开机画面 ──────── */

@media (prefers-reduced-motion: reduce) {
  .splash-visual,
  .splash-logo,
  .splash-title,
  .splash-subtitle,
  .splash-progress,
  .splash-progress-fill,
  .splash-meta,
  .splash-caret {
    animation: none !important;
  }

  .splash-visual,
  .splash-brand,
  .splash-progress,
  .splash-meta {
    transition: opacity 0.4s ease;
  }

  .splash-leaving .splash-visual,
  .splash-leaving .splash-brand {
    transform: none;
  }
}

.sr-only {
  position: absolute;
  width: 1px;
  height: 1px;
  padding: 0;
  margin: -1px;
  overflow: hidden;
  clip: rect(0, 0, 0, 0);
  white-space: nowrap;
  border: 0;
}
</style>
