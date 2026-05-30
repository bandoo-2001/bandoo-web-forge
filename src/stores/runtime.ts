import { defineStore } from 'pinia'
import { invoke } from '@tauri-apps/api/core'

export interface RuntimeInfo {
  os: string
  family: string
  arch: string
  linuxPrimary: boolean
  macosSupported: boolean
  desktopIntegrationSupported: boolean
}

function isTauriRuntime() {
  return Boolean('__TAURI_INTERNALS__' in window)
}

export const useRuntimeStore = defineStore('runtime', {
  state: () => ({
    info: {
      os: 'browser',
      family: 'web',
      arch: 'unknown',
      linuxPrimary: false,
      macosSupported: false,
      desktopIntegrationSupported: false,
    } as RuntimeInfo,
  }),
  actions: {
    async load() {
      this.info = isTauriRuntime()
        ? await invoke<RuntimeInfo>('runtime_info')
        : this.info
    },
  },
})
