<script setup lang="ts">
import { ref, computed, watch, onMounted } from 'vue';
import { fileStore } from '../stores/fileStore';

type PeopleEntry = {
  title: string;
  value: string;
  color: string;
};

const { people, peopleCategories } = fileStore;

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

const peopleList = computed(() => {
  return Object.entries(counts.value).sort((a, b) => {
      let x = a[1][0];
      let y = b[1][0];
      if (props.sort === 'photographer') {
        x = a[1][1];
        y = b[1][1];
      }
      return y - x;
    })
    .map((entry) => {
      let count = entry[1][0];
      if (props.sort === 'photographer') {
        count = entry[1][1];
      }
      return {
        color: peopleCategories[people[entry[0]].data.category].data.color,
        title: `${people[entry[0]].data.name} (${count})`,
        value: entry[0],
      };
    });
});

const prevPeople = ref<PeopleEntry[]>([]);
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
      counts.value[id] = [p.count, p.photographerCount];
      return {
        color: peopleCategories[p.data.category].data.color,
        title: `${p.data.name} (${count})`,
        value: p.Id,
      };
    });
    prevPeople.value = tempPeople.value;
  },
);

onMounted(() => {
  Object.values(people).forEach((person) => {
    counts.value[person.Id] = [person.count, person.photographerCount];
  });
});
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
      (value) => {
        value.forEach((person) => {
          if (prevPeople.find((p) => p.value === person.value) === undefined) {
            if (sort === 'count') {
              counts[person.value][0] += 1;
            } else {
              counts[person.value][1] += 1;
            }
          }
        });
        prevPeople.forEach((person) => {
          if (value.find((p) => p.value === person.value) === undefined) {
            if (sort === 'count') {
              counts[person.value][0] -= 1;
            } else {
              counts[person.value][1] -= 1;
            }
          }
        });
        emit(
          'update',
          tempPeople.map((p) => p.value),
        );
        prevPeople = value;
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
