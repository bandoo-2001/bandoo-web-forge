<script setup lang="ts">
import { onMounted, reactive, ref } from 'vue'
import { storeToRefs } from 'pinia'
import { useRuntimeStore } from '@/stores/runtime'
import { useThemeStore } from '@/stores/themes'
import { defaultChromeConfig } from '@/stores/webapps'

const runtime = useRuntimeStore()
const themeStore = useThemeStore()
const { info } = storeToRefs(runtime)
const { presets, settings } = storeToRefs(themeStore)
const themeName = ref('Custom Theme')
const importText = ref('')
const message = ref('')
const draftChrome = reactive(defaultChromeConfig())

onMounted(() => {
  void runtime.load()
  void themeStore.load().then(() => {
    Object.assign(draftChrome, settings.value.defaultChromeConfig)
  })
})

async function saveDefaultAppearance() {
  await themeStore.saveSettings({
    ...settings.value,
    defaultChromeConfig: { ...draftChrome },
  })
  message.value = '默认窗口外观已保存'
}

async function saveAsPreset() {
  await themeStore.createPreset(themeName.value.trim() || 'Custom Theme', { ...draftChrome })
  message.value = '主题预设已保存'
}

function exportThemes() {
  const blob = new Blob([themeStore.exportJson()], { type: 'application/json' })
  const url = URL.createObjectURL(blob)
  const anchor = document.createElement('a')
  anchor.href = url
  anchor.download = 'bandoo-webforge-themes.json'
  anchor.click()
  URL.revokeObjectURL(url)
}

async function importThemes() {
  await themeStore.importJson(importText.value)
  importText.value = ''
  message.value = '主题配置已导入'
}
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

    <h2>Bridge 诊断</h2>
    <div class="bridge-grid">
      <span>注入对象</span>
      <code>window.__BANDOO__</code>
      <span>页面 API</span>
      <code>app, permissions, page, clipboard, automation, notify</code>
      <span>路由事件</span>
      <code>bandoo:route-change</code>
      <span>验证方式</span>
      <code>window.__BANDOO__?.getRoute()</code>
    </div>

    <h2>窗口个性化</h2>
    <p v-if="message" class="message success">{{ message }}</p>
    <div class="appearance-editor">
      <div class="field-row">
        <label>
          <span>顶部栏高度</span>
          <input v-model.number="draftChrome.titlebarHeight" type="number" min="32" max="88" />
        </label>
        <label>
          <span>窗口圆角</span>
          <input v-model.number="draftChrome.cornerRadius" type="number" min="0" max="32" />
        </label>
      </div>
      <div class="field-row">
        <label>
          <span>顶部栏背景</span>
          <input v-model="draftChrome.backgroundColor" type="color" />
        </label>
        <label>
          <span>文字颜色</span>
          <input v-model="draftChrome.foregroundColor" type="color" />
        </label>
      </div>
      <label>
        <span>透明度 {{ Math.round(draftChrome.opacity * 100) }}%</span>
        <input v-model.number="draftChrome.opacity" type="range" min="0.72" max="1" step="0.01" />
      </label>
      <div class="field-row">
        <label>
          <span>窗口按钮位置</span>
          <select v-model="draftChrome.controlsPosition">
            <option value="right">右侧</option>
            <option value="left">左侧</option>
          </select>
        </label>
        <label>
          <span>按钮风格</span>
          <select v-model="draftChrome.controlsStyle">
            <option value="windows">Windows</option>
            <option value="traffic-light">Traffic light</option>
            <option value="minimal">Minimal</option>
          </select>
        </label>
      </div>
      <div class="toggle-row">
        <label><input v-model="draftChrome.enabled" type="checkbox" /> 自绘顶部栏</label>
        <label><input v-model="draftChrome.shadow" type="checkbox" /> 阴影</label>
        <label><input v-model="draftChrome.showTitle" type="checkbox" /> 标题</label>
        <label><input v-model="draftChrome.showIcon" type="checkbox" /> 图标</label>
        <label><input v-model="draftChrome.showUrl" type="checkbox" /> URL</label>
      </div>
      <div class="app-actions">
        <button type="button" @click="saveDefaultAppearance">保存为默认外观</button>
        <input v-model="themeName" placeholder="主题名称" />
        <button type="button" @click="saveAsPreset">保存为主题预设</button>
        <button type="button" @click="exportThemes">导出主题 JSON</button>
      </div>
      <textarea v-model="importText" rows="5" placeholder="粘贴主题 JSON 后导入" />
      <button type="button" @click="importThemes">导入主题 JSON</button>
    </div>

    <h2>主题预设</h2>
    <article v-for="preset in presets" :key="preset.id" class="app-item">
      <div>
        <strong>{{ preset.name }}</strong>
        <span>{{ preset.chromeConfig.backgroundColor }} / {{ preset.chromeConfig.foregroundColor }}</span>
      </div>
      <div class="app-actions">
        <button type="button" @click="Object.assign(draftChrome, preset.chromeConfig)">载入</button>
        <button type="button" class="danger" @click="themeStore.removePreset(preset.id)">删除</button>
      </div>
    </article>
  </section>
</template>
