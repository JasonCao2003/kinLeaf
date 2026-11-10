<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { createFamily, exportBackup, listFamilies, restoreBackup, type FamilySummary } from './api'
import MemberEditor from './components/MemberEditor.vue'

const families = ref<FamilySummary[]>([])
const selectedId = ref('')
const name = ref('')
const founderName = ref('')
const pending = ref(false)
const error = ref('')
const notice = ref('')
const backupPending = ref(false)
const unreadableCount = ref(0)
const creating = ref(false)
const selected = computed(() => families.value.find(family => family.id === selectedId.value))
const nameCounts = computed(() => {
  const counts = new Map<string, number>()
  for (const family of families.value) {
    counts.set(family.name, (counts.get(family.name) ?? 0) + 1)
  }
  return counts
})

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
  if (pending.value || backupPending.value || !name.value.trim() || !founderName.value.trim()) return
  pending.value = true
  error.value = ''
  try {
    const family = await createFamily(name.value, founderName.value)
    families.value = [...families.value, family]
    selectedId.value = family.id
    creating.value = false
    name.value = ''
    founderName.value = ''
  } catch {
    error.value = '建谱失败，请重试。'
  } finally {
    pending.value = false
  }
}

async function useBackup(action: 'export' | 'restore') {
  if (pending.value || backupPending.value) return
  const family = selected.value
  if (action === 'export' && !family) return
  backupPending.value = true
  error.value = ''
  notice.value = ''
  try {
    if (action === 'export' && family) {
      if (await exportBackup(family.id)) notice.value = '族谱备份已保存。'
    } else {
      const restored = await restoreBackup()
      if (restored) {
        families.value = [...families.value, restored]
        selectedId.value = restored.id
        creating.value = false
        notice.value = '已恢复为独立的族谱副本。'
      }
    }
  } catch {
    error.value = action === 'export'
      ? '无法保存备份。请选择有写入权限的目录和新文件名。'
      : '无法恢复备份。请确认文件完整，且由兼容版本的枝间生成。'
  } finally {
    backupPending.value = false
  }
}
</script>

<template>
  <main class="paper-app">
    <header class="masthead">
      <h1 class="brand"><span class="brand-seal" aria-hidden="true">枝</span> 枝间 <small>Kinleaf</small></h1>
      <div class="family-controls">
        <label>
          打开族谱
          <select v-model="selectedId" :disabled="pending || backupPending || families.length === 0" @change="creating = false">
            <option disabled value="">请选择族谱</option>
            <option v-for="family in families" :key="family.id" :value="family.id">
              {{ family.name }}{{ (nameCounts.get(family.name) ?? 0) > 1 ? ' · ' + family.id.slice(0, 8) : '' }}
            </option>
          </select>
        </label>
        <button type="button" :disabled="pending || backupPending" aria-controls="new-family" :aria-expanded="creating" @click="creating = !creating">
          {{ creating ? '收起建谱' : '新建族谱' }}
        </button>
        <details class="backup-menu">
          <summary>系统工具</summary>
          <div>
            <button type="button" :disabled="pending || backupPending || !selected" @click="useBackup('export')">备份当前族谱</button>
            <button type="button" :disabled="pending || backupPending" @click="useBackup('restore')">恢复为新族谱</button>
          </div>
        </details>
      </div>
    </header>
    <div class="paper-body">
      <p v-if="pending && !creating" class="notice" role="status">正在读取本地族谱…</p>
      <p v-if="unreadableCount" class="notice" role="status">{{ unreadableCount }} 份族谱文件无法读取。</p>
      <p v-if="error" class="notice notice-error" role="alert">{{ error }}</p>
      <p v-if="notice" class="notice" role="status">{{ notice }}</p>
      <section v-if="creating" id="new-family" class="cover">
        <h2>新建族谱</h2>
        <p>从一位始祖开始，逐代记下家人的名字。</p>
        <form class="paper-form" @submit.prevent="submit">
          <label>谱名 <input v-model="name" required></label>
          <label>始祖姓名 <input v-model="founderName" required></label>
          <button class="ink-button" :disabled="pending || backupPending || !name.trim() || !founderName.trim()">创建族谱</button>
        </form>
      </section>
      <MemberEditor v-if="selected" :key="selected.id" :family="selected" />
      <section v-else-if="!creating && !pending" class="cover empty-cover">
        <span class="cover-seal" aria-hidden="true">谱</span>
        <h2>把一家人的故事，慢慢写下去</h2>
        <p>{{ families.length ? '从页首打开一份本机族谱，开始查阅世系。' : '新建一份族谱，从始祖开始记下家族世系。' }}</p>
        <button class="ink-button" type="button" :disabled="backupPending" @click="creating = true">新建族谱</button>
      </section>
    </div>
  </main>
</template>
