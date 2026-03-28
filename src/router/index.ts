import { createRouter, createWebHistory, RouteRecordRaw } from 'vue-router'
import HostsManager from '@/views/hosts/HostsManager.vue'
import WallpaperManager from '@/views/wallpapers/WallpaperManager.vue'
import PodcastManager from '@/views/podcast/PodcastManager.vue'
import AiChat from '@/views/AiChat/index.vue'

const routes: Array<RouteRecordRaw> = [
    {
        path: '/',
        redirect: '/chat'
    },
    {
        path: '/hosts',
        name: 'Hosts',
        component: HostsManager,
        meta: {
            title: 'Hosts Manager'
        }
    },
    {
        path: '/wallpapers',
        name: 'Wallpapers',
        component: WallpaperManager,
        meta: {
            title: 'Wallpapers'
        }
    },
    {
        path: '/podcast',
        name: 'Podcast',
        component: PodcastManager,
        meta: {
            title: 'Podcast'
        }
    },
    {
        path: '/chat',
        name: 'AiChat',
        component: AiChat,
        meta: {
            title: 'Chat'
        }
    }
]

const router = createRouter({
    history: createWebHistory(),
    routes
})

export default router
