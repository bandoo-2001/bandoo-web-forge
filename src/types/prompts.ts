export interface PromptTemplate {
  id: string
  name: string
  category: string
  instruction: string
  createdAt: number
  updatedAt?: number
}

export type PromptTemplateDraft = Omit<PromptTemplate, 'id' | 'createdAt' | 'updatedAt'>
