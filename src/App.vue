<script setup lang="ts">
import { onMounted, ref } from 'vue';
import { fileStore } from './stores/fileStore';

const { workingDir, theme } = fileStore;

const saving = ref(false);
const saveError = ref(false);
const generatingThumbnails = ref(false);
const thumbnailProgress = ref(0);
const drawer = ref(false);
const darkMode = ref(false);

fileStore.on('toggleTheme', () => {
  darkMode.value = !darkMode.value;
});

fileStore.on('saving', state => {
  saving.value = state;
});

fileStore.on('saveError', () => {
  saveError.value = true;
});

fileStore.on('thumbnailProgress', progress => {
  if (!generatingThumbnails.value) {
    generatingThumbnails.value = true;
  }
  thumbnailProgress.value = progress;
  if (progress === 100) {
    generatingThumbnails.value = false;
  }
});

onMounted(() => {
  darkMode.value = theme;
});
</script>

<template>
  <v-app :theme="darkMode ? 'dark' : 'default'">
    <v-layout>
      <v-app-bar>
        <v-btn icon @click="drawer = true">
          <v-icon>mdi-menu</v-icon>
        </v-btn>
        <v-btn icon to="/">
          <v-icon>mdi-home</v-icon>
        </v-btn>
        <v-toolbar-title>{{ workingDir }}</v-toolbar-title>
        <v-spacer />
        <span v-if="saving">
          <v-progress-circular indeterminate />
        </span>
        <span v-else>Saved</span>
      </v-app-bar>
      <v-navigation-drawer v-model="drawer" expand-on-hover rail>
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
    <v-snackbar v-model="saveError">Changes could not be saved!</v-snackbar>
    <v-snackbar v-model="generatingThumbnails" :timeout="-1">
      Generating Thumbnails... ({{ thumbnailProgress }}%)
      <v-progress-linear :model-value="thumbnailProgress" />
    </v-snackbar>
  </v-app>
</template>
