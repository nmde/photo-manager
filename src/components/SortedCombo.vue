<template>
  <v-combobox
    v-model="localValue"
    v-model:search="query"
    aria-autocomplete="none"
    :chips="chips"
    clearable
    color="primary"
    :custom-filter="() => true"
    :items="itemList"
    :label="label"
    :loading="loading"
    :multiple="multiple"
    @update:model-value="
      selected => {
        updateCounts(value, selected);
        if (typeof selected === 'string') {
          emit('update', [selected]);
        } else {
          emit('update', selected ?? []);
        }
      }
    "
  >
    <template #item="{ item, props: lprops }">
      <v-list-item
        v-bind="lprops"
        :prepend-avatar="items[item.raw ?? '']?.photo"
        :style="{
          color:
            colorRepo[items[item.raw ?? '']?.[colorKey as keyof SortableItem] as string]?.color,
        }"
        :title="`${items[item.raw ?? '']?.name} (${items[item.raw ?? '']?.count})`"
      />
    </template>
    <template #chip="{ item, props: cprops }">
      <v-chip
        v-if="chips"
        v-bind="cprops"
        :color="colorRepo[items[item.raw ?? '']?.[colorKey as keyof SortableItem] as string]?.color"
        :prepend-avatar="items[item.raw ?? '']?.photo"
        size="x-large"
      >
        {{ items[item.raw ?? '']?.name }}
      </v-chip>
      <span v-else>{{ items[item.raw ?? '']?.name }}</span>
    </template>
  </v-combobox>
</template>

<script setup lang="ts">
  import type { ColorableItem } from '@/classes/ColorableItem';
  import type { SortableItem } from '@/classes/SortableItem';
  import stringSimilarity from 'string-similarity-js';

  const props = defineProps<{
    avatars?: boolean;
    chips?: boolean;
    colorKey: string;
    colorRepo: Record<string, ColorableItem>;
    items: Record<string, SortableItem>;
    itemKey?: string;
    label: string;
    loading?: boolean;
    multiple?: boolean;
    value: string[];
  }>();

  const emit = defineEmits<{
    (e: 'update', value: readonly string[]): void;
  }>();

  const counts = ref<Record<string, number>>({});
  const localValue = ref<string[]>([]);
  const query = ref('');

  const k = computed(() =>
    typeof props.itemKey === 'string' ? (props.itemKey as keyof SortableItem) : 'id',
  );

  const itemList = computed(() => {
    const items = Object.values(props.items);
    if (query.value.length === 0) {
      return items
        .toSorted((a, b) => {
          const aVal = a[k.value];
          const bVal = b[k.value];
          if (aVal && bVal) {
            return (counts.value[bVal] ?? 0) - (counts.value[aVal] ?? 0);
          }
          return 0;
        })
        .map(item => item[k.value]);
    }
    return items
      .entries()
      .map(([i, item]) => ({
        value: stringSimilarity(query.value, item.name ?? ''),
        index: i,
      }))
      .toArray()
      .toSorted((a, b) => b.value - a.value)
      .map(s => items[s.index]?.[k.value]);
  });

  function updateCounts(prevValue: string[], newValue: string[]) {
    for (const val of prevValue) {
      if (!newValue.includes(val) && typeof counts.value[val] === 'number') {
        counts.value[val] -= 1;
      }
    }
    for (const val of newValue) {
      if (!prevValue.includes(val) && typeof counts.value[val] === 'number') {
        counts.value[val] += 1;
      }
    }
  }

  watch(
    () => props.items,
    () => {
      localValue.value = props.value;
      for (const id in props.items) {
        counts.value[id] = props.items[id]?.count ?? 0;
      }
    },
  );
</script>
