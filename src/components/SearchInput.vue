<script setup lang="ts">
  import { fileStore } from '../stores/fileStore';

  defineEmits<{
    (e: 'search', query: string[]): void;
  }>();

  const props = defineProps<{
    value: string[];
    loading?: boolean;
  }>();

  const { tags } = fileStore;

  const localQuery = ref<string[]>([]);

  onMounted(() => {
    localQuery.value = props.value;
  });
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
