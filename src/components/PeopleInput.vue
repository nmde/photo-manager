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
  const prevPeople = ref<PeopleEntry[]>([]);
  const tempPeople = ref<PeopleEntry[]>([]);

  watch(
    () => props.value,
    async () => {
      const people = Person.createPeople(
        Object.values(await invoke<Record<string, PersonData>>('get_people')),
      );
      const categories = PersonCategory.createCategories(await invoke<PersonCategoryData[]>('get_people_categories'));
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
      prevPeople.value = tempPeople.value;
      peopleList.value = Object.entries(counts.value)
        .toSorted((a, b) => {
          let x = a[1][0];
          let y = b[1][0];
          if (props.sort === 'photographer') {
            x = a[1][1];
            y = b[1][1];
          }
          return (y ?? 0) - (x ?? 0);
        })
        .map(entry => {
          let count = entry[1][0];
          if (props.sort === 'photographer') {
            count = entry[1][1];
          }
          const p = people[entry[0]];
          if (p) {
            return {
              color: categories[p.category]?.color,
              title: `${p.name} (${count?.toString() ?? '0'})`,
              value: entry[0],
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
    },
  );

  function updatePeople(value: PeopleEntry[]) {
    for (const person of value) {
      if (!prevPeople.value.some(p => p.value === person.value)) {
        if (props.sort === 'count') {
          counts.value[person.value][0] += 1;
        } else {
          counts.value[person.value][1] += 1;
        }
      }
    }
    for (const person of prevPeople.value) {
      if (!value.some(p => p.value === person.value)) {
        if (props.sort === 'count') {
          counts.value[person.value][0] -= 1;
        } else {
          counts.value[person.value][1] -= 1;
        }
      }
    }
    emit(
      'update',
      tempPeople.value.map(p => p.value),
    );
    prevPeople.value = value;
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
    @update:model-value="value => updatePeople(value)"
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
