<script setup lang="ts">
export type WikiItem = {
  name: string;
  path: string;
  files: Record<string, WikiItem>;
  folders: Record<string, WikiItem>;
  id: string;
};

defineProps<{
  page: WikiItem;
}>();
</script>

<template>
  <v-list-item
    v-if="Object.keys(page.folders).length === 0 && Object.keys(page.files).length === 0"
    :to="`/wiki/${page.path}`.replace('//', '/')"
    >{{ page.name }}</v-list-item
  >
  <v-list-group v-else>
    <template #activator="{ props }">
      <v-list-item v-bind="props" :title="page.name"></v-list-item>
    </template>
    <WikiFolder v-for="child in page.folders" :key="child.id" :page="child"></WikiFolder>
    <v-list-item v-for="child in page.files" :key="child.id" :to="`/wiki${child.path}`">{{
      page.name
    }}</v-list-item>
  </v-list-group>
</template>
