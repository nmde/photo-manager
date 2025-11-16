<script setup lang="ts">
  import { fileStore } from '../stores/fileStore';

  defineEmits<{
    (e: 'search', query: string[]): void;
  }>();

  defineProps<{
    value: string[];
    loading?: boolean;
  }>();

  const { tags } = fileStore;

  const localQuery = ref<string[]>([]);
</script>

<template>
  <v-combobox
    v-model="localQuery"
    chips
    clearable
    :items="tags"
    label="Search"
    :loading="loading"
    multiple
    @update:model-value="() => {}"
  />
  <v-btn @click="$emit('search', localQuery)">Search</v-btn>
</template>
