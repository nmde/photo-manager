<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue';
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

const peopleList = computed(() =>
  Object.entries(counts.value)
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
          color: peopleCategories[p.data.category]?.data.color,
          title: `${p.data.name} (${count?.toString() ?? '0'})`,
          value: entry[0],
        };
      }
      return {
        color: '',
        title: '',
        value: '',
      };
    }),
);

const prevPeople = ref<PeopleEntry[]>([]);
const tempPeople = ref<PeopleEntry[]>([]);

watch(
  () => props.value,
  () => {
    tempPeople.value = props.value.map(id => {
      const p = people[id];
      if (p) {
        let count = p.count;
        if (props.sort === 'photographer') {
          count = p.photographerCount;
        }
        counts.value[id] = [p.count, p.photographerCount];
        return {
          color: peopleCategories[p.data.category]?.data.color ?? '',
          title: `${p.data.name} (${count.toString()})`,
          value: p.Id,
        };
      }
      return {
        color: '',
        title: '',
        value: '',
      };
    });
    prevPeople.value = tempPeople.value;
  },
);

onMounted(() => {
  for (const person of Object.values(people)) {
    counts.value[person.Id] = [person.count, person.photographerCount];
  }
});
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
    @update:model-value="
      value => {
        value.forEach(person => {
          if (!prevPeople.some(p => p.value === person.value)) {
            if (sort === 'count') {
              counts[person.value][0] += 1;
            } else {
              counts[person.value][1] += 1;
            }
          }
        });
        prevPeople.forEach(person => {
          if (!value.some(p => p.value === person.value)) {
            if (sort === 'count') {
              counts[person.value][0] -= 1;
            } else {
              counts[person.value][1] -= 1;
            }
          }
        });
        emit(
          'update',
          tempPeople.map(p => p.value),
        );
        prevPeople = value;
      }
    "
  >
    <template #item="{ item, props: lprops }">
      <v-list-item
        v-bind="lprops"
        :prepend-avatar="people[item.raw.value]?.data.photo"
        :style="{ color: item.raw.color }"
      />
    </template>
    <template #chip="{ item, props: cprops }">
      <v-chip
        v-bind="cprops"
        :color="item.raw.color"
        :prepend-avatar="people[item.raw.value]?.data.photo"
        size="x-large"
      />
    </template>
  </v-combobox>
</template>
