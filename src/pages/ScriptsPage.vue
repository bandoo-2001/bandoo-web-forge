<script setup lang="ts">
import { onMounted, reactive } from 'vue'
import { storeToRefs } from 'pinia'
import { useScriptStore } from '@/stores/scripts'
import type { UserScriptDraft } from '@/types/scripts'

const store = useScriptStore()
const { items } = storeToRefs(store)

const draft = reactive<UserScriptDraft>({
  webAppId: '',
  name: '页面标题增强',
  enabled: true,
  requiredPermissions: ['page'],
  code: "console.log('Bandoo title:', window.__BANDOO__?.getTitle())",
})

async function submit() {
  await store.create({ ...draft, requiredPermissions: [...draft.requiredPermissions] })
}

onMounted(() => {
  void store.load()
})
</script>

<template>
  <section class="page-header">
    <div>
      <p class="eyebrow">MVP · 第三阶段</p>
      <h1>用户脚本</h1>
    </div>
  </section>

  <section class="dashboard-grid">
    <form class="panel create-form" @submit.prevent="submit">
      <h2>脚本管理</h2>
      <label>
        <span>名称</span>
        <input v-model="draft.name" required />
      </label>
      <label>
        <span>绑定 WebApp ID</span>
        <input v-model="draft.webAppId" />
      </label>
      <label>
        <span>脚本代码</span>
        <textarea v-model="draft.code" rows="10" spellcheck="false" />
      </label>
      <label class="inline-toggle"><input v-model="draft.enabled" type="checkbox" /> 启用</label>
      <button class="primary-button" type="submit">保存脚本</button>
    </form>

    <section class="panel app-list">
      <div class="panel-title">
        <h2>脚本列表</h2>
      </div>
      <article v-for="item in items" :key="item.id" class="app-item">
        <div>
          <strong>{{ item.name }}</strong>
          <span>{{ item.enabled ? '已启用' : '已停用' }} · {{ item.webAppId || '未绑定' }}</span>
        </div>
        <div class="app-actions">
          <button type="button" class="danger" @click="store.remove(item.id)">删除</button>
        </div>
      </article>
    </section>
  </section>
</template>
