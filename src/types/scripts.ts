export interface UserScriptConfig {
  id: string
  webAppId: string
  name: string
  enabled: boolean
  code: string
  language: 'javascript' | 'typescript'
  compiledCode?: string
  runAt: 'manual' | 'page-load' | 'url-change' | 'shortcut'
  matchPatterns: string[]
  requiredPermissions: string[]
  createdAt: number
  updatedAt?: number
}

export type UserScriptDraft = Omit<UserScriptConfig, 'id' | 'createdAt' | 'updatedAt'>

export interface UserScriptRunResult {
  runId: string
  scriptId: string
  webAppId: string
  dispatched: boolean
  message: string
  startedAt: number
  finishedAt?: number
  durationMs?: number
  error?: string
}
