import { defineStore } from 'pinia'
import { invoke } from '@tauri-apps/api/core'
import type {
  DesktopIntegrationResult,
  DesktopIntegrationTarget,
  WebApp,
  WebAppDraft,
} from '@/types/webapp'

const STORAGE_KEY = 'bandoo-webforge.webapps'

function isTauriRuntime() {
  return Boolean('__TAURI_INTERNALS__' in window)
}

function createId() {
  return crypto.randomUUID ? crypto.randomUUID() : `webapp-${Date.now()}`
}

function readLocalWebApps(): WebApp[] {
  const raw = localStorage.getItem(STORAGE_KEY)
  return raw ? (JSON.parse(raw) as WebApp[]) : []
}

function writeLocalWebApps(items: WebApp[]) {
  localStorage.setItem(STORAGE_KEY, JSON.stringify(items))
}

function normalizeWebApp(item: WebApp): WebApp {
  return {
    ...item,
    permissions: {
      page: item.permissions.page ?? true,
      clipboard: item.permissions.clipboard ?? false,
      shell: item.permissions.shell ?? false,
      filesystem: item.permissions.filesystem ?? false,
      network: item.permissions.network ?? false,
      notification: item.permissions.notification ?? false,
    },
    scriptConfig: item.scriptConfig ?? {
      injectBridge: true,
      customScriptEnabled: false,
      customScript: '',
    },
  }
}

export const useWebAppStore = defineStore('webapps', {
  state: () => ({
    items: [] as WebApp[],
    loading: false,
  }),
  actions: {
    async load() {
      this.loading = true
      try {
        const items = isTauriRuntime()
          ? await invoke<WebApp[]>('list_webapps')
          : readLocalWebApps()
        this.items = items.map(normalizeWebApp)
      } finally {
        this.loading = false
      }
    },
    async save(webapp: WebApp) {
      const item = normalizeWebApp(webapp)
      if (isTauriRuntime()) {
        this.items = (await invoke<WebApp[]>('upsert_webapp', { webapp: item })).map(normalizeWebApp)
      } else {
        this.items = [item, ...this.items.filter((candidate) => candidate.id !== item.id)]
        writeLocalWebApps(this.items)
      }
    },
    async create(draft: WebAppDraft) {
      const item: WebApp = {
        ...draft,
        id: createId(),
        createdAt: Date.now(),
      }

      await this.save(item)
    },
    async update(id: string, draft: WebAppDraft) {
      const current = this.items.find((item) => item.id === id)
      if (!current) return

      await this.save({
        ...current,
        ...draft,
        windowConfig: { ...draft.windowConfig },
        permissions: { ...draft.permissions },
        updatedAt: Date.now(),
      })
    },
    async remove(id: string) {
      if (isTauriRuntime()) {
        this.items = await invoke<WebApp[]>('delete_webapp', { id })
      } else {
        this.items = this.items.filter((item) => item.id !== id)
        writeLocalWebApps(this.items)
      }
    },
    async launch(id: string) {
      if (isTauriRuntime()) {
        await invoke('launch_webapp', { id })
        return
      }

      const app = this.items.find((item) => item.id === id)
      if (app) {
        window.open(app.url, '_blank', 'noopener,noreferrer')
      }
    },
    exportJson() {
      return JSON.stringify(this.items.map(normalizeWebApp), null, 2)
    },
    async importJson(raw: string) {
      const parsed = JSON.parse(raw) as WebApp[]
      if (!Array.isArray(parsed)) {
        throw new Error('Imported WebApp JSON must be an array')
      }

      const imported = parsed.map(normalizeWebApp)
      for (const item of imported) {
        await this.save(item)
      }
    },
    async installIntegration(id: string, target: DesktopIntegrationTarget) {
      if (!isTauriRuntime()) {
        throw new Error('Desktop integration is only available in the Tauri runtime')
      }
      return await invoke<DesktopIntegrationResult>('install_desktop_entry', { id, target })
    },
    async removeIntegration(id: string, target: DesktopIntegrationTarget) {
      if (!isTauriRuntime()) {
        throw new Error('Desktop integration is only available in the Tauri runtime')
      }
      return await invoke<DesktopIntegrationResult>('remove_desktop_entry', { id, target })
    },
  },
})
