import { defineStore } from 'pinia'
import { invoke } from '@tauri-apps/api/core'
import type { UserScriptConfig, UserScriptDraft } from '@/types/scripts'

const STORAGE_KEY = 'bandoo-webforge.user-scripts'

function isTauriRuntime() {
  return Boolean('__TAURI_INTERNALS__' in window)
}

function createId() {
  return crypto.randomUUID ? crypto.randomUUID() : `script-${Date.now()}`
}

function readLocal(): UserScriptConfig[] {
  const raw = localStorage.getItem(STORAGE_KEY)
  return raw ? (JSON.parse(raw) as UserScriptConfig[]) : []
}

function writeLocal(items: UserScriptConfig[]) {
  localStorage.setItem(STORAGE_KEY, JSON.stringify(items))
}

export const useScriptStore = defineStore('scripts', {
  state: () => ({
    items: [] as UserScriptConfig[],
  }),
  actions: {
    async load() {
      this.items = isTauriRuntime()
        ? await invoke<UserScriptConfig[]>('list_user_scripts')
        : readLocal()
    },
    async save(item: UserScriptConfig) {
      if (isTauriRuntime()) {
        this.items = await invoke<UserScriptConfig[]>('upsert_user_script', { script: item })
      } else {
        this.items = [item, ...this.items.filter((candidate) => candidate.id !== item.id)]
        writeLocal(this.items)
      }
    },
    async create(draft: UserScriptDraft) {
      await this.save({ ...draft, id: createId(), createdAt: Date.now() })
    },
    async remove(id: string) {
      if (isTauriRuntime()) {
        this.items = await invoke<UserScriptConfig[]>('delete_user_script', { id })
      } else {
        this.items = this.items.filter((item) => item.id !== id)
        writeLocal(this.items)
      }
    },
  },
})
