<script setup lang="ts">
  import { onMounted, ref } from 'vue';
  import { fileStore } from '../stores/fileStore';

  const { tags, search, query } = fileStore;

  const localQuery = ref<string[]>([]);

  onMounted(() => {
    localQuery.value = query;
  });
</script>

<template>
  <v-combobox
    v-model="localQuery"
    chips
    clearable
    :items="tags"
    label="Search"
    multiple
    @update:model-value="() => {}"
  />
  <v-btn @click="search(localQuery)">Search</v-btn>
</template>
