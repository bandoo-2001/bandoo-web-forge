<script setup lang="ts">
import { computed, onMounted, reactive, ref } from 'vue'
import { storeToRefs } from 'pinia'
import { useScriptStore } from '@/stores/scripts'
import { useAutomationStore } from '@/stores/automations'
import type { UserScriptConfig, UserScriptDraft, UserScriptRunResult } from '@/types/scripts'

const store = useScriptStore()
const automationStore = useAutomationStore()
const { items } = storeToRefs(store)
const { logs } = storeToRefs(automationStore)
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
  language: 'javascript',
  compiledCode: '',
  runAt: 'manual',
  matchPatterns: ['*'],
  requiredPermissions: ['page', 'notification'],
  code: `workflow.log('title:', bandoo.getTitle())
workflow.log('route:', bandoo.getRoute())
notification.send('Bandoo 用户脚本', \`当前页面：\${app.name}\`)`,
})
const matchPreview = computed(() => draft.matchPatterns.join('、') || '*')
const scriptLogs = computed(() => logs.value.filter((log) => log.kind === 'user-script').slice(0, 6))

function resetDraft() {
  editingId.value = null
  draft.webAppId = ''
  draft.name = 'Bridge 诊断脚本'
  draft.enabled = true
  draft.language = 'javascript'
  draft.compiledCode = ''
  draft.runAt = 'manual'
  draft.matchPatterns = ['*']
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
  draft.language = item.language
  draft.compiledCode = item.compiledCode ?? ''
  draft.runAt = item.runAt
  draft.matchPatterns = [...item.matchPatterns]
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

function updateMatchPatterns(event: Event) {
  draft.matchPatterns = (event.target as HTMLInputElement).value
    .split(',')
    .map((item) => item.trim())
    .filter(Boolean)
}

async function submit() {
  const payload = {
    ...draft,
    matchPatterns: [...draft.matchPatterns],
    requiredPermissions: [...draft.requiredPermissions],
  }
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
    await automationStore.loadLogs()
  } catch (error) {
    lastResult.value = null
    message.type = 'error'
    message.text = error instanceof Error ? error.message : String(error)
  }
}

onMounted(() => {
  void Promise.all([store.load(), automationStore.loadLogs()])
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
      <div class="field-row">
        <label>
          <span>语言</span>
          <select v-model="draft.language">
            <option value="javascript">JavaScript</option>
            <option value="typescript">TypeScript</option>
          </select>
        </label>
        <label>
          <span>运行时机</span>
          <select v-model="draft.runAt">
            <option value="manual">手动</option>
            <option value="page-load">页面加载</option>
            <option value="url-change">URL 变化</option>
            <option value="shortcut">快捷键</option>
          </select>
        </label>
      </div>
      <label>
        <span>匹配规则</span>
        <input
          :value="draft.matchPatterns.join(', ')"
          placeholder="*, chatgpt.com, /docs"
          @input="updateMatchPatterns"
        />
        <small>当前匹配：{{ matchPreview }}</small>
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
        <label>
          <input
            :checked="draft.requiredPermissions.includes('shell')"
            type="checkbox"
            @change="onPermissionChange('shell', $event)"
          />
          Shell
        </label>
        <label>
          <input
            :checked="draft.requiredPermissions.includes('filesystem')"
            type="checkbox"
            @change="onPermissionChange('filesystem', $event)"
          />
          文件
        </label>
        <label>
          <input
            :checked="draft.requiredPermissions.includes('network')"
            type="checkbox"
            @change="onPermissionChange('network', $event)"
          />
          网络
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
      <div v-if="scriptLogs.length > 0" class="step-results">
        <strong>最近脚本日志</strong>
        <ol>
          <li v-for="log in scriptLogs" :key="log.id">
            <span>{{ log.sourceId }}</span>
            <em :class="log.status">{{ log.status }}</em>
            <small>{{ log.message }}</small>
          </li>
        </ol>
      </div>
      <article v-for="item in items" :key="item.id" class="app-item">
        <div>
          <strong>{{ item.name }}</strong>
          <span>{{ item.enabled ? '已启用' : '已停用' }} · {{ item.webAppId || '未绑定' }}</span>
          <small>权限：{{ item.requiredPermissions.join('、') || '无' }}</small>
          <small>{{ item.language }} · {{ item.runAt }} · {{ item.matchPatterns.join('、') || '*' }}</small>
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
