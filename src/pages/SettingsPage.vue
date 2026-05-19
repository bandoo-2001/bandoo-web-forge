<script setup lang="ts">
import { onMounted } from 'vue'
import { storeToRefs } from 'pinia'
import { useRuntimeStore } from '@/stores/runtime'

const runtime = useRuntimeStore()
const { info } = storeToRefs(runtime)

onMounted(() => {
  void runtime.load()
})
</script>

<template>
  <section class="page-header">
    <div>
      <p class="eyebrow">Runtime</p>
      <h1>设置中心</h1>
    </div>
  </section>

  <section class="panel settings-panel">
    <h2>运行平台</h2>
    <div class="runtime-grid">
      <span>OS</span>
      <strong>{{ info.os }}</strong>
      <span>Family</span>
      <strong>{{ info.family }}</strong>
      <span>Arch</span>
      <strong>{{ info.arch }}</strong>
      <span>Linux 优先</span>
      <strong>{{ info.linuxPrimary ? '是' : '否' }}</strong>
    </div>

    <h2>权限默认值</h2>
    <label><input type="checkbox" checked /> 默认最小权限</label>
    <label><input type="checkbox" /> 允许 Shell 权限申请</label>
    <label><input type="checkbox" /> 启用用户脚本实验能力</label>
  </section>
</template>
