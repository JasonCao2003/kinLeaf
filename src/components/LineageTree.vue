<script setup lang="ts">
import { computed } from 'vue'
import type { Registration } from '../api'

const props = defineProps<{
  entries: Registration[]
  rootId: string
  selectedId: string
  expandedIds: Set<string>
}>()
const emit = defineEmits<{ select: [id: string]; toggle: [id: string] }>()

const visible = computed(() => {
  const root = props.entries.find(entry => entry.id === props.rootId)
  if (!root) return []

  const byParent = new Map<string, Registration[]>()
  for (const entry of props.entries) {
    if (!entry.parentEntryId) continue
    const siblings = byParent.get(entry.parentEntryId) ?? []
    siblings.push(entry)
    byParent.set(entry.parentEntryId, siblings)
  }
  for (const siblings of byParent.values()) {
    siblings.sort((a, b) =>
      (a.fields.rank ?? Number.MAX_SAFE_INTEGER) - (b.fields.rank ?? Number.MAX_SAFE_INTEGER)
      || a.id.localeCompare(b.id))
  }

  type TreeRow = { entry: Registration; generation: number; followingSiblings: boolean[] }
  const pending: TreeRow[] = [{ entry: root, generation: 1, followingSiblings: [] }]
  const result: (TreeRow & { hasChildren: boolean })[] = []
  while (pending.length) {
    const node = pending.pop()!
    const children = byParent.get(node.entry.id) ?? []
    result.push({ ...node, hasChildren: children.length > 0 })
    if (props.expandedIds.has(node.entry.id)) {
      for (let index = children.length - 1; index >= 0; index--) {
        pending.push({
          entry: children[index],
          generation: node.generation + 1,
          followingSiblings: [...node.followingSiblings, index < children.length - 1],
        })
      }
    }
  }
  return result
})
</script>

<template>
  <ul aria-label="世系树" class="lineage">
    <li v-for="node in visible" :key="node.entry.id">
      <span
        v-for="(continues, index) in node.followingSiblings.slice(-10)"
        :key="index"
        class="lineage-connector"
        :data-elbow="index === Math.min(node.followingSiblings.length, 10) - 1"
        :data-last="!continues"
        aria-hidden="true"
      ></span>
      <button
        v-if="node.hasChildren"
        class="branch-toggle"
        type="button"
        :aria-expanded="expandedIds.has(node.entry.id)"
        :aria-label="(expandedIds.has(node.entry.id) ? '折叠 ' : '展开 ') + node.entry.name"
        @click="emit('toggle', node.entry.id)"
      >{{ expandedIds.has(node.entry.id) ? '−' : '+' }}</button>
      <span v-else class="branch-toggle" aria-hidden="true"></span>
      <button
        class="lineage-entry"
        type="button"
        :aria-current="selectedId === node.entry.id ? 'true' : undefined"
        :data-entry-id="node.entry.id"
        @click="emit('select', node.entry.id)"
      ><span>{{ node.entry.name }}</span><small>第 {{ node.generation }} 世</small></button>
    </li>
  </ul>
</template>
