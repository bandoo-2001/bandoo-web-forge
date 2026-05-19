<script setup lang="ts">
import { onMounted, reactive } from 'vue'
import { storeToRefs } from 'pinia'
import { usePromptStore } from '@/stores/prompts'
import type { PromptTemplateDraft } from '@/types/prompts'

const store = usePromptStore()
const { items } = storeToRefs(store)

const draft = reactive<PromptTemplateDraft>({
  name: '解释',
  category: 'AI 快捷操作',
  instruction: '解释以下内容，并给出简洁结论：\\n\\n{{selection}}',
})

async function submit() {
  await store.create({ ...draft })
}

onMounted(() => {
  void store.load()
})
</script>

<template>
  <section class="page-header">
    <div>
      <p class="eyebrow">后续规划 · AI</p>
      <h1>Prompt 模板</h1>
    </div>
  </section>

  <section class="dashboard-grid">
    <form class="panel create-form" @submit.prevent="submit">
      <h2>AI 模板</h2>
      <label>
        <span>名称</span>
        <input v-model="draft.name" required />
      </label>
      <label>
        <span>分类</span>
        <input v-model="draft.category" required />
      </label>
      <label>
        <span>模板内容</span>
        <textarea v-model="draft.instruction" rows="8" />
      </label>
      <button class="primary-button" type="submit">保存模板</button>
    </form>

    <section class="panel app-list">
      <div class="panel-title">
        <h2>模板列表</h2>
      </div>
      <article v-for="item in items" :key="item.id" class="app-item">
        <div>
          <strong>{{ item.name }}</strong>
          <span>{{ item.category }}</span>
        </div>
        <div class="app-actions">
          <button type="button" class="danger" @click="store.remove(item.id)">删除</button>
        </div>
      </article>
    </section>
  </section>
</template>
