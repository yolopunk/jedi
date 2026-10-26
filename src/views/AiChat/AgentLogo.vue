<script setup lang="ts">
import antigravityLogo from '@lobehub/icons-static-svg/icons/antigravity-color.svg?url'
import claudeCodeLogo from '@lobehub/icons-static-svg/icons/claudecode-color.svg?url'
import codexLogo from '@lobehub/icons-static-svg/icons/codex-color.svg?url'
import geminiCliLogo from '@lobehub/icons-static-svg/icons/geminicli-color.svg?url'
import kimiLogo from '@lobehub/icons-static-svg/icons/kimi-color.svg?url'
import qwenLogo from '@lobehub/icons-static-svg/icons/qwen-color.svg?url'
import { computed } from 'vue'

const props = withDefaults(
  defineProps<{
    agentId?: string
    size?: number
  }>(),
  {
    agentId: 'builtin',
    size: 24,
  }
)

type LogoDefinition = {
  src: string
  monochrome?: boolean
}

const logos: Record<string, LogoDefinition> = {
  builtin: { src: '/branding/jedi-mark.svg', monochrome: true },
  'jedi-builtin': { src: '/branding/jedi-mark.svg', monochrome: true },
  'claude-code': { src: claudeCodeLogo },
  codex: { src: codexLogo },
  'gemini-cli': { src: geminiCliLogo },
  'qwen-code': { src: qwenLogo },
  'cursor-cli': { src: '/agent-logos/cursor.svg' },
  opencode: { src: '/agent-logos/opencode.svg' },
  pi: { src: '/agent-logos/pi.svg' },
  antigravity: { src: antigravityLogo },
  goose: { src: '/agent-logos/goose.png', monochrome: true },
  'kimi-code': { src: kimiLogo },
  crush: { src: '/agent-logos/crush.svg' },
  droid: { src: '/agent-logos/droid.svg', monochrome: true },
  zcode: { src: '/agent-logos/zcode.png' },
}

const normalizedId = computed(() => props.agentId.toLowerCase().split('@')[0])
const logo = computed(() => logos[normalizedId.value])
const initials = computed(() =>
  normalizedId.value
    .split(/[-_\s]+/)
    .map(part => part[0])
    .join('')
    .slice(0, 2)
    .toUpperCase()
)
const style = computed(() => ({ '--agent-logo-size': `${props.size}px` }))
</script>

<template>
  <span class="agent-logo" :class="{ fallback: !logo }" :style="style" aria-hidden="true">
    <span
      v-if="logo?.monochrome"
      class="agent-logo-mask"
      :style="{ maskImage: `url(${logo.src})`, WebkitMaskImage: `url(${logo.src})` }"
    ></span>
    <img v-else-if="logo" :src="logo.src" alt="" draggable="false" />
    <span v-else class="agent-logo-fallback">{{ initials }}</span>
  </span>
</template>

<style scoped>
.agent-logo {
  width: var(--agent-logo-size);
  height: var(--agent-logo-size);
  flex: 0 0 var(--agent-logo-size);
  display: grid;
  place-items: center;
  color: var(--text);
}

.agent-logo img,
.agent-logo-mask {
  width: 100%;
  height: 100%;
  display: block;
}

.agent-logo img { object-fit: contain; }

.agent-logo-mask {
  background: currentColor;
  mask-position: center;
  mask-repeat: no-repeat;
  mask-size: contain;
  -webkit-mask-position: center;
  -webkit-mask-repeat: no-repeat;
  -webkit-mask-size: contain;
}

.agent-logo.fallback {
  border: 1px solid rgb(var(--text-rgb) / 0.14);
  border-radius: 28%;
  background: rgb(var(--text-rgb) / 0.055);
}

.agent-logo-fallback {
  font: 700 calc(var(--agent-logo-size) * 0.36) var(--jedi-font-mono);
  letter-spacing: -0.04em;
}
</style>
