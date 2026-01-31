<script setup lang="ts">
import { TresCanvas } from '@tresjs/core'
import { OrbitControls, GLTFModel } from '@tresjs/cientos'
import { SRGBColorSpace, WebGLRenderer, ACESFilmicToneMapping } from 'three'

const onCreated = ({ renderer, scene }: { renderer: WebGLRenderer; scene: any }) => {
  // 诊断性设置：尝试半透明红色，确认 alpha 是否生效
  renderer.setClearColor(0xff0000, 0.5)
  renderer.setClearAlpha(0.5)
  scene.background = null
  
  // 材质表现优化
  renderer.outputColorSpace = SRGBColorSpace
  renderer.toneMapping = ACESFilmicToneMapping
  renderer.toneMappingExposure = 1.2
}
</script>

<template>
  <div class="koala-3d-container">
    <TresCanvas 
      shadows
      alpha
      antialias
      :premultiplied-alpha="false"
      @created="onCreated"
    >
      <TresPerspectiveCamera :position="[0, 1, 4]" :look-at="[0, 0, 0]" />
      <OrbitControls :enableZoom="false" :enablePan="false" />
      
      <TresAmbientLight :intensity="2.0" />
      <TresDirectionalLight :position="[2, 5, 2]" :intensity="2.5" />
      <TresDirectionalLight :position="[-2, 2, 5]" :intensity="1.5" />
      
      <Suspense>
        <GLTFModel path="/koala-yoda.glb" :scale="3" :position="[0, -1.5, 0]" />
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
