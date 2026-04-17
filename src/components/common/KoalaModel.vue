<script setup lang="ts">
import { GLTFModel, OrbitControls } from '@tresjs/cientos'
import { TresCanvas } from '@tresjs/core'
import { ACESFilmicToneMapping, Color, SRGBColorSpace, type WebGLRenderer } from 'three'
import { shallowRef, watch } from 'vue'

const onCreated = ({ renderer, scene }: { renderer: WebGLRenderer; scene: any }) => {
  // 确保背景完全透明
  renderer.setClearColor(new Color(0x000000), 0)
  renderer.setClearAlpha(0)
  scene.background = null
  scene.environment = null

  // 材质表现优化
  renderer.outputColorSpace = SRGBColorSpace
  renderer.toneMapping = ACESFilmicToneMapping
  renderer.toneMappingExposure = 1.2
}

const modelRef = shallowRef<any>(null)

watch(modelRef, model => {
  if (!model) return
  model.traverse((child: any) => {
    if (!child?.isMesh) return
    const materials = Array.isArray(child.material) ? child.material : [child.material]
    const hasTexture = materials.some((m: any) => !!m?.map)
    const isPureBlack =
      materials.length > 0 &&
      materials.every((m: any) => {
        if (!m?.color) return false
        return m.color.getHex() === 0x000000
      })
    if (!hasTexture && isPureBlack) {
      child.visible = false
      return
    }
    materials.forEach((m: any) => {
      if (!m) return
      m.transparent = true
      if ('alphaTest' in m) m.alphaTest = 0.01
      if ('depthWrite' in m) m.depthWrite = true
      if ('opacity' in m) m.opacity = 1
      if ('needsUpdate' in m) m.needsUpdate = true
    })
  })
})
</script>

<template>
  <div class="koala-3d-container">
    <TresCanvas 
      shadows
      :alpha="true"
      antialias
      @created="onCreated"
    >
      <TresPerspectiveCamera :position="[0, 1, 4]" :look-at="[0, 0, 0]" />
      <OrbitControls :enableZoom="false" :enablePan="false" />
      
      <TresAmbientLight :intensity="2.0" />
      <TresDirectionalLight :position="[2, 5, 2]" :intensity="2.5" />
      <TresDirectionalLight :position="[-2, 2, 5]" :intensity="1.5" />
      
      <Suspense>
        <GLTFModel ref="modelRef" path="/koala-yoda.glb" :scale="3" :position="[0, -1.5, 0]" />
      </Suspense>
    </TresCanvas>
  </div>
</template>

<style scoped>
.koala-3d-container {
  width: 64px;
  height: 64px;
  background: transparent !important;
}

.koala-3d-container :deep(canvas) {
  background: transparent !important;
}
</style>
