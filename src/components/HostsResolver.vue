<template>
  <!-- 应用标题区域 - 绝地武士主题 -->
  <div class="app-header mb-6 pa-4 rounded-lg" style="background: linear-gradient(135deg, #4a90e2 0%, #2c5282 100%);">
    <div class="d-flex justify-space-between align-center">
      <div class="d-flex align-center">
        <div class="jedi-logo mr-4">
          <v-icon :icon="mdiSwordCross" size="x-large" color="white" class="lightsaber-icon"></v-icon>
        </div>
        <div>
          <h1 class="text-h4 font-weight-bold mb-1 jedi-title">
            <span class="jedi-text" style="color: white; text-shadow: 0 0 10px rgba(255,255,255,0.8);">JEDI</span>
            <span class="hosts-text" style="color: white; text-shadow: 0 0 10px rgba(255,255,255,0.8);">HOSTS</span>
            <span class="text-h5 font-weight-medium" style="color: white; text-shadow: 0 0 10px rgba(255,255,255,0.8);">管理器</span>
          </h1>
          <p class="text-subtitle-1 mb-0" style="color: white; font-weight: 500;">
            管理您的 hosts 文件配置，提高开发效率
          </p>
        </div>
      </div>
      <div class="d-flex align-center">
        <v-chip
          :color="hostsResolveSwitch ? 'var(--jedi-green)' : 'var(--jedi-grey-dark)'"
          :text-color="'white'"
          size="large"
          class="mr-3 px-4 force-chip"
          variant="elevated"
          elevation="3"
        >
          <v-icon start :icon="hostsResolveSwitch ? mdiCheckCircle : mdiPauseCircle"></v-icon>
          <span style="font-weight: 500;">{{ hostsResolveSwitch ? '已启用' : '已禁用' }}</span>
        </v-chip>
        <div class="d-flex align-center">
          <span class="mr-2" style="color: white; font-weight: 500; text-shadow: 0 0 5px rgba(0,0,0,0.3);">全局开关</span>
          <v-switch
            v-model="hostsResolveSwitch"
            hide-details
            :color="hostsResolveSwitch ? 'var(--jedi-green)' : 'var(--jedi-grey-medium)'"
            :track-color="hostsResolveSwitch ? 'var(--jedi-green-light)' : '#EEEEEE'"
            @update:model-value="(val) => val !== null && handleHostsSwitch(val)"
            density="comfortable"
            class="global-switch lightsaber-switch"
          ></v-switch>
        </div>
      </div>
    </div>
  </div>

  <!-- 分组选择区域 - 绝地武士主题 -->
  <v-card class="mb-6 rounded-xl elevation-3 overflow-hidden border">
    <v-card-text class="pa-0">
      <div class="d-flex align-center px-4 py-2" style="background-color: #4a90e2;">
        <v-icon :icon="mdiDomain" class="mr-2" color="white"></v-icon>
        <span class="text-subtitle-1 font-weight-medium text-white">分组管理</span>
        <v-spacer></v-spacer>
        <v-btn
          color="white"
          variant="elevated"
          class="ml-2 lightsaber-btn"
          :prepend-icon="mdiPlus"
          @click="showAddGroupDialog = true"
          size="small"
          rounded="pill"
          style="color: #4a90e2; font-weight: 500;"
        >
          添加分组
        </v-btn>
      </div>
      <v-divider></v-divider>
      <v-tabs
        v-model="selectedTag"
        color="var(--lightsaber-blue)"
        align-tabs="start"
        show-arrows
        slider-color="var(--lightsaber-blue)"
        class="px-2 jedi-tabs"
        height="56"
        bg-color="#f5f7fa"
      >
        <v-tab
          v-for="group in tags"
          :key="group.tag"
          :value="group.tag"
          class="text-capitalize font-weight-medium jedi-tab pa-2"
          rounded="pill"
        >
          <v-chip
            :color="selectedTag === group.tag ? 'var(--jedi-blue)' : '#f0f0f0'"
            :text-color="selectedTag === group.tag ? 'var(--jedi-text-light)' : 'var(--jedi-text-medium)'"
            size="small"
            class="px-2 py-1 group-tab-chip"
            :elevation="selectedTag === group.tag ? 2 : 1"
            :style="selectedTag === group.tag ? 'border: 1px solid rgba(255,255,255,0.2)' : 'border: 1px solid #d0d0d0'"
          >
            <v-icon :icon="mdiDomain" size="x-small" class="mr-1"></v-icon>
            {{ group.tag }}
          </v-chip>
        </v-tab>
      </v-tabs>
    </v-card-text>
  </v-card>

  <!-- 数据展示区域 - 绝地武士主题 -->
  <v-card class="rounded-xl elevation-3 overflow-hidden border">
    <v-card-text v-if="tags.length && selectedTag" class="pa-0">
      <div v-for="group in tags.filter(t => t.tag === selectedTag)" :key="group.tag" class="pa-4">
        <div class="d-flex justify-space-between align-center mb-5">
          <div class="d-flex align-center">
            <v-btn
              color="#4CAF50"
              variant="elevated"
              :prepend-icon="mdiPlus"
              @click="openAddHostDialog(group.tag)"
              rounded="pill"
              elevation="3"
              class="px-4 add-host-btn"
              style="color: white; font-weight: 500;"
            >
              新增条目
            </v-btn>
            <div class="ml-4 pa-2 rounded-lg d-flex align-center" style="background-color: #edf2ff;">
              <span class="text-caption mr-2" style="color: #333;">当前分组:</span>
              <v-chip
                color="#4a90e2"
                size="small"
                variant="elevated"
                class="font-weight-medium"
                text-color="white"
              >
                {{ group.tag }}
              </v-chip>
            </div>
          </div>
          <v-text-field
            v-model="search"
            label="搜索域名或IP"
            :prepend-inner-icon="mdiMagnify"
            variant="outlined"
            hide-details
            density="compact"
            class="max-width-300"
            bg-color="white"
            color="#333333"
            rounded="pill"
          ></v-text-field>
        </div>

        <v-card class="mb-4 border rounded-lg overflow-hidden" elevation="2">
          <v-data-table
            :headers="headers"
            :items="getHostsAsItems(group.hosts)"
            :search="search"
            density="comfortable"
            hover
            class="hosts-table jedi-table"
            :items-per-page="10"
            bg-color="white"
            :footer-props="{
              'items-per-page-options': [5, 10, 15, 20, -1],
              'items-per-page-text': '每页显示',
              'page-text': '{0}-{1} 共 {2}'
            }"
          >
            <template v-slot:item.enabled="{ item }">
              <div class="d-flex align-center">
                <v-switch
                  v-model="item.enabled"
                  hide-details
                  :color="item.enabled ? 'var(--jedi-green)' : 'var(--jedi-grey-medium)'"
                  :track-color="item.enabled ? 'var(--jedi-green-light)' : '#EEEEEE'"
                  density="comfortable"
                  @update:model-value="updateHostStatus(item)"
                  class="ma-0 pa-0 lightsaber-switch"
                  :ripple="false"
                  style="background-color: transparent;"
                ></v-switch>
                <v-chip
                  size="x-small"
                  :color="item.enabled ? 'var(--jedi-green-light)' : '#EEEEEE'"
                  :text-color="item.enabled ? 'var(--jedi-green)' : 'var(--jedi-grey-medium)'"
                  class="ml-1 status-chip"
                  :style="item.enabled ? 'border: 1px solid var(--jedi-green)' : 'border: 1px solid #d0d0d0'"
                >
                  <span style="font-weight: 500;">{{ item.enabled ? '已启用' : '已禁用' }}</span>
                </v-chip>
              </div>
            </template>
            <template v-slot:item.domain="{ item }">
              <div class="d-flex align-center">
                <div class="font-weight-medium">{{ item.domain }}</div>
                <v-tooltip text="打开域名" location="top">
                  <template v-slot:activator="{ props }">
                    <v-btn
                      icon
                      variant="text"
                      size="x-small"
                      class="ml-2"
                      v-bind="props"
                      @click="openDomainLink(item.domain)"
                    >
                      <v-icon :icon="mdiWeb" size="small" color="#4a90e2"></v-icon>
                    </v-btn>
                  </template>
                </v-tooltip>
              </div>
            </template>
            <template v-slot:item.ip="{ item }">
              <v-chip
                size="small"
                color="#4a90e2"
                variant="elevated"
                class="font-monospace"
              >
                {{ item.ip }}
              </v-chip>
            </template>
            <template v-slot:item.actions="{ item }">
              <div class="d-flex">
                <v-tooltip text="编辑条目" location="top">
                  <template v-slot:activator="{ props }">
                    <v-btn
                      icon
                      variant="flat"
                      color="#4a90e2"
                      class="mr-1"
                      @click="openEditHostDialog(item)"
                      v-bind="props"
                      size="small"
                      style="background-color: #e3f2fd;"
                    >
                      <v-icon :icon="mdiPencil" color="#4a90e2"></v-icon>
                    </v-btn>
                  </template>
                </v-tooltip>
                <v-tooltip text="删除条目" location="top">
                  <template v-slot:activator="{ props }">
                    <v-btn
                      icon
                      variant="flat"
                      color="#F44336"
                      @click="removeHost(item)"
                      v-bind="props"
                      size="small"
                      style="background-color: #ffebee;"
                    >
                      <v-icon :icon="mdiDelete" color="#F44336"></v-icon>
                    </v-btn>
                  </template>
                </v-tooltip>
              </div>
            </template>
          </v-data-table>
        </v-card>
      </div>
    </v-card-text>
    <v-card-text v-else class="text-center pa-12" style="background-color: #f5f7fa;">
      <div class="empty-state-container py-8">
        <div class="death-star-icon mb-6">
          <v-icon :icon="mdiDomain" size="80" color="#4a90e2" class="death-star"></v-icon>
        </div>
        <h2 class="text-h5 font-weight-bold mb-2">暂无解析配置</h2>
        <p class="text-body-1 text-grey-darken-1 mb-8 max-width-500 mx-auto">您可以手动添加分组或使用默认配置来开始管理您的 hosts 文件</p>
        <div class="d-flex justify-center">
          <v-btn
            color="var(--lightsaber-blue)"
            variant="elevated"
            class="mr-4 px-6 lightsaber-btn blue"
            rounded="pill"
            size="large"
            :prepend-icon="mdiPlus"
            elevation="3"
            @click="showAddGroupDialog = true"
          >
            添加分组
          </v-btn>
          <v-btn
            color="var(--lightsaber-green)"
            variant="elevated"
            class="px-6 lightsaber-btn green"
            rounded="pill"
            size="large"
            :prepend-icon="mdiDomain"
            @click="initializeDefaultConfig"
          >
            使用默认配置
          </v-btn>
        </div>
      </div>
    </v-card-text>
  </v-card>

  <!-- 添加分组对话框 - 绝地武士主题 -->
  <v-dialog v-model="showAddGroupDialog" max-width="550" persistent>
    <v-card class="rounded-lg overflow-hidden">
      <v-toolbar color="#4a90e2" class="px-4">
        <v-icon :icon="mdiDomainPlus" class="mr-2" color="var(--lightsaber-blue)"></v-icon>
        <v-toolbar-title class="font-weight-medium">添加分组</v-toolbar-title>
        <v-spacer></v-spacer>
        <v-btn icon @click="showAddGroupDialog = false">
          <v-icon :icon="mdiClose" color="white"></v-icon>
        </v-btn>
      </v-toolbar>
      <v-card-text class="pa-6">
        <v-text-field
          v-model="newGroupTag"
          label="分组名称 (tag)"
          variant="outlined"
          placeholder="例如: development, production"
          required
          class="mb-4"
          bg-color="white"
        ></v-text-field>

        <v-card class="mb-4 pa-3 rounded-lg" style="background-color: #edf2ff;">
          <div class="d-flex align-center">
            <v-switch
              v-model="newGroupIsRemote"
              label="使用远程配置"
              color="var(--lightsaber-blue)"
              hide-details
              density="comfortable"
              inset
              class="lightsaber-switch"
            ></v-switch>
            <v-tooltip text="从远程 URL 加载 hosts 配置" location="top">
              <template v-slot:activator="{ props }">
                <v-icon v-bind="props" class="ml-2" size="small" color="var(--lightsaber-blue)" :icon="mdiInformationOutline"></v-icon>
              </template>
            </v-tooltip>
          </div>
        </v-card>

        <v-expand-transition>
          <v-text-field
            v-if="newGroupIsRemote"
            v-model="newGroupUrl"
            label="远程配置 URL"
            variant="outlined"
            placeholder="https://example.com/hosts.json"
            required
            :prepend-inner-icon="mdiLinkVariant"
            bg-color="white"
          ></v-text-field>
        </v-expand-transition>

        <v-expand-transition>
          <div v-if="!newGroupIsRemote">
            <v-textarea
              v-model="newGroupHosts"
              label="Hosts 列表"
              variant="outlined"
              placeholder="格式: IP 域名，每行一条\n例如:\n127.0.0.1 localhost\n192.168.1.1 router.local"
              rows="6"
              auto-grow
              required
              bg-color="white"
              class="font-monospace"
            ></v-textarea>
            <v-alert
              type="info"
              variant="tonal"
              density="compact"
              class="mt-2 text-body-2"
            >
              <v-icon start :icon="mdiInformationOutline" color="var(--lightsaber-blue)"></v-icon>
              每行一条记录，格式为 "IP地址 域名"
            </v-alert>
          </div>
        </v-expand-transition>
      </v-card-text>
      <v-divider></v-divider>
      <v-card-actions class="pa-4" style="background-color: #f5f7fa;">
        <v-spacer></v-spacer>
        <v-btn
          variant="text"
          @click="showAddGroupDialog = false"
          class="mr-2"
          color="grey-darken-1"
        >
          取消
        </v-btn>
        <v-btn
          color="var(--lightsaber-blue)"
          variant="elevated"
          @click="confirmAddGroup"
          class="lightsaber-btn blue"
        >
          确认
        </v-btn>
      </v-card-actions>
    </v-card>
  </v-dialog>

  <!-- 添加条目对话框 - 绝地武士主题 -->
  <v-dialog v-model="showAddHostDialog" max-width="500" persistent>
    <v-card class="rounded-lg overflow-hidden">
      <v-toolbar color="#42b983" class="px-4">
        <v-icon :icon="mdiDns" class="mr-2" color="var(--lightsaber-green)"></v-icon>
        <v-toolbar-title class="font-weight-medium">新增条目</v-toolbar-title>
        <v-spacer></v-spacer>
        <v-btn icon @click="showAddHostDialog = false">
          <v-icon :icon="mdiClose" color="white"></v-icon>
        </v-btn>
      </v-toolbar>
      <v-card-text class="pa-6">
        <v-text-field
          v-model="newHostIp"
          label="IP地址"
          variant="outlined"
          placeholder="例如: 127.0.0.1"
          required
          class="mb-4"
          bg-color="white"
          :prepend-inner-icon="mdiIpNetwork"
        ></v-text-field>
        <v-text-field
          v-model="newHostDomain"
          label="域名"
          variant="outlined"
          placeholder="例如: example.local"
          required
          bg-color="white"
          :prepend-inner-icon="mdiWeb"
        ></v-text-field>
        <v-alert
          type="info"
          variant="tonal"
          density="compact"
          class="mt-4 text-body-2"
        >
          <v-icon start :icon="mdiInformationOutline" color="var(--lightsaber-green)"></v-icon>
          此条目将属于当前选中的分组
        </v-alert>
      </v-card-text>
      <v-divider></v-divider>
      <v-card-actions class="pa-4" style="background-color: #f5f7fa;">
        <v-spacer></v-spacer>
        <v-btn
          variant="text"
          @click="showAddHostDialog = false"
          class="mr-2"
          color="grey-darken-1"
        >
          取消
        </v-btn>
        <v-btn
          color="var(--lightsaber-green)"
          variant="elevated"
          @click="confirmAddHost"
          class="lightsaber-btn green"
        >
          确认
        </v-btn>
      </v-card-actions>
    </v-card>
  </v-dialog>

  <!-- 编辑条目对话框 - 绝地武士主题 -->
  <v-dialog v-model="showEditHostDialog" max-width="500" persistent>
    <v-card class="rounded-lg overflow-hidden">
      <v-toolbar color="#4a90e2" class="px-4">
        <v-icon :icon="mdiPencil" class="mr-2" color="var(--lightsaber-blue)"></v-icon>
        <v-toolbar-title class="font-weight-medium">编辑条目</v-toolbar-title>
        <v-spacer></v-spacer>
        <v-btn icon @click="showEditHostDialog = false">
          <v-icon :icon="mdiClose" color="white"></v-icon>
        </v-btn>
      </v-toolbar>
      <v-card-text class="pa-6">
        <v-text-field
          v-model="editHostIp"
          label="IP地址"
          variant="outlined"
          placeholder="例如: 127.0.0.1"
          required
          class="mb-4"
          bg-color="white"
          :prepend-inner-icon="mdiIpNetwork"
        ></v-text-field>
        <v-text-field
          v-model="editHostDomain"
          label="域名"
          variant="outlined"
          placeholder="例如: example.local"
          required
          bg-color="white"
          :prepend-inner-icon="mdiWeb"
        ></v-text-field>
        <v-alert
          type="info"
          variant="tonal"
          density="compact"
          class="mt-4 text-body-2"
        >
          <v-icon start :icon="mdiInformationOutline" color="var(--lightsaber-blue)"></v-icon>
          编辑条目将保留其启用/禁用状态
        </v-alert>
      </v-card-text>
      <v-divider></v-divider>
      <v-card-actions class="pa-4" style="background-color: #f5f7fa;">
        <v-spacer></v-spacer>
        <v-btn
          variant="text"
          @click="showEditHostDialog = false"
          class="mr-2"
          color="grey-darken-1"
        >
          取消
        </v-btn>
        <v-btn
          color="var(--lightsaber-blue)"
          variant="elevated"
          @click="confirmEditHost"
          class="lightsaber-btn blue"
        >
          保存
        </v-btn>
      </v-card-actions>
    </v-card>
  </v-dialog>

  <!-- 提示消息 - 绝地武士主题 -->
  <v-snackbar
    v-model="showSnackbar"
    :color="snackbarColor"
    :timeout="3000"
    location="top"
    rounded="pill"
    class="mt-6"
  >
    <div class="d-flex align-center">
      <v-icon
        :icon="snackbarColor === 'success' ? mdiCheckCircle : snackbarColor === 'error' ? mdiAlertCircle : mdiInformation"
        class="mr-2"
      ></v-icon>
      {{ snackbarText }}
    </div>
    <template v-slot:actions>
      <v-btn
        variant="text"
        @click="showSnackbar = false"
        icon
        size="small"
      >
        <v-icon :icon="mdiClose"></v-icon>
      </v-btn>
    </template>
  </v-snackbar>
</template>

<script setup lang="ts">
import { ref, onMounted, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import {
  mdiPlus,
  mdiDelete,
  mdiMagnify,
  mdiDomain,
  mdiDomainPlus,
  mdiDns,
  mdiPencil,
  mdiCheckCircle,
  mdiPauseCircle,
  mdiClose,
  mdiInformationOutline,
  mdiLinkVariant,
  mdiWeb,
  mdiIpNetwork,
  mdiAlertCircle,
  mdiInformation,
  mdiSwordCross
} from '@mdi/js'

// 数据表格配置
const headers = [
  { title: 'IP地址', key: 'ip', sortable: true },
  { title: '域名', key: 'domain', sortable: true },
  { title: '状态', key: 'enabled', sortable: false },
  { title: '操作', key: 'actions', sortable: false }
]

// 状态变量
const hostsResolveSwitch = ref(false)
// 注释掉未使用的变量
// const remoteUrl = ref('')
const tags = ref<Array<{ tag: string; hosts: Array<Record<string, string>> }>>([])
const selectedTag = ref<string>('')
const search = ref('')

// 对话框状态
const showAddGroupDialog = ref(false)
const showAddHostDialog = ref(false)
const showEditHostDialog = ref(false)
const newGroupTag = ref('')
const newGroupIsRemote = ref(false)
const newGroupUrl = ref('')
const newGroupHosts = ref('')
const newHostIp = ref('')
const newHostDomain = ref('')
const editHostIp = ref('')
const editHostDomain = ref('')
const currentAddGroupTag = ref('')
const currentEditHost = ref<any>(null)

// 提示消息
const showSnackbar = ref(false)
const snackbarText = ref('')
const snackbarColor = ref('success')

// 生命周期钩子
onMounted(async () => {
  await getOsInfo()
  await loadSystemHosts()
})

// 监听全局开关状态变化
watch(hostsResolveSwitch, (newValue) => {
  handleHostsSwitch(newValue)
})

// 将hosts数组转换为数据表格项目
function getHostsAsItems(hosts: Array<Record<string, string>>) {
  return hosts.map((hostMap, index) => {
    // 检查是否禁用
    const isDisabled = hostMap.hasOwnProperty('__disabled')

    // 提取域名和IP（跳过特殊键）
    let domain = ''
    let ip = ''

    for (const key in hostMap) {
      if (key !== '__disabled') {
        domain = key
        ip = hostMap[key]
        break
      }
    }

    return {
      id: index,
      domain,
      ip,
      enabled: !isDisabled,
      originalMap: hostMap
    }
  })
}

// 处理全局开关状态变化
async function handleHostsSwitch(switchState: boolean) {
  try {
    console.log('切换全局开关状态：', switchState)
    if (switchState) {
      // 如果开启，先将所有条目启用（移除__disabled标记）
      for (const tag of tags.value) {
        for (const host of tag.hosts) {
          if (host.hasOwnProperty('__disabled')) {
            delete host['__disabled']
          }
        }
      }
      // 然后更新hosts文件
      await updateHosts()
      showNotification('Hosts解析已启用，所有条目已生效', 'success')
    } else {
      // 如果关闭，先将所有条目禁用（添加__disabled标记）
      for (const tag of tags.value) {
        for (const host of tag.hosts) {
          if (!host.hasOwnProperty('__disabled')) {
            host['__disabled'] = 'true'
          }
        }
      }
      // 然后调用revertHosts来注释所有条目
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

// 更新hosts文件
async function updateHosts() {
  try {
    // 使用当前界面上的配置
    console.log('开始更新hosts文件，当前标签数量：', tags.value.length)

    // 直接将当前界面上的标签数据传递给后端
    await invoke('update_hosts_with_tag', {
      source: 'current',
      url: null,
      tags: tags.value
    })

    console.log('hosts文件更新成功')
  } catch (error) {
    console.error('更新hosts失败', error)
    throw error
  }
}

// 初始化默认配置
async function initializeDefaultConfig() {
  try {
    await invoke('update_hosts_with_tag', {
      source: 'default',
      url: null,
      tags: null
    })
    showNotification('默认配置初始化成功', 'success')
    await loadSystemHosts() // 重新加载配置
  } catch (error) {
    console.error('初始化默认配置失败', error)
    showNotification('初始化失败: ' + (error as Error).message, 'error')
  }
}

// 恢复hosts文件
async function revertHosts() {
  try {
    await invoke('revert_hosts')
  } catch (error) {
    console.error('恢复hosts失败', error)
    throw error
  }
}

// 加载系统hosts配置
async function loadSystemHosts() {
  try {
    // 调用后端API来读取系统hosts文件
    const result = await invoke('read_system_hosts');

    // 如果有数据，则使用返回的数据
    if (Array.isArray(result) && result.length > 0) {
      tags.value = result;
      selectedTag.value = result[0].tag;

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
      showNotification('成功加载系统 Hosts 配置', 'success');
    } else {
      // 如果没有数据，使用默认空标签
      tags.value = [
        {
          tag: '默认',
          hosts: []
        }
      ];
      selectedTag.value = '默认';
      hostsResolveSwitch.value = false;
    }
  } catch (error) {
    console.error('加载系统 Hosts 失败:', error);
    showNotification('加载系统 Hosts 失败: ' + (error as Error).message, 'error');

    // 出错时使用默认空标签
    tags.value = [
      {
        tag: '默认',
        hosts: []
      }
    ];
    selectedTag.value = '默认';
    hostsResolveSwitch.value = false;
  }
}

// 加载远程配置
// 注释掉未使用的函数以避免 TypeScript 警告
/*
async function loadConfig() {
  try {
    const result: Array<{ tag: string; hosts: Array<Record<string, string>> }> =
      await invoke('fetch_remote_config', { url: remoteUrl.value })

    if (result.length > 0) {
      tags.value = result
      selectedTag.value = result[0].tag
      showNotification('远程配置加载成功', 'success')
    } else {
      showNotification('远程配置为空', 'warning')
    }
  } catch (error) {
    console.error('加载配置失败', error)
    showNotification('加载配置失败: ' + (error as Error).message, 'error')
  }
}
*/

// 打开添加主机对话框
function openAddHostDialog(tag: string) {
  currentAddGroupTag.value = tag
  newHostIp.value = ''
  newHostDomain.value = ''
  showAddHostDialog.value = true
}

// 打开编辑主机对话框
function openEditHostDialog(host: any) {
  currentEditHost.value = host
  editHostIp.value = host.ip
  editHostDomain.value = host.domain
  showEditHostDialog.value = true
}

// 确认添加主机
function confirmAddHost() {
  if (!newHostIp.value.trim() || !newHostDomain.value.trim()) {
    showNotification('IP和域名不能为空', 'error')
    return
  }

  const group = tags.value.find(t => t.tag === currentAddGroupTag.value)
  if (!group) {
    showNotification('未找到对应分组', 'error')
    return
  }

  group.hosts.push({ [newHostDomain.value.trim()]: newHostIp.value.trim() })

  selectedTag.value = currentAddGroupTag.value
  showAddHostDialog.value = false
  newHostIp.value = ''
  newHostDomain.value = ''
  showNotification('条目添加成功', 'success')
}

// 确认编辑主机
function confirmEditHost() {
  if (!editHostIp.value.trim() || !editHostDomain.value.trim()) {
    showNotification('IP和域名不能为空', 'error')
    return
  }

  if (!currentEditHost.value) {
    showNotification('编辑数据丢失', 'error')
    return
  }

  const group = tags.value.find(t => t.tag === selectedTag.value)
  if (!group) {
    showNotification('未找到对应分组', 'error')
    return
  }

  // 找到对应的主机条目
  const hostEntry = group.hosts.find(h => {
    for (const key in h) {
      if (key !== '__disabled' && key === currentEditHost.value.domain && h[key] === currentEditHost.value.ip) {
        return true
      }
    }
    return false
  })

  if (hostEntry) {
    // 保存禁用状态
    const isDisabled = hostEntry.hasOwnProperty('__disabled')

    // 删除旧条目
    const index = group.hosts.indexOf(hostEntry)
    if (index !== -1) {
      group.hosts.splice(index, 1)
    }

    // 添加新条目
    const newHostEntry = { [editHostDomain.value.trim()]: editHostIp.value.trim() }
    if (isDisabled) {
      newHostEntry['__disabled'] = 'true'
    }
    group.hosts.push(newHostEntry)

    // 更新hosts文件
    updateHosts().then(() => {
      showNotification('条目编辑成功', 'success')
      showEditHostDialog.value = false
      editHostIp.value = ''
      editHostDomain.value = ''
      currentEditHost.value = null
    }).catch(error => {
      console.error('更新状态失败', error)
      showNotification('更新状态失败: ' + (error as Error).message, 'error')
    })
  } else {
    showNotification('未找到要编辑的条目', 'error')
  }
}

// 确认添加分组
async function confirmAddGroup() {
  if (!newGroupTag.value) {
    showNotification('分组名称不能为空', 'error')
    return
  }

  if (newGroupIsRemote.value && !newGroupUrl.value) {
    showNotification('远程配置URL不能为空', 'error')
    return
  }

  let hostsArray: Array<Record<string, string>> = []

  if (newGroupIsRemote.value) {
    try {
      const result: Array<{ tag: string; hosts: Array<Record<string, string>> }> =
        await invoke('fetch_remote_config', { url: newGroupUrl.value })

      const found = result.find(r => r.tag === newGroupTag.value)
      if (found) {
        hostsArray = found.hosts
      } else {
        showNotification('远程配置中未找到该分组', 'error')
        return
      }
    } catch (error) {
      console.error('远程加载失败', error)
      showNotification('远程加载失败: ' + (error as Error).message, 'error')
      return
    }
  } else {
    hostsArray = newGroupHosts.value
      .split('\n')
      .map(line => line.trim())
      .filter(line => line.length > 0)
      .map(line => {
        const [ip, domain] = line.split(/\s+/)
        return { [domain]: ip }
      })
  }

  tags.value.push({
    tag: newGroupTag.value,
    hosts: hostsArray
  })

  selectedTag.value = newGroupTag.value

  showAddGroupDialog.value = false
  newGroupTag.value = ''
  newGroupIsRemote.value = false
  newGroupUrl.value = ''
  newGroupHosts.value = ''
  showNotification('分组添加成功', 'success')
}

// 更新主机状态
function updateHostStatus(host: any) {
  console.log('更新主机状态:', host)

  // 获取当前分组
  const group = tags.value.find(t => t.tag === selectedTag.value)
  if (!group) return

  // 找到对应的主机条目
  const hostEntry = group.hosts.find(h => {
    for (const key in h) {
      if (key !== '__disabled' && key === host.domain && h[key] === host.ip) {
        return true
      }
    }
    return false
  })

  if (hostEntry) {
    // 更新启用/禁用状态
    if (host.enabled) {
      // 如果启用，删除禁用标记
      delete hostEntry['__disabled']
    } else {
      // 如果禁用，添加禁用标记
      hostEntry['__disabled'] = 'true'
    }

    // 更新hosts文件
    updateHosts().then(() => {
      showNotification(
        host.enabled ? '条目已启用' : '条目已禁用',
        host.enabled ? 'success' : 'info'
      )
    }).catch(error => {
      console.error('更新状态失败', error)
      showNotification('更新状态失败: ' + (error as Error).message, 'error')
      // 恢复状态
      host.enabled = !host.enabled
    })
  }
}

// 移除主机
function removeHost(host: any) {
  const group = tags.value.find(t => t.tag === selectedTag.value)
  if (!group) return

  const index = group.hosts.findIndex(h => {
    const domain = Object.keys(h)[0]
    return domain === host.domain && h[domain] === host.ip
  })

  if (index !== -1) {
    group.hosts.splice(index, 1)
    showNotification('条目已删除', 'info')
  }
}

// 打开域名链接
function openDomainLink(domain: string) {
  // 根据域名构建 URL
  let url = domain
  if (!url.startsWith('http://') && !url.startsWith('https://')) {
    url = 'http://' + url
  }

  // 在新标签页中打开链接
  window.open(url, '_blank')
  showNotification('正在打开: ' + domain, 'info')
}

// 显示通知
function showNotification(text: string, color: 'success' | 'error' | 'info' | 'warning') {
  snackbarText.value = text
  snackbarColor.value = color
  showSnackbar.value = true
}

// 获取系统信息
async function getOsInfo() {
  await invoke('get_os_info')
}
</script>

<style scoped>
/* 绝地武士主题样式 - 优化配色 */
:root {
  --jedi-blue: #2c5aa0;         /* 更柔和的蓝色 */
  --jedi-light-blue: #5d8dc9;   /* 更柔和的浅蓝色 */
  --jedi-dark-blue: #1a3a6a;    /* 更深沉的蓝色 */
  --lightsaber-blue: #4a90e2;   /* 更柔和的光剑蓝 */
  --lightsaber-green: #42b983;  /* 更柔和的光剑绿 */
  --lightsaber-red: #e74c3c;    /* 更柔和的光剑红 */
  --empire-gray: #34495e;       /* 更柔和的灰色 */
  --star-yellow: #f1c40f;       /* 更柔和的黄色 */
}

.app-header {
  background: linear-gradient(135deg, var(--jedi-blue) 0%, var(--jedi-dark-blue) 100%);
  color: white;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
  border-radius: 8px;
  position: relative;
  overflow: hidden;
}

.tabs-header {
  background-color: var(--empire-gray);
  color: white;
}

.max-width-200 {
  max-width: 200px;
}

.max-width-300 {
  max-width: 300px;
}

.max-width-500 {
  max-width: 500px;
}

.border {
  border: 1px solid rgba(0, 0, 0, 0.1);
}

.hosts-table :deep(th) {
  background-color: #e0e7ff !important; /* 浅蓝色背景 */
  font-weight: 600 !important;
  color: #1a3a6a !important; /* 深蓝色文字 */
  letter-spacing: 0.5px;
}

.hosts-table :deep(tbody tr) {
  background-color: white !important;
  color: #333 !important;
}

.hosts-table :deep(tbody tr:nth-child(even)) {
  background-color: #f8faff !important; /* 非常浅的蓝色 */
}

.hosts-table :deep(tbody tr:hover) {
  background-color: #edf2ff !important; /* 浅蓝色高亮 */
}

.switch-large :deep(.v-switch__track) {
  opacity: 1;
  background-color: var(--empire-gray);
  border: none;
}

.switch-large :deep(.v-switch__track--active) {
  background-color: var(--lightsaber-green) !important;
}

.switch-large :deep(.v-switch__thumb) {
  background-color: white;
}

.empty-state-container {
  max-width: 600px;
  margin: 0 auto;
}

/* 光剑开关效果 - 减弱版 */
.lightsaber-switch :deep(.v-switch__track) {
  height: 14px !important;
  opacity: 1 !important;
  background-color: rgba(0, 0, 0, 0.2) !important;
  border: none !important;
}

.lightsaber-switch :deep(.v-switch__track--active) {
  background-color: var(--lightsaber-blue) !important;
  box-shadow: 0 0 5px var(--lightsaber-blue) !important;
}

.lightsaber-switch.green :deep(.v-switch__track--active) {
  background-color: var(--lightsaber-green) !important;
  box-shadow: 0 0 5px var(--lightsaber-green) !important;
}

.lightsaber-switch.red :deep(.v-switch__track--active) {
  background-color: var(--lightsaber-red) !important;
  box-shadow: 0 0 5px var(--lightsaber-red) !important;
}

/* 星空背景 - 减弱版 */
.star-bg {
  background-color: var(--jedi-dark-blue);
  background-image:
    radial-gradient(white, rgba(255,255,255,.1) 1px, transparent 2px),
    radial-gradient(white, rgba(255,255,255,.05) 1px, transparent 1px);
  background-size: 550px 550px, 350px 350px;
  background-position: 0 0, 40px 60px;
}

/* 光剑按钮 - 减弱版 */
.lightsaber-btn {
  position: relative;
  overflow: hidden;
  transition: all 0.3s ease;
}

.lightsaber-btn.blue {
  background-color: var(--jedi-blue) !important;
}

.lightsaber-btn.blue:hover {
  box-shadow: 0 0 5px var(--lightsaber-blue) !important;
}

.lightsaber-btn.green {
  background-color: var(--lightsaber-green) !important;
}

.lightsaber-btn.green:hover {
  box-shadow: 0 0 5px var(--lightsaber-green) !important;
}

.lightsaber-btn.red {
  background-color: var(--lightsaber-red) !important;
}

.lightsaber-btn.red:hover {
  box-shadow: 0 0 5px var(--lightsaber-red) !important;
}
</style>
