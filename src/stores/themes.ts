import { defineStore } from 'pinia'
import { invoke } from '@tauri-apps/api/core'
import type { AppSettings, ThemePreset, WebAppChromeConfig } from '@/types/webapp'
import { builtInThemePresets, defaultAppSettings, defaultChromeConfig } from './webapps'

const PRESETS_KEY = 'bandoo-webforge.theme-presets'
const SETTINGS_KEY = 'bandoo-webforge.app-settings'

function isTauriRuntime() {
  return Boolean('__TAURI_INTERNALS__' in window)
}

function createId() {
  return crypto.randomUUID ? crypto.randomUUID() : `theme-${Date.now()}`
}

function normalizeChrome(config?: Partial<WebAppChromeConfig>): WebAppChromeConfig {
  const merged = {
    ...defaultChromeConfig(),
    ...(config ?? {}),
  }
  return {
    ...merged,
    controlsPosition: merged.controlsPosition === 'left' ? 'left' : 'right',
    controlsStyle: ['windows', 'traffic-light', 'minimal'].includes(merged.controlsStyle)
      ? merged.controlsStyle
      : 'windows',
  }
}

function readLocalPresets(): ThemePreset[] {
  const raw = localStorage.getItem(PRESETS_KEY)
  return raw ? (JSON.parse(raw) as ThemePreset[]) : builtInThemePresets()
}

function writeLocalPresets(items: ThemePreset[]) {
  localStorage.setItem(PRESETS_KEY, JSON.stringify(items))
}

function readLocalSettings(): AppSettings {
  const raw = localStorage.getItem(SETTINGS_KEY)
  return raw ? (JSON.parse(raw) as AppSettings) : defaultAppSettings()
}

function writeLocalSettings(settings: AppSettings) {
  localStorage.setItem(SETTINGS_KEY, JSON.stringify(settings))
}

export function mergeChromeConfig(
  appConfig?: Partial<WebAppChromeConfig>,
  settings = defaultAppSettings(),
  presets = builtInThemePresets(),
) {
  const presetId = appConfig?.themePresetId ?? settings.defaultThemePresetId
  const preset = presets.find((item) => item.id === presetId)
  return normalizeChrome({
    ...settings.defaultChromeConfig,
    ...(preset?.chromeConfig ?? {}),
    ...(appConfig ?? {}),
  })
}

export const useThemeStore = defineStore('themes', {
  state: () => ({
    presets: [] as ThemePreset[],
    settings: defaultAppSettings(),
    loading: false,
  }),
  actions: {
    async load() {
      this.loading = true
      try {
        if (isTauriRuntime()) {
          const [presets, settings] = await Promise.all([
            invoke<ThemePreset[]>('list_theme_presets'),
            invoke<AppSettings>('app_settings'),
          ])
          this.presets = presets.map((preset) => ({
            ...preset,
            chromeConfig: normalizeChrome(preset.chromeConfig),
          }))
          this.settings = {
            ...settings,
            defaultChromeConfig: normalizeChrome(settings.defaultChromeConfig),
          }
        } else {
          this.presets = readLocalPresets().map((preset) => ({
            ...preset,
            chromeConfig: normalizeChrome(preset.chromeConfig),
          }))
          this.settings = {
            ...readLocalSettings(),
            defaultChromeConfig: normalizeChrome(readLocalSettings().defaultChromeConfig),
          }
        }
      } finally {
        this.loading = false
      }
    },
    async savePreset(preset: ThemePreset) {
      const item = {
        ...preset,
        chromeConfig: normalizeChrome(preset.chromeConfig),
      }
      if (isTauriRuntime()) {
        this.presets = await invoke<ThemePreset[]>('upsert_theme_preset', { preset: item })
      } else {
        this.presets = [item, ...this.presets.filter((candidate) => candidate.id !== item.id)]
        writeLocalPresets(this.presets)
      }
    },
    async createPreset(name: string, chromeConfig: WebAppChromeConfig) {
      await this.savePreset({
        id: createId(),
        name,
        chromeConfig: normalizeChrome(chromeConfig),
        createdAt: Date.now(),
      })
    },
    async removePreset(id: string) {
      if (isTauriRuntime()) {
        this.presets = await invoke<ThemePreset[]>('delete_theme_preset', { id })
      } else {
        this.presets = this.presets.filter((item) => item.id !== id)
        writeLocalPresets(this.presets)
      }
    },
    async saveSettings(settings: AppSettings) {
      const normalized = {
        ...settings,
        defaultChromeConfig: normalizeChrome(settings.defaultChromeConfig),
      }
      if (isTauriRuntime()) {
        this.settings = await invoke<AppSettings>('save_app_settings', { settings: normalized })
      } else {
        this.settings = normalized
        writeLocalSettings(normalized)
      }
    },
    exportJson() {
      return JSON.stringify({ presets: this.presets, settings: this.settings }, null, 2)
    },
    async importJson(raw: string) {
      const parsed = JSON.parse(raw) as { presets?: ThemePreset[]; settings?: AppSettings }
      if (parsed.settings) {
        await this.saveSettings(parsed.settings)
      }
      for (const preset of parsed.presets ?? []) {
        await this.savePreset({
          ...preset,
          chromeConfig: normalizeChrome(preset.chromeConfig),
        })
      }
    },
  },
})
