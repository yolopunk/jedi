import { ref } from 'vue'

export type AppNotificationColor = 'success' | 'error' | 'info' | 'warning'

/** 通知时长规范（ms）：success/info 3s、warning 5s、error 6s */
const DURATION_BY_COLOR: Record<AppNotificationColor, number> = {
  success: 3000,
  info: 3000,
  warning: 5000,
  error: 6000,
}

// 模块级单例状态：全应用共享同一条顶部通知
const visible = ref(false)
const text = ref('')
const color = ref<AppNotificationColor>('success')

let timer: ReturnType<typeof setTimeout> | null = null

function clearTimer() {
  if (timer !== null) {
    clearTimeout(timer)
    timer = null
  }
}

/** 立即关闭通知并清空计时器 */
function dismiss() {
  clearTimer()
  visible.value = false
}

/**
 * 弹出全局通知。
 * 连续调用会重置计时器并替换内容；
 * 自动消失时长按时色规范：success/info 3000ms、warning 5000ms、error 6000ms。
 */
function notify(message: string, nextColor: AppNotificationColor = 'success') {
  clearTimer()
  text.value = message
  color.value = nextColor
  visible.value = true
  timer = setTimeout(() => {
    timer = null
    visible.value = false
  }, DURATION_BY_COLOR[nextColor])
}

export function useAppNotification() {
  return { notify, visible, text, color, dismiss }
}
