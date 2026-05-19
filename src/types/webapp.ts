export interface WebAppWindowConfig {
  width: number
  height: number
  maximized?: boolean
}

export interface WebAppPermissions {
  clipboard: boolean
  shell: boolean
  filesystem: boolean
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
  permissions: WebAppPermissions
  createdAt: number
}

export type WebAppDraft = Omit<WebApp, 'id' | 'createdAt'>
