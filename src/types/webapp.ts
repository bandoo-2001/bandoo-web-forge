export interface WebAppWindowConfig {
  width: number
  height: number
  maximized?: boolean
  transparent: boolean
  decorations: boolean
  stableFallback: boolean
}

export interface WebAppWindowState {
  x: number
  y: number
  width: number
  height: number
  maximized: boolean
}

export interface WebAppPermissions {
  page: boolean
  clipboard: boolean
  shell: boolean
  filesystem: boolean
  network: boolean
  notification: boolean
}

export interface WebAppScriptConfig {
  injectBridge: boolean
  customScriptEnabled: boolean
  customScript: string
}

export interface WebAppChromeConfig {
  enabled: boolean
  titlebarHeight: number
  backgroundColor: string
  foregroundColor: string
  opacity: number
  cornerRadius: number
  shadow: boolean
  controlsPosition: 'left' | 'right'
  controlsStyle: 'windows' | 'traffic-light' | 'minimal'
  showTitle: boolean
  showIcon: boolean
  showUrl: boolean
  themePresetId?: string
}

export interface ThemePreset {
  id: string
  name: string
  chromeConfig: WebAppChromeConfig
  createdAt: number
  updatedAt?: number
}

export interface AppSettings {
  defaultThemePresetId?: string
  defaultChromeConfig: WebAppChromeConfig
}

export interface WebApp {
  id: string
  name: string
  icon?: string
  url: string
  userAgent?: string
  startOnBoot?: boolean
  tray?: boolean
  windowConfig: WebAppWindowConfig
  lastWindowState?: WebAppWindowState
  permissions: WebAppPermissions
  scriptConfig: WebAppScriptConfig
  chromeConfig: WebAppChromeConfig
  createdAt: number
  updatedAt?: number
}

export type WebAppDraft = Omit<WebApp, 'id' | 'createdAt' | 'updatedAt' | 'lastWindowState'>

export type DesktopIntegrationTarget = 'applications' | 'desktop' | 'autostart'

export interface DesktopIntegrationResult {
  path: string
  installed: boolean
}

export interface DesktopIntegrationStatus {
  target: DesktopIntegrationTarget
  path: string
  installed: boolean
}
