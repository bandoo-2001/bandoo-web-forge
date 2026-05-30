<script setup lang="ts">
import { computed, onMounted } from 'vue'
import { useRoute } from 'vue-router'
import { storeToRefs } from 'pinia'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { useWebAppStore } from '@/stores/webapps'
import { mergeChromeConfig, useThemeStore } from '@/stores/themes'

const route = useRoute()
const store = useWebAppStore()
const themeStore = useThemeStore()
const { items } = storeToRefs(store)
const { presets, settings } = storeToRefs(themeStore)
const windowRef = getCurrentWindow()

const webAppId = computed(() => String(route.params.id ?? ''))
const app = computed(() => items.value.find((item) => item.id === webAppId.value))
const chrome = computed(() => mergeChromeConfig(app.value?.chromeConfig, settings.value, presets.value))
const titlebarStyle = computed(() => ({
  height: `${chrome.value.titlebarHeight}px`,
  backgroundColor: chrome.value.backgroundColor,
  color: chrome.value.foregroundColor,
  opacity: chrome.value.opacity,
  borderTopLeftRadius: `${chrome.value.cornerRadius}px`,
  borderTopRightRadius: `${chrome.value.cornerRadius}px`,
}))
const controlsOnLeft = computed(() => chrome.value.controlsPosition === 'left')

async function minimize() {
  await windowRef.minimize()
}

async function toggleMaximize() {
  await windowRef.toggleMaximize()
}

async function closeWindow() {
  await windowRef.close()
}

onMounted(() => {
  void Promise.all([store.load(), themeStore.load()])
})
</script>

<template>
  <main class="webapp-shell" :style="{ height: `${chrome.titlebarHeight}px` }">
    <header
      class="webapp-titlebar"
      :class="[`controls-${chrome.controlsPosition}`, `style-${chrome.controlsStyle}`]"
      :style="titlebarStyle"
      data-tauri-drag-region
    >
      <div v-if="controlsOnLeft" class="window-controls">
        <button type="button" class="control close" aria-label="关闭" @click="closeWindow" />
        <button type="button" class="control minimize" aria-label="最小化" @click="minimize" />
        <button type="button" class="control maximize" aria-label="最大化" @click="toggleMaximize" />
      </div>

      <div class="titlebar-identity" data-tauri-drag-region>
        <img v-if="chrome.showIcon && app?.icon" :src="app.icon" alt="" />
        <span v-else-if="chrome.showIcon" class="fallback-icon">B</span>
        <div data-tauri-drag-region>
          <strong v-if="chrome.showTitle">{{ app?.name || 'Bandoo WebApp' }}</strong>
          <small v-if="chrome.showUrl">{{ app?.url }}</small>
        </div>
      </div>

      <div v-if="!controlsOnLeft" class="window-controls">
        <button type="button" class="control minimize" aria-label="最小化" @click="minimize" />
        <button type="button" class="control maximize" aria-label="最大化" @click="toggleMaximize" />
        <button type="button" class="control close" aria-label="关闭" @click="closeWindow" />
      </div>
    </header>
  </main>
</template>
