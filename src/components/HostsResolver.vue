<template>
  <v-card flat class="d-flex align-center pa-2">
    <v-btn rounded color="deep-purple-accent-2" class="ma-2 px-4 text-uppercase font-weight-bold"
      @click="showAddGroupDialog = true">
      添加分组
    </v-btn>
    <v-tabs v-model="selectedTag" class="ml-4" color="deep-purple-accent-4">
      <v-tab v-for="group in tags" :key="group.tag" :value="group.tag" class="text-uppercase font-weight-bold">
        {{ group.tag }}
      </v-tab>
    </v-tabs>
  </v-card>

  <v-card flat class="pa-4 mt-2">
    <div v-if="tags.length">
      <div v-for="group in tags.filter(t => t.tag === selectedTag)" :key="group.tag" class="mb-6">
        <div class="d-flex justify-space-between align-center mb-3">
          <v-btn color="indigo" @click="openAddHostDialog(group.tag)">新增条目</v-btn>
          <v-switch v-model="hostsResolveSwitch" hide-details label="开启解析" color="indigo-darken-3"></v-switch>
        </div>
        <v-table class="mt-2" density="compact">
          <thead>
            <tr>
              <th>IP地址</th>
              <th>域名</th>
              <th>开关</th>
              <th>操作</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="(hostMap, idx) in group.hosts" :key="idx">
              <td style="vertical-align: middle; padding: 4px 8px;">
                {{ group.hosts[idx][Object.keys(hostMap)[0]] }}
              </td>
              <td style="vertical-align: middle; padding: 4px 8px;">
                {{ Object.keys(hostMap)[0] }}
              </td>
              <td style="vertical-align: middle; padding: 4px 8px;">
                <v-switch class="my-0" style="margin-top: 0; margin-bottom: 0; vertical-align: middle;"></v-switch>
              </td>
              <td style="vertical-align: middle; padding: 4px 8px;">
                <v-btn variant="text" :icon="mdiDelete" @click="group.hosts.splice(idx, 1)" color="red-lighten-2"></v-btn>
              </td>
            </tr>
          </tbody>
        </v-table>
      </div>
    </div>
    <div style="text-align: center;" v-else>暂无解析</div>
  </v-card>

  <v-dialog v-model="showAddGroupDialog" max-width="500">
    <v-card>
      <v-card-title>添加分组</v-card-title>
      <v-card-text>
        <v-text-field v-model="newGroupTag" label="分组名称 (tag)"></v-text-field>
        <v-switch v-model="newGroupIsRemote" label="是否远程配置"></v-switch>
        <v-text-field v-if="newGroupIsRemote" v-model="newGroupUrl" label="远程配置 URL"></v-text-field>
        <div v-if="!newGroupIsRemote">
          <v-textarea v-model="newGroupHosts" label="Hosts 列表 (格式: IP 域名，每行一条)"></v-textarea>
        </div>
      </v-card-text>
      <v-card-actions>
        <v-spacer></v-spacer>
        <v-btn text @click="showAddGroupDialog = false">取消</v-btn>
        <v-btn color="indigo" @click="confirmAddGroup">确认</v-btn>
      </v-card-actions>
    </v-card>
  </v-dialog>

  <v-dialog v-model="showAddHostDialog" max-width="400">
    <v-card>
      <v-card-title>新增条目</v-card-title>
      <v-card-text>
        <v-text-field v-model="newHostIp" label="IP地址"></v-text-field>
        <v-text-field v-model="newHostDomain" label="域名"></v-text-field>
      </v-card-text>
      <v-card-actions>
        <v-spacer></v-spacer>
        <v-btn text @click="showAddHostDialog = false">取消</v-btn>
        <v-btn color="indigo" @click="confirmAddHost">确认</v-btn>
      </v-card-actions>
    </v-card>
  </v-dialog>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { mdiPlus, mdiDelete } from '@mdi/js'

const hostsResolveSwitch = ref(false)

const remoteUrl = ref('')
const tags = ref<Array<{ tag: string; hosts: Array<Record<string, string>> }>>([])
const selectedTag = ref<string>('')
const showSnackbar = ref(false)

const showAddGroupDialog = ref(false)
const newGroupTag = ref('')
const newGroupIsRemote = ref(false)
const newGroupUrl = ref('')
const newGroupHosts = ref('')

const showAddHostDialog = ref(false)
const newHostIp = ref('')
const newHostDomain = ref('')
const currentAddGroupTag = ref('')

onMounted(() => {
  getOsInfo()
})

function handleHostsSwitch(switchState: boolean | null) {
  if (switchState) updateHosts()
  else revertHosts()
}

async function revertHosts() {
  await invoke('revert_hosts')
}

async function loadConfig() {
  try {
    const result: Array<{ tag: string; hosts: Array<Record<string, string>> }> = await invoke('fetch_remote_config', { url: remoteUrl.value })
    tags.value = result
    if (result.length > 0) {
      selectedTag.value = result[0].tag
    }
  } catch (e) {
    console.error('加载配置失败', e)
  }
}

function openAddHostDialog(tag: string) {
  currentAddGroupTag.value = tag
  newHostIp.value = ''
  newHostDomain.value = ''
  showAddHostDialog.value = true
}

function confirmAddHost() {
  if (!newHostIp.value.trim() || !newHostDomain.value.trim()) {
    alert('IP和域名不能为空')
    return
  }

  const group = tags.value.find(t => t.tag === currentAddGroupTag.value)
  if (!group) {
    alert('未找到对应分组')
    return
  }

  group.hosts.push({ [newHostDomain.value.trim()]: newHostIp.value.trim() })

  selectedTag.value = currentAddGroupTag.value
  showAddHostDialog.value = false
  newHostIp.value = ''
  newHostDomain.value = ''
}

async function confirmAddGroup() {
  if (!newGroupTag.value) {
    alert('分组名称不能为空')
    return
  }

  if (newGroupIsRemote.value && !newGroupUrl.value) {
    alert('远程配置URL不能为空')
    return
  }

  let hostsArray: Array<Record<string, string>> = []

  if (newGroupIsRemote.value) {
    try {
      const result: Array<{ tag: string; hosts: Array<Record<string, string>> }> = await invoke('fetch_remote_config', { url: newGroupUrl.value })
      const found = result.find(r => r.tag === newGroupTag.value)
      if (found) {
        hostsArray = found.hosts
      } else {
        alert('远程配置中未找到该分组')
        return
      }
    } catch (e) {
      console.error('远程加载失败', e)
      alert('远程加载失败')
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
}

async function getOsInfo() {
  await invoke('get_os_info')
}
</script>