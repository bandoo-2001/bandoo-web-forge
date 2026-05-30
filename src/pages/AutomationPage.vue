<script setup lang="ts">
import { computed, onMounted, onUnmounted, reactive, ref } from 'vue'
import { storeToRefs } from 'pinia'
import { listen } from '@tauri-apps/api/event'
import { useAutomationStore } from '@/stores/automations'
import type { AutomationAction, AutomationDraft, AutomationRunResult } from '@/types/automation'

const store = useAutomationStore()
const { items, loading, logs } = storeToRefs(store)
const message = reactive({
  type: '',
  text: '',
})
const lastResult = ref<AutomationRunResult | null>(null)
const captureTargetIndex = ref<number | null>(null)
const recording = ref(false)
let unlistenSelector: (() => void) | null = null
let unlistenActions: (() => void) | null = null
const enabledShortcuts = computed(() =>
  items.value
    .filter((item) => item.enabled && item.trigger.kind === 'shortcut' && item.trigger.shortcut)
    .map((item) => item.trigger.shortcut),
)

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
    { kind: 'page-focus', selector: '#prompt-textarea, [data-testid="prompt-textarea"], textarea, [contenteditable="true"]' },
    { kind: 'page-type', selector: '#prompt-textarea, [data-testid="prompt-textarea"], textarea, [contenteditable="true"]', text: '{{clipboard}}' },
    { kind: 'notify', text: 'Bandoo WebForge', value: '已填入剪贴板内容' },
  ],
})

const actionOptions = [
  ['clipboard-read', '读取剪贴板'],
  ['clipboard-write', '写入剪贴板'],
  ['page-focus', '聚焦元素'],
  ['page-click', '点击元素'],
  ['page-type', '输入文本'],
  ['wait', '等待'],
  ['js', '执行 JS'],
  ['notify', '通知'],
  ['shell', 'Shell'],
  ['fs-read', '读取文件'],
  ['fs-write', '写入文件'],
  ['network-fetch', '网络请求'],
] as const

function addAction(kind = 'wait') {
  draft.actions.push({
    kind,
    timeoutMs: kind === 'wait' ? 500 : undefined,
  })
}

function removeAction(index: number) {
  draft.actions.splice(index, 1)
}

function isTauriRuntime() {
  return Boolean('__TAURI_INTERNALS__' in window)
}

async function captureSelector(index: number) {
  try {
    if (!draft.webAppId.trim()) {
      throw new Error('请先填写绑定 WebApp ID')
    }
    captureTargetIndex.value = index
    await store.startSelectorCapture(draft.webAppId.trim())
    message.type = 'success'
    message.text = '已进入元素选择模式'
  } catch (error) {
    captureTargetIndex.value = null
    message.type = 'error'
    message.text = error instanceof Error ? error.message : String(error)
  }
}

async function startRecording() {
  try {
    if (!draft.webAppId.trim()) {
      throw new Error('请先填写绑定 WebApp ID')
    }
    recording.value = true
    await store.startActionRecording(draft.webAppId.trim())
    message.type = 'success'
    message.text = '录制已开始，20 秒后自动回填'
  } catch (error) {
    recording.value = false
    message.type = 'error'
    message.text = error instanceof Error ? error.message : String(error)
  }
}

async function submit() {
  if (draft.actions[1]?.selector && draft.actions[2]) {
    draft.actions[2].selector = draft.actions[1].selector
  }
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
    lastResult.value = result
    message.type = result.dispatched ? 'success' : 'error'
    message.text = result.message
    await store.loadLogs()
  } catch (error) {
    lastResult.value = null
    message.type = 'error'
    message.text = error instanceof Error ? error.message : String(error)
  }
}

onMounted(() => {
  void Promise.all([store.load(), store.loadLogs()])
  if (isTauriRuntime()) {
    void listen<{ webAppId: string; selector: string }>('bandoo-selector-captured', (event) => {
      if (event.payload.webAppId !== draft.webAppId.trim()) return
      const index = captureTargetIndex.value
      if (index === null || !draft.actions[index]) return
      draft.actions[index].selector = event.payload.selector
      captureTargetIndex.value = null
      message.type = 'success'
      message.text = `已采集选择器：${event.payload.selector}`
      void store.loadLogs()
    }).then((unlisten) => {
      unlistenSelector = unlisten
    })
    void listen<{ webAppId: string; actions: AutomationAction[] }>('bandoo-actions-recorded', (event) => {
      if (event.payload.webAppId !== draft.webAppId.trim()) return
      const actions = event.payload.actions ?? []
      draft.actions.push(...actions.map((action) => ({ ...action })))
      recording.value = false
      message.type = 'success'
      message.text = `已回填 ${actions.length} 个录制步骤`
      void store.loadLogs()
    }).then((unlisten) => {
      unlistenActions = unlisten
    })
  }
})

onUnmounted(() => {
  unlistenSelector?.()
  unlistenActions?.()
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
      <div class="step-editor">
        <div class="panel-title">
          <h2>步骤编排</h2>
          <div class="app-actions">
            <button type="button" @click="startRecording">
              {{ recording ? '录制中' : '录制' }}
            </button>
            <button type="button" @click="addAction()">添加步骤</button>
          </div>
        </div>
        <article v-for="(action, index) in draft.actions" :key="index" class="step-card">
          <div class="field-row">
            <label>
              <span>动作</span>
              <select v-model="action.kind">
                <option v-for="[value, label] in actionOptions" :key="value" :value="value">
                  {{ label }}
                </option>
              </select>
            </label>
            <label>
              <span>超时 ms</span>
              <input v-model.number="action.timeoutMs" type="number" min="0" />
            </label>
          </div>
          <label>
            <span>Selector</span>
            <input v-model="action.selector" placeholder="textarea, button, [data-testid]" />
          </label>
          <label>
            <span>文本</span>
            <textarea v-model="action.text" rows="2" />
          </label>
          <label>
            <span>值 / 路径 / URL</span>
            <input v-model="action.value" />
          </label>
          <label>
            <span>JS</span>
            <textarea v-model="action.script" rows="3" spellcheck="false" />
          </label>
          <div class="app-actions">
            <label class="inline-toggle">
              <input v-model="action.continueOnError" type="checkbox" />
              失败后继续
            </label>
            <button type="button" class="danger" @click="removeAction(index)">删除步骤</button>
            <button type="button" @click="captureSelector(index)">
              {{ captureTargetIndex === index ? '采集中' : '采集 Selector' }}
            </button>
          </div>
        </article>
      </div>
      <label class="inline-toggle"><input v-model="draft.enabled" type="checkbox" /> 启用</label>
      <button class="primary-button" type="submit">保存工作流</button>
    </form>

    <section class="panel app-list">
      <div class="panel-title">
        <h2>工作流列表</h2>
        <span v-if="loading">加载中</span>
      </div>
      <p class="hint">已注册快捷键：{{ enabledShortcuts.join('、') || '暂无' }}</p>
      <p v-if="message.text" class="message" :class="message.type">{{ message.text }}</p>
      <div v-if="lastResult" class="step-results">
        <strong>最近执行：{{ lastResult.automationId }}</strong>
        <ol>
          <li v-for="step in lastResult.steps" :key="`${lastResult.automationId}-${step.index}`">
            <span>{{ step.index }}. {{ step.actionKind }}</span>
            <em :class="step.status">{{ step.status }}</em>
            <small>{{ step.message }}</small>
          </li>
        </ol>
      </div>
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
      <div v-if="logs.length > 0" class="step-results">
        <strong>最近运行日志</strong>
        <ol>
          <li v-for="log in logs.slice(0, 6)" :key="log.id">
            <span>{{ log.kind }} · {{ log.sourceId }}</span>
            <em :class="log.status">{{ log.status }}</em>
            <small>{{ log.message }}</small>
          </li>
        </ol>
      </div>
    </section>
  </section>
</template>
