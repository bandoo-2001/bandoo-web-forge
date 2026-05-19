export interface UserScriptConfig {
  id: string
  webAppId: string
  name: string
  enabled: boolean
  code: string
  requiredPermissions: string[]
  createdAt: number
  updatedAt?: number
}

export type UserScriptDraft = Omit<UserScriptConfig, 'id' | 'createdAt' | 'updatedAt'>
