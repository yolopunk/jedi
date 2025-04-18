<template>
  <v-app>
    <v-navigation-drawer permanent :border="0" :width="200" class="bg-grey-lighten-4 text-center">
      <v-container class="py-10">
        <v-img src="/icon.png"></v-img>
        <v-list nav>
          <v-list-item :active="true"  color="indigo" rounded="xl" title="Hosts 管理">
          </v-list-item>
        </v-list>
      </v-container>
      <template v-slot:append>
        <p class="text-indigo-lighten-2 my-2"><strong>v0.1.0</strong></p>
      </template>
    </v-navigation-drawer>
    <v-main class="bg-grey-lighten-4">
      <v-container class="py-8 px-6" height="100%">
        <hosts-resolver></hosts-resolver>
      </v-container>
    </v-main>

    <!-- 系统信息栏 -->
    <system-info-bar></system-info-bar>
  </v-app>
</template>
<script setup lang="ts">
import HostsResolver from '@/components/HostsResolver.vue'
import SystemInfoBar from '@/components/common/SystemInfoBar.vue'
</script>


<style scoped>
.logo.vite:hover {
  filter: drop-shadow(0 0 2em #747bff);
}

.logo.vue:hover {
  filter: drop-shadow(0 0 2em #249b73);
}
</style>
