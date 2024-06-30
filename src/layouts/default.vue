<script setup lang="ts">
import { fileStore } from '../stores/fileStore';

const { workingDir } = fileStore;

const saving = ref(false);
const saveError = ref(false);
const generatingThumbnails = ref(false);
const thumbnailProgress = ref(0);
const drawer = ref(false);

fileStore.on('saving', (state) => {
  saving.value = state;
});

fileStore.on('saveError', () => {
  saveError.value = true;
});

fileStore.on('thumbnailProgress', (progress) => {
  if (!generatingThumbnails.value) {
    generatingThumbnails.value = true;
  }
  thumbnailProgress.value = progress;
  if (progress === 100) {
    generatingThumbnails.value = false;
  }
});
</script>

<template>
  <v-app>
    <v-layout>
      <v-app-bar>
        <v-btn icon @click="drawer = true">
          <v-icon>mdi-menu</v-icon>
        </v-btn>
        <v-btn to="/" icon>
          <v-icon>mdi-home</v-icon>
        </v-btn>
        <v-toolbar-title>{{ workingDir }}</v-toolbar-title>
        <v-spacer></v-spacer>
        <span v-if="saving">
          <v-progress-circular indeterminate></v-progress-circular>
        </span>
        <span v-else>Saved</span>
      </v-app-bar>
      <v-navigation-drawer expand-on-hover rail v-model="drawer">
        <v-list density="compact" nav>
          <v-list-item prepend-icon="mdi-image" title="Photos" to="/tagger"></v-list-item>
          <v-list-item prepend-icon="mdi-tag" title="Manage Tags" to="/tags"></v-list-item>
          <v-list-item prepend-icon="mdi-map-marker" title="Locations" to="/locations"></v-list-item>
          <v-list-item prepend-icon="mdi-calendar" title="Calendar" to="/calendar"></v-list-item>
          <v-list-item prepend-icon="mdi-notebook" title="Journal" to="/journal"></v-list-item>
          <v-list-item prepend-icon="mdi-music" title="Music" to="/music"></v-list-item>
        </v-list>
      </v-navigation-drawer>
      <slot></slot>
    </v-layout>
    <v-snackbar v-model="saveError">Changes could not be saved!</v-snackbar>
    <v-snackbar v-model="generatingThumbnails" :timeout="-1">
      Generating Thumbnails... ({{ thumbnailProgress }}%)
      <v-progress-linear :model-value="thumbnailProgress"></v-progress-linear>
    </v-snackbar>
  </v-app>
</template>
