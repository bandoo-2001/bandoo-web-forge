<script setup lang="ts">
import { onMounted, reactive } from 'vue'
import { storeToRefs } from 'pinia'
import { useWebAppStore } from '@/stores/webapps'
import type { WebAppDraft } from '@/types/webapp'

const store = useWebAppStore()
const { items, loading } = storeToRefs(store)

const draft = reactive<WebAppDraft>({
  name: 'ChatGPT',
  url: 'https://chatgpt.com',
  windowConfig: {
    width: 1280,
    height: 860,
  },
  permissions: {
    clipboard: true,
    shell: false,
    filesystem: false,
  },
  startOnBoot: false,
  tray: true,
})

async function submit() {
  await store.create({ ...draft, windowConfig: { ...draft.windowConfig }, permissions: { ...draft.permissions } })
}

onMounted(() => {
  void store.load()
})
</script>

<template>
  <section class="page-header">
    <div>
      <p class="eyebrow">MVP · 第一阶段</p>
      <h1>WebApp 管理</h1>
    </div>
  </section>

  <section class="dashboard-grid">
    <form class="panel create-form" @submit.prevent="submit">
      <h2>创建 WebApp</h2>
      <label>
        <span>名称</span>
        <input v-model="draft.name" required placeholder="ChatGPT" />
      </label>
      <label>
        <span>URL</span>
        <input v-model="draft.url" required type="url" placeholder="https://chatgpt.com" />
      </label>
      <div class="field-row">
        <label>
          <span>宽度</span>
          <input v-model.number="draft.windowConfig.width" required type="number" min="640" />
        </label>
        <label>
          <span>高度</span>
          <input v-model.number="draft.windowConfig.height" required type="number" min="480" />
        </label>
      </div>
      <div class="toggle-row">
        <label><input v-model="draft.permissions.clipboard" type="checkbox" /> 剪贴板</label>
        <label><input v-model="draft.tray" type="checkbox" /> 托盘</label>
        <label><input v-model="draft.startOnBoot" type="checkbox" /> 开机启动</label>
      </div>
      <button class="primary-button" type="submit">创建应用</button>
    </form>

    <section class="panel app-list">
      <div class="panel-title">
        <h2>应用列表</h2>
        <span v-if="loading">加载中</span>
      </div>

      <div v-if="items.length === 0" class="empty-state">还没有 WebApp，先创建一个。</div>

      <article v-for="app in items" :key="app.id" class="app-item">
        <div>
          <strong>{{ app.name }}</strong>
          <span>{{ app.url }}</span>
        </div>
        <div class="app-actions">
          <button type="button" @click="store.launch(app.id)">启动</button>
          <button type="button" class="danger" @click="store.remove(app.id)">删除</button>
        </div>
      </article>
    </section>
  </section>
</template>
