<script setup lang="ts">
import { storeToRefs } from 'pinia';
import { useFileStore } from '../stores/fileStore';

const fileStore = useFileStore();
const { getTagColor } = fileStore;
const { tags } = storeToRefs(fileStore);

const props = defineProps<{
  label: string;
  value: string[];
  single?: boolean;
}>();

const emit = defineEmits<{
  (e: 'update', tags: string[]): void;
}>();

const selected = ref<string[]>([]);

function initialize() {
  selected.value = props.value;
}

watch(() => props.value, initialize);
</script>

<template>
  <v-combobox
    :label="props.label"
    :items="tags"
    :multiple="props.single ? false : true"
    chips
    clearable
    v-model="selected"
    @update:model-value="emit('update', selected)"
  >
    <template v-slot:item="{ item, props }">
      <v-list-item v-bind="props" :style="{ color: getTagColor(item.title) }"></v-list-item>
    </template>
    <template v-slot:chip="{ item, props }">
      <v-chip v-bind="props" :color="getTagColor(item.title)"></v-chip>
    </template>
  </v-combobox>
</template>
