// @vitest-environment jsdom
import { flushPromises, mount } from '@vue/test-utils'
import { beforeEach, expect, test, vi } from 'vitest'
import { invoke } from '@tauri-apps/api/core'
import App from './App.vue'

vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }))

const family = { id: 'family', name: '家谱', rootEntryId: 'root' }
const other = { id: 'other', name: '另一份谱', rootEntryId: 'other-root' }
const ids = ['root', 'branch', 'leaf']
const names = ['始祖', '分支', '目标']
const path = ids.map((entryId, index) => ({
  entryId, name: names[index], personId: entryId, rank: null, hasChildren: index < 2,
}))
const hit = { entry: path[2], generation: 3, path }
const fields = {
  name: '目标', spectrumName: null, generationName: null, birthText: null, deathText: null, notes: null,
}

let search: () => Promise<unknown>
beforeEach(() => {
  search = async () => [hit]
  vi.mocked(invoke).mockImplementation(async (command, args) => {
    if (command === 'list_families') return { families: [family, other], unreadableCount: 0 } as never
    if (command === 'find_people') return [] as never
    if (command === 'list_registrations') {
      const entries = args?.familyId === 'other' ? [{ id: 'other-root', name: '另一位始祖', personId: 'other-root', parentEntryId: null, fields: {} }] :
        path.map((item, index) => ({ id: item.entryId, name: item.name, personId: item.personId, parentEntryId: ids[index - 1] ?? null, fields: {} }))
      return entries as never
    }
    if (command === 'read_member') return { fields, spouseIds: [], registrationCount: 1 } as never
    if (command === 'get_entry') return {
      hit: args?.entryId === 'leaf' ? hit : { entry: { name: args?.entryId === 'other-root' ? '另一位始祖' : '始祖' }, generation: 1, path: [] },
      person: fields, spouses: [], otherRegistrations: [],
    } as never
    if (command === 'search_entries') return await search() as never
    throw new Error(command)
  })
})

test('search opens the complete lineage path and selects the result', async () => {
  const wrapper = mount(App)
  await flushPromises()
  await wrapper.get('select').setValue(family.id)
  await flushPromises()
  expect(wrapper.find('[data-entry-id="leaf"]').exists()).toBe(false)
  await wrapper.get('input[type="search"]').setValue('目标')
  await wrapper.get('form[role="search"]').trigger('submit')
  await flushPromises()
  await wrapper.get('[data-search-hit="leaf"]').trigger('click')
  await flushPromises()
  expect(wrapper.get('[data-entry-id="leaf"]').attributes('aria-current')).toBe('true')
  expect(wrapper.get('[aria-label="世系详情"]').text()).toContain('第 3 世')
  wrapper.unmount()
})

test('switching families discards a delayed search from the previous family', async () => {
  let finishSearch!: (value: unknown) => void
  search = () => new Promise(resolve => { finishSearch = resolve })
  const wrapper = mount(App)
  await flushPromises()
  await wrapper.get('select').setValue(family.id)
  await flushPromises()
  await wrapper.get('input[type="search"]').setValue('目标')
  await wrapper.get('form[role="search"]').trigger('submit')
  await wrapper.get('select').setValue(other.id)
  await flushPromises()
  finishSearch([hit])
  await flushPromises()
  expect(wrapper.find('[data-search-hit="leaf"]').exists()).toBe(false)
  expect(wrapper.get('[data-entry-id="other-root"]').text()).toContain('另一位始祖')
  wrapper.unmount()
})
