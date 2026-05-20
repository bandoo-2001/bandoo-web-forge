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
}

export interface AutomationRunResult {
  automationId: string
  webAppId: string
  dispatched: boolean
  message: string
  steps: AutomationStepResult[]
}
