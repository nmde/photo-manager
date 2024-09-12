<script setup lang="ts">
import type { Person } from '~/classes/Person';
import { fileStore } from '~/stores/fileStore';

type PeopleEntry = {
  title: string;
  value: string;
  color: string;
};

const { people, peopleMap, peopleCategories } = fileStore;

const emit = defineEmits<{
  (e: 'update', value: string[]): void;
}>();

const props = defineProps<{
  label: string;
  multiple?: boolean;
  value: string[];
  sort: 'count' | 'photographer';
}>();

const peopleList = computed(() => {
  let flatPeople: Person[] = [];
  Object.values(peopleMap).forEach((persons) => {
    flatPeople = flatPeople.concat(persons);
  });
  return flatPeople
    .sort((a, b) => {
      let x = a.count;
      let y = b.count;
      if (props.sort === 'photographer') {
        x = a.photographerCount;
        y = b.photographerCount;
      }
      return y - x;
    })
    .map((person) => {
      let count = person.count;
      if (props.sort === 'photographer') {
        count = person.photographerCount;
      }
      return {
        color: peopleCategories[person.data.category].data.color,
        title: `${person.data.name} (${count})`,
        value: person.Id,
      };
    });
});

const tempPeople = ref<PeopleEntry[]>([]);

watch(
  () => props.value,
  () => {
    tempPeople.value = props.value.map((id) => {
      const p = people[id];
      let count = p.count;
      if (props.sort === 'photographer') {
        count = p.photographerCount;
      }
      return {
        color: peopleCategories[p.data.category].data.color,
        title: `${p.data.name} (${count})`,
        value: p.Id,
      };
    });
  },
);
</script>

<template>
  <v-combobox
    :label="props.label"
    :items="peopleList"
    multiple
    chips
    clearable
    item-value="value"
    v-model="tempPeople"
    @update:model-value="
      () => {
        emit(
          'update',
          tempPeople.map((p) => p.value),
        );
      }
    "
  >
    <template v-slot:item="{ item, props }">
      <v-list-item
        v-bind="props"
        :style="{ color: item.raw.color }"
        :prepend-avatar="people[item.raw.value].data.photo"
      ></v-list-item>
    </template>
    <template v-slot:chip="{ item, props }">
      <v-chip
        v-bind="props"
        size="x-large"
        :color="item.raw.color"
        :prepend-avatar="people[item.raw.value].data.photo"
      ></v-chip>
    </template>
  </v-combobox>
</template>
