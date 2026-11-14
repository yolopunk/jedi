import { createRouter, createWebHistory, type RouteRecordRaw } from 'vue-router'
import AiChat from '@/views/AiChat/index.vue'
import HostsManager from '@/views/hosts/HostsManager.vue'
import PodcastManager from '@/views/podcast/PodcastManager.vue'
import ScreenshotManager from '@/views/Screenshot/index.vue'
import ScreenshotOverlay from '@/views/Screenshot/Overlay.vue'
import ScreenshotToolbar from '@/views/Screenshot/Toolbar.vue'
import SkillsManager from '@/views/skills/SkillsManager.vue'
import WallpaperManager from '@/views/wallpapers/WallpaperManager.vue'

const routes: Array<RouteRecordRaw> = [
  {
    path: '/',
    redirect: '/chat',
  },
  {
    path: '/skills',
    name: 'Skills',
    component: SkillsManager,
    meta: {
      title: 'Skills',
    },
  },
  {
    path: '/hosts',
    name: 'Hosts',
    component: HostsManager,
    meta: {
      title: 'Hosts Manager',
    },
  },
  {
    path: '/wallpapers',
    name: 'Wallpapers',
    component: WallpaperManager,
    meta: {
      title: 'Wallpapers',
    },
  },
  {
    path: '/podcast',
    name: 'Podcast',
    component: PodcastManager,
    meta: {
      title: 'Podcast',
    },
  },
  {
    path: '/screenshot',
    name: 'Screenshot',
    component: ScreenshotManager,
    meta: {
      title: 'Screenshot',
    },
  },
  {
    // 截图覆盖窗口专用路由：绕过主布局，仅在有截图会话时由 Rust 创建的窗口加载
    path: '/screenshot-overlay',
    name: 'ScreenshotOverlay',
    component: ScreenshotOverlay,
    meta: {
      title: 'Screenshot Overlay',
      bare: true,
    },
  },
  {
    // 截图悬浮控制条窗口：滚动截长图 / 录屏模式由 Rust 创建
    path: '/screenshot-toolbar',
    name: 'ScreenshotToolbar',
    component: ScreenshotToolbar,
    meta: {
      title: 'Screenshot Toolbar',
      bare: true,
    },
  },
  {
    path: '/chat',
    name: 'AiChat',
    component: AiChat,
    meta: {
      title: 'Chat',
    },
  },
]

const router = createRouter({
  history: createWebHistory(),
  routes,
})

export default router
