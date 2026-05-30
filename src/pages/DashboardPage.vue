<script setup lang="ts">
import { computed, onMounted, reactive, ref } from 'vue'
import { storeToRefs } from 'pinia'
import { defaultChromeConfig, defaultWindowConfig, useWebAppStore } from '@/stores/webapps'
import { useThemeStore } from '@/stores/themes'
import { useAutomationStore } from '@/stores/automations'
import type { DesktopIntegrationStatus, DesktopIntegrationTarget } from '@/types/webapp'
import type { WebApp, WebAppDraft } from '@/types/webapp'

const store = useWebAppStore()
const themeStore = useThemeStore()
const automationStore = useAutomationStore()
const { items, loading } = storeToRefs(store)
const { presets } = storeToRefs(themeStore)
const { logs: runLogs } = storeToRefs(automationStore)
const editingId = ref<string | null>(null)
const errorMessage = ref('')
const statusMessage = ref('')
const importText = ref('')
const activityLog = ref<string[]>([])
const integrationStatuses = reactive<Record<string, DesktopIntegrationStatus[]>>({})

function defaultDraft(): WebAppDraft {
  return {
    name: 'ChatGPT',
    url: 'https://chatgpt.com',
    userAgent: '',
    icon: '',
    windowConfig: defaultWindowConfig(),
    permissions: {
      page: true,
      clipboard: true,
      shell: false,
      filesystem: false,
      network: false,
      notification: true,
    },
    scriptConfig: {
      injectBridge: true,
      customScriptEnabled: false,
      customScript: '',
    },
    chromeConfig: defaultChromeConfig(),
    startOnBoot: false,
    tray: true,
  }
}

const draft = reactive<WebAppDraft>(defaultDraft())
const formTitle = computed(() => (editingId.value ? '编辑 WebApp' : '创建 WebApp'))
const submitText = computed(() => (editingId.value ? '保存修改' : '创建应用'))
const iconPreview = computed(() => draft.icon?.trim() || '')
const highRiskCapabilities = computed(() =>
  [
    {
      key: 'shell',
      name: 'Shell',
      active: draft.permissions.shell,
      detail: '允许用户脚本和自动化执行本机命令。',
    },
    {
      key: 'filesystem',
      name: '文件系统',
      active: draft.permissions.filesystem,
      detail: '允许读取、写入、创建和删除本机文件。',
    },
    {
      key: 'network',
      name: '网络',
      active: draft.permissions.network,
      detail: '允许远程页面通过受控 Bridge 发起网络请求。',
    },
  ].filter((item) => item.active),
)
const recentBridgeLogs = computed(() =>
  runLogs.value
    .filter((item) => item.kind === 'bridge' && (!editingId.value || item.webAppId === editingId.value))
    .slice(0, 5),
)
const previewControlsOnLeft = computed(() => draft.chromeConfig.controlsPosition === 'left')
const chromePreviewFrameStyle = computed(() => ({
  borderRadius: `${Math.min(Math.max(draft.chromeConfig.cornerRadius, 0), 32)}px`,
  boxShadow: draft.chromeConfig.shadow ? '0 16px 40px rgb(15 23 42 / 0.16)' : 'none',
}))
const chromePreviewTitlebarStyle = computed(() => ({
  minHeight: `${Math.min(Math.max(draft.chromeConfig.titlebarHeight, 32), 88)}px`,
  backgroundColor: draft.chromeConfig.backgroundColor,
  color: draft.chromeConfig.foregroundColor,
  opacity: draft.chromeConfig.enabled ? draft.chromeConfig.opacity : 0.35,
}))

function log(message: string) {
  activityLog.value = [`${new Date().toLocaleTimeString()} ${message}`, ...activityLog.value].slice(0, 6)
}

function assignDraft(nextDraft: WebAppDraft) {
  Object.assign(draft, {
    ...nextDraft,
    windowConfig: { ...nextDraft.windowConfig },
    permissions: { ...nextDraft.permissions },
    scriptConfig: { ...nextDraft.scriptConfig },
    chromeConfig: { ...nextDraft.chromeConfig },
  })
}

function resetForm() {
  editingId.value = null
  assignDraft(defaultDraft())
}

function edit(app: WebApp) {
  editingId.value = app.id
  assignDraft({
    name: app.name,
    icon: app.icon,
    url: app.url,
    userAgent: app.userAgent ?? '',
    startOnBoot: app.startOnBoot ?? false,
    tray: app.tray ?? true,
    windowConfig: {
      width: app.windowConfig.width,
      height: app.windowConfig.height,
      maximized: app.windowConfig.maximized ?? false,
      transparent: app.windowConfig.transparent ?? true,
      decorations: app.windowConfig.decorations ?? false,
      stableFallback: app.windowConfig.stableFallback ?? true,
    },
    permissions: {
      clipboard: app.permissions.clipboard,
      shell: app.permissions.shell,
      filesystem: app.permissions.filesystem,
      page: app.permissions.page,
      network: app.permissions.network,
      notification: app.permissions.notification,
    },
    scriptConfig: {
      injectBridge: app.scriptConfig?.injectBridge ?? true,
      customScriptEnabled: app.scriptConfig?.customScriptEnabled ?? false,
      customScript: app.scriptConfig?.customScript ?? '',
    },
    chromeConfig: {
      ...defaultChromeConfig(),
      ...app.chromeConfig,
    },
  })
}

function normalizedDraft(): WebAppDraft {
  const url = new URL(draft.url.trim())
  if (!['http:', 'https:'].includes(url.protocol)) {
    throw new Error('URL 只支持 http 或 https')
  }

  return {
    ...draft,
    name: draft.name.trim(),
    url: url.toString(),
    icon: draft.icon?.trim() || undefined,
    userAgent: draft.userAgent?.trim() || undefined,
    windowConfig: { ...draft.windowConfig },
    permissions: { ...draft.permissions },
    scriptConfig: { ...draft.scriptConfig },
    chromeConfig: { ...draft.chromeConfig },
  }
}

async function submit() {
  try {
    errorMessage.value = ''
    statusMessage.value = ''
    const payload = normalizedDraft()
    const riskyPermissions = []
    if (payload.permissions.shell) riskyPermissions.push('Shell')
    if (payload.permissions.filesystem) riskyPermissions.push('文件系统')
    if (payload.permissions.network) riskyPermissions.push('网络')
    if (
      riskyPermissions.length > 0 &&
      !window.confirm(`将开启高风险权限：${riskyPermissions.join('、')}。确认继续？`)
    ) {
      return
    }

    if (editingId.value) {
      await store.update(editingId.value, payload)
      statusMessage.value = '已保存修改'
      log(`保存 WebApp：${payload.name}`)
    } else {
      await store.create(payload)
      statusMessage.value = '已创建应用'
      log(`创建 WebApp：${payload.name}`)
    }
    resetForm()
    await refreshAllIntegrationStatuses()
  } catch (error) {
    errorMessage.value = error instanceof Error ? error.message : String(error)
  }
}

async function launch(id: string) {
  try {
    errorMessage.value = ''
    await store.launch(id)
    statusMessage.value = '已发送启动请求'
    log(`启动 WebApp：${id}`)
  } catch (error) {
    errorMessage.value = error instanceof Error ? error.message : String(error)
  }
}

async function install(app: WebApp, target: DesktopIntegrationTarget) {
  try {
    errorMessage.value = ''
    const result = await store.installIntegration(app.id, target)
    statusMessage.value = `已写入 ${result.path}`
    log(`安装 ${app.name} 的 ${target} 入口`)
    await refreshIntegrationStatus(app.id)
  } catch (error) {
    errorMessage.value = error instanceof Error ? error.message : String(error)
  }
}

async function uninstall(app: WebApp, target: DesktopIntegrationTarget) {
  try {
    errorMessage.value = ''
    const result = await store.removeIntegration(app.id, target)
    statusMessage.value = `已移除 ${result.path}`
    log(`移除 ${app.name} 的 ${target} 入口`)
    await refreshIntegrationStatus(app.id)
  } catch (error) {
    errorMessage.value = error instanceof Error ? error.message : String(error)
  }
}

function exportJson() {
  const blob = new Blob([store.exportJson()], { type: 'application/json' })
  const url = URL.createObjectURL(blob)
  const anchor = document.createElement('a')
  anchor.href = url
  anchor.download = 'bandoo-webforge-webapps.json'
  anchor.click()
  URL.revokeObjectURL(url)
}

async function importJson() {
  try {
    errorMessage.value = ''
    if (!importText.value.trim()) {
      throw new Error('请先粘贴 WebApp JSON 数组')
    }
    await store.importJson(importText.value)
    importText.value = ''
    statusMessage.value = '已导入 WebApp 配置'
    log('导入 WebApp JSON 配置')
    await refreshAllIntegrationStatuses()
  } catch (error) {
    errorMessage.value = error instanceof Error ? error.message : String(error)
  }
}

async function remove(app: WebApp) {
  if (!window.confirm(`确认删除 ${app.name}？此操作会删除 WebApp 配置。`)) {
    return
  }
  await store.remove(app.id)
  delete integrationStatuses[app.id]
  log(`删除 WebApp：${app.name}`)
}

async function refreshIntegrationStatus(id: string) {
  integrationStatuses[id] = await store.integrationStatuses(id)
}

async function refreshAllIntegrationStatuses() {
  await Promise.all(items.value.map((item) => refreshIntegrationStatus(item.id)))
}

function installed(app: WebApp, target: DesktopIntegrationTarget) {
  return integrationStatuses[app.id]?.find((item) => item.target === target)?.installed ?? false
}

onMounted(() => {
  void Promise.all([store.load(), themeStore.load(), automationStore.loadLogs()]).then(refreshAllIntegrationStatuses)
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
      <div class="form-title">
        <h2>{{ formTitle }}</h2>
        <button v-if="editingId" type="button" @click="resetForm">取消</button>
      </div>
      <label>
        <span>名称</span>
        <input v-model="draft.name" required placeholder="ChatGPT" />
      </label>
      <label>
        <span>URL</span>
        <input v-model="draft.url" required type="url" placeholder="https://chatgpt.com" />
      </label>
      <label>
        <span>图标路径</span>
        <input v-model="draft.icon" placeholder="Linux .desktop 可复用本地图标路径" />
      </label>
      <img v-if="iconPreview" class="icon-preview" :src="iconPreview" alt="" />
      <label>
        <span>UserAgent</span>
        <input v-model="draft.userAgent" placeholder="留空则使用系统 WebView 默认值" />
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
        <label><input v-model="draft.permissions.page" type="checkbox" /> 页面</label>
        <label><input v-model="draft.permissions.clipboard" type="checkbox" /> 剪贴板</label>
        <label><input v-model="draft.permissions.filesystem" type="checkbox" /> 文件</label>
        <label><input v-model="draft.permissions.shell" type="checkbox" /> Shell</label>
        <label><input v-model="draft.permissions.network" type="checkbox" /> 网络</label>
        <label><input v-model="draft.permissions.notification" type="checkbox" /> 通知</label>
        <label><input v-model="draft.tray" type="checkbox" /> 托盘</label>
        <label><input v-model="draft.startOnBoot" type="checkbox" /> 开机启动</label>
        <label><input v-model="draft.windowConfig.maximized" type="checkbox" /> 默认最大化</label>
        <label><input v-model="draft.windowConfig.transparent" type="checkbox" /> 透明窗口</label>
        <label><input v-model="draft.windowConfig.decorations" type="checkbox" /> 系统标题栏</label>
      </div>
      <div v-if="highRiskCapabilities.length > 0" class="risk-panel">
        <strong>高风险权限已开启</strong>
        <ul>
          <li v-for="capability in highRiskCapabilities" :key="capability.key">
            <span>{{ capability.name }}</span>
            <small>{{ capability.detail }}</small>
          </li>
        </ul>
        <div v-if="recentBridgeLogs.length > 0" class="risk-log">
          <span>最近 Bridge 调用</span>
          <small v-for="log in recentBridgeLogs" :key="log.id">
            {{ new Date(log.startedAt).toLocaleTimeString() }} · {{ log.sourceId }} · {{ log.status }} · {{ log.message }}
          </small>
        </div>
      </div>
      <div class="appearance-editor">
        <h3>窗口个性化</h3>
        <label>
          <span>主题预设</span>
          <select v-model="draft.chromeConfig.themePresetId">
            <option value="">不使用预设</option>
            <option v-for="preset in presets" :key="preset.id" :value="preset.id">
              {{ preset.name }}
            </option>
          </select>
        </label>
        <div class="field-row">
          <label>
            <span>顶部栏高度</span>
            <input v-model.number="draft.chromeConfig.titlebarHeight" type="number" min="32" max="88" />
          </label>
          <label>
            <span>窗口圆角</span>
            <input v-model.number="draft.chromeConfig.cornerRadius" type="number" min="0" max="32" />
          </label>
        </div>
        <div class="field-row">
          <label>
            <span>顶部栏背景</span>
            <input v-model="draft.chromeConfig.backgroundColor" type="color" />
          </label>
          <label>
            <span>文字颜色</span>
            <input v-model="draft.chromeConfig.foregroundColor" type="color" />
          </label>
        </div>
        <label>
          <span>透明度 {{ Math.round(draft.chromeConfig.opacity * 100) }}%</span>
          <input v-model.number="draft.chromeConfig.opacity" type="range" min="0.72" max="1" step="0.01" />
        </label>
        <div class="field-row">
          <label>
            <span>窗口按钮位置</span>
            <select v-model="draft.chromeConfig.controlsPosition">
              <option value="right">右侧</option>
              <option value="left">左侧</option>
            </select>
          </label>
          <label>
            <span>按钮风格</span>
            <select v-model="draft.chromeConfig.controlsStyle">
              <option value="windows">Windows</option>
              <option value="traffic-light">Traffic light</option>
              <option value="minimal">Minimal</option>
            </select>
          </label>
        </div>
        <div class="toggle-row">
          <label><input v-model="draft.chromeConfig.enabled" type="checkbox" /> 自绘顶部栏</label>
          <label><input v-model="draft.chromeConfig.shadow" type="checkbox" /> 阴影</label>
          <label><input v-model="draft.chromeConfig.showTitle" type="checkbox" /> 标题</label>
          <label><input v-model="draft.chromeConfig.showIcon" type="checkbox" /> 图标</label>
          <label><input v-model="draft.chromeConfig.showUrl" type="checkbox" /> URL</label>
        </div>
        <div class="chrome-preview" :style="chromePreviewFrameStyle">
          <div
            class="chrome-preview-titlebar"
            :class="[`controls-${draft.chromeConfig.controlsPosition}`, `style-${draft.chromeConfig.controlsStyle}`]"
            :style="chromePreviewTitlebarStyle"
          >
            <div v-if="previewControlsOnLeft" class="window-controls">
              <span class="control close" />
              <span class="control minimize" />
              <span class="control maximize" />
            </div>
            <div class="titlebar-identity">
              <span v-if="draft.chromeConfig.showIcon" class="fallback-icon">B</span>
              <div>
                <strong v-if="draft.chromeConfig.showTitle">{{ draft.name || 'Bandoo WebApp' }}</strong>
                <small v-if="draft.chromeConfig.showUrl">{{ draft.url || 'https://example.com' }}</small>
              </div>
            </div>
            <div v-if="!previewControlsOnLeft" class="window-controls">
              <span class="control minimize" />
              <span class="control maximize" />
              <span class="control close" />
            </div>
          </div>
          <div class="chrome-preview-content">
            <span>{{ draft.url || 'https://example.com' }}</span>
          </div>
        </div>
      </div>
      <div class="toggle-row">
        <label><input v-model="draft.scriptConfig.injectBridge" type="checkbox" /> 注入 Bridge</label>
        <label><input v-model="draft.scriptConfig.customScriptEnabled" type="checkbox" /> 启用自定义脚本</label>
      </div>
      <label>
        <span>自定义注入脚本</span>
        <textarea v-model="draft.scriptConfig.customScript" rows="5" spellcheck="false" />
      </label>
      <button class="primary-button" type="submit">{{ submitText }}</button>
    </form>

    <section class="panel app-list">
      <div class="panel-title">
        <h2>应用列表</h2>
        <span v-if="loading">加载中</span>
      </div>
      <p v-if="errorMessage" class="message error">{{ errorMessage }}</p>
      <p v-if="statusMessage" class="message success">{{ statusMessage }}</p>
      <div v-if="activityLog.length > 0" class="activity-log">
        <strong>最近操作</strong>
        <span v-for="entry in activityLog" :key="entry">{{ entry }}</span>
      </div>

      <div v-if="items.length === 0" class="empty-state">还没有 WebApp，先创建一个。</div>

      <article v-for="app in items" :key="app.id" class="app-item">
        <div>
          <strong>{{ app.name }}</strong>
          <span>{{ app.url }}</span>
          <small v-if="app.lastWindowState">
            {{ app.lastWindowState.width }} × {{ app.lastWindowState.height }}
          </small>
          <small>
            菜单 {{ installed(app, 'applications') ? '已安装' : '未安装' }} · 桌面
            {{ installed(app, 'desktop') ? '已安装' : '未安装' }} · 自启动
            {{ installed(app, 'autostart') ? '已启用' : '未启用' }}
          </small>
        </div>
        <div class="app-actions">
          <button type="button" @click="launch(app.id)">启动</button>
          <button type="button" @click="edit(app)">编辑</button>
          <button type="button" @click="install(app, 'applications')">菜单入口</button>
          <button type="button" @click="install(app, 'desktop')">桌面入口</button>
          <button type="button" @click="install(app, 'autostart')">自启动</button>
          <button type="button" @click="uninstall(app, 'autostart')">取消自启动</button>
          <button type="button" class="danger" @click="remove(app)">删除</button>
        </div>
      </article>

      <div class="import-export">
        <button type="button" @click="exportJson">导出 JSON</button>
        <textarea v-model="importText" rows="5" placeholder="粘贴 WebApp JSON 数组后导入" />
        <button type="button" @click="importJson">导入 JSON</button>
      </div>
    </section>
  </section>
</template>
