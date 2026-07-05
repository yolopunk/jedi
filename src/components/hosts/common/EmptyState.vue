<template>
  <!-- 空状态显示 -->
  <div class="d-flex flex-column align-center justify-center py-12 px-4 h-100">
    <div class="mb-6">
      <v-icon
        :icon="mdiDomain"
        size="80"
        color="var(--text-subtle)"
        class="death-star jedi-hover-scale"
      ></v-icon>
    </div>
    <h2 class="text-h5 font-weight-bold mb-2" style="color: var(--text);">{{ $t('hosts.empty.title') }}</h2>
    <p class="text-body-1 text-secondary mb-8 text-center" style="max-width: 500px;">
      {{ $t('hosts.empty.description') }}
    </p>
    <div class="d-flex justify-center gap-4">
      <v-btn
        color="success"
        variant="flat"
        class="mr-4 px-6"
        size="large"
        :prepend-icon="mdiPlus"
        @click="$emit('add-group')"
      >
        {{ $t('hosts.empty.addGroup') }}
      </v-btn>
      <v-btn
        color="primary"
        variant="outlined"
        class="px-6"
        size="large"
        :prepend-icon="mdiDomain"
        @click="$emit('use-default')"
      >
        {{ $t('hosts.empty.useDefault') }}
      </v-btn>
    </div>
  </div>
</template>

<script setup lang="ts">
import { mdiDomain, mdiPlus } from '@mdi/js'

// 定义组件事件
defineEmits<{
  (e: 'add-group'): void
  (e: 'use-default'): void
}>()
</script>

<style scoped>
.death-star {
  filter: drop-shadow(0 4px 8px rgba(0, 0, 0, 0.1));
  transition: transform 0.3s ease;
}

.gap-4 {
  gap: 16px;
}
</style>
