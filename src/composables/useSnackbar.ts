import { ref } from 'vue'

export type SnackbarColor = 'success' | 'error' | 'info' | 'warning'

export function useSnackbar() {
  const show = ref(false)
  const text = ref('')
  const color = ref<SnackbarColor>('success')

  function notify(message: string, type: SnackbarColor = 'success') {
    text.value = message
    color.value = type
    show.value = true
  }

  return { show, text, color, notify }
}
