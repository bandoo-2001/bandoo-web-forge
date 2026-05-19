export interface WebAppWindowConfig {
  width: number
  height: number
  maximized?: boolean
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
  createdAt: number
  updatedAt?: number
}

export type WebAppDraft = Omit<WebApp, 'id' | 'createdAt' | 'updatedAt' | 'lastWindowState'>

export type DesktopIntegrationTarget = 'applications' | 'desktop' | 'autostart'

export interface DesktopIntegrationResult {
  path: string
  installed: boolean
}
