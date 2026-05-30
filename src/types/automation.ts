export interface AutomationTrigger {
  kind: string
  shortcut?: string
  url?: string
  menuId?: string
}

export interface AutomationCondition {
  kind: string
  value?: string
  negate?: boolean
}

export interface AutomationAction {
  kind: string
  selector?: string
  text?: string
  script?: string
  value?: string
  timeoutMs?: number
  continueOnError?: boolean
}

export interface AutomationConfig {
  id: string
  webAppId: string
  name: string
  enabled: boolean
  trigger: AutomationTrigger
  conditions: AutomationCondition[]
  actions: AutomationAction[]
  createdAt: number
  updatedAt?: number
}

export type AutomationDraft = Omit<AutomationConfig, 'id' | 'createdAt' | 'updatedAt'>

export interface AutomationStepResult {
  index: number
  actionKind: string
  status: string
  message: string
  durationMs?: number
}

export interface AutomationRunResult {
  runId: string
  automationId: string
  webAppId: string
  dispatched: boolean
  message: string
  steps: AutomationStepResult[]
  startedAt: number
  finishedAt?: number
  durationMs?: number
  error?: string
}

export interface AutomationRunLog {
  id: string
  sourceId: string
  webAppId: string
  kind: string
  status: string
  message: string
  steps: AutomationStepResult[]
  startedAt: number
  finishedAt?: number
  durationMs?: number
  error?: string
}
