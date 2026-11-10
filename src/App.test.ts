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
    if (command === 'read_member') return { fields: { ...fields }, spouseIds: [], registrationCount: 1 } as never
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
  const leafRow = wrapper.get('[data-entry-id="leaf"]').element.closest('li')!
  expect(leafRow.querySelector('.branch-toggle[aria-hidden="true"]')).not.toBeNull()
  await wrapper.get('[aria-controls="relatives-panel"]').trigger('click')
  const relatives = wrapper.get('#relatives-panel')
  expect(relatives.text()).toContain('谱系上级：分支')
  expect(relatives.text()).toContain('添加配偶')
  expect(relatives.text()).toContain('添加子女')
  expect(relatives.text()).not.toMatch(/生父|生母/)
  expect(relatives.findAll('select')).toHaveLength(0)
  expect(relatives.findAll('input')).toHaveLength(2)
  expect(leafRow.querySelector('.lineage-connector[data-last="true"]')).not.toBeNull()
  await wrapper.get('button[aria-label="折叠 始祖"]').trigger('click')
  await wrapper.get('button[aria-label="展开 始祖"]').trigger('click')
  expect(wrapper.find('[data-entry-id="branch"]').exists()).toBe(true)
  expect(wrapper.find('[data-entry-id="leaf"]').exists()).toBe(false)
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

test('switching archive sections preserves the current profile draft', async () => {
  const wrapper = mount(App)
  await flushPromises()
  await wrapper.get('select').setValue(family.id)
  await flushPromises()
  await wrapper.get('#profile-alias').setValue('草稿谱名')
  await wrapper.get('[aria-controls="registration-panel"]').trigger('click')
  expect(wrapper.get('#registration-panel').isVisible()).toBe(true)
  expect(wrapper.get('#profile-panel').isVisible()).toBe(false)
  await wrapper.get('[aria-controls="profile-panel"]').trigger('click')
  await wrapper.get('[data-entry-id="root"]').trigger('click')
  await flushPromises()
  expect((wrapper.get('#profile-alias').element as HTMLInputElement).value).toBe('草稿谱名')
  wrapper.unmount()
})

test('restoring a backup selects the new copy and cancellation leaves it selected', async () => {
  const existing = vi.mocked(invoke).getMockImplementation()!
  const restored = { ...family, id: 'restored' }
  const results = [restored, null]
  vi.mocked(invoke).mockImplementation((command, args) =>
    command === 'restore_backup' ? Promise.resolve(results.shift() as never) : existing(command, args))
  const wrapper = mount(App)
  await flushPromises()
  await wrapper.get('select').setValue(family.id)
  await flushPromises()
  const restoreButton = () => wrapper.findAll('button').find(button => button.text() === '恢复为新族谱')!
  await restoreButton().trigger('click')
  await flushPromises()
  expect((wrapper.get('select').element as HTMLSelectElement).value).toBe(restored.id)
  expect(wrapper.get('option[value="restored"]').text()).toContain('· restored')
  expect(wrapper.text()).toContain('已恢复为独立的族谱副本')
  await restoreButton().trigger('click')
  await flushPromises()
  expect((wrapper.get('select').element as HTMLSelectElement).value).toBe(restored.id)
  wrapper.unmount()
})
