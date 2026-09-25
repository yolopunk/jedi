import { createRouter, createWebHistory, type RouteRecordRaw } from 'vue-router'
import AiChat from '@/views/AiChat/index.vue'
import HostsManager from '@/views/hosts/HostsManager.vue'
import PodcastManager from '@/views/podcast/PodcastManager.vue'
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
