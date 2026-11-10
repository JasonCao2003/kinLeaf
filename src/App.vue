<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { createFamily, listFamilies, type FamilySummary } from './api'

const families = ref<FamilySummary[]>([])
const selectedId = ref('')
const name = ref('')
const founderName = ref('')
const pending = ref(false)
const error = ref('')
const unreadableCount = ref(0)
const selected = computed(() => families.value.find(family => family.id === selectedId.value))

onMounted(async () => {
  pending.value = true
  try {
    const catalog = await listFamilies()
    families.value = catalog.families
    unreadableCount.value = catalog.unreadableCount
  } catch {
    error.value = '无法读取本地族谱。'
  } finally {
    pending.value = false
  }
})

async function submit() {
  if (pending.value || !name.value.trim() || !founderName.value.trim()) return
  pending.value = true
  error.value = ''
  try {
    const family = await createFamily(name.value, founderName.value)
    families.value = [...families.value, family]
    selectedId.value = family.id
    name.value = ''
    founderName.value = ''
  } catch {
    error.value = '建谱失败，请重试。'
  } finally {
    pending.value = false
  }
}
</script>

<template>
  <main>
    <h1>枝间 Kinleaf</h1>
    <label>
      打开族谱
      <select v-model="selectedId" :disabled="pending || families.length === 0">
        <option disabled value="">请选择族谱</option>
        <option v-for="family in families" :key="family.id" :value="family.id">{{ family.name }}</option>
      </select>
    </label>
    <p v-if="selected">已打开 {{ selected.name }}</p>
    <p v-if="unreadableCount" role="status">{{ unreadableCount }} 份族谱文件无法读取。</p>
    <h2>新建族谱</h2>
    <form @submit.prevent="submit">
      <label>谱名 <input v-model="name" required></label>
      <label>始祖姓名 <input v-model="founderName" required></label>
      <button :disabled="pending || !name.trim() || !founderName.trim()">创建族谱</button>
    </form>
    <p v-if="error" role="alert">{{ error }}</p>
  </main>
</template>
