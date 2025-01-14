<script setup lang="ts">
import { fileStore } from '../stores/fileStore';

const { tags, search } = fileStore;

const props = defineProps<{
  value?: string[];
}>();

const query = ref<string[]>([]);

onMounted(() => {
  if (props.value) {
    query.value = props.value;
  }
});
</script>

<template>
  <v-combobox
    label="Search"
    :items="tags"
    multiple
    chips
    clearable
    v-model="query"
    @update:model-value="() => {}"
  >
  </v-combobox>
  <v-btn @click="search(...query)">Search</v-btn>
</template>
