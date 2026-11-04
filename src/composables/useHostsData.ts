import { computed, ref, shallowRef } from 'vue'
import {
  fetchRemoteConfig,
  getHostsSources,
  initializeDefaultConfig as initDefaultConfig,
  readSystemHosts,
  removeHostsSource,
  revertHosts,
  saveHostsSource,
  updateHostsWithGroups,
} from '@/api/hosts'
import type { Group, HostEntry, HostsSource } from '@/types/hosts'
import {
  disableAllHosts,
  enableAllHosts,
  findHostEntry,
  findHostIndex,
  updateHostEntryStatus,
} from '@/utils/hostsUtils'

export type NotifyFunction = (text: string, color: 'success' | 'error' | 'info' | 'warning') => void

export function useHostsData(notify: NotifyFunction) {
  // ===== 状态变量 =====
  const groups = ref<Group[]>([])
  const selectedGroup = ref<string>('')
  const hostsResolveSwitch = ref(false)
  const loading = shallowRef(false)
  // 远程数据源（URL），与分组一一对应
  const remoteSources = ref<HostsSource[]>([])
  const syncing = shallowRef(false)

  // 批量更新状态
  const pendingUpdates = ref(false)
  const updateDebounceTimeout = ref<NodeJS.Timeout | null>(null)

  // ===== 计算属性 =====
  const currentGroup = computed(() => {
    return groups.value.find((g: Group) => g.name === selectedGroup.value)
  })

  // ===== 内部辅助方法 =====

  function updateGlobalSwitchState(result: Group[]) {
    let hasEnabledEntries = false
    for (const group of result) {
      for (const host of group.hosts) {
        if (!host.disabled) {
          hasEnabledEntries = true
          break
        }
      }
      if (hasEnabledEntries) break
    }
    hostsResolveSwitch.value = hasEnabledEntries
  }

  function initializeEmptyGroups() {
    groups.value = []
    selectedGroup.value = ''
    hostsResolveSwitch.value = false
  }

  function getCurrentGroup(): Group | null {
    const group = groups.value.find((g: Group) => g.name === selectedGroup.value)
    if (!group) {
      notify('未找到对应分组', 'error')
      return null
    }
    return group
  }

  // ===== 远程数据源同步 =====

  /** 拉取数据源并合并到对应分组：同域名保留本地禁用状态，远端新增进入、已删除移除 */
  async function refreshSource(
    source: HostsSource,
    options?: { silent?: boolean }
  ): Promise<boolean> {
    try {
      const result = await fetchRemoteConfig(source.url)

      if (!Array.isArray(result) || result.length === 0) {
        if (!options?.silent) notify('远程数据为空或格式不正确', 'error')
        return false
      }

      const matchedGroup =
        result.find((g: Group) => g.name === source.name) ||
        (result.length === 1 ? result[0] : null)
      if (!matchedGroup || !Array.isArray(matchedGroup.hosts)) {
        if (!options?.silent) notify(`远程数据中未找到分组 ${source.name}`, 'error')
        return false
      }

      // 保留本地手动禁用/启用的状态，其余跟随远端
      const localGroup = groups.value.find((g: Group) => g.name === source.name)
      const localDisabled = new Map<string, boolean>()
      if (localGroup) {
        for (const host of localGroup.hosts) {
          localDisabled.set(host.domain, host.disabled)
        }
      }

      // 全局解析开关处于「关闭」时（已有分组且无任何启用条目），
      // 远端新增条目也必须保持禁用，否则自动同步会把「全部关闭」状态悄悄打破
      const globalResolveOff = groups.value.length > 0 && !hostsResolveSwitch.value

      const mergedHosts: HostEntry[] = matchedGroup.hosts
        .filter((host: HostEntry) => host && host.domain && host.ip)
        .map((host: HostEntry) => {
          if (localDisabled.has(host.domain)) {
            return {
              ip: host.ip,
              domain: host.domain,
              disabled: localDisabled.get(host.domain) as boolean,
            }
          }
          return {
            ip: host.ip,
            domain: host.domain,
            disabled: globalResolveOff ? true : !!host.disabled,
          }
        })

      if (localGroup) {
        localGroup.hosts = mergedHosts
      } else {
        // 分组被手动清空过：按数据源重建
        groups.value.push({ name: source.name, hosts: mergedHosts })
      }

      await debouncedUpdateHosts()

      const syncedAt = Math.floor(Date.now() / 1000)
      remoteSources.value = await saveHostsSource({ ...source, lastSyncedAt: syncedAt })

      if (!options?.silent) {
        notify(`已同步分组 ${source.name}（${mergedHosts.length} 条）`, 'success')
      }
      return true
    } catch (error) {
      console.error('同步远程数据源失败:', error)
      notify(`同步 ${source.name} 失败: ${(error as Error).message || error}`, 'error')
      return false
    }
  }

  /** 加载远程数据源列表并后台自动同步一次（打开页面时调用，不阻塞加载） */
  async function syncRemoteSources() {
    try {
      remoteSources.value = await getHostsSources()
    } catch (error) {
      console.error('加载远程数据源失败:', error)
      return
    }
    if (remoteSources.value.length === 0) return

    syncing.value = true
    try {
      for (const source of remoteSources.value) {
        // 静默模式：成功不提示，失败仍提示（集群不可达时用户需要感知）
        await refreshSource(source, { silent: true })
      }
    } finally {
      syncing.value = false
    }
  }

  // ===== 核心业务方法 =====

  async function loadSystemHosts() {
    loading.value = true
    try {
      const result = await readSystemHosts()

      if (Array.isArray(result) && result.length > 0) {
        groups.value = result
        if (groups.value.length > 0) {
          selectedGroup.value = groups.value[0].name
        }
        updateGlobalSwitchState(result)
        notify('成功加载系统 Hosts 配置', 'success')
      } else {
        initializeEmptyGroups()
      }
    } catch (error) {
      console.error('加载系统 Hosts 失败:', error)
      notify(`加载系统 Hosts 失败: ${(error as Error).message}`, 'error')
      initializeEmptyGroups()
    } finally {
      loading.value = false
    }

    // 后台同步远程数据源（URL / K8s API），不阻塞页面加载
    void syncRemoteSources()
  }

  async function updateHosts() {
    loading.value = true
    try {
      await updateHostsWithGroups(groups.value)
    } catch (error) {
      console.error('更新hosts失败', error)
      throw error
    } finally {
      loading.value = false
    }
  }

  // 防抖写入：多次调用合并为一次落盘，且所有调用者的 promise 一起 settle。
  // 旧实现每次调用都会清掉上一个定时器，而被清掉那次调用的 promise 永远不会
  // settle —— 自动同步与用户操作并发时会让同步流程永久挂起（syncing 卡为 true）。
  let updateWaiters: Array<{ resolve: () => void; reject: (reason?: unknown) => void }> = []

  function debouncedUpdateHosts(): Promise<void> {
    pendingUpdates.value = true

    if (updateDebounceTimeout.value) {
      clearTimeout(updateDebounceTimeout.value)
    }

    return new Promise<void>((resolve, reject) => {
      updateWaiters.push({ resolve, reject })

      updateDebounceTimeout.value = setTimeout(async () => {
        updateDebounceTimeout.value = null
        pendingUpdates.value = false

        const waiters = updateWaiters
        updateWaiters = []

        try {
          loading.value = true
          await updateHostsWithGroups(groups.value)
          for (const waiter of waiters) waiter.resolve()
        } catch (error) {
          console.error('批量更新hosts失败', error)
          for (const waiter of waiters) waiter.reject(error)
        } finally {
          loading.value = false
        }
      }, 500)
    })
  }

  async function handleHostsSwitch(switchState: boolean) {
    loading.value = true
    try {
      if (switchState) {
        enableAllHosts(groups.value)
        await updateHosts()
        notify('Hosts解析已启用，所有条目已生效', 'success')
      } else {
        disableAllHosts(groups.value)
        await revertHosts()
        notify('Hosts解析已禁用，所有条目已暂停生效，但配置已保留', 'info')
      }
    } catch (error) {
      console.error('切换Hosts解析状态失败', error)
      notify(`操作失败: ${(error as Error).message}`, 'error')
      hostsResolveSwitch.value = !switchState
    } finally {
      loading.value = false
    }
  }

  async function initializeDefaultConfig() {
    loading.value = true
    try {
      await initDefaultConfig()
      notify('默认配置初始化成功', 'success')
      await loadSystemHosts()
    } catch (error) {
      console.error('初始化默认配置失败', error)
      notify(`初始化失败: ${(error as Error).message}`, 'error')
    } finally {
      loading.value = false
    }
  }

  // ===== CRUD 操作 =====

  async function addGroup(data: {
    name: string
    isRemote: boolean
    url?: string
    hosts?: HostEntry[]
  }) {
    groups.value.push({
      name: data.name,
      hosts: data.hosts || [],
    })

    // 远程数据源持久化，供后续自动/手动同步
    let sourceSaved = false
    if (data.isRemote && data.url) {
      const source: HostsSource = {
        name: data.name,
        url: data.url,
        lastSyncedAt: Math.floor(Date.now() / 1000),
      }
      try {
        remoteSources.value = await saveHostsSource(source)
        sourceSaved = true
      } catch (error) {
        // 数据源保存失败不阻塞分组创建，仅无法自动刷新
        console.error('保存远程数据源失败:', error)
      }
    }

    try {
      await debouncedUpdateHosts()
      selectedGroup.value = data.name
      notify('分组添加成功', 'success')
    } catch (error) {
      console.error('添加分组失败:', error)
      notify(`添加失败: ${(error as Error).message}`, 'error')
      groups.value.pop()
      if (sourceSaved) {
        try {
          remoteSources.value = await removeHostsSource(data.name)
        } catch (rollbackError) {
          console.error('回滚远程数据源失败:', rollbackError)
        }
      }
    }
  }

  async function addHost(data: { groupName: string; ip: string; domain: string }) {
    const group = groups.value.find((g: Group) => g.name === data.groupName)
    if (!group) {
      notify('未找到对应分组', 'error')
      return
    }

    const domainExists = group.hosts.some(host => host.domain === data.domain)
    if (domainExists) {
      notify('该域名已存在于当前分组中', 'error')
      return
    }

    group.hosts.push({
      domain: data.domain,
      ip: data.ip,
      disabled: false,
    })

    try {
      selectedGroup.value = data.groupName
      await debouncedUpdateHosts()
      notify('条目添加成功', 'success')
    } catch (error) {
      console.error('添加条目失败', error)
      notify(`添加失败: ${(error as Error).message}`, 'error')
      group.hosts.pop()
    }
  }

  async function editHost(data: { originalHost: HostEntry; ip: string; domain: string }) {
    const group = getCurrentGroup()
    if (!group) return

    const hostEntry = findHostEntry(group, data.originalHost)
    if (!hostEntry) {
      notify('未找到要编辑的条目', 'error')
      return
    }

    if (data.domain !== data.originalHost.domain) {
      const domainExists = group.hosts.some(
        host => host.domain === data.domain && host !== hostEntry
      )
      if (domainExists) {
        notify('该域名已存在于当前分组中', 'error')
        return
      }
    }

    const isDisabled = hostEntry.disabled
    const index = group.hosts.indexOf(hostEntry)
    if (index !== -1) {
      group.hosts.splice(index, 1)
    }

    group.hosts.push({
      domain: data.domain,
      ip: data.ip,
      disabled: isDisabled,
    })

    try {
      await debouncedUpdateHosts()
      notify('条目编辑成功', 'success')
    } catch (error) {
      console.error('更新状态失败', error)
      notify(`更新状态失败: ${(error as Error).message}`, 'error')
    }
  }

  async function updateHostStatus(host: HostEntry, enabled?: boolean) {
    const group = getCurrentGroup()
    if (!group) return

    const hostEntry = findHostEntry(group, host)
    if (!hostEntry) return

    const isEnabled = enabled ?? !hostEntry.disabled
    updateHostEntryStatus(hostEntry, isEnabled)

    try {
      await debouncedUpdateHosts()
      notify(isEnabled ? '条目已启用' : '条目已禁用', isEnabled ? 'success' : 'info')
    } catch (error) {
      console.error('更新状态失败', error)
      notify(`更新状态失败: ${(error as Error).message}`, 'error')
      hostEntry.disabled = !isEnabled
    }
  }

  async function confirmDeleteHost(host: HostEntry) {
    const group = getCurrentGroup()
    if (!group) return

    const index = findHostIndex(group, host)

    if (index !== -1) {
      group.hosts.splice(index, 1)

      if (group.hosts.length === 0) {
        const groupIndex = groups.value.findIndex((g: Group) => g.name === group.name)
        if (groupIndex !== -1) {
          groups.value.splice(groupIndex, 1)
          if (groups.value.length > 0) {
            selectedGroup.value = groups.value[0].name
          } else {
            selectedGroup.value = ''
          }
          // 分组被清空删除时，同步移除远程数据源
          if (remoteSources.value.some((s: HostsSource) => s.name === group.name)) {
            try {
              remoteSources.value = await removeHostsSource(group.name)
            } catch (error) {
              console.error('移除远程数据源失败:', error)
            }
          }
        }
      }

      try {
        await debouncedUpdateHosts()
        notify('条目已删除', 'info')
      } catch (error) {
        console.error('删除条目失败', error)
        notify(`删除失败: ${(error as Error).message}`, 'error')
      }
    }
  }

  async function renameGroup(oldName: string, newName: string) {
    const trimmed = newName.trim()
    if (!trimmed) {
      notify('分组名称不能为空', 'error')
      return
    }

    if (oldName === trimmed) {
      return
    }

    const existing = groups.value.find((g: Group) => g.name === trimmed)
    if (existing) {
      notify('已存在同名分组', 'error')
      return
    }

    const group = groups.value.find((g: Group) => g.name === oldName)
    if (!group) {
      notify('未找到对应分组', 'error')
      return
    }

    const previousName = group.name
    group.name = trimmed

    if (selectedGroup.value === oldName) {
      selectedGroup.value = trimmed
    }

    try {
      await debouncedUpdateHosts()
      const result = await readSystemHosts()
      if (Array.isArray(result) && result.length > 0) {
        groups.value = result
        const target = result.find((g: Group) => g.name === trimmed)
        if (target) {
          selectedGroup.value = target.name
        } else {
          selectedGroup.value = result[0].name
        }
        updateGlobalSwitchState(result)
      }
      // 远程数据源跟随分组重命名（保存新名 + 移除旧名）
      const source = remoteSources.value.find((s: HostsSource) => s.name === oldName)
      if (source) {
        try {
          remoteSources.value = await saveHostsSource({ ...source, name: trimmed })
          remoteSources.value = await removeHostsSource(oldName)
        } catch (sourceError) {
          console.error('更新远程数据源名称失败:', sourceError)
        }
      }
      notify('分组重命名成功', 'success')
    } catch (error) {
      console.error('重命名分组失败', error)
      notify(`重命名失败: ${(error as Error).message}`, 'error')
      group.name = previousName
      if (selectedGroup.value === trimmed) {
        selectedGroup.value = previousName
      }
    }
  }

  return {
    // State
    groups,
    selectedGroup,
    hostsResolveSwitch,
    loading,
    currentGroup,
    remoteSources,
    syncing,

    // Actions
    loadSystemHosts,
    handleHostsSwitch,
    initializeDefaultConfig,
    addGroup,
    addHost,
    editHost,
    updateHostStatus,
    confirmDeleteHost,
    renameGroup,
    refreshSource,
    syncRemoteSources,
  }
}
