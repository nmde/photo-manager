<script setup lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { Person, type PersonData } from '@/classes/Person';
  import { PersonCategory, type PersonCategoryData } from '@/classes/PersonCategory';

  type PeopleEntry = {
    title: string;
    value: string;
    color: string;
    photo: string;
  };

  const emit = defineEmits<{
    (e: 'update', value: string[]): void;
  }>();

  const props = defineProps<{
    label: string;
    multiple?: boolean;
    value: string[];
    sort: 'count' | 'photographer';
  }>();

  const counts = ref<Record<string, number[]>>({});
  const peopleList = ref<PeopleEntry[]>([]);
  const tempPeople = ref<PeopleEntry[]>([]);

  watch(
    () => props.value,
    async () => {
      const people = Person.createPeople(
        Object.values(await invoke<Record<string, PersonData>>('get_people')),
      );
      const categories = PersonCategory.createCategories(
        await invoke<PersonCategoryData[]>('get_people_categories'),
      );
      tempPeople.value = props.value.map(id => {
        const p = people[id];
        if (p) {
          let count = p.count;
          if (props.sort === 'photographer') {
            count = p.photographerCount;
          }
          counts.value[id] = [p.count, p.photographerCount];
          return {
            color: categories[p.category]?.color ?? '',
            title: `${p.name} (${count.toString()})`,
            value: p.id,
            photo: p.photo,
          };
        }
        return {
          color: '',
          title: '',
          value: '',
          photo: '',
        };
      });
      peopleList.value = Object.values(people)
        .toSorted((a, b) =>
          props.sort === 'photographer'
            ? b.photographerCount - a.photographerCount
            : b.count - a.count,
        )
        .map(entry => ({
          color: categories[entry.category]?.color,
          title: `${entry.name} (${
            (props.sort === 'photographer' ? entry.photographerCount : entry.count).toString() ??
            '0'
          })`,
          value: entry.id,
          photo: entry.photo,
        }));
    },
  );

  function updatePeople() {
    emit(
      'update',
      tempPeople.value.map(p => p.value),
    );
  }
</script>

<template>
  <v-combobox
    v-model="tempPeople"
    chips
    clearable
    item-value="value"
    :items="peopleList"
    :label="props.label"
    multiple
    @update:model-value="() => updatePeople()"
  >
    <template #item="{ item, props: lprops }">
      <v-list-item
        v-bind="lprops"
        :prepend-avatar="item.raw.photo"
        :style="{ color: item.raw.color }"
      />
    </template>
    <template #chip="{ item, props: cprops }">
      <v-chip
        v-bind="cprops"
        :color="item.raw.color"
        :prepend-avatar="item.raw.photo"
        size="x-large"
      />
    </template>
  </v-combobox>
</template>
