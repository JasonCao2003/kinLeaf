<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { memberApi, type FamilySummary, type PersonFields } from '../api'

const props = defineProps<{ family: FamilySummary }>()
const api = memberApi(props.family.id)
const registrations = ref<Awaited<ReturnType<typeof api.registrations>>>([])
const people = ref<Awaited<ReturnType<typeof api.people>>>([])
const member = ref<Awaited<ReturnType<typeof api.read>> | null>(null)
const selectedId = ref(props.family.rootEntryId)
const selected = computed(() => registrations.value.find(entry => entry.id === selectedId.value))
const relatives = computed(() => people.value.filter(person => person.id !== selected.value?.personId))
const busy = ref(false)
const error = ref('')
const parentId = ref('')
const rank = ref<number | ''>('')
const branchNotes = ref('')
const childName = ref('')
const existingPerson = ref('')
const relativeName = ref('')
const spouseId = ref('')

function newPerson(name: string): PersonFields {
  return { name, spectrumName: null, generationName: null, birthText: null, deathText: null, notes: null }
}

function entryLabel(entry: { id: string; name: string }) {
  return entry.id === props.family.rootEntryId ? `${entry.name}（始祖）` : `${entry.name} · ${entry.id.slice(0, 8)}`
}

async function readSelection() {
  member.value = null
  const entry = selected.value
  if (!entry) return
  parentId.value = entry.parentEntryId ?? ''
  rank.value = entry.fields.rank ?? ''
  branchNotes.value = entry.fields.branchNotes ?? ''
  spouseId.value = ''
  member.value = await api.read(entry.personId)
}

async function refreshCatalog() {
  const [entries, profiles] = await Promise.all([api.registrations(), api.people()])
  registrations.value = entries
  people.value = profiles
}

async function refreshSelection() {
  await refreshCatalog()
  await readSelection()
}

async function run(action: () => Promise<void>) {
  if (busy.value) return
  busy.value = true
  error.value = ''
  try {
    await action()
  } catch (failure) {
    const message = (failure as { message?: string } | null)?.message
    error.value = message ? `操作未完成（${message}）` : '操作未完成，请重试。'
  } finally {
    busy.value = false
  }
}

function saveRegistration() {
  return run(async () => {
    await api.updateEntry(selectedId.value, parentId.value || null, {
      rank: rank.value === '' ? null : rank.value,
      branchNotes: branchNotes.value || null,
    })
    await refreshCatalog()
  })
}

function deleteRegistration() {
  return run(async () => {
    if (!window.confirm('删除这条族员登记？个人档案和亲属关系会保留。')) return
    await api.deleteEntry(selectedId.value)
    selectedId.value = props.family.rootEntryId
    await refreshSelection()
  })
}

function saveProfile() {
  const profile = member.value
  const entry = selected.value
  if (!profile || !entry) return
  return run(async () => {
    await api.updatePerson(entry.personId, profile.fields)
    await refreshCatalog()
  })
}

function saveParents() {
  const profile = member.value
  const entry = selected.value
  if (!profile || !entry) return
  return run(async () => {
    await api.setParents(entry.personId, profile.fatherId || null, profile.motherId || null)
  })
}

function changeSpouse(id: string, linked: boolean) {
  const entry = selected.value
  const profile = member.value
  if (!entry || !profile || !id) return
  return run(async () => {
    await api.setSpouse(entry.personId, id, linked)
    profile.spouseIds = (await api.read(entry.personId)).spouseIds
    spouseId.value = ''
  })
}

function addChild() {
  return run(async () => {
    const person = existingPerson.value
      ? { existing: existingPerson.value }
      : { new: newPerson(childName.value) }
    selectedId.value = await api.addChild(selectedId.value, person, { rank: null, branchNotes: null })
    childName.value = ''
    existingPerson.value = ''
    await refreshSelection()
  })
}

function addRelative() {
  return run(async () => {
    await api.createPerson(newPerson(relativeName.value))
    relativeName.value = ''
    people.value = await api.people()
  })
}

onMounted(() => run(refreshSelection))
</script>

<template>
  <section aria-labelledby="member-editor">
    <h2 id="member-editor">族员管理</h2>
    <p v-if="error" role="alert">{{ error }}</p>
    <fieldset :disabled="busy">
      <legend>选择族员登记</legend>
      <select v-model="selectedId" aria-label="族员登记" @change="run(readSelection)">
        <option v-for="entry in registrations" :key="entry.id" :value="entry.id">
          {{ entryLabel(entry) }}
        </option>
      </select>
    </fieldset>

    <fieldset v-if="selected && member" :disabled="busy">
      <legend>登记信息</legend>
      <form @submit.prevent="saveRegistration">
        <label v-if="selected.parentEntryId">
          谱系上级
          <select v-model="parentId" required>
            <option v-for="entry in registrations.filter(item => item.id !== selectedId)" :key="entry.id" :value="entry.id">
              {{ entryLabel(entry) }}
            </option>
          </select>
        </label>
        <label>排行 <input v-model.number="rank" type="number" min="1" step="1"></label>
        <label>支系备注 <textarea v-model="branchNotes" /></label>
        <button>保存登记</button>
        <button v-if="selected.parentEntryId" type="button" @click="deleteRegistration">删除登记</button>
      </form>
    </fieldset>

    <fieldset v-if="selected && member" :disabled="busy">
      <legend>个人档案</legend>
      <p v-if="member.registrationCount > 1" role="status">
        这份个人档案用于 {{ member.registrationCount }} 条登记，保存会同步更新所有登记中的个人资料。
      </p>
      <form @submit.prevent="saveProfile">
        <label>姓名 <input v-model="member.fields.name" required></label>
        <label>谱名 <input v-model="member.fields.spectrumName"></label>
        <label>字辈 <input v-model="member.fields.generationName"></label>
        <label>出生 <input v-model="member.fields.birthText"></label>
        <label>逝世 <input v-model="member.fields.deathText"></label>
        <label>备注 <textarea v-model="member.fields.notes" /></label>
        <button>保存档案</button>
      </form>
    </fieldset>

    <fieldset v-if="selected && member" :disabled="busy">
      <legend>亲属关系</legend>
      <form @submit.prevent="saveParents">
        <label>生父
          <select v-model="member.fatherId">
            <option :value="null">未知</option>
            <option v-for="person in relatives" :key="person.id" :value="person.id">{{ person.name }} · {{ person.id.slice(0, 8) }}</option>
          </select>
        </label>
        <label>生母
          <select v-model="member.motherId">
            <option :value="null">未知</option>
            <option v-for="person in relatives" :key="person.id" :value="person.id">{{ person.name }} · {{ person.id.slice(0, 8) }}</option>
          </select>
        </label>
        <button>保存父母关系</button>
      </form>
      <ul v-if="member.spouseIds.length">
        <li v-for="id in member.spouseIds" :key="id">
          {{ people.find(person => person.id === id)?.name }}
          <button type="button" @click="changeSpouse(id, false)">解除配偶关联</button>
        </li>
      </ul>
      <form @submit.prevent="changeSpouse(spouseId, true)">
        <label>配偶
          <select v-model="spouseId" required>
            <option value="" disabled>选择档案</option>
            <option v-for="person in relatives.filter(person => !member?.spouseIds.includes(person.id))" :key="person.id" :value="person.id">
              {{ person.name }} · {{ person.id.slice(0, 8) }}
            </option>
          </select>
        </label>
        <button>添加配偶</button>
      </form>
      <form @submit.prevent="addRelative">
        <label>新建亲属档案 <input v-model="relativeName" required></label>
        <button>创建档案</button>
      </form>
    </fieldset>

    <fieldset v-if="selected && member" :disabled="busy">
      <legend>登记后代</legend>
      <form @submit.prevent="addChild">
        <label>档案来源
          <select v-model="existingPerson">
            <option value="">新建个人档案</option>
            <option v-for="person in people" :key="person.id" :value="person.id">
              {{ person.name }} · {{ person.id.slice(0, 8) }}
            </option>
          </select>
        </label>
        <label v-if="!existingPerson">姓名 <input v-model="childName" required></label>
        <button>{{ existingPerson ? '登记现有档案（兼祧）' : '新增后代' }}</button>
      </form>
    </fieldset>
  </section>
</template>
