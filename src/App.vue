<script setup lang="ts">
  import { onMounted, ref } from 'vue';
  import { fileStore } from './stores/fileStore';

  const { theme } = fileStore;

  const darkMode = ref(false);

  fileStore.on('toggleTheme', () => {
    darkMode.value = !darkMode.value;
  });

  onMounted(() => {
    darkMode.value = theme;
  });
</script>

<template>
  <v-app :theme="darkMode ? 'dark' : 'default'">
    <v-layout>
      <v-navigation-drawer expand-on-hover permanent rail>
        <v-list density="compact" nav>
          <v-list-item prepend-icon="mdi-image" title="Photos" to="/tagger" />
          <v-list-item prepend-icon="mdi-tag" title="Manage Tags" to="/tags" />
          <v-list-item prepend-icon="mdi-map-marker" title="Locations" to="/locations" />
          <v-list-item prepend-icon="mdi-calendar" title="Calendar" to="/calendar" />
          <v-list-item prepend-icon="mdi-account" title="People" to="/people" />
          <v-list-item prepend-icon="mdi-notebook" title="Journal" to="/journal" />
          <v-list-item prepend-icon="mdi-book" title="Wiki" to="/wiki" />
          <v-list-item prepend-icon="mdi-chart-line" title="Statistics" to="/stats" />
          <v-list-item prepend-icon="mdi-cog" title="Settings" to="/settings" />
        </v-list>
      </v-navigation-drawer>
      <RouterView />
    </v-layout>
  </v-app>
</template>
