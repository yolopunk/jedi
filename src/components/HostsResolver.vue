<template>
  <div class="text-h6 text-md-h5 text-lg-h4 font-weight-bold text-indigo">平台服务</div>
  <v-switch v-model="hostsResolveSwitch" hide-details label="开启解析" @update:model-value="handleHostsSwitch"
    color="indigo-darken-3"></v-switch>
  <v-row>
    <v-col v-for="host in hosts" :key="host.title" cols="auto">
      <v-card :title="host.title" :prepend-icon="host.icon" :append-icon="mdiOpenInNew" :href="host.href"
        :disabled="!hostsResolveSwitch" color="indigo" width="256" target="_blank" rel="noopener"></v-card>
    </v-col>
  </v-row>

  <!-- <v-sheet class="my-4">
    <div class="text-h6 text-md-h5 text-lg-h4 font-weight-bold text-indigo">系统信息</div>
    <p class="pa-2" v-for="info in systemInfo">{{ info }}</p>
  </v-sheet> -->

  <v-snackbar
    :timeout="2000"
    color="indigo-darken-1"
    v-model="showSnackbar"
  >
    解析已生效 
  </v-snackbar>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import {
  mdiGitlab,
  mdiKubernetes,
  mdiWeb,
  mdiOpenInNew,
  mdiNas,
  mdiAccount,
  mdiRouterNetwork,
  mdiRabbit,
  mdiBug
} from '@mdi/js'
import { invoke } from '@tauri-apps/api/core'

const hostsResolveSwitch = ref(false)
const showSnackbar = ref(false)
const hosts = ref([
  { title: 'Gitlab', icon: mdiGitlab, href: 'http://gitlab.dtc.io' },
  { title: 'Rancher', icon: mdiKubernetes, href: 'http://rancher.dtc.io' },
  { title: 'Traefik', icon: mdiRouterNetwork, href: 'http://traefik.dtc.io' },
  { title: 'Minio', icon: mdiWeb, href: 'http://minio.dtc.io' },
  { title: 'Portainer', icon: mdiWeb, href: 'http://portainer.dtc.io' },
  { title: 'Consul', icon: mdiWeb, href: 'http://consul.dtc.io' },
  { title: 'DasV', icon: mdiWeb, href: 'http://dasv.dtc.io' },
  { title: 'Runtime(DasV)', icon: mdiWeb, href: 'http://dasv-runtime.dtc.io' },
  { title: 'DM', icon: mdiWeb, href: 'http://dm.dtc.io' },
  { title: 'DM-Runtime', icon: mdiWeb, href: 'http://dm-runtime.dtc.io' },
  { title: 'NAS', icon: mdiNas, href: 'http://nas.dtc.io' },
  { title: 'LDAP', icon: mdiWeb, href: 'http://ldap.dtc.io' },
  { title: 'SSO', icon: mdiWeb, href: 'http://sso.dtc.io' },
  { title: 'OSS', icon: mdiAccount, href: 'http://oss.dtc.io' },
  { title: 'RabbitMQ', icon: mdiRabbit, href: 'http://rabbitmq.dtc.io' },
  { title: 'Zentao', icon: mdiBug, href: 'http://zentao.dtc.io' },
])
const systemInfo = ref('')

onMounted(() => {
  getOsInfo()
})

function handleHostsSwitch(switchState: boolean | null) {
  if (switchState) updateHosts()
  else revertHosts()
}

async function updateHosts() {
  await invoke('update_hosts')
  showSnackbar.value = true
}

async function revertHosts() {
  await invoke('revert_hosts')
  showSnackbar.value = true
}

async function getOsInfo() {
  systemInfo.value = await invoke('get_os_info')
}
</script>