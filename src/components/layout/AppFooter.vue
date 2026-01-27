<template>
  <div class="nav-footer">
    <!-- 深色模式：星空背景 -->
    <template v-if="isDark">
      <!-- 深空背景 -->
      <div class="deep-space-bg"></div>
      
      <!-- 星星层 -->
      <div class="stars-layer small-stars"></div>
      <div class="stars-layer medium-stars"></div>
      
      <!-- 极光/星云效果 -->
      <div class="nebula-glow"></div>
    </template>

    <!-- 浅色模式：曼达洛人日落风格 -->
    <template v-else>
      <!-- 日落背景 -->
      <div class="sunset-bg"></div>
      
      <!-- 暖色光晕 -->
      <div class="sunset-glow"></div>
      
      <!-- 尘埃粒子 -->
      <div class="dust-particles"></div>
    </template>

    <!-- 底部控制台容器 -->
    <div class="footer-console d-flex flex-column align-center px-4 pt-3 pb-2" :class="{'light-console': !isDark}">
      <!-- 装饰线 -->
      <div class="console-line" :class="{'light-line': !isDark}"></div>
      
      <!-- 底部按钮 -->
      <div class="d-flex justify-center w-100 mb-1 button-group">
        <v-btn variant="text" :color="isDark ? 'white' : 'brown-darken-3'" size="small" class="mx-1 footer-btn jedi-icon-btn" @click="toggleTheme" icon>
          <v-icon :icon="themeIcon"></v-icon>
          <v-tooltip activator="parent" location="top">{{ themeTooltip }}</v-tooltip>
        </v-btn>
        <v-btn :icon="mdiGithub" variant="text" :color="isDark ? 'white' : 'brown-darken-3'" @click="$emit('open-github')" size="small" class="mx-1 footer-btn jedi-icon-btn"></v-btn>
        <v-btn :icon="mdiHelpCircle" variant="text" :color="isDark ? 'white' : 'brown-darken-3'" size="small" class="mx-1 footer-btn jedi-icon-btn" @click="$emit('show-help')"></v-btn>
        <v-btn :icon="mdiCog" variant="text" :color="isDark ? 'white' : 'brown-darken-3'" size="small" class="mx-1 footer-btn jedi-icon-btn" @click="$emit('show-settings')"></v-btn>
        <v-btn :icon="mdiInformation" variant="text" :color="isDark ? 'white' : 'brown-darken-3'" size="small" class="mx-1 footer-btn jedi-icon-btn" @click="$emit('show-about')"></v-btn>
      </div>

      <!-- 版权信息 -->
      <div class="copyright-text mt-1" :class="{'light-text': !isDark}">
        <span :class="isDark ? 'text-white' : 'text-brown-darken-4'">© 2025</span> <span class="brand-text" :class="{'light-brand': !isDark}">Jedi</span>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { mdiGithub, mdiHelpCircle, mdiCog, mdiInformation, mdiWeatherNight, mdiWeatherSunny, mdiThemeLightDark } from '@mdi/js'
import { useTheme } from '@/composables/useTheme'

// 获取主题状态
const { t } = useI18n()
const { isDark, themeMode, setTheme } = useTheme()

// 计算属性图标
const themeIcon = computed(() => {
  if (themeMode.value === 'dark') return mdiWeatherNight
  if (themeMode.value === 'light') return mdiWeatherSunny
  return mdiThemeLightDark
})

// 主题文本提示
const themeTooltip = computed(() => {
  if (themeMode.value === 'dark') return t('theme.dark')
  if (themeMode.value === 'light') return t('theme.light')
  return t('theme.system')
})

// 切换主题
function toggleTheme() {
  // 切换顺序: 浅色 -> 深色 -> 跟随系统 -> 浅色
  if (themeMode.value === 'light') {
    setTheme('dark')
  } else if (themeMode.value === 'dark') {
    setTheme('system')
  } else {
    setTheme('light')
  }
}

// 定义组件事件
defineEmits<{
  (e: 'open-github'): void;
  (e: 'open-website'): void;
  (e: 'open-email'): void;
  (e: 'show-help'): void;
  (e: 'show-settings'): void;
  (e: 'show-about'): void;
}>()
</script>

<style scoped>
.nav-footer {
  position: absolute;
  bottom: 0;
  left: 0;
  width: 100%;
  background-color: transparent;
  z-index: 10;
  overflow: hidden;
  height: 120px; /* 稍微增加高度以容纳效果 */
  display: flex;
  align-items: flex-end;
  /* 顶部边缘淡出，实现与侧边栏的无缝融合 */
  -webkit-mask-image: linear-gradient(to bottom, transparent 0%, black 30%);
  mask-image: linear-gradient(to bottom, transparent 0%, black 30%);
  pointer-events: none; /* 让鼠标事件穿透非按钮区域 */
}

/* 恢复按钮区域的交互 */
.footer-console {
  pointer-events: auto;
}

/* ================== 深色模式样式 ================== */

/* 深空背景 */
.deep-space-bg {
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  background: linear-gradient(180deg, #020409 0%, #050b1a 40%, #0a1835 100%);
  z-index: -2;
}

/* 星云发光 */
.nebula-glow {
  position: absolute;
  bottom: -50px;
  left: 50%;
  transform: translateX(-50%);
  width: 200%;
  height: 150px;
  background: radial-gradient(ellipse at center, rgba(60, 100, 255, 0.25) 0%, transparent 70%);
  z-index: -1;
  pointer-events: none;
  animation: pulse-glow 8s infinite alternate;
}

/* 简单的星星生成 (使用 box-shadow) */
.stars-layer {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  z-index: -1;
}

.small-stars {
  width: 1px;
  height: 1px;
  background: transparent;
  box-shadow: 
    10px 10px #FFF, 50px 80px #FFF, 120px 30px #FFF, 
    180px 10px #FFF, 40px 110px #FFF, 150px 90px #FFF,
    90px 40px #FFF, 20px 60px #FFF, 160px 50px #FFF,
    70px 20px #FFF, 110px 100px #FFF, 190px 70px #FFF,
    30px 30px rgba(255,255,255,0.5), 140px 20px rgba(255,255,255,0.5);
  animation: stars-move-1 100s linear infinite;
}

.medium-stars {
  width: 2px;
  height: 2px;
  background: transparent;
  box-shadow: 
    30px 50px rgba(255,255,255,0.6), 100px 20px rgba(255,255,255,0.6), 
    160px 80px rgba(255,255,255,0.6), 60px 90px rgba(255,255,255,0.6);
  animation: stars-move-2 150s linear infinite;
}

/* ================== 浅色模式样式 (曼达洛人日落) ================== */

/* 日落背景 */
.sunset-bg {
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  /* 沙漠日落渐变: 顶部深红褐 -> 中部橙红 -> 底部金黄 */
  background: linear-gradient(180deg, #5D4037 0%, #D84315 40%, #FFA000 100%);
  z-index: -2;
}

/* 暖色光晕 */
.sunset-glow {
  position: absolute;
  bottom: -30px;
  left: 50%;
  transform: translateX(-50%);
  width: 150%;
  height: 120px;
  background: radial-gradient(ellipse at center, rgba(255, 236, 179, 0.4) 0%, rgba(255, 111, 0, 0.1) 60%, transparent 80%);
  z-index: -1;
  pointer-events: none;
  animation: sunset-pulse 6s infinite alternate;
}

/* 尘埃粒子效果 */
.dust-particles {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  z-index: -1;
  background-image: 
    radial-gradient(1px 1px at 20px 30px, rgba(255, 255, 255, 0.6), transparent),
    radial-gradient(1px 1px at 40px 70px, rgba(255, 255, 255, 0.6), transparent),
    radial-gradient(2px 2px at 90px 40px, rgba(255, 224, 178, 0.5), transparent),
    radial-gradient(1px 1px at 160px 120px, rgba(255, 255, 255, 0.6), transparent);
  background-size: 200px 200px;
  animation: dust-float 30s linear infinite;
}

/* ================== 通用控制台样式 ================== */

/* 底部控制台 */
.footer-console {
  width: 100%;
  position: relative;
  z-index: 2;
  backdrop-filter: blur(2px);
  background: linear-gradient(to top, rgba(10, 20, 40, 0.9), rgba(10, 20, 40, 0));
  transition: background 0.3s ease;
}

.footer-console.light-console {
  background: linear-gradient(to top, rgba(93, 64, 55, 0.8), rgba(93, 64, 55, 0));
}

/* 装饰线 */
.console-line {
  width: 60%;
  height: 1px;
  background: linear-gradient(90deg, transparent, rgba(100, 150, 255, 0.6), transparent);
  margin-bottom: 12px;
  opacity: 0.6;
}

.console-line.light-line {
  background: linear-gradient(90deg, transparent, rgba(255, 160, 0, 0.6), transparent);
}

/* 版权文字 */
.copyright-text {
  /* font-family: var(--jedi-font-ui); */
  font-size: 11px;
  color: rgba(255, 255, 255, 0.8);
  letter-spacing: 1px;
  text-transform: uppercase;
  font-weight: 500;
  text-shadow: 0 0 5px rgba(0, 0, 0, 0.5);
}

.copyright-text.light-text {
  color: rgba(255, 248, 225, 0.9);
  text-shadow: 0 0 3px rgba(62, 39, 35, 0.5);
}

.brand-text {
  color: #a0c4ff;
  font-weight: 700;
  text-shadow: 0 0 8px rgba(60, 100, 255, 0.8), 0 0 15px rgba(60, 100, 255, 0.4);
  margin-left: 4px;
}

.brand-text.light-brand {
  color: #FFECB3;
  text-shadow: 0 0 8px rgba(255, 111, 0, 0.8), 0 0 15px rgba(255, 111, 0, 0.4);
}

/* 按钮样式 */
.jedi-icon-btn {
  opacity: 0.7;
  transition: all 0.3s cubic-bezier(0.25, 0.8, 0.25, 1);
}

.jedi-icon-btn:hover {
  opacity: 1;
  transform: translateY(-2px);
  text-shadow: 0 0 8px rgba(255, 255, 255, 0.6);
  background: rgba(255, 255, 255, 0.05);
}

/* ================== 动画定义 ================== */

@keyframes pulse-glow {
  0% { opacity: 0.6; transform: translateX(-50%) scale(1); }
  100% { opacity: 1; transform: translateX(-50%) scale(1.1); }
}

@keyframes sunset-pulse {
  0% { opacity: 0.7; transform: translateX(-50%) scale(1); }
  100% { opacity: 1; transform: translateX(-50%) scale(1.05); }
}

@keyframes stars-move-1 {
  from { transform: translateY(0); }
  to { transform: translateY(-200px); }
}

@keyframes stars-move-2 {
  from { transform: translateY(0); }
  to { transform: translateY(-150px); }
}

@keyframes dust-float {
  0% { background-position: 0 0; }
  100% { background-position: 50px -100px; }
}
</style>
