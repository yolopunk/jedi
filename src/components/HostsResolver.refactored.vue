<template>
  <!--
  ====================================
  JEDIHOSTS管理器 - 主界面
  功能：管理hosts文件配置，支持分组、启用/禁用、添加/编辑/删除条目
  ====================================
  -->

  <!-- 1. 应用标题区域 -->
  <header-section v-model="hostsResolveSwitch" @update:model-value="handleHostsSwitch" />

  <!-- 2. 主内容区域 -->
  <v-card class="jedi-card" style="border-radius: 12px; overflow: hidden; box-shadow: 0 2px 8px rgba(0,0,0,0.08);">
    <!-- 分组管理区域 -->
    <group-manager
      v-if="groups.length"
      v-model="selectedGroup"
      :groups="groups"
      @add-group="showAddGroupDialog = true"
    />

    <!-- 数据展示区域 -->
    <template v-if="groups.length && selectedGroup">
      <hosts-table
        v-if="currentGroup"
        :current-group="currentGroup"
        v-model:search="search"
        @update-status="updateHostStatus"
        @edit-host="openEditHostDialog"
        @delete-host="removeHost"
        @add-host="openAddHostDialog"
        @open-domain="handleOpenDomain"
      />
    </template>

    <!-- 空状态显示 -->
    <template v-else>
      <empty-state
        @add-group="showAddGroupDialog = true"
        @use-default="initializeDefaultConfig"
      />
    </template>
  </v-card>

  <!-- 3. 对话框区域 -->
  <!-- 添加分组对话框 -->
  <add-group-dialog
    v-model="showAddGroupDialog"
    @add="addGroup"
    @error="showNotification($event, 'error')"
  />

  <!-- 添加条目对话框 -->
  <add-host-dialog
    v-model="showAddHostDialog"
    :group-name="currentAddGroupName"
    @add="addHost"
    @error="showNotification($event, 'error')"
  />

  <!-- 编辑条目对话框 -->
  <edit-host-dialog
    v-model="showEditHostDialog"
    :host="currentEditHost"
    @edit="editHost"
    @error="showNotification($event, 'error')"
  />

  <!-- 删除确认对话框 -->
  <delete-confirm-dialog
    v-model="showDeleteConfirmDialog"
    :host="hostToDelete"
    @delete="confirmDeleteHost"
  />

  <!-- 4. 通知消息组件 -->
  <notification-snackbar
    v-model="showSnackbar"
    :text="snackbarText"
    :color="snackbarColor"
    :timeout="3000"
  />
</template>

<script setup lang="ts">
/**
 * JEDIHOSTS管理器 - 主组件
 * 功能：管理hosts文件配置，支持分组、启用/禁用、添加/编辑/删除条目
 */

// ===== 导入依赖 =====
import { ref, computed, onMounted } from 'vue'

// 导入类型定义
import { Group, Tag, HostEntry, tagToGroup, groupToTag } from '@/types/hosts'

// 导入子组件
import HeaderSection from '@/components/hosts/common/HeaderSection.vue'
import GroupManager from '@/components/hosts/common/GroupManager.vue'
import HostsTable from '@/components/hosts/tables/HostsTable.vue'
import EmptyState from '@/components/hosts/common/EmptyState.vue'
import AddGroupDialog from '@/components/hosts/dialogs/AddGroupDialog.vue'
import AddHostDialog from '@/components/hosts/dialogs/AddHostDialog.vue'
import EditHostDialog from '@/components/hosts/dialogs/EditHostDialog.vue'
import DeleteConfirmDialog from '@/components/hosts/dialogs/DeleteConfirmDialog.vue'
import NotificationSnackbar from '@/components/hosts/common/NotificationSnackbar.vue'

// 导入工具和服务
import {
  findHostEntry,
  findHostIndex,
  updateHostEntryStatus,
  enableAllHosts,
  disableAllHosts
} from '@/utils/hostsUtils'
import {
  getOsInfo,
  readSystemHosts,
  updateHostsWithTag,
  revertHosts,
  initializeDefaultConfig as initDefaultConfig
} from '@/services/hostsService'

// ===== 状态变量 =====

/**
 * 全局开关状态
 * @description 控制所有hosts条目的启用/禁用状态
 */
const hostsResolveSwitch = ref(false)

/**
 * 分组数据 - 新版本
 * @description 存储所有分组及其hosts条目
 */
const groups = ref<Group[]>([])

/**
 * 当前选中的分组 - 新版本
 */
const selectedGroup = ref<string>('')

/**
 * 分组数据 - 兼容层
 * @description 存储所有分组及其hosts条目
 */
const tags = computed<Tag[]>(() => {
  return groups.value.map(group => groupToTag(group))
})

/**
 * 当前选中的分组 - 兼容层
 */
const selectedTag = computed<string>({
  get: () => selectedGroup.value,
  set: (value) => { selectedGroup.value = value }
})

/**
 * 搜索关键词
 */
const search = ref('')

// ===== 对话框状态 =====

/**
 * 添加分组对话框状态
 */
const showAddGroupDialog = ref(false)

/**
 * 添加条目对话框状态
 */
const showAddHostDialog = ref(false)
const currentAddGroupName = ref('')

/**
 * 编辑条目对话框状态
 */
const showEditHostDialog = ref(false)
const currentEditHost = ref<any>(null)

/**
 * 删除确认对话框状态
 */
const showDeleteConfirmDialog = ref(false)
const hostToDelete = ref<any>(null)

/**
 * 提示消息状态
 */
const showSnackbar = ref(false)
const snackbarText = ref('')
const snackbarColor = ref<'success' | 'error' | 'info' | 'warning'>('success')

// ===== 计算属性 =====

/**
 * 当前选中的分组数据
 */
const currentGroup = computed(() => {
  return groups.value.find(g => g.name === selectedGroup.value)
})

// ===== 生命周期钩子 =====

/**
 * 组件挂载时初始化
 */
onMounted(async () => {
  await getOsInfo()
  await loadSystemHosts()
})

// ===== 方法 =====

/**
 * 处理全局开关状态变化
 * @param switchState 开关状态
 */
async function handleHostsSwitch(switchState: boolean) {
  try {
    if (switchState) {
      // 如果开启，启用所有条目
      enableAllHosts(tags.value)
      // 更新hosts文件
      await updateHostsWithTag(tags.value)
      showNotification('Hosts解析已启用，所有条目已生效', 'success')
    } else {
      // 如果关闭，禁用所有条目
      disableAllHosts(tags.value)
      // 恢复hosts文件
      await revertHosts()
      showNotification('Hosts解析已禁用，所有条目已暂停生效，但配置已保留', 'info')
    }
  } catch (error) {
    console.error('切换Hosts解析状态失败', error)
    showNotification('操作失败: ' + (error as Error).message, 'error')
    // 恢复开关状态
    hostsResolveSwitch.value = !switchState
  }
}

/**
 * 加载系统hosts配置
 * @description 从系统中读取hosts配置并更新界面
 */
async function loadSystemHosts() {
  try {
    // 调用后端API来读取系统hosts文件
    const result = await readSystemHosts();

    // 如果有数据，则使用返回的数据
    if (Array.isArray(result) && result.length > 0) {
      // 更新数据
      updateGroupsData(result);

      // 更新全局开关状态
      updateGlobalSwitchState(result);

      showNotification('成功加载系统 Hosts 配置', 'success');
    } else {
      // 如果没有数据，使用默认空分组
      initializeEmptyGroups();
    }
  } catch (error) {
    console.error('加载系统 Hosts 失败:', error);
    showNotification('加载系统 Hosts 失败: ' + (error as Error).message, 'error');

    // 出错时使用默认空分组
    initializeEmptyGroups();
  }
}

/**
 * 更新分组数据
 * @param result 从后端获取的数据
 */
function updateGroupsData(result: Array<{ tag: string; hosts: Array<Record<string, string>> }>) {
  // 将旧格式转换为新格式
  groups.value = result.map(tag => tagToGroup(tag));
  selectedGroup.value = result[0].tag;
}

/**
 * 更新全局开关状态
 * @param result 从后端获取的数据
 */
function updateGlobalSwitchState(result: Array<{ tag: string; hosts: Array<Record<string, string>> }>) {
  // 检查是否有未禁用的条目，如果有，则设置全局开关为开
  let hasEnabledEntries = false;
  for (const tag of result) {
    for (const host of tag.hosts) {
      if (!host.hasOwnProperty('__disabled')) {
        hasEnabledEntries = true;
        break;
      }
    }
    if (hasEnabledEntries) break;
  }
  hostsResolveSwitch.value = hasEnabledEntries;
}

/**
 * 初始化空分组
 * @description 当没有数据或出错时初始化默认空分组
 */
function initializeEmptyGroups() {
  groups.value = [
    {
      name: '默认',
      hosts: []
    }
  ];
  selectedGroup.value = '默认';
  hostsResolveSwitch.value = false;
}

/**
 * 更新hosts文件
 * @description 将当前界面上的配置写入hosts文件
 */
async function updateHosts() {
  try {
    // 直接将当前界面上的标签数据传递给后端
    await updateHostsWithTag(tags.value)
  } catch (error) {
    console.error('更新hosts失败', error)
    throw error
  }
}

/**
 * 初始化默认配置
 * @description 使用默认配置初始化hosts文件
 */
async function initializeDefaultConfig() {
  try {
    await initDefaultConfig()
    showNotification('默认配置初始化成功', 'success')
    await loadSystemHosts() // 重新加载配置
  } catch (error) {
    console.error('初始化默认配置失败', error)
    showNotification('初始化失败: ' + (error as Error).message, 'error')
  }
}

/**
 * 获取当前分组
 * @returns 当前选中的分组
 */
function getCurrentGroup() {
  const group = groups.value.find(g => g.name === selectedGroup.value);
  if (!group) {
    showNotification('未找到对应分组', 'error');
    return null;
  }
  return group;
}

/**
 * 打开添加主机对话框
 * @param groupName 分组名称
 * @description 打开添加主机对话框，并初始化表单
 */
function openAddHostDialog(groupName: string) {
  // 设置当前分组
  currentAddGroupName.value = groupName
  // 显示对话框
  showAddHostDialog.value = true
}

/**
 * 打开编辑主机对话框
 * @param host 要编辑的主机信息
 * @description 打开编辑主机对话框，并填充表单
 */
function openEditHostDialog(host: any) {
  // 保存当前编辑的主机
  currentEditHost.value = host
  // 显示对话框
  showEditHostDialog.value = true
}

/**
 * 添加分组
 * @param data 分组数据
 */
async function addGroup(data: { name: string; isRemote: boolean; url?: string; hosts?: HostEntry[] }) {
  // 添加新分组
  groups.value.push({
    name: data.name,
    hosts: data.hosts || []
  })

  // 更新hosts文件
  try {
    await updateHosts()
    selectedGroup.value = data.name
    showNotification('分组添加成功', 'success')
  } catch (error) {
    console.error('添加分组失败', error)
    showNotification('添加失败: ' + (error as Error).message, 'error')
    // 回滚操作
    groups.value.pop()
  }
}

/**
 * 添加主机
 * @param data 主机数据
 */
async function addHost(data: { groupName: string; ip: string; domain: string }) {
  const group = groups.value.find(g => g.name === data.groupName)
  if (!group) {
    showNotification('未找到对应分组', 'error')
    return
  }

  // 添加新条目
  group.hosts.push({ [data.domain]: data.ip })

  // 更新hosts文件
  try {
    await updateHosts()
    // 更新UI状态
    selectedGroup.value = data.groupName
    showNotification('条目添加成功', 'success')
  } catch (error) {
    console.error('添加条目失败', error)
    showNotification('添加失败: ' + (error as Error).message, 'error')
    // 回滚操作
    group.hosts.pop()
  }
}

/**
 * 编辑主机
 * @param data 编辑数据
 */
async function editHost(data: { originalHost: any; ip: string; domain: string }) {
  // 获取当前分组
  const group = getCurrentGroup()
  if (!group) return

  // 找到对应的主机条目
  const hostEntry = findHostEntry(groupToTag(group), data.originalHost)
  if (!hostEntry) {
    showNotification('未找到要编辑的条目', 'error')
    return
  }

  // 保存禁用状态
  const isDisabled = hostEntry.hasOwnProperty('__disabled')

  // 删除旧条目
  const index = group.hosts.indexOf(hostEntry)
  if (index !== -1) {
    group.hosts.splice(index, 1)
  }

  // 添加新条目
  const newHostEntry = { [data.domain]: data.ip } as HostEntry
  if (isDisabled) {
    newHostEntry['__disabled'] = 'true'
  }
  group.hosts.push(newHostEntry)

  // 更新hosts文件
  try {
    await updateHosts()
    showNotification('条目编辑成功', 'success')
  } catch (error) {
    console.error('更新状态失败', error)
    showNotification('更新状态失败: ' + (error as Error).message, 'error')
  }
}

/**
 * 更新主机状态
 * @param host 要更新的主机信息
 * @description 更新hosts条目的启用/禁用状态
 */
async function updateHostStatus(host: any) {
  // 获取当前分组
  const group = getCurrentGroup()
  if (!group) return

  // 找到对应的主机条目
  const hostEntry = findHostEntry(groupToTag(group), host)
  if (!hostEntry) return

  // 更新启用/禁用状态
  updateHostEntryStatus(hostEntry, host.enabled)

  // 更新hosts文件
  try {
    await updateHosts()
    showNotification(
      host.enabled ? '条目已启用' : '条目已禁用',
      host.enabled ? 'success' : 'info'
    )
  } catch (error) {
    console.error('更新状态失败', error)
    showNotification('更新状态失败: ' + (error as Error).message, 'error')
    // 恢复状态
    host.enabled = !host.enabled
  }
}

/**
 * 打开删除确认对话框
 * @param host 要删除的主机信息
 */
function removeHost(host: any) {
  hostToDelete.value = host
  showDeleteConfirmDialog.value = true
}

/**
 * 确认删除主机
 * @param host 要删除的主机信息
 * @description 删除指定的hosts条目并更新hosts文件
 */
async function confirmDeleteHost(host: any) {
  // 获取当前分组
  const group = getCurrentGroup()
  if (!group) return

  // 查找要删除的条目索引
  const index = findHostIndex(groupToTag(group), host)

  // 如果找到了条目，则删除并更新hosts文件
  if (index !== -1) {
    // 删除条目
    group.hosts.splice(index, 1)

    // 如果分组中没有条目了，则删除该分组
    if (group.hosts.length === 0) {
      const groupIndex = groups.value.findIndex(g => g.name === group.name)
      if (groupIndex !== -1) {
        groups.value.splice(groupIndex, 1)

        // 如果还有其他分组，则选中第一个分组
        if (groups.value.length > 0) {
          selectedGroup.value = groups.value[0].name
        } else {
          selectedGroup.value = ''
        }
      }
    }

    // 更新hosts文件
    try {
      await updateHosts()
      showNotification('条目已删除', 'info')
    } catch (error) {
      console.error('删除条目失败', error)
      showNotification('删除失败: ' + (error as Error).message, 'error')
    }
  }
}

/**
 * 处理打开域名链接
 * @param domain 域名
 * @param message 通知消息
 */
function handleOpenDomain(_domain: string, message: string) {
  showNotification(message, 'info')
}

/**
 * 显示通知
 * @param text 通知文本
 * @param color 通知颜色
 * @description 显示一个通知消息，用于反馈操作结果
 */
function showNotification(text: string, color: 'success' | 'error' | 'info' | 'warning') {
  snackbarText.value = text
  snackbarColor.value = color
  showSnackbar.value = true
}
</script>

<style scoped>
/* 引入全局样式 */
@import '@/assets/style.css';
</style>
