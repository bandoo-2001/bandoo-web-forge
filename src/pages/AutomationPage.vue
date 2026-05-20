<script setup lang="ts">
import { onMounted, reactive } from 'vue'
import { storeToRefs } from 'pinia'
import { useAutomationStore } from '@/stores/automations'
import type { AutomationDraft } from '@/types/automation'

const store = useAutomationStore()
const { items, loading } = storeToRefs(store)
const message = reactive({
  type: '',
  text: '',
})

const draft = reactive<AutomationDraft>({
  webAppId: '',
  name: '剪贴板输入到当前页面',
  enabled: true,
  trigger: {
    kind: 'shortcut',
    shortcut: 'Ctrl+Alt+A',
  },
  conditions: [
    {
      kind: 'url-contains',
      value: 'chatgpt.com',
    },
  ],
  actions: [
    { kind: 'clipboard-read' },
    { kind: 'page-focus', selector: 'textarea' },
    { kind: 'page-type', text: '{{clipboard}}' },
  ],
})

async function submit() {
  await store.create({
    ...draft,
    trigger: { ...draft.trigger },
    conditions: draft.conditions.map((item) => ({ ...item })),
    actions: draft.actions.map((item) => ({ ...item })),
  })
  message.type = 'success'
  message.text = '已保存工作流'
}

async function execute(id: string) {
  try {
    const result = await store.execute(id)
    message.type = result.dispatched ? 'success' : 'error'
    message.text = result.message
  } catch (error) {
    message.type = 'error'
    message.text = error instanceof Error ? error.message : String(error)
  }
}

onMounted(() => {
  void store.load()
})
</script>

<template>
  <section class="page-header">
    <div>
      <p class="eyebrow">MVP · 第二阶段</p>
      <h1>自动化工作流</h1>
    </div>
  </section>

  <section class="dashboard-grid">
    <form class="panel create-form" @submit.prevent="submit">
      <h2>步骤流工作流</h2>
      <label>
        <span>名称</span>
        <input v-model="draft.name" required />
      </label>
      <label>
        <span>绑定 WebApp ID</span>
        <input v-model="draft.webAppId" placeholder="留空表示稍后绑定" />
      </label>
      <div class="field-row">
        <label>
          <span>触发器</span>
          <select v-model="draft.trigger.kind">
            <option value="shortcut">全局快捷键</option>
            <option value="page-load">页面加载</option>
            <option value="url-change">URL 变化</option>
            <option value="menu-click">菜单点击</option>
          </select>
        </label>
        <label>
          <span>快捷键</span>
          <input v-model="draft.trigger.shortcut" />
        </label>
      </div>
      <label>
        <span>URL 条件</span>
        <input v-model="draft.conditions[0].value" placeholder="chatgpt.com" />
      </label>
      <label>
        <span>元素选择器</span>
        <input v-model="draft.actions[1].selector" placeholder="textarea" />
      </label>
      <label>
        <span>输入模板</span>
        <textarea v-model="draft.actions[2].text" rows="4" />
      </label>
      <label class="inline-toggle"><input v-model="draft.enabled" type="checkbox" /> 启用</label>
      <button class="primary-button" type="submit">保存工作流</button>
    </form>

    <section class="panel app-list">
      <div class="panel-title">
        <h2>工作流列表</h2>
        <span v-if="loading">加载中</span>
      </div>
      <p v-if="message.text" class="message" :class="message.type">{{ message.text }}</p>
      <article v-for="item in items" :key="item.id" class="app-item">
        <div>
          <strong>{{ item.name }}</strong>
          <span>{{ item.webAppId || '未绑定 WebApp' }} · {{ item.trigger.kind }} · {{ item.trigger.shortcut || item.trigger.url || '无参数' }}</span>
        </div>
        <div class="app-actions">
          <button type="button" @click="execute(item.id)">执行</button>
          <button type="button" class="danger" @click="store.remove(item.id)">删除</button>
        </div>
      </article>
    </section>
  </section>
</template>
