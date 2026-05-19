import { defineStore } from 'pinia'
import { invoke } from '@tauri-apps/api/core'
import type { PromptTemplate, PromptTemplateDraft } from '@/types/prompts'

const STORAGE_KEY = 'bandoo-webforge.prompt-templates'

function isTauriRuntime() {
  return Boolean('__TAURI_INTERNALS__' in window)
}

function createId() {
  return crypto.randomUUID ? crypto.randomUUID() : `prompt-${Date.now()}`
}

function readLocal(): PromptTemplate[] {
  const raw = localStorage.getItem(STORAGE_KEY)
  return raw ? (JSON.parse(raw) as PromptTemplate[]) : []
}

function writeLocal(items: PromptTemplate[]) {
  localStorage.setItem(STORAGE_KEY, JSON.stringify(items))
}

export const usePromptStore = defineStore('prompts', {
  state: () => ({
    items: [] as PromptTemplate[],
  }),
  actions: {
    async load() {
      this.items = isTauriRuntime()
        ? await invoke<PromptTemplate[]>('list_prompt_templates')
        : readLocal()
    },
    async save(item: PromptTemplate) {
      if (isTauriRuntime()) {
        this.items = await invoke<PromptTemplate[]>('upsert_prompt_template', { template: item })
      } else {
        this.items = [item, ...this.items.filter((candidate) => candidate.id !== item.id)]
        writeLocal(this.items)
      }
    },
    async create(draft: PromptTemplateDraft) {
      await this.save({ ...draft, id: createId(), createdAt: Date.now() })
    },
    async remove(id: string) {
      if (isTauriRuntime()) {
        this.items = await invoke<PromptTemplate[]>('delete_prompt_template', { id })
      } else {
        this.items = this.items.filter((item) => item.id !== id)
        writeLocal(this.items)
      }
    },
  },
})
