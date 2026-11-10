import { invoke } from '@tauri-apps/api/core'

export interface FamilySummary {
  id: string
  name: string
  rootEntryId: string
}

export function listFamilies() {
  return invoke<{ families: FamilySummary[]; unreadableCount: number }>('list_families')
}

export function createFamily(name: string, founderName: string) {
  return invoke<FamilySummary>('create_family', { name, founderName })
}

export interface PersonFields {
  name: string
  spectrumName: string | null
  generationName: string | null
  birthText: string | null
  deathText: string | null
  notes: string | null
}

export interface EntryFields {
  rank: number | null
  branchNotes: string | null
}

export interface Registration {
  id: string
  personId: string
  parentEntryId: string | null
  name: string
  fields: EntryFields
}

interface Member {
  fields: PersonFields
  fatherId: string | null
  motherId: string | null
  spouseIds: string[]
  registrationCount: number
}

interface PersonSummary {
  id: string
  name: string
  spectrumName: string | null
}

export function memberApi(familyId: string) {
  return {
    registrations: () => invoke<Registration[]>('list_registrations', { familyId }),
    people: () => invoke<PersonSummary[]>('find_people', { familyId, query: '' }),
    read: (personId: string) => invoke<Member>('read_member', { familyId, personId }),
    addChild: (parentEntryId: string, person: { new: PersonFields } | { existing: string }, fields: EntryFields) =>
      invoke<string>('add_child', { familyId, parentEntryId, person, fields }),
    updateEntry: (entryId: string, parentEntryId: string | null, fields: EntryFields) =>
      invoke<void>('update_entry', { familyId, entryId, parentEntryId, fields }),
    deleteEntry: (entryId: string) => invoke<void>('delete_entry', { familyId, entryId }),
    createPerson: (fields: PersonFields) => invoke<string>('create_person', { familyId, fields }),
    updatePerson: (personId: string, fields: PersonFields) =>
      invoke<void>('update_person', { familyId, personId, fields }),
    setParents: (personId: string, fatherId: string | null, motherId: string | null) =>
      invoke<void>('set_parents', { familyId, personId, fatherId, motherId }),
    setSpouse: (personId: string, otherId: string, linked: boolean) =>
      invoke<void>('set_spouse', { familyId, personId, otherId, linked }),
  }
}
