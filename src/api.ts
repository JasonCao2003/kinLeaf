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
