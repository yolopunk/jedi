import { ref, watch, onMounted } from 'vue'
import { useStorage } from '@/composables/useStorage'

// 主题类型
export type ThemeMode = 'light' | 'dark' | 'system'

// 创建主题存储
const storage = useStorage()

// 主题状态
const themeMode = ref<ThemeMode>('system')
const isDark = ref(false)

// 检测系统主题
const prefersDark = window.matchMedia('(prefers-color-scheme: dark)')

// 监听系统主题变化
const updateSystemTheme = () => {
  if (themeMode.value === 'system') {
    isDark.value = prefersDark.matches
    applyTheme()
  }
}

// 应用主题到DOM
const applyTheme = () => {
  // 应用到文档根元素
  if (isDark.value) {
    document.documentElement.classList.add('dark-theme')
    document.documentElement.classList.remove('light-theme')
  } else {
    document.documentElement.classList.add('light-theme')
    document.documentElement.classList.remove('dark-theme')
  }
}

// 设置主题
export const setTheme = (mode: ThemeMode) => {
  themeMode.value = mode
  
  // 根据模式设置暗色状态
  if (mode === 'light') {
    isDark.value = false
  } else if (mode === 'dark') {
    isDark.value = true
  } else {
    // 系统模式，跟随系统设置
    isDark.value = prefersDark.matches
  }
  
  // 保存设置到本地存储
  storage.setItem('theme-mode', mode)
  
  // 应用主题
  applyTheme()
}

// 初始化主题
export const initTheme = async () => {
  // 从存储中读取主题设置
  const savedMode = await storage.getItem<ThemeMode>('theme-mode')
  if (savedMode) {
    setTheme(savedMode)
  } else {
    // 默认使用系统主题
    setTheme('system')
  }
}

// 监听主题变化
watch(themeMode, () => {
  applyTheme()
})

// 导出组合式函数
export function useTheme() {
  onMounted(() => {
    // 初始化主题
    initTheme()
    
    // 监听系统主题变化
    prefersDark.addEventListener('change', updateSystemTheme)
  })
  
  return {
    themeMode,
    isDark,
    setTheme
  }
}
