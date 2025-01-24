<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { fileStore } from '../stores/fileStore';

const { tags, search, query } = fileStore;

const localQuery = ref<string[]>([]);

onMounted(() => {
  localQuery.value = query;
});
</script>

<template>
  <v-combobox
    label="Search"
    :items="tags"
    multiple
    chips
    clearable
    v-model="localQuery"
    @update:model-value="() => {}"
  >
  </v-combobox>
  <v-btn @click="search(...localQuery)">Search</v-btn>
</template>
