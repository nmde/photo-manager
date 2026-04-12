<script setup lang="ts" generic="T extends SortableItem">
  import type { ColorableItem } from '@/classes/ColorableItem';
  import type { SortableItem } from '@/classes/SortableItem';
  import stringSimilarity from 'string-similarity-js';

  const props = defineProps<{
    avatars?: boolean;
    chips?: boolean;
    colorKey: string;
    colorRepo: Record<string, ColorableItem>;
    disabled?: boolean;
    errorMessages?: string;
    id?: string; // This is for photo-detail to ensure switching photos triggers the local values changing
    items: Record<string, T>;
    itemKey?: string;
    itemSize?: string;
    label: string;
    loading?: boolean;
    multiple?: boolean;
    sortKey?: keyof T;
    value: string[];
  }>();

  const emit = defineEmits<{
    (e: 'update', value: string[]): void;
    (e: 'focused', value: boolean): void;
  }>();

  const counts = ref<Record<string, number>>({});
  const localValue = ref<string[]>([]);
  const query = ref('');

  const sortBy = computed(() => (typeof props.sortKey === 'string' ? props.sortKey : 'count'));

  const k = computed(() =>
    typeof props.itemKey === 'string' ? (props.itemKey as keyof SortableItem) : 'id',
  );

  const itemList = computed(() => {
    const items = Object.values(props.items).toSorted(
      (a, b) => (b[sortBy.value] as number) - (a[sortBy.value] as number),
    );
    if (query.value.length < 2) {
      return items.map(s => s[k.value]);
    }
    return items
      .entries()
      .map(([i, item]) => ({
        value: stringSimilarity(query.value, item.name ?? ''),
        index: i,
        count: item[sortBy.value],
      }))
      .toArray()
      .filter(i => i.value > 0)
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

  function sync() {
    localValue.value = props.value;
    for (const id in props.items) {
      counts.value[id] = props.items[id]?.[sortBy.value] as number;
    }
  }

  watch([() => props.id, () => props.value], () => {
    if (props.value !== localValue.value) {
      sync();
    }
  });

  onMounted(sync);
</script>

<template>
  <v-combobox
    v-model="localValue"
    v-model:search="query"
    aria-autocomplete="none"
    :chips="chips"
    clearable
    color="primary"
    :custom-filter="() => true"
    :disabled="disabled"
    :error-messages="errorMessages"
    :items="itemList"
    :label="label"
    :loading="loading"
    :multiple="multiple"
    @update:focused="val => emit('focused', val)"
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
        :prepend-avatar="items[item ?? '']?.photo ?? ''"
        :style="{
          color:
            colorRepo[items[item ?? '']?.[colorKey as keyof SortableItem] as string]?.color,
        }"
        :title="`${items[item ?? '']?.name} (${items[item ?? '']?.[sortBy]})`"
      />
    </template>
    <template #chip="{ item, props: cprops }">
      <v-chip
        v-if="chips"
        v-bind="cprops"
        :color="colorRepo[items[item ?? '']?.[colorKey as keyof SortableItem] as string]?.color"
        :prepend-avatar="items[item ?? '']?.photo ?? ''"
        :size="typeof itemSize === 'string' ? itemSize : 'default'"
      >
        {{ items[item ?? ''] ? items[item ?? '']?.name : item }}
      </v-chip>
      <span v-else>{{ items[item ?? '']?.name }}</span>
    </template>
  </v-combobox>
</template>
