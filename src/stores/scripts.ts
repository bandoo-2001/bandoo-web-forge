import { defineStore } from 'pinia'
import { invoke } from '@tauri-apps/api/core'
import ts from 'typescript'
import type { UserScriptConfig, UserScriptDraft, UserScriptRunResult } from '@/types/scripts'

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

function normalizeScript(item: UserScriptConfig): UserScriptConfig {
  const language = item.language ?? 'javascript'
  const compiledCode =
    language === 'typescript'
      ? ts.transpileModule(item.code, {
          compilerOptions: {
            target: ts.ScriptTarget.ES2020,
            module: ts.ModuleKind.ESNext,
          },
        }).outputText
      : item.compiledCode

  return {
    ...item,
    language,
    compiledCode,
    runAt: item.runAt ?? 'manual',
    matchPatterns: item.matchPatterns ?? [],
    requiredPermissions: item.requiredPermissions ?? [],
  }
}

export const useScriptStore = defineStore('scripts', {
  state: () => ({
    items: [] as UserScriptConfig[],
  }),
  actions: {
    async load() {
      const items = isTauriRuntime()
        ? await invoke<UserScriptConfig[]>('list_user_scripts')
        : readLocal()
      this.items = items.map(normalizeScript)
    },
    async save(item: UserScriptConfig) {
      const normalized = normalizeScript(item)
      if (isTauriRuntime()) {
        this.items = (await invoke<UserScriptConfig[]>('upsert_user_script', { script: normalized })).map(normalizeScript)
      } else {
        this.items = [normalized, ...this.items.filter((candidate) => candidate.id !== normalized.id)]
        writeLocal(this.items)
      }
    },
    async create(draft: UserScriptDraft) {
      await this.save(normalizeScript({ ...draft, id: createId(), createdAt: Date.now() }))
    },
    async remove(id: string) {
      if (isTauriRuntime()) {
        this.items = await invoke<UserScriptConfig[]>('delete_user_script', { id })
      } else {
        this.items = this.items.filter((item) => item.id !== id)
        writeLocal(this.items)
      }
    },
    async execute(id: string) {
      const script = this.items.find((item) => item.id === id)
      if (!script) {
        throw new Error('User script not found')
      }
      if (!isTauriRuntime()) {
        throw new Error('User script execution is only available in the Tauri runtime')
      }
      return await invoke<UserScriptRunResult>('execute_user_script', { script })
    },
  },
})
