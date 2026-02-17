import { ref, shallowRef, computed } from 'vue'
import { Group, HostEntry } from '@/types/hosts'
import {
  readSystemHosts,
  updateHostsWithGroups,
  revertHosts,
  initializeDefaultConfig as initDefaultConfig
} from '@/api/hosts'
import {
  enableAllHosts,
  disableAllHosts,
  findHostEntry,
  findHostIndex,
  updateHostEntryStatus
} from '@/utils/hostsUtils'

export type NotifyFunction = (text: string, color: 'success' | 'error' | 'info' | 'warning') => void

export function useHostsData(notify: NotifyFunction) {
  // ===== 状态变量 =====
  const groups = ref<Group[]>([])
  const selectedGroup = ref<string>('')
  const hostsResolveSwitch = ref(false)
  const loading = shallowRef(false)
  
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
      notify('加载系统 Hosts 失败: ' + (error as Error).message, 'error')
      initializeEmptyGroups()
    } finally {
      loading.value = false
    }
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

  async function debouncedUpdateHosts() {
    pendingUpdates.value = true
    
    if (updateDebounceTimeout.value) {
      clearTimeout(updateDebounceTimeout.value)
    }
    
    return new Promise<void>((resolve, reject) => {
      updateDebounceTimeout.value = setTimeout(async () => {
        if (pendingUpdates.value) {
          try {
            loading.value = true
            await updateHostsWithGroups(groups.value)
            pendingUpdates.value = false
            resolve()
          } catch (error) {
            console.error('批量更新hosts失败', error)
            reject(error)
          } finally {
            loading.value = false
          }
        } else {
          resolve()
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
      notify('操作失败: ' + (error as Error).message, 'error')
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
      notify('初始化失败: ' + (error as Error).message, 'error')
      loading.value = false
    }
  }

  // ===== CRUD 操作 =====

  async function addGroup(data: { name: string; isRemote: boolean; url?: string; hosts?: HostEntry[] }) {
    groups.value.push({
      name: data.name,
      hosts: data.hosts || []
    })

    try {
      await debouncedUpdateHosts()
      selectedGroup.value = data.name
      notify('分组添加成功', 'success')
    } catch (error) {
      console.error('添加分组失败', error)
      notify('添加失败: ' + (error as Error).message, 'error')
      groups.value.pop()
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
      disabled: false
    })

    try {
      selectedGroup.value = data.groupName
      await debouncedUpdateHosts()
      notify('条目添加成功', 'success')
    } catch (error) {
      console.error('添加条目失败', error)
      notify('添加失败: ' + (error as Error).message, 'error')
      group.hosts.pop()
    }
  }

  async function editHost(data: { originalHost: any; ip: string; domain: string }) {
    const group = getCurrentGroup()
    if (!group) return

    const hostEntry = findHostEntry(group, data.originalHost)
    if (!hostEntry) {
      notify('未找到要编辑的条目', 'error')
      return
    }

    if (data.domain !== data.originalHost.domain) {
      const domainExists = group.hosts.some(host => host.domain === data.domain && host !== hostEntry)
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
      disabled: isDisabled
    })

    try {
      await debouncedUpdateHosts()
      notify('条目编辑成功', 'success')
    } catch (error) {
      console.error('更新状态失败', error)
      notify('更新状态失败: ' + (error as Error).message, 'error')
    }
  }

  async function updateHostStatus(host: any) {
    const group = getCurrentGroup()
    if (!group) return

    const hostEntry = findHostEntry(group, host)
    if (!hostEntry) return

    updateHostEntryStatus(hostEntry, host.enabled)

    try {
      await debouncedUpdateHosts()
      notify(
        host.enabled ? '条目已启用' : '条目已禁用',
        host.enabled ? 'success' : 'info'
      )
    } catch (error) {
      console.error('更新状态失败', error)
      notify('更新状态失败: ' + (error as Error).message, 'error')
      host.enabled = !host.enabled
    }
  }

  async function confirmDeleteHost(host: any) {
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
        }
      }

      try {
        await debouncedUpdateHosts()
        notify('条目已删除', 'info')
      } catch (error) {
        console.error('删除条目失败', error)
        notify('删除失败: ' + (error as Error).message, 'error')
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
      notify('分组重命名成功', 'success')
    } catch (error) {
      console.error('重命名分组失败', error)
      notify('重命名失败: ' + (error as Error).message, 'error')
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
    
    // Actions
    loadSystemHosts,
    handleHostsSwitch,
    initializeDefaultConfig,
    addGroup,
    addHost,
    editHost,
    updateHostStatus,
    confirmDeleteHost,
    renameGroup
  }
}
