<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { memberApi, type FamilySummary, type PersonFields, type SearchHit } from '../api'
import LineageTree from './LineageTree.vue'
import MemberSearch from './MemberSearch.vue'

const props = defineProps<{ family: FamilySummary }>()
const api = memberApi(props.family.id)
const registrations = ref<Awaited<ReturnType<typeof api.registrations>>>([])
const people = ref<Awaited<ReturnType<typeof api.people>>>([])
const member = ref<Awaited<ReturnType<typeof api.read>> | null>(null)
const detail = ref<Awaited<ReturnType<typeof api.detail>> | null>(null)
const expandedIds = ref(new Set<string>())
const searchRevision = ref(0)
const selectedId = ref(props.family.rootEntryId)
const selected = computed(() => registrations.value.find(entry => entry.id === selectedId.value))
const relatives = computed(() => people.value.filter(person => person.id !== selected.value?.personId))
const activePanel = ref<'profile' | 'registration' | 'relatives'>('profile')
const addingChild = ref(false)
const panels = [
  { id: 'profile', label: '个人档案' },
  { id: 'registration', label: '支系登记' },
  { id: 'relatives', label: '亲属关系' },
] as const
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
  detail.value = null
  const entry = selected.value
  if (!entry) return
  parentId.value = entry.parentEntryId ?? ''
  rank.value = entry.fields.rank ?? ''
  branchNotes.value = entry.fields.branchNotes ?? ''
  spouseId.value = ''
  childName.value = ''
  existingPerson.value = ''
  addingChild.value = false
  const [profile, lineage] = await Promise.all([api.read(entry.personId), api.detail(entry.id)])
  member.value = profile
  detail.value = lineage
  expandSelectionPath()
}

async function refreshCatalog() {
  const [entries, profiles] = await Promise.all([api.registrations(), api.people()])
  registrations.value = entries
  people.value = profiles
  searchRevision.value++
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

function toggleBranch(id: string) {
  if (expandedIds.value.has(id)) expandedIds.value.delete(id)
  else expandedIds.value.add(id)
}

function expandSelectionPath() {
  for (const ancestor of detail.value?.hit.path.slice(0, -1) ?? []) {
    expandedIds.value.add(ancestor.entryId)
  }
}

function selectEntry(id: string) {
  return run(async () => {
    if (id === selectedId.value && detail.value) {
      expandSelectionPath()
      return
    }
    selectedId.value = id
    await readSelection()
  })
}

function locate(hit: SearchHit) {
  return selectEntry(hit.entry.entryId)
}

async function refreshDetail() {
  detail.value = await api.detail(selectedId.value)
}

function saveRegistration() {
  return run(async () => {
    await api.updateEntry(selectedId.value, parentId.value || null, {
      rank: rank.value === '' ? null : rank.value,
      branchNotes: branchNotes.value || null,
    })
    await refreshCatalog()
    await refreshDetail()
    expandSelectionPath()
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
    await refreshDetail()
  })
}

function saveParents() {
  const profile = member.value
  const entry = selected.value
  if (!profile || !entry) return
  return run(async () => {
    await api.setParents(entry.personId, profile.fatherId || null, profile.motherId || null)
    await refreshDetail()
  })
}

function changeSpouse(id: string, linked: boolean) {
  const entry = selected.value
  const profile = member.value
  if (!entry || !profile || !id) return
  return run(async () => {
    await api.setSpouse(entry.personId, id, linked)
    await refreshDetail()
    profile.spouseIds = detail.value?.spouses.map(person => person.id) ?? []
    spouseId.value = ''
  })
}

function addChild() {
  return run(async () => {
    const person = existingPerson.value
      ? { existing: existingPerson.value }
      : { new: newPerson(childName.value) }
    const parent = selectedId.value
    selectedId.value = await api.addChild(parent, person, { rank: null, branchNotes: null })
    expandedIds.value.add(parent)
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
  <section class="genealogy" aria-label="族谱工作区">
    <aside class="lineage-sidebar" aria-labelledby="lineage-title">
      <div class="sidebar-heading">
        <h2 id="lineage-title">{{ family.name }}</h2>
        <p>{{ registrations.length }} 条族员登记</p>
      </div>
      <MemberSearch :key="searchRevision" :search="api.search" :disabled="busy" @locate="locate" />
      <fieldset class="tree-fieldset" :disabled="busy">
        <legend>世系</legend>
        <LineageTree
          :entries="registrations"
          :root-id="family.rootEntryId"
          :selected-id="selectedId"
          :expanded-ids="expandedIds"
          @select="selectEntry"
          @toggle="toggleBranch"
        />
      </fieldset>
    </aside>
    <div class="archive">
      <p v-if="error" class="notice notice-error" role="alert">{{ error }}</p>
      <p v-if="busy && !detail" class="notice" role="status">正在读取族员资料…</p>
      <header v-if="detail" class="archive-heading" aria-label="世系详情">
        <p class="ancestor-path">{{ detail.hit.path.map(entry => entry.name).join(' / ') || '始祖' }}</p>
        <div class="archive-title">
          <h2>{{ detail.hit.entry.name }} <span class="generation">第 {{ detail.hit.generation }} 世</span></h2>
          <button
            class="ink-button" type="button" :disabled="busy" aria-controls="add-child-panel"
            :aria-expanded="addingChild" @click="addingChild = !addingChild"
          >{{ addingChild ? '收起登记' : '登记后代' }}</button>
        </div>
        <ul v-if="detail.otherRegistrations.length" class="linked-entries" aria-label="其他支系登记">
          <li v-for="hit in detail.otherRegistrations" :key="hit.entry.entryId">
            <button type="button" :disabled="busy" @click="locate(hit)">{{ hit.path.map(entry => entry.name).join(' › ') }}</button>
          </li>
        </ul>
      </header>
      <fieldset v-if="selected && member" v-show="addingChild" id="add-child-panel" class="archive-panel" :disabled="busy">
        <legend>登记后代</legend>
        <form class="paper-form" @submit.prevent="addChild">
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
      <div v-if="selected && member" class="archive-sections" role="group" aria-label="档案分区">
        <button
          v-for="panel in panels" :key="panel.id" type="button" :aria-pressed="activePanel === panel.id"
          :aria-controls="panel.id + '-panel'" @click="activePanel = panel.id"
        >{{ panel.label }}</button>
      </div>

      <fieldset v-if="selected && member" v-show="activePanel === 'registration'" id="registration-panel" class="archive-panel" :disabled="busy">
        <legend>登记信息</legend>
        <form class="paper-form" @submit.prevent="saveRegistration">
          <label v-if="selected.parentEntryId">
            谱系上级
            <select v-model="parentId" required>
              <option v-for="entry in registrations.filter(item => item.id !== selectedId)" :key="entry.id" :value="entry.id">
                {{ entryLabel(entry) }}
              </option>
            </select>
          </label>
          <label>排行 <input v-model.number="rank" type="number" min="1" step="1"></label>
          <label class="full-field">支系备注 <textarea v-model="branchNotes" /></label>
          <button class="ink-button">保存登记</button>
          <button v-if="selected.parentEntryId" class="danger-button" type="button" @click="deleteRegistration">删除登记</button>
        </form>
      </fieldset>

      <fieldset v-if="selected && member" v-show="activePanel === 'profile'" id="profile-panel" class="archive-panel" :disabled="busy">
        <legend>个人档案</legend>
        <p v-if="member.registrationCount > 1" role="status">
          这份个人档案用于 {{ member.registrationCount }} 条登记，保存会同步更新所有登记中的个人资料。
        </p>
        <form class="paper-form" @submit.prevent="saveProfile">
          <label>姓名 <input v-model="member.fields.name" required></label>
          <label>谱名 <input id="profile-alias" v-model="member.fields.spectrumName"></label>
          <label>字辈 <input v-model="member.fields.generationName"></label>
          <label>出生 <input v-model="member.fields.birthText"></label>
          <label>逝世 <input v-model="member.fields.deathText"></label>
          <label class="full-field">备注 <textarea v-model="member.fields.notes" /></label>
          <button class="ink-button">保存档案</button>
        </form>
      </fieldset>

      <fieldset v-if="selected && member" v-show="activePanel === 'relatives'" id="relatives-panel" class="archive-panel" :disabled="busy">
        <legend>亲属关系</legend>
        <form class="paper-form" @submit.prevent="saveParents">
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
          <button class="ink-button">保存父母关系</button>
        </form>
        <ul v-if="member.spouseIds.length">
          <li v-for="id in member.spouseIds" :key="id">
            {{ people.find(person => person.id === id)?.name }}
            <button type="button" @click="changeSpouse(id, false)">解除配偶关联</button>
          </li>
        </ul>
        <form class="paper-form" @submit.prevent="changeSpouse(spouseId, true)">
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
        <form class="paper-form" @submit.prevent="addRelative">
          <label>新建亲属档案 <input v-model="relativeName" required></label>
          <button>创建档案</button>
        </form>
      </fieldset>


    </div>
  </section>
</template>
