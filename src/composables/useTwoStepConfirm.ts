// src/composables/useTwoStepConfirm.ts
//
// Two-step confirm state for destructive actions (delete / reset): the first
// call arms the id, a second call on the same id within the window confirms.
// Auto-disarms after 3s so a stray armed state doesn't linger.

import { ref } from 'vue'

const RESET_MS = 3000

export function useTwoStepConfirm() {
  const armedId = ref<string | null>(null)
  let timer: ReturnType<typeof setTimeout> | null = null

  function disarm(): void {
    armedId.value = null
    if (timer) {
      clearTimeout(timer)
      timer = null
    }
  }

  /** 对同一 id 连续调用两次视为确认；返回本次调用是否构成确认 */
  function arm(id: string): boolean {
    if (armedId.value === id) {
      disarm()
      return true
    }
    armedId.value = id
    if (timer) clearTimeout(timer)
    timer = setTimeout(disarm, RESET_MS)
    return false
  }

  return { armedId, arm, disarm }
}
