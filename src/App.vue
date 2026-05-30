<script setup lang="ts">
import { RouterLink, RouterView } from 'vue-router'
import { computed } from 'vue'
import { useRoute } from 'vue-router'

const route = useRoute()
const shellMode = computed(() => route.meta.shell === true)
</script>

<template>
  <RouterView v-if="shellMode" />
  <div v-else class="shell">
    <aside class="sidebar">
      <div class="brand">
        <span class="brand-mark">B</span>
        <div>
          <strong>Bandoo WebForge</strong>
          <span>WebApp Runtime</span>
        </div>
      </div>

      <nav class="nav">
        <RouterLink to="/">WebApps</RouterLink>
        <RouterLink to="/automation">自动化</RouterLink>
        <RouterLink to="/scripts">用户脚本</RouterLink>
        <RouterLink to="/ai">AI 模板</RouterLink>
        <RouterLink to="/settings">设置</RouterLink>
      </nav>
    </aside>

    <main class="content">
      <RouterView />
    </main>
  </div>
</template>
