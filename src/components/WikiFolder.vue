<script setup lang="ts">
  import type { WikiItem } from '@/types/WikiItem';

  defineProps<{
    page: WikiItem;
  }>();
</script>

<template>
  <v-list-item
    v-if="Object.keys(page.folders).length === 0 && Object.keys(page.files).length === 0"
    :to="`/wiki/${page.path}`.replace('//', '/')"
  >
    {{ page.name }}
  </v-list-item>
  <v-list-group v-else>
    <template #activator="{ props }">
      <v-list-item v-bind="props" :title="page.name" />
    </template>
    <WikiFolder v-for="child in page.folders" :key="child.id" :page="child" />
    <v-list-item v-for="child in page.files" :key="child.id" :to="`/wiki${child.path}`">{{
      page.name
    }}</v-list-item>
  </v-list-group>
</template>
