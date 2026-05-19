import { defineStore } from 'pinia'
import { invoke } from '@tauri-apps/api/core'
import type { WebApp, WebAppDraft } from '@/types/webapp'

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

export const useWebAppStore = defineStore('webapps', {
  state: () => ({
    items: [] as WebApp[],
    loading: false,
  }),
  actions: {
    async load() {
      this.loading = true
      try {
        this.items = isTauriRuntime()
          ? await invoke<WebApp[]>('list_webapps')
          : readLocalWebApps()
      } finally {
        this.loading = false
      }
    },
    async create(draft: WebAppDraft) {
      const item: WebApp = {
        ...draft,
        id: createId(),
        createdAt: Date.now(),
      }

      if (isTauriRuntime()) {
        this.items = await invoke<WebApp[]>('upsert_webapp', { webapp: item })
      } else {
        this.items = [item, ...this.items]
        writeLocalWebApps(this.items)
      }
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
  },
})
