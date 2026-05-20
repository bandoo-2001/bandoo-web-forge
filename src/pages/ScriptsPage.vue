<script setup lang="ts">
import { onMounted, reactive, ref } from 'vue'
import { storeToRefs } from 'pinia'
import { useScriptStore } from '@/stores/scripts'
import type { UserScriptConfig, UserScriptDraft, UserScriptRunResult } from '@/types/scripts'

const store = useScriptStore()
const { items } = storeToRefs(store)
const editingId = ref<string | null>(null)
const lastResult = ref<UserScriptRunResult | null>(null)
const message = reactive({
  type: '',
  text: '',
})

const draft = reactive<UserScriptDraft>({
  webAppId: '',
  name: 'Bridge 诊断脚本',
  enabled: true,
  requiredPermissions: ['page', 'notification'],
  code: `workflow.log('title:', bandoo.getTitle())
workflow.log('route:', bandoo.getRoute())
notification.send('Bandoo 用户脚本', \`当前页面：\${app.name}\`)`,
})

function resetDraft() {
  editingId.value = null
  draft.webAppId = ''
  draft.name = 'Bridge 诊断脚本'
  draft.enabled = true
  draft.requiredPermissions = ['page', 'notification']
  draft.code = `workflow.log('title:', bandoo.getTitle())
workflow.log('route:', bandoo.getRoute())
notification.send('Bandoo 用户脚本', \`当前页面：\${app.name}\`)`
}

function edit(item: UserScriptConfig) {
  editingId.value = item.id
  draft.webAppId = item.webAppId
  draft.name = item.name
  draft.enabled = item.enabled
  draft.requiredPermissions = [...item.requiredPermissions]
  draft.code = item.code
}

function togglePermission(permission: string, enabled: boolean) {
  draft.requiredPermissions = enabled
    ? Array.from(new Set([...draft.requiredPermissions, permission]))
    : draft.requiredPermissions.filter((item) => item !== permission)
}

function onPermissionChange(permission: string, event: Event) {
  togglePermission(permission, (event.target as HTMLInputElement).checked)
}

async function submit() {
  const payload = { ...draft, requiredPermissions: [...draft.requiredPermissions] }
  if (editingId.value) {
    const existing = items.value.find((item) => item.id === editingId.value)
    if (!existing) {
      throw new Error('User script not found')
    }
    await store.save({
      ...payload,
      id: existing.id,
      createdAt: existing.createdAt,
      updatedAt: Date.now(),
    })
    message.text = '已更新脚本'
  } else {
    await store.create(payload)
    message.text = '已保存脚本'
  }
  message.type = 'success'
}

async function execute(id: string) {
  try {
    const result = await store.execute(id)
    lastResult.value = result
    message.type = result.dispatched ? 'success' : 'error'
    message.text = result.message
  } catch (error) {
    lastResult.value = null
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
      <p class="eyebrow">MVP · 第三阶段</p>
      <h1>用户脚本</h1>
    </div>
  </section>

  <section class="dashboard-grid">
    <form class="panel create-form" @submit.prevent="submit">
      <div class="form-title">
        <h2>{{ editingId ? '编辑脚本' : '脚本管理' }}</h2>
        <button v-if="editingId" type="button" @click="resetDraft">新建</button>
      </div>
      <label>
        <span>名称</span>
        <input v-model="draft.name" required />
      </label>
      <label>
        <span>绑定 WebApp ID</span>
        <input v-model="draft.webAppId" required />
      </label>
      <div class="toggle-row">
        <label>
          <input
            :checked="draft.requiredPermissions.includes('page')"
            type="checkbox"
            @change="onPermissionChange('page', $event)"
          />
          页面
        </label>
        <label>
          <input
            :checked="draft.requiredPermissions.includes('clipboard')"
            type="checkbox"
            @change="onPermissionChange('clipboard', $event)"
          />
          剪贴板
        </label>
        <label>
          <input
            :checked="draft.requiredPermissions.includes('notification')"
            type="checkbox"
            @change="onPermissionChange('notification', $event)"
          />
          通知
        </label>
      </div>
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
      <p v-if="message.text" class="message" :class="message.type">{{ message.text }}</p>
      <div v-if="lastResult" class="script-result">
        <strong>最近运行：{{ lastResult.scriptId }}</strong>
        <span>{{ lastResult.webAppId }} · {{ lastResult.dispatched ? '已派发' : '未派发' }}</span>
        <small>{{ lastResult.message }}</small>
      </div>
      <article v-for="item in items" :key="item.id" class="app-item">
        <div>
          <strong>{{ item.name }}</strong>
          <span>{{ item.enabled ? '已启用' : '已停用' }} · {{ item.webAppId || '未绑定' }}</span>
          <small>权限：{{ item.requiredPermissions.join('、') || '无' }}</small>
        </div>
        <div class="app-actions">
          <button type="button" @click="execute(item.id)">运行</button>
          <button type="button" @click="edit(item)">编辑</button>
          <button type="button" class="danger" @click="store.remove(item.id)">删除</button>
        </div>
      </article>
    </section>
  </section>
</template>
