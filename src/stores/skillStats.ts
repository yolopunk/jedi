// src/stores/skillStats.ts
//
// Per-skill execution stats. recordToolEnd() is called from withStatsHooks
// on every tool execution (foreground chat and background pool workers).
// Counters are aggregated in memory and flushed to the backend file
// (~/.jedi/skill_stats.json) in whole-map batches, debounced by 3s — the
// data is small and bounded, so at most the last 3s of counters can be lost.

import { defineStore } from 'pinia'
import { ref } from 'vue'
import type { SkillStatEntry } from '@/api/skills'
import { clearSkillStats, listSkillStats, saveSkillStats } from '@/api/skills'

const FLUSH_DELAY_MS = 3000

export const useSkillStatsStore = defineStore('skillStats', () => {
  const stats = ref<Record<string, SkillStatEntry>>({})
  const loaded = ref(false)
  let flushTimer: ReturnType<typeof setTimeout> | null = null

  /** 启动时加载落盘统计（幂等） */
  async function loadStats(): Promise<void> {
    if (loaded.value) return
    loaded.value = true
    try {
      stats.value = await listSkillStats()
    } catch (e) {
      console.error('Failed to load skill stats:', e)
    }
  }

  /** 工具执行结束记账：计数、耗时、错误（含拒绝，v1 不区分） */
  function recordToolEnd(event: { skillId: string; startedAt: number; error?: string }): void {
    const entry: SkillStatEntry = stats.value[event.skillId] ?? {
      calls: 0,
      failures: 0,
      totalMs: 0,
      lastUsedAt: 0,
      recentErrors: [],
    }
    entry.calls += 1
    entry.totalMs += Math.max(0, Date.now() - event.startedAt)
    entry.lastUsedAt = Date.now()
    if (event.error) {
      entry.failures += 1
      entry.recentErrors.push({ ts: Date.now(), msg: event.error.slice(0, 200) })
      if (entry.recentErrors.length > 10) {
        entry.recentErrors = entry.recentErrors.slice(-10)
      }
    }
    stats.value[event.skillId] = entry
    scheduleFlush()
  }

  function scheduleFlush(): void {
    if (flushTimer) return
    flushTimer = setTimeout(() => {
      flushTimer = null
      void flush()
    }, FLUSH_DELAY_MS)
  }

  /** 整表批量落盘 */
  async function flush(): Promise<void> {
    try {
      await saveSkillStats(stats.value)
    } catch (e) {
      console.error('Failed to flush skill stats:', e)
    }
  }

  /** 清零全部统计（内存 + 落盘） */
  async function clearAll(): Promise<void> {
    stats.value = {}
    if (flushTimer) {
      clearTimeout(flushTimer)
      flushTimer = null
    }
    try {
      await clearSkillStats()
    } catch (e) {
      console.error('Failed to clear skill stats:', e)
    }
  }

  /** 查询单个技能统计；无记录返回 null */
  function statsFor(skillId: string): SkillStatEntry | null {
    return stats.value[skillId] ?? null
  }

  return {
    stats,
    loaded,
    loadStats,
    recordToolEnd,
    flush,
    clearAll,
    statsFor,
  }
})
