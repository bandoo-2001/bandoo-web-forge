<script setup lang="ts">
import { RouterLink, RouterView } from 'vue-router'
</script>

<template>
  <div class="shell">
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
        <RouterLink to="/settings">设置</RouterLink>
      </nav>
    </aside>

    <main class="content">
      <RouterView />
    </main>
  </div>
</template>
