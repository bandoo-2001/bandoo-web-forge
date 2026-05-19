import { defineStore } from 'pinia'
import { invoke } from '@tauri-apps/api/core'
import type { AutomationConfig, AutomationDraft } from '@/types/automation'

const STORAGE_KEY = 'bandoo-webforge.automations'

function isTauriRuntime() {
  return Boolean('__TAURI_INTERNALS__' in window)
}

function createId() {
  return crypto.randomUUID ? crypto.randomUUID() : `automation-${Date.now()}`
}

function readLocal(): AutomationConfig[] {
  const raw = localStorage.getItem(STORAGE_KEY)
  return raw ? (JSON.parse(raw) as AutomationConfig[]) : []
}

function writeLocal(items: AutomationConfig[]) {
  localStorage.setItem(STORAGE_KEY, JSON.stringify(items))
}

export const useAutomationStore = defineStore('automations', {
  state: () => ({
    items: [] as AutomationConfig[],
    loading: false,
  }),
  actions: {
    async load() {
      this.loading = true
      try {
        this.items = isTauriRuntime()
          ? await invoke<AutomationConfig[]>('list_automations')
          : readLocal()
      } finally {
        this.loading = false
      }
    },
    async save(item: AutomationConfig) {
      if (isTauriRuntime()) {
        this.items = await invoke<AutomationConfig[]>('upsert_automation', { automation: item })
      } else {
        this.items = [item, ...this.items.filter((candidate) => candidate.id !== item.id)]
        writeLocal(this.items)
      }
    },
    async create(draft: AutomationDraft) {
      await this.save({ ...draft, id: createId(), createdAt: Date.now() })
    },
    async remove(id: string) {
      if (isTauriRuntime()) {
        this.items = await invoke<AutomationConfig[]>('delete_automation', { id })
      } else {
        this.items = this.items.filter((item) => item.id !== id)
        writeLocal(this.items)
      }
    },
  },
})
