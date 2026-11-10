<script setup lang="ts">
import { onBeforeUnmount, ref, watch } from 'vue'
import type { SearchHit } from '../api'

const props = defineProps<{
  search: (query: string) => Promise<SearchHit[]>
  disabled: boolean
}>()
const emit = defineEmits<{ locate: [hit: SearchHit] }>()
const query = ref('')
const hits = ref<SearchHit[]>([])
const state = ref<'idle' | 'loading' | 'done' | 'error'>('idle')
let request = 0

watch(query, () => {
  request++
  hits.value = []
  state.value = 'idle'
}, { flush: 'sync' })

onBeforeUnmount(() => { request++ })

async function submit() {
  if (!query.value.trim()) return
  const current = ++request
  hits.value = []
  state.value = 'loading'
  try {
    const results = await props.search(query.value)
    if (current !== request) return
    hits.value = results
    state.value = 'done'
  } catch {
    if (current === request) state.value = 'error'
  }
}
</script>

<template>
  <form class="member-search" role="search" @submit.prevent="submit">
    <label>按姓名或谱名搜索 <input v-model="query" type="search" :disabled="disabled"></label>
    <button :disabled="disabled || !query.trim()">搜索</button>
  </form>
  <p v-if="state === 'loading'" role="status">正在搜索…</p>
  <p v-if="state === 'error'" role="alert">搜索失败，请重试。</p>
  <p v-if="state === 'done' && !hits.length" role="status">未找到匹配的族员登记。</p>
  <ul v-if="hits.length" class="search-results" aria-label="搜索结果">
    <li v-for="hit in hits" :key="hit.entry.entryId">
      <button
        type="button"
        :disabled="disabled"
        :data-search-hit="hit.entry.entryId"
        @click="emit('locate', hit)"
      >{{ hit.path.map(entry => entry.name).join(' › ') }} · 第 {{ hit.generation }} 世</button>
    </li>
  </ul>
</template>
