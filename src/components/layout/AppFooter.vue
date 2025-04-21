<template>
  <div class="nav-footer px-4 pt-4">
    <!-- 星空背景效果 -->
    <div class="starfield-container">
      <div class="stars-background"></div>
      <div class="shooting-star star-1"></div>
      <div class="shooting-star star-2"></div>
      <div class="shooting-star star-3"></div>
      <div class="blue-star"></div>
    </div>

    <div class="d-flex flex-column align-center">
      <!-- 底部按钮 -->
      <div class="d-flex justify-center w-100 mb-1">
        <v-btn :icon="mdiGithub" variant="text" color="white" @click="$emit('open-github')" size="small" class="mx-1 footer-btn"></v-btn>
        <v-btn :icon="mdiHelpCircle" variant="text" color="white" size="small" class="mx-1 footer-btn" @click="$emit('show-help')"></v-btn>
        <v-btn :icon="mdiCog" variant="text" color="white" size="small" class="mx-1 footer-btn" @click="$emit('show-settings')"></v-btn>
        <v-btn :icon="mdiInformation" variant="text" color="white" size="small" class="mx-1 footer-btn" @click="$emit('show-about')"></v-btn>
      </div>

      <!-- 版权信息 -->
      <div class="text-caption text-white mt-1">© 2025 Jedi</div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { mdiGithub, mdiHelpCircle, mdiCog, mdiInformation } from '@mdi/js'

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
  position: absolute; /* 改为absolute定位，避免与页面滚动冲突 */
  bottom: 0;
  left: 0;
  width: 100%; /* 使用100%宽度，但在父容器中已限制为200px */
  background-color: transparent;
  z-index: 1;
  padding-bottom: 8px;
  overflow: hidden;
  transition: all 0.3s ease;
}

/* 星空背景效果 */
.starfield-container {
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  overflow: hidden;
  z-index: -1;
  background: linear-gradient(to top, rgba(44, 62, 80, 0.85), rgba(44, 62, 80, 0.7));
  border-top: 1px solid rgba(52, 152, 219, 0.3);
  box-shadow: 0 -3px 10px rgba(44, 62, 80, 0.4);
  backdrop-filter: blur(3px);
}

.stars-background {
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  background-image:
    /* 小星星 - 降低亮度 */
    radial-gradient(1px 1px at 25px 5px, rgba(255, 255, 255, 0.7), transparent),
    radial-gradient(1px 1px at 50px 25px, rgba(255, 255, 255, 0.7), transparent),
    radial-gradient(1px 1px at 125px 20px, rgba(255, 255, 255, 0.7), transparent),
    radial-gradient(1px 1px at 50px 75px, rgba(255, 255, 255, 0.7), transparent),
    radial-gradient(1px 1px at 15px 45px, rgba(255, 255, 255, 0.7), transparent),
    radial-gradient(1px 1px at 110px 70px, rgba(255, 255, 255, 0.7), transparent),
    /* 减少星星数量 */
    radial-gradient(1px 1px at 160px 15px, rgba(255, 255, 255, 0.7), transparent),
    radial-gradient(1px 1px at 85px 65px, rgba(255, 255, 255, 0.7), transparent),
    /* 中等星星 - 降低亮度 */
    radial-gradient(1.5px 1.5px at 40px 30px, rgba(255, 255, 255, 0.8), transparent),
    radial-gradient(1.5px 1.5px at 100px 50px, rgba(255, 255, 255, 0.8), transparent),
    /* 大星星 - 保留但降低亮度 */
    radial-gradient(2px 2px at 70px 60px, rgba(255, 255, 255, 0.8), transparent),
    /* 蓝色星星 - 与主题色调协调 */
    radial-gradient(1.5px 1.5px at 55px 45px, rgba(52, 152, 219, 0.8), transparent),
    radial-gradient(1.5px 1.5px at 120px 60px, rgba(52, 152, 219, 0.8), transparent);
  animation: twinkle 5s infinite alternate;
}

.shooting-star {
  position: absolute;
  width: 60px; /* 减小流星长度 */
  height: 1px; /* 减小流星宽度 */
  background: linear-gradient(to right, transparent, rgba(255, 255, 255, 0.8), transparent);
  transform: rotate(15deg);
  opacity: 0;
  animation: shooting-star 12s infinite ease-out; /* 减慢动画频率 */
  box-shadow: 0 0 6px rgba(255, 255, 255, 0.7); /* 减弱发光效果 */
  z-index: 0;
}

.star-1 {
  top: 20px;
  left: -80px;
  animation-delay: 2s;
}

.star-2 {
  top: 40px;
  left: -80px;
  width: 60px;
  transform: rotate(5deg);
  animation-delay: 6s;
}

.star-3 {
  top: 10px;
  left: -60px;
  width: 40px;
  transform: rotate(25deg);
  animation-delay: 10s;
}

.blue-star {
  position: absolute;
  top: 30px;
  left: 50%;
  width: 3px; /* 减小尺寸 */
  height: 3px;
  background-color: rgba(52, 152, 219, 0.9); /* 降低亮度 */
  border-radius: 50%;
  box-shadow: 0 0 10px rgba(52, 152, 219, 0.8); /* 减弱发光效果 */
  animation: blue-star-pulse 4s infinite alternate; /* 放慢动画 */
  z-index: 0;
}

.footer-btn {
  position: relative;
  z-index: 2;
  background-color: rgba(52, 152, 219, 0.15); /* 降低背景色强度 */
  border-radius: 50%;
  transition: all 0.3s ease;
}

.footer-btn:hover {
  background-color: rgba(52, 152, 219, 0.3); /* 降低悬停背景色强度 */
  transform: translateY(-2px);
  box-shadow: 0 0 10px rgba(52, 152, 219, 0.4); /* 减弱发光效果 */
}

@keyframes twinkle {
  0% {
    opacity: 0.5; /* 降低最小不透明度 */
  }
  100% {
    opacity: 0.8; /* 降低最大不透明度 */
  }
}

@keyframes shooting-star {
  0% {
    transform: translateX(0) rotate(15deg);
    opacity: 0;
  }
  3% {
    opacity: 0.7; /* 降低最大不透明度 */
  }
  10% {
    transform: translateX(180px) rotate(15deg); /* 减少移动距离 */
    opacity: 0;
  }
  100% {
    opacity: 0;
    transform: translateX(180px) rotate(15deg);
  }
}

@keyframes blue-star-pulse {
  0% {
    transform: scale(1);
    opacity: 0.7; /* 降低最小不透明度 */
    box-shadow: 0 0 8px rgba(52, 152, 219, 0.6); /* 减弱发光效果 */
  }
  50% {
    transform: scale(1.3); /* 减少缩放幅度 */
    opacity: 0.9; /* 降低最大不透明度 */
    box-shadow: 0 0 12px rgba(52, 152, 219, 0.8); /* 减弱发光效果 */
  }
  100% {
    transform: scale(1);
    opacity: 0.7;
    box-shadow: 0 0 8px rgba(52, 152, 219, 0.6);
  }
}
</style>
